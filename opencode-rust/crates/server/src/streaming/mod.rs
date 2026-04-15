use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use opencode_core::bus::InternalEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod conn_state;
pub mod heartbeat;
pub mod stress_test;

const DEFAULT_REPLAY_LIMIT: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    Message {
        session_id: String,
        content: String,
        role: String,
    },
    ToolCall {
        session_id: String,
        tool_name: String,
        args: serde_json::Value,
        call_id: String,
    },
    ToolResult {
        session_id: String,
        call_id: String,
        output: String,
        success: bool,
    },
    SessionUpdate {
        session_id: String,
        status: String,
    },
    Heartbeat {
        timestamp: i64,
    },
    Error {
        session_id: Option<String>,
        error: String,
        code: String,
        message: String,
    },
    Connected {
        session_id: Option<String>,
    },
}

impl StreamMessage {
    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::Message { session_id, .. }
            | Self::ToolCall { session_id, .. }
            | Self::ToolResult { session_id, .. }
            | Self::SessionUpdate { session_id, .. } => Some(session_id),
            Self::Error { session_id, .. } => session_id.as_deref(),
            Self::Heartbeat { .. } | Self::Connected { .. } => None,
        }
    }

    pub fn from_internal_event(event: &InternalEvent) -> Option<Self> {
        match event {
            InternalEvent::MessageAdded {
                session_id,
                message_id,
            } => Some(Self::Message {
                session_id: session_id.clone(),
                content: format!("message_added:{message_id}"),
                role: "system".to_string(),
            }),
            InternalEvent::MessageUpdated {
                session_id,
                message_id,
            } => Some(Self::Message {
                session_id: session_id.clone(),
                content: format!("message_updated:{message_id}"),
                role: "system".to_string(),
            }),
            InternalEvent::ToolCallStarted {
                session_id,
                tool_name,
                call_id,
            } => Some(Self::ToolCall {
                session_id: session_id.clone(),
                tool_name: tool_name.clone(),
                args: serde_json::Value::Null,
                call_id: call_id.clone(),
            }),
            InternalEvent::ToolCallEnded {
                session_id,
                call_id,
                success,
            } => Some(Self::ToolResult {
                session_id: session_id.clone(),
                call_id: call_id.clone(),
                output: String::new(),
                success: *success,
            }),
            InternalEvent::ToolCallOutput {
                session_id,
                call_id,
                output,
            } => Some(Self::ToolResult {
                session_id: session_id.clone(),
                call_id: call_id.clone(),
                output: output.clone(),
                success: true,
            }),
            InternalEvent::AgentStatusChanged { session_id, status } => Some(Self::SessionUpdate {
                session_id: session_id.clone(),
                status: status.clone(),
            }),
            InternalEvent::SessionStarted(session_id) => Some(Self::SessionUpdate {
                session_id: session_id.clone(),
                status: "started".to_string(),
            }),
            InternalEvent::SessionEnded(session_id) => Some(Self::SessionUpdate {
                session_id: session_id.clone(),
                status: "ended".to_string(),
            }),
            InternalEvent::Error { source, message } => Some(Self::Error {
                session_id: None,
                error: source.clone(),
                code: source.clone(),
                message: message.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplayEntry {
    pub sequence: u64,
    pub message: StreamMessage,
}

#[derive(Debug, Clone)]
pub struct ReconnectionStore {
    replay_limit: usize,
    inner: Arc<Mutex<HashMap<String, SessionReplay>>>,
}

#[derive(Debug, Default)]
struct SessionReplay {
    next_sequence: u64,
    messages: VecDeque<ReplayEntry>,
    tokens: HashMap<String, u64>,
}

impl ReconnectionStore {
    pub fn new(replay_limit: usize) -> Self {
        Self {
            replay_limit,
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record_message(&self, session_id: &str, message: StreamMessage) -> u64 {
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let replay = guard.entry(session_id.to_string()).or_default();
        replay.next_sequence = replay.next_sequence.saturating_add(1);
        let sequence = replay.next_sequence;
        replay.messages.push_back(ReplayEntry { sequence, message });
        while replay.messages.len() > self.replay_limit {
            replay.messages.pop_front();
        }
        sequence
    }

    pub fn replay_from(&self, session_id: &str, sequence: u64) -> Vec<ReplayEntry> {
        let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        guard
            .get(session_id)
            .map(|replay| {
                replay
                    .messages
                    .iter()
                    .filter(|entry| entry.sequence > sequence)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn generate_token(&self, session_id: &str, last_sequence: Option<u64>) -> String {
        let token = Uuid::new_v4().to_string();
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        let replay = guard.entry(session_id.to_string()).or_default();
        let sequence = last_sequence.unwrap_or(replay.next_sequence);
        replay.tokens.insert(token.clone(), sequence);
        token
    }

    pub fn validate_token(&self, session_id: &str, token: &str) -> Option<u64> {
        let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        guard
            .get(session_id)
            .and_then(|replay| replay.tokens.get(token).copied())
    }
}

impl Default for ReconnectionStore {
    fn default() -> Self {
        Self::new(DEFAULT_REPLAY_LIMIT)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReconnectionStore, StreamMessage};

    #[test]
    fn stream_message_serialization_deserialization() {
        let message = StreamMessage::Message {
            session_id: "session-1".to_string(),
            content: "hello".to_string(),
            role: "assistant".to_string(),
        };

        let serialized = serde_json::to_string(&message).expect("serialize should work");
        let deserialized: StreamMessage =
            serde_json::from_str(&serialized).expect("deserialize should work");

        match deserialized {
            StreamMessage::Message {
                session_id,
                content,
                role,
            } => {
                assert_eq!(session_id, "session-1");
                assert_eq!(content, "hello");
                assert_eq!(role, "assistant");
            }
            _ => panic!("unexpected stream message variant"),
        }
    }

    #[test]
    fn error_message_format_is_standardized() {
        let error = StreamMessage::Error {
            session_id: Some("session-err".to_string()),
            error: "PARSE_ERROR".to_string(),
            code: "PARSE_ERROR".to_string(),
            message: "invalid payload".to_string(),
        };

        let value = serde_json::to_value(&error).expect("serialize should work");
        assert_eq!(value["type"], "error");
        assert_eq!(value["session_id"], "session-err");
        assert_eq!(value["error"], "PARSE_ERROR");
        assert_eq!(value["code"], "PARSE_ERROR");
        assert_eq!(value["message"], "invalid payload");
    }

    #[test]
    fn reconnection_token_generation_and_validation() {
        let store = ReconnectionStore::new(16);
        store.record_message(
            "session-a",
            StreamMessage::SessionUpdate {
                session_id: "session-a".to_string(),
                status: "active".to_string(),
            },
        );

        let token = store.generate_token("session-a", None);
        let sequence = store
            .validate_token("session-a", &token)
            .expect("token should validate");
        assert_eq!(sequence, 1);

        assert!(store.validate_token("session-a", "missing-token").is_none());
    }
}

pub use conn_state::{
    ConnectionEvent, ConnectionInfo, ConnectionMonitor, ConnectionStats, ConnectionStatus,
    ConnectionType,
};
pub use stress_test::{ConnectionStressTester, StressTestConfig, StressTestResult};
