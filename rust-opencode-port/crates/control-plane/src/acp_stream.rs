use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpAgentEvent {
    pub agent_id: String,
    pub event_type: AcpEventType,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AcpEventType {
    StatusChanged,
    ToolCallStarted,
    ToolCallCompleted,
    ToolCallFailed,
    MessageGenerated,
    SessionStarted,
    SessionEnded,
    LogLine,
    Heartbeat,
}

impl AcpAgentEvent {
    pub fn new(
        agent_id: impl Into<String>,
        event_type: AcpEventType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            event_type,
            payload,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn status(agent_id: impl Into<String>, status: &str) -> Self {
        Self::new(
            agent_id,
            AcpEventType::StatusChanged,
            serde_json::json!({ "status": status }),
        )
    }

    pub fn tool_started(agent_id: impl Into<String>, tool: &str, call_id: &str) -> Self {
        Self::new(
            agent_id,
            AcpEventType::ToolCallStarted,
            serde_json::json!({
                "tool": tool,
                "call_id": call_id
            }),
        )
    }

    pub fn tool_completed(agent_id: impl Into<String>, call_id: &str, success: bool) -> Self {
        Self::new(
            agent_id,
            AcpEventType::ToolCallCompleted,
            serde_json::json!({
                "call_id": call_id,
                "success": success
            }),
        )
    }

    pub fn log(agent_id: impl Into<String>, line: &str) -> Self {
        Self::new(
            agent_id,
            AcpEventType::LogLine,
            serde_json::json!({ "line": line }),
        )
    }

    pub fn to_sse(&self) -> String {
        let data = serde_json::to_string(self).unwrap_or_default();
        format!("event: acp\ndata: {}\n\n", data)
    }
}

pub struct AcpEventStream {
    tx: broadcast::Sender<AcpAgentEvent>,
    agent_status: Arc<Mutex<HashMap<String, String>>>,
}

impl AcpEventStream {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(512);
        Self {
            tx,
            agent_status: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AcpAgentEvent> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: AcpAgentEvent) {
        if event.event_type == AcpEventType::StatusChanged {
            if let Some(status) = event.payload.get("status").and_then(|s| s.as_str()) {
                if let Ok(mut map) = self.agent_status.lock() {
                    map.insert(event.agent_id.clone(), status.to_string());
                }
            }
        }
        let _ = self.tx.send(event);
    }

    pub fn agent_status(&self, agent_id: &str) -> Option<String> {
        self.agent_status.lock().ok()?.get(agent_id).cloned()
    }

    pub fn all_agent_statuses(&self) -> HashMap<String, String> {
        self.agent_status
            .lock()
            .ok()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for AcpEventStream {
    fn default() -> Self {
        Self::new()
    }
}

pub type SharedAcpStream = Arc<AcpEventStream>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_event_status() {
        let ev = AcpAgentEvent::status("agent-1", "running");
        assert_eq!(ev.agent_id, "agent-1");
        assert_eq!(ev.event_type, AcpEventType::StatusChanged);
    }

    #[test]
    fn test_acp_event_sse_format() {
        let ev = AcpAgentEvent::status("agent-1", "idle");
        let sse = ev.to_sse();
        assert!(sse.starts_with("event: acp\ndata: "));
        assert!(sse.ends_with("\n\n"));
    }

    #[test]
    fn test_acp_stream_publish_subscribe() {
        let stream = AcpEventStream::new();
        let mut rx = stream.subscribe();
        stream.publish(AcpAgentEvent::status("a1", "running"));
        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap().agent_id, "a1");
    }

    #[test]
    fn test_acp_stream_tracks_status() {
        let stream = AcpEventStream::new();
        stream.publish(AcpAgentEvent::status("a1", "running"));
        assert_eq!(stream.agent_status("a1").as_deref(), Some("running"));
        stream.publish(AcpAgentEvent::status("a1", "idle"));
        assert_eq!(stream.agent_status("a1").as_deref(), Some("idle"));
    }
}
