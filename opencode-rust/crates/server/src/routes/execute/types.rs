//! Execute API types for session execution endpoint.
//!
//! Provides request/response types for POST /api/sessions/{id}/execute

use serde::{Deserialize, Serialize};

/// Request type for session execution endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequest {
    /// The prompt to send to the agent.
    pub prompt: String,

    /// Execution mode for the agent.
    #[serde(default)]
    pub mode: Option<ExecuteMode>,

    /// Whether to stream responses. Defaults to true.
    #[serde(default = "default_stream", skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

fn default_stream() -> Option<bool> {
    Some(true)
}

/// Agent execution modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecuteMode {
    /// Build mode - full access for implementing user requests.
    Build,
    /// Plan mode - read-only mode for creating execution plans.
    Plan,
    /// General mode - helpful assistant without tool access.
    General,
}

impl ExecuteMode {
    /// Returns the system prompt for this mode.
    pub fn system_prompt(self) -> &'static str {
        match self {
            ExecuteMode::Build => {
                "You are OpenCode's BUILD agent. Implement user requests with concise, actionable output."
            }
            ExecuteMode::Plan => {
                "You are OpenCode's PLAN agent. Produce an explicit and practical execution plan."
            }
            ExecuteMode::General => {
                "You are OpenCode's GENERAL agent. Respond helpfully and clearly."
            }
        }
    }
}

/// Streaming events emitted during agent execution.
/// Used for Server-Sent Events (SSE) responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ExecuteEvent {
    /// Tool call event - emitted when agent requests tool execution.
    ToolCall {
        tool: String,
        params: serde_json::Value,
        call_id: String,
    },

    /// Tool result event - emitted after tool execution completes.
    ToolResult {
        tool: String,
        result: serde_json::Value,
        call_id: String,
        success: bool,
    },

    /// Message event - emitted when agent sends a message.
    Message { role: String, content: String },

    /// Token event - emitted for each token of LLM output (for real-time streaming).
    Token { content: String },

    /// Error event - emitted when an error occurs.
    Error { code: String, message: String },

    /// Complete event - emitted when execution finishes successfully.
    Complete { session_state: serde_json::Value },
}

impl ExecuteEvent {
    /// Creates a tool_call event.
    pub fn tool_call(
        tool: impl Into<String>,
        params: serde_json::Value,
        call_id: impl Into<String>,
    ) -> Self {
        Self::ToolCall {
            tool: tool.into(),
            params,
            call_id: call_id.into(),
        }
    }

    /// Creates a tool_result event.
    pub fn tool_result(
        tool: impl Into<String>,
        result: serde_json::Value,
        call_id: impl Into<String>,
        success: bool,
    ) -> Self {
        Self::ToolResult {
            tool: tool.into(),
            result,
            call_id: call_id.into(),
            success,
        }
    }

    /// Creates a message event.
    pub fn message(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Message {
            role: role.into(),
            content: content.into(),
        }
    }

    /// Creates a token event.
    pub fn token(content: impl Into<String>) -> Self {
        Self::Token {
            content: content.into(),
        }
    }

    /// Creates an error event.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Creates a complete event.
    pub fn complete(session_state: serde_json::Value) -> Self {
        Self::Complete { session_state }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecuteEvent, ExecuteMode, ExecuteRequest};

    #[test]
    fn test_execute_request_deserialization() {
        // Test basic deserialization
        let json = r#"{"prompt": "Hello, world!"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(req.prompt, "Hello, world!");
        assert!(req.mode.is_none());
        // stream defaults to Some(true) when not specified
        assert_eq!(req.stream, Some(true));

        // Test with mode
        let json = r#"{"prompt": "Test prompt", "mode": "build"}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(req.prompt, "Test prompt");
        assert_eq!(req.mode, Some(ExecuteMode::Build));
        // stream still defaults to Some(true)
        assert_eq!(req.stream, Some(true));

        // Test with all fields
        let json = r#"{"prompt": "Test", "mode": "plan", "stream": false}"#;
        let req: ExecuteRequest = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(req.mode, Some(ExecuteMode::Plan));
        assert_eq!(req.stream, Some(false));
    }

