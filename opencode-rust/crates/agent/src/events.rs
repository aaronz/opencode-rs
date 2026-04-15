use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentEvent {
    #[serde(rename = "tool_call")]
    ToolCall { tool: String, params: Value },
    #[serde(rename = "tool_result")]
    ToolResult { tool: String, result: Value },
    #[serde(rename = "thinking")]
    Thinking { content: String },
    #[serde(rename = "token")]
    Token { content: String },
    #[serde(rename = "message")]
    Message { role: String, content: String },
    #[serde(rename = "error")]
    Error { error: String },
    #[serde(rename = "complete")]
    Complete { summary: String },
}

impl AgentEvent {
    pub fn tool_call(tool: impl Into<String>, params: Value) -> Self {
        Self::ToolCall {
            tool: tool.into(),
            params,
        }
    }

    pub fn tool_result(tool: impl Into<String>, result: Value) -> Self {
        Self::ToolResult {
            tool: tool.into(),
            result,
        }
    }

    pub fn thinking(content: impl Into<String>) -> Self {
        Self::Thinking {
            content: content.into(),
        }
    }

    pub fn token(content: impl Into<String>) -> Self {
        Self::Token {
            content: content.into(),
        }
    }

    pub fn message(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Message {
            role: role.into(),
            content: content.into(),
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self::Error {
            error: error.into(),
        }
    }

    pub fn complete(summary: impl Into<String>) -> Self {
        Self::Complete {
            summary: summary.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_event_tool_call_serialization() {
        let event = AgentEvent::tool_call("read", json!({"path": "/tmp/test"}));
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("\"type\":\"tool_call\""));
        assert!(json_str.contains("\"tool\":\"read\""));
        assert!(json_str.contains("\"params\""));
        let parsed: AgentEvent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            AgentEvent::ToolCall { tool, params } => {
                assert_eq!(tool, "read");
                assert_eq!(params["path"], "/tmp/test");
            }
            _ => panic!("Expected ToolCall variant"),
        }
    }

    #[test]
    fn test_agent_event_tool_result_serialization() {
        let event = AgentEvent::tool_result("read", json!({"content": "hello world"}));
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("\"type\":\"tool_result\""));
        assert!(json_str.contains("\"tool\":\"read\""));
        let parsed: AgentEvent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            AgentEvent::ToolResult { tool, result } => {
                assert_eq!(tool, "read");
                assert_eq!(result["content"], "hello world");
            }
            _ => panic!("Expected ToolResult variant"),
        }
    }

    #[test]
    fn test_agent_event_thinking_serialization() {
        let event = AgentEvent::thinking(" 分析中...");
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("\"type\":\"thinking\""));
        assert!(json_str.contains("分析中"));
        let parsed: AgentEvent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            AgentEvent::Thinking { content } => {
                assert_eq!(content, " 分析中...");
            }
            _ => panic!("Expected Thinking variant"),
        }
    }

    #[test]
    fn test_agent_event_token_serialization() {
        let event = AgentEvent::token("Hello");
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("\"type\":\"token\""));
        assert!(json_str.contains("\"content\":\"Hello\""));
        let parsed: AgentEvent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            AgentEvent::Token { content } => {
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected Token variant"),
        }
    }

    #[test]
    fn test_agent_event_message_serialization() {
        let event = AgentEvent::message("assistant", "Hello, how can I help?");
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("\"type\":\"message\""));
        assert!(json_str.contains("\"role\":\"assistant\""));
        assert!(json_str.contains("\"content\":\"Hello, how can I help?\""));
        let parsed: AgentEvent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            AgentEvent::Message { role, content } => {
                assert_eq!(role, "assistant");
                assert_eq!(content, "Hello, how can I help?");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_agent_event_error_serialization() {
        let event = AgentEvent::error("Tool execution failed: file not found");
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("\"type\":\"error\""));
        assert!(json_str.contains("\"error\":\"Tool execution failed: file not found\""));
        let parsed: AgentEvent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            AgentEvent::Error { error } => {
                assert_eq!(error, "Tool execution failed: file not found");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_agent_event_complete_serialization() {
        let event = AgentEvent::complete("Task completed successfully");
        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("\"type\":\"complete\""));
        assert!(json_str.contains("\"summary\":\"Task completed successfully\""));
        let parsed: AgentEvent = serde_json::from_str(&json_str).unwrap();
        match parsed {
            AgentEvent::Complete { summary } => {
                assert_eq!(summary, "Task completed successfully");
            }
            _ => panic!("Expected Complete variant"),
        }
    }

    #[test]
    fn test_all_variants_defined() {
        let variants = [
            AgentEvent::tool_call("test", json!({})),
            AgentEvent::tool_result("test", json!({})),
            AgentEvent::thinking("thinking"),
            AgentEvent::token("t"),
            AgentEvent::message("user", "msg"),
            AgentEvent::error("err"),
            AgentEvent::complete("done"),
        ];
        assert_eq!(variants.len(), 7);
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
            drop(parsed);
        }
    }

    #[test]
    fn test_events_are_cloneable() {
        let event = AgentEvent::tool_call("read", json!({"path": "/tmp"}));
        let cloned = event.clone();
        assert_eq!(
            serde_json::to_string(&event).unwrap(),
            serde_json::to_string(&cloned).unwrap()
        );
    }

    #[test]
    fn test_events_are_debug() {
        let event = AgentEvent::error("test error");
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Error"));
        assert!(debug_str.contains("test error"));
    }
}
