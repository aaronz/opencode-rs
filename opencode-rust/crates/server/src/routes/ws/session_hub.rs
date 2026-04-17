//! Session hub for managing multiple WebSocket clients across sessions.
//!
//! This module provides the `SessionHub` which manages WebSocket client connections
//! and enables broadcasting messages to all clients in a session.
//!
//! ## Architecture
//!
//! - `SessionHub`: Central registry mapping session_id -> SessionClients
//! - `SessionClients`: Per-session client registry mapping client_id -> broadcast::Sender
//! - `ClientInfo`: Metadata about each connected client
//!
//! ## Broadcasting
//!
//! Messages sent via `broadcast()` are delivered to all clients in the specified session.
//! Messages sent via `broadcast_all()` are delivered to all clients across all sessions.
//!
//! ## Example
//!
//! ```ignore
//! let hub = SessionHub::new(256); // capacity 256 per broadcast channel
//!
//! // Register a client
//! let mut receiver = hub.register_client("session-1", "client-1").await;
//!
//! // Broadcast to session
//! hub.broadcast("session-1", StreamMessage::Message { ... }).await;
//!
//! // Unregister when client disconnects
//! hub.unregister_client("session-1", "client-1").await;
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::streaming::StreamMessage;

/// Information about a connected WebSocket client.
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Unique identifier for this client connection
    pub client_id: String,
    /// Broadcast sender for sending messages to this client
    pub sender: broadcast::Sender<StreamMessage>,
}

/// Collection of clients connected to a specific session.
///
/// Maintains a mapping of client_id -> broadcast::Sender for efficient broadcasting.
#[derive(Debug, Clone)]
pub struct SessionClients {
    /// The session ID these clients are connected to
    pub session_id: String,
    /// Map of client_id to their message sender
    pub clients: HashMap<String, broadcast::Sender<StreamMessage>>,
}

impl SessionClients {
    /// Creates a new empty SessionClients for the given session_id.
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            clients: HashMap::new(),
        }
    }

    /// Adds a client to this session.
    pub fn add_client(&mut self, client_id: String, sender: broadcast::Sender<StreamMessage>) {
        self.clients.insert(client_id, sender);
    }

    /// Removes a client from this session.
    pub fn remove_client(&mut self, client_id: &str) {
        self.clients.remove(client_id);
    }

    /// Returns the number of clients in this session.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Returns true if no clients are connected.
    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}

/// Central hub for managing WebSocket client sessions.
///
/// `SessionHub` provides:
/// - Registration/unregistration of clients
/// - Broadcasting messages to all clients in a session
/// - Global broadcast to all clients across all sessions
/// - Session and client count statistics
///
/// Uses tokio broadcast channels for efficient message distribution.
#[derive(Debug)]
pub struct SessionHub {
    sessions: Arc<RwLock<HashMap<String, SessionClients>>>,
    broadcast_capacity: usize,
}

impl SessionHub {
    /// Creates a new SessionHub with the specified broadcast channel capacity.
    ///
    /// The capacity determines how many messages can be buffered per broadcast channel.
    /// Higher values allow for more burst tolerance but use more memory.
    pub fn new(broadcast_capacity: usize) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            broadcast_capacity,
        }
    }

    /// Registers a new client to a session.
    ///
    /// Returns a receiver that will receive all messages broadcast to this session.
    /// The receiver must be kept alive as long as the client is connected.
    pub async fn register_client(
        &self,
        session_id: &str,
        client_id: &str,
    ) -> broadcast::Receiver<StreamMessage> {
        let (sender, receiver) = broadcast::channel(self.broadcast_capacity);

        let mut sessions = self.sessions.write().await;
        let clients = sessions
            .entry(session_id.to_string())
            .or_insert_with(|| SessionClients::new(session_id.to_string()));
        clients.add_client(client_id.to_string(), sender.clone());

        receiver
    }

    /// Unregisters a client from a session.
    ///
    /// If this was the last client in the session, the session is removed.
    pub async fn unregister_client(&self, session_id: &str, client_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(clients) = sessions.get_mut(session_id) {
            clients.remove_client(client_id);
            if clients.is_empty() {
                sessions.remove(session_id);
            }
        }
    }

    /// Broadcasts a message to all clients in the specified session.
    pub async fn broadcast(&self, session_id: &str, message: StreamMessage) {
        let sessions = self.sessions.read().await;
        if let Some(clients) = sessions.get(session_id) {
            for sender in clients.clients.values() {
                let _ = sender.send(message.clone());
            }
        }
    }

    /// Broadcasts a message to all clients across all sessions.
    pub async fn broadcast_all(&self, message: StreamMessage) {
        let sessions = self.sessions.read().await;
        for clients in sessions.values() {
            for sender in clients.clients.values() {
                let _ = sender.send(message.clone());
            }
        }
    }

    /// Returns the number of clients connected to a specific session.
    pub async fn get_session_client_count(&self, session_id: &str) -> usize {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .map(|c| c.client_count())
            .unwrap_or(0)
    }

    /// Returns the total number of clients across all sessions.
    pub async fn total_client_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.values().map(|c| c.client_count()).sum()
    }

    /// Returns the number of active sessions.
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Returns a list of all active session IDs.
    pub async fn get_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }
}