    #[test]
    fn test_execute_request_serialization() {
        let req = ExecuteRequest {
            prompt: "Test prompt".to_string(),
            mode: Some(ExecuteMode::General),
            stream: Some(true),
        };
        let json = serde_json::to_string(&req).expect("should serialize");
        assert!(json.contains(r#""prompt":"Test prompt""#));
        assert!(json.contains(r#""mode":"general""#));
        assert!(json.contains(r#""stream":true"#));
    }

    #[test]
    fn test_execute_mode_system_prompt() {
        assert!(ExecuteMode::Build.system_prompt().contains("BUILD"));
        assert!(ExecuteMode::Plan.system_prompt().contains("PLAN"));
        assert!(ExecuteMode::General.system_prompt().contains("GENERAL"));
    }

    #[test]
    fn test_execute_event_tool_call_serialization() {
        let event = ExecuteEvent::tool_call("read", serde_json::json!({"path": "/test"}), "call-1");
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains(r#""event":"tool_call""#));
        assert!(json.contains(r#""tool":"read""#));
        assert!(json.contains(r#""call_id":"call-1""#));
    }

    #[test]
    fn test_execute_event_tool_result_serialization() {
        let event = ExecuteEvent::tool_result(
            "read",
            serde_json::json!({"content": "file contents"}),
            "call-1",
            true,
        );
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains(r#""event":"tool_result""#));
        assert!(json.contains(r#""tool":"read""#));
        assert!(json.contains(r#""success":true"#));
    }

    #[test]
    fn test_execute_event_message_serialization() {
        let event = ExecuteEvent::message("assistant", "Hello there!");
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains(r#""event":"message""#));
        assert!(json.contains(r#""role":"assistant""#));
        assert!(json.contains(r#""content":"Hello there!""#));
    }

    #[test]
    fn test_execute_event_complete_serialization() {
        let state = serde_json::json!({"messages": 5, "tools_used": ["read", "write"]});
        let event = ExecuteEvent::complete(state);
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains(r#""event":"complete""#));
        assert!(json.contains(r#""session_state""#));
    }

    #[test]
    fn test_execute_event_error_serialization() {
        let event = ExecuteEvent::error("TOOL_NOT_FOUND", "The tool 'foo' was not found");
        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains(r#""event":"error""#));
        assert!(json.contains(r#""code":"TOOL_NOT_FOUND""#));
    }

    #[test]
    fn test_execute_event_deserialization() {
        // Test tool_call deserialization
        let json = r#"{"event": "tool_call", "tool": "read", "params": {}, "call_id": "c1"}"#;
        let event: ExecuteEvent = serde_json::from_str(json).expect("should deserialize");
        match event {
            ExecuteEvent::ToolCall {
                tool,
                params,
                call_id,
            } => {
                assert_eq!(tool, "read");
                assert_eq!(call_id, "c1");
            }
            _ => panic!("expected ToolCall variant"),
        }

        // Test message deserialization
        let json = r#"{"event": "message", "role": "assistant", "content": "Hi!"}"#;
        let event: ExecuteEvent = serde_json::from_str(json).expect("should deserialize");
        match event {
            ExecuteEvent::Message { role, content } => {
                assert_eq!(role, "assistant");
                assert_eq!(content, "Hi!");
            }
            _ => panic!("expected Message variant"),
        }

        // Test error deserialization
        let json = r#"{"event": "error", "code": "ERR", "message": "oops"}"#;
        let event: ExecuteEvent = serde_json::from_str(json).expect("should deserialize");
        match event {
            ExecuteEvent::Error { code, message } => {
                assert_eq!(code, "ERR");
                assert_eq!(message, "oops");
            }
            _ => panic!("expected Error variant"),
        }
    }

    #[test]
    fn test_execute_mode_deserialization() {
        assert_eq!(
            serde_json::from_str::<ExecuteMode>(r#""build""#).unwrap(),
            ExecuteMode::Build
        );
        assert_eq!(
            serde_json::from_str::<ExecuteMode>(r#""plan""#).unwrap(),
            ExecuteMode::Plan
        );
        assert_eq!(
            serde_json::from_str::<ExecuteMode>(r#""general""#).unwrap(),
            ExecuteMode::General
        );
    }

    #[test]
    fn test_execute_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&ExecuteMode::Build).unwrap(),
            r#""build""#
        );
        assert_eq!(
            serde_json::to_string(&ExecuteMode::Plan).unwrap(),
            r#""plan""#
        );
        assert_eq!(
            serde_json::to_string(&ExecuteMode::General).unwrap(),
            r#""general""#
        );
    }
}
