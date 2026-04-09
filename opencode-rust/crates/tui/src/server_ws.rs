use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, error, info, warn};

use crate::server_protocol::{ServerError, ServerMessage, ServerRequest};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const RECONNECT_BASE_DELAY: Duration = Duration::from_secs(1);
const RECONNECT_MAX_DELAY: Duration = Duration::from_secs(30);
const RECONNECT_MAX_ATTEMPTS: u32 = 5;

pub enum TuiServerEvent {
    Connected,
    Disconnected,
    Message(ServerMessage),
    Error(ServerError),
    Reconnecting(u32),
    ReconnectFailed,
}

pub struct ServerClient {
    url: String,
    event_tx: mpsc::Sender<TuiServerEvent>,
    reconnect_attempts: u32,
}

impl ServerClient {
    pub fn new(url: String, event_tx: mpsc::Sender<TuiServerEvent>) -> Self {
        Self {
            url,
            event_tx,
            reconnect_attempts: 0,
        }
    }

    pub async fn connect(&mut self) {
        loop {
            match self.try_connect().await {
                Ok(()) => {
                    self.reconnect_attempts = 0;
                    break;
                }
                Err(_e) => {
                    self.reconnect_attempts += 1;
                    if self.reconnect_attempts > RECONNECT_MAX_ATTEMPTS {
                        error!(
                            "Max reconnect attempts ({}) reached",
                            RECONNECT_MAX_ATTEMPTS
                        );
                        let _ = self.event_tx.send(TuiServerEvent::ReconnectFailed).await;
                        break;
                    }

                    let delay = std::cmp::min(
                        RECONNECT_BASE_DELAY * 2u32.pow(self.reconnect_attempts - 1),
                        RECONNECT_MAX_DELAY,
                    );

                    warn!(
                        "Connection failed, reconnecting in {:?} (attempt {}/{})",
                        delay, self.reconnect_attempts, RECONNECT_MAX_ATTEMPTS
                    );

                    let _ = self
                        .event_tx
                        .send(TuiServerEvent::Reconnecting(self.reconnect_attempts))
                        .await;

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn try_connect(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url).await?;
        info!("Connected to server: {}", self.url);

        let _ = self.event_tx.send(TuiServerEvent::Connected).await;

        let (mut write, mut read) = ws_stream.split();

        let (heartbeat_tx, mut heartbeat_rx) = mpsc::channel::<()>(1);

        let heartbeat_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            loop {
                interval.tick().await;
                if heartbeat_tx.send(()).await.is_err() {
                    break;
                }
            }
        });

        loop {
            tokio::select! {
                Some(_) = heartbeat_rx.recv() => {
                    let ping = ServerMessage::Ping {
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    if let Ok(json) = serde_json::to_string(&ping) {
                        if write.send(WsMessage::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
                Some(msg) = read.next() => {
                    let msg = msg?;
                    match msg {
                        WsMessage::Text(text) => {
                            debug!("Server message: {}", text);
                            if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                let _ = self.event_tx.send(TuiServerEvent::Message(server_msg)).await;
                            }
                        }
                        WsMessage::Close(_) => {
                            info!("Server closed connection");
                            break;
                        }
                        WsMessage::Ping(bytes) => {
                            if write.send(WsMessage::Pong(bytes)).await.is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                else => break,
            }
        }

        heartbeat_handle.abort();
        let _ = self.event_tx.send(TuiServerEvent::Disconnected).await;
        Err("Connection closed".into())
    }

    pub async fn send_request(&self, request: ServerRequest) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string(&request)?;
        debug!("Sending request: {}", json);
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.reconnect_attempts == 0
    }
}