impl Default for SessionHub {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientInfo, SessionClients, SessionHub};
    use crate::streaming::StreamMessage;

    #[tokio::test]
    async fn test_session_hub_can_store_multiple_client_connections() {
        let hub = SessionHub::new(256);

        let receiver1 = hub.register_client("session-1", "client-1").await;
        let receiver2 = hub.register_client("session-1", "client-2").await;
        let receiver3 = hub.register_client("session-2", "client-3").await;

        assert_eq!(hub.get_session_client_count("session-1").await, 2);
        assert_eq!(hub.get_session_client_count("session-2").await, 1);
        assert_eq!(hub.get_session_client_count("session-3").await, 0);

        assert_eq!(hub.total_client_count().await, 3);
        assert_eq!(hub.session_count().await, 2);

        drop(receiver1);
        drop(receiver2);
        drop(receiver3);
    }

    #[tokio::test]
    async fn test_broadcast_sends_to_all_connected_clients() {
        let hub = SessionHub::new(256);

        let mut receiver1 = hub.register_client("session-1", "client-1").await;
        let mut receiver2 = hub.register_client("session-1", "client-2").await;
        let mut receiver3 = hub.register_client("session-1", "client-3").await;

        let message = StreamMessage::Message {
            session_id: "session-1".to_string(),
            content: "Hello all clients!".to_string(),
            role: "assistant".to_string(),
        };

        hub.broadcast("session-1", message.clone()).await;

        let msg1 = receiver1.recv().await.expect("should receive message");
        let msg2 = receiver2.recv().await.expect("should receive message");
        let msg3 = receiver3.recv().await.expect("should receive message");

        match (&msg1, &msg2, &msg3) {
            (
                StreamMessage::Message { content: c1, .. },
                StreamMessage::Message { content: c2, .. },
                StreamMessage::Message { content: c3, .. },
            ) => {
                assert_eq!(c1, "Hello all clients!");
                assert_eq!(c2, "Hello all clients!");
                assert_eq!(c3, "Hello all clients!");
            }
            _ => panic!("expected Message variant"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_to_specific_session_only() {
        let hub = SessionHub::new(256);

        let mut receiver1 = hub.register_client("session-1", "client-1").await;
        let mut receiver2 = hub.register_client("session-2", "client-2").await;

        let message = StreamMessage::Message {
            session_id: "session-1".to_string(),
            content: "Only for session-1".to_string(),
            role: "assistant".to_string(),
        };

        hub.broadcast("session-1", message.clone()).await;

        let msg1 = receiver1.recv().await.expect("should receive message");
        match msg1 {
            StreamMessage::Message { content, .. } => {
                assert_eq!(content, "Only for session-1");
            }
            _ => panic!("expected Message variant"),
        }

        let msg2 = receiver2.try_recv();
        assert!(
            msg2.is_err(),
            "session-2 client should not receive the message"
        );
    }

    #[tokio::test]
    async fn test_unregister_client_removes_from_session() {
        let hub = SessionHub::new(256);

        let _receiver1 = hub.register_client("session-1", "client-1").await;
        let _receiver2 = hub.register_client("session-1", "client-2").await;

        assert_eq!(hub.get_session_client_count("session-1").await, 2);

        hub.unregister_client("session-1", "client-1").await;

        assert_eq!(hub.get_session_client_count("session-1").await, 1);
        assert_eq!(hub.total_client_count().await, 1);
    }

    #[tokio::test]
    async fn test_session_removed_when_last_client_unregisters() {
        let hub = SessionHub::new(256);

        let _receiver1 = hub.register_client("session-1", "client-1").await;
        assert_eq!(hub.session_count().await, 1);

        hub.unregister_client("session-1", "client-1").await;
        assert_eq!(hub.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast_all_to_all_sessions() {
        let hub = SessionHub::new(256);

        let mut receiver1 = hub.register_client("session-1", "client-1").await;
        let mut receiver2 = hub.register_client("session-2", "client-2").await;

        let message = StreamMessage::SessionUpdate {
            session_id: "broadcast".to_string(),
            status: "update_all".to_string(),
        };

        hub.broadcast_all(message.clone()).await;

        let msg1 = receiver1.recv().await.expect("should receive message");
        let msg2 = receiver2.recv().await.expect("should receive message");

        match (&msg1, &msg2) {
            (
                StreamMessage::SessionUpdate { status: s1, .. },
                StreamMessage::SessionUpdate { status: s2, .. },
            ) => {
                assert_eq!(s1, "update_all");
                assert_eq!(s2, "update_all");
            }
            _ => panic!("expected SessionUpdate variant"),
        }
    }

    #[test]
    fn test_session_clients_new() {
        let clients = SessionClients::new("test-session".to_string());
        assert_eq!(clients.session_id, "test-session");
        assert!(clients.is_empty());
        assert_eq!(clients.client_count(), 0);
    }

    #[test]
    fn test_session_clients_add_remove_client() {
        let mut clients = SessionClients::new("test-session".to_string());
        let (sender1, _) = tokio::sync::broadcast::channel(256);
        let (sender2, _) = tokio::sync::broadcast::channel(256);

        clients.add_client("client-1".to_string(), sender1);
        clients.add_client("client-2".to_string(), sender2);

        assert_eq!(clients.client_count(), 2);
        assert!(!clients.is_empty());

        clients.remove_client("client-1");
        assert_eq!(clients.client_count(), 1);

        clients.remove_client("client-2");
        assert!(clients.is_empty());
    }
}
