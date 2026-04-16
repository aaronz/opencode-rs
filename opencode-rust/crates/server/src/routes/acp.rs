use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use futures::stream::{self, Stream};
use opencode_control_plane::{AcpAgentEvent, SharedAcpStream};
use opencode_core::acp::{AcpHandshakeAck, AcpHandshakeRequest, AcpProtocol};
use opencode_permission::{ApprovalQueue, PermissionScope};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::mpsc;
use tracing::info;

use opencode_storage::{SqliteProjectRepository, SqliteSessionRepository, StorageService};

use crate::ServerState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpStatusResponse {
    pub status: String,
    pub version: String,
    pub acp_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpHandshakeResponse {
    pub version: String,
    pub server_id: String,
    pub session_id: String,
    pub accepted: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpConnectRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpConnectResponse {
    pub status: String,
    pub url: String,
    pub connection_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpEventRequest {
    pub event_type: String,
    pub payload: serde_json::Value,
}

pub struct AcpState {
    pub protocol: Arc<RwLock<AcpProtocol>>,
    pub enabled: bool,
}

#[allow(dead_code)]
impl AcpState {
    pub(crate) fn new(enabled: bool) -> Self {
        Self {
            protocol: Arc::new(RwLock::new(AcpProtocol::new("server", "1.0"))),
            enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::acp_ws::{AcpClientRegistry, SharedAcpClientRegistry};
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::Responder;

    fn create_test_server_state() -> ServerState {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();
        let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
        let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));
        ServerState {
            storage: Arc::new(StorageService::new(session_repo, project_repo, pool)),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(opencode_core::Config::default())),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: crate::streaming::ReconnectionStore::default(),
            temp_db_dir: None,
            connection_monitor: std::sync::Arc::new(
                crate::streaming::conn_state::ConnectionMonitor::new(),
            ),
            share_server: std::sync::Arc::new(std::sync::RwLock::new(
                crate::routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: true,
            acp_stream: opencode_control_plane::AcpEventStream::new().into(),
            acp_client_registry: std::sync::Arc::new(tokio::sync::RwLock::new(
                AcpClientRegistry::new(),
            )),
            tool_registry: std::sync::Arc::new(opencode_tools::ToolRegistry::new()),
            session_hub: std::sync::Arc::new(crate::routes::ws::SessionHub::new(256)),
            server_start_time: std::time::SystemTime::now(),
            permission_manager: std::sync::Arc::new(std::sync::RwLock::new(
                opencode_core::PermissionManager::default(),
            )),
            approval_queue: std::sync::Arc::new(std::sync::RwLock::new(ApprovalQueue::new(
                PermissionScope::Full,
            ))),
        }
    }

    fn create_disabled_server_state() -> ServerState {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut config = opencode_core::Config::default();
        let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();
        let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
        let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));
        ServerState {
            storage: Arc::new(StorageService::new(session_repo, project_repo, pool)),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(config)),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: crate::streaming::ReconnectionStore::default(),
            temp_db_dir: None,
            connection_monitor: std::sync::Arc::new(
                crate::streaming::conn_state::ConnectionMonitor::new(),
            ),
            share_server: std::sync::Arc::new(std::sync::RwLock::new(
                crate::routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: false,
            acp_stream: opencode_control_plane::AcpEventStream::new().into(),
            acp_client_registry: std::sync::Arc::new(tokio::sync::RwLock::new(
                AcpClientRegistry::new(),
            )),
            tool_registry: std::sync::Arc::new(opencode_tools::ToolRegistry::new()),
            session_hub: std::sync::Arc::new(crate::routes::ws::SessionHub::new(256)),
            server_start_time: std::time::SystemTime::now(),
            permission_manager: std::sync::Arc::new(std::sync::RwLock::new(
                opencode_core::PermissionManager::default(),
            )),
            approval_queue: std::sync::Arc::new(std::sync::RwLock::new(ApprovalQueue::new(
                PermissionScope::Full,
            ))),
        }
    }

    #[actix_web::test]
    async fn test_acp_status_when_enabled() {
        let state = web::Data::new(create_test_server_state());
        let req = TestRequest::default().to_http_request();
        let resp = acp_status(state).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_status_when_disabled() {
        let state = web::Data::new(create_disabled_server_state());
        let req = TestRequest::default().to_http_request();
        let resp = acp_status(state).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_handshake_when_disabled() {
        let state = web::Data::new(create_disabled_server_state());
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: "test-client".to_string(),
            capabilities: vec![],
        });
        let resp = acp_handshake(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_handshake_when_enabled() {
        let state = web::Data::new(create_test_server_state());
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: "test-client".to_string(),
            capabilities: vec!["chat".to_string()],
        });
        let resp = acp_handshake(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_handshake_version_mismatch() {
        let state = web::Data::new(create_test_server_state());
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeRequest {
            version: "2.0".to_string(),
            client_id: "test-client".to_string(),
            capabilities: vec![],
        });
        let resp = acp_handshake(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_ack_when_disabled() {
        let state = web::Data::new(create_disabled_server_state());
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeAck {
            session_id: "test-session".to_string(),
            confirmed: true,
        });
        let resp = acp_ack(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_connect_when_disabled() {
        let state = web::Data::new(create_disabled_server_state());
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpConnectRequest {
            url: "http://localhost:9000".to_string(),
        });
        let resp = acp_connect(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/status").route(web::get().to(acp_status)));
    cfg.service(web::resource("/handshake").route(web::post().to(acp_handshake)));
    cfg.service(web::resource("/connect").route(web::post().to(acp_connect)));
    cfg.service(web::resource("/ack").route(web::post().to(acp_ack)));
    cfg.service(web::resource("/events").route(web::get().to(acp_events)));
}

async fn acp_status(state: web::Data<ServerState>) -> impl Responder {
    HttpResponse::Ok().json(AcpStatusResponse {
        status: if state.acp_enabled {
            "ready"
        } else {
            "disabled"
        }
        .to_string(),
        version: "1.0".to_string(),
        acp_enabled: state.acp_enabled,
    })
}

async fn acp_handshake(
    state: web::Data<ServerState>,
    body: web::Json<AcpHandshakeRequest>,
) -> impl Responder {
    if !state.acp_enabled {
        return HttpResponse::Ok().json(AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: String::new(),
            session_id: String::new(),
            accepted: false,
            error: Some("ACP is disabled".to_string()),
        });
    }

    let protocol = AcpProtocol::new("server", "1.0");
    let response = protocol.process_handshake(body.into_inner());

    if response.accepted {
        let event = AcpAgentEvent::status("server", "handshake_completed");
        state.acp_stream.publish(event);
    }

    HttpResponse::Ok().json(AcpHandshakeResponse {
        version: response.version,
        server_id: response.server_id,
        session_id: response.session_id,
        accepted: response.accepted,
        error: response.error,
    })
}

async fn acp_connect(
    state: web::Data<ServerState>,
    body: web::Json<AcpConnectRequest>,
) -> impl Responder {
    if !state.acp_enabled {
        return HttpResponse::Ok().json(AcpConnectResponse {
            status: "disabled".to_string(),
            url: body.url.clone(),
            connection_id: None,
        });
    }

    let connection_id = format!("acp-conn-{}", uuid::Uuid::new_v4());

    let event = AcpAgentEvent::new(
        "server",
        opencode_control_plane::AcpEventType::StatusChanged,
        serde_json::json!({
            "connection_id": connection_id,
            "url": body.url,
            "status": "connecting"
        }),
    );
    state.acp_stream.publish(event);

    HttpResponse::Ok().json(AcpConnectResponse {
        status: "connected".to_string(),
        url: body.url.clone(),
        connection_id: Some(connection_id),
    })
}

async fn acp_ack(
    state: web::Data<ServerState>,
    body: web::Json<AcpHandshakeAck>,
) -> impl Responder {
    if !state.acp_enabled {
        return HttpResponse::Ok().json(serde_json::json!({
            "confirmed": false,
            "error": "ACP is disabled",
            "code": 6001
        }));
    }

    let protocol = AcpProtocol::new("server", "1.0");
    let confirmed = protocol.confirm_handshake(body.into_inner());

    if confirmed {
        let event = AcpAgentEvent::status("server", "session_confirmed");
        state.acp_stream.publish(event);
    }

    HttpResponse::Ok().json(serde_json::json!({
        "confirmed": confirmed
    }))
}

async fn acp_events(
    req: HttpRequest,
    state: web::Data<ServerState>,
) -> Result<HttpResponse, Error> {
    let connection_id = format!("acp-sse-{}", uuid::Uuid::new_v4());

    info!("ACP SSE connect: connection_id={}", connection_id);

    let stream = create_acp_event_stream(state.into_inner(), connection_id.clone());

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(Box::pin(stream)))
}

fn create_acp_event_stream(
    state: Arc<ServerState>,
    connection_id: String,
) -> impl Stream<Item = Result<web::Bytes, Error>> {
    let (tx, rx) = mpsc::channel::<String>(128);

    let tx_init = tx.clone();
    let acp_stream = state.acp_stream.clone();
    actix_rt::spawn(async move {
        let connected_event = AcpAgentEvent::status("server", "connected");
        let sse_data = connected_event.to_sse();
        let _ = tx_init.send(sse_data).await;
    });

    let mut bus_rx = acp_stream.subscribe();
    let tx_bus = tx.clone();
    actix_rt::spawn(async move {
        loop {
            match bus_rx.recv().await {
                Ok(event) => {
                    let sse_data = event.to_sse();
                    if tx_bus.send(sse_data).await.is_err() {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let connection_monitor = Arc::clone(&state.connection_monitor);
    let conn_id = connection_id.clone();
    stream::unfold(rx, move |mut rx| {
        let connection_monitor = Arc::clone(&connection_monitor);
        let conn_id = conn_id.clone();
        async move {
            match rx.recv().await {
                Some(data) => {
                    connection_monitor.heartbeat_success(&conn_id).await;
                    Some((Ok::<_, Error>(web::Bytes::from(data.clone())), rx))
                }
                None => {
                    connection_monitor
                        .unregister_connection(&conn_id, "stream_ended")
                        .await;
                    None
                }
            }
        }
    })
}
