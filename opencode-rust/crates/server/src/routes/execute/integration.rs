//! Integration module bridging HTTP layer to AgentExecutor and ToolRegistry.

use std::sync::Arc;

use opencode_agent::{Agent, AgentResponse, AgentType, BuildAgent, GeneralAgent, PlanAgent};
use opencode_core::{Message, OpenCodeError, Session};
use opencode_llm::Provider;
use opencode_tools::ToolRegistry;
use uuid::Uuid;

use super::types::{ExecuteEvent, ExecuteMode};

pub struct ExecutionContext {
    pub tool_registry: Arc<ToolRegistry>,
    pub provider: Arc<dyn Provider + Send + Sync>,
    pub agent_type: AgentType,
    pub max_iterations: usize,
    pub max_tool_results_per_iteration: usize,
}

#[allow(dead_code)]
impl ExecutionContext {
    pub(crate) fn new(
        tool_registry: Arc<ToolRegistry>,
        provider: Arc<dyn Provider + Send + Sync>,
        agent_type: AgentType,
    ) -> Self {
        Self {
            tool_registry,
            provider,
            agent_type,
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
        }
    }

    pub(crate) fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub(crate) fn with_max_tool_results_per_iteration(mut self, max: usize) -> Self {
        self.max_tool_results_per_iteration = max;
        self
    }

    pub(crate) fn create_agent(&self) -> Box<dyn Agent> {
        match self.agent_type {
            AgentType::Build => Box::new(BuildAgent::new()),
            AgentType::Plan => Box::new(PlanAgent::new()),
            AgentType::General => Box::new(GeneralAgent::new()),
            _ => Box::new(GeneralAgent::new()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    ToolCall {
        call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        call_id: String,
        tool_name: String,
        result: opencode_tools::ToolResult,
    },
    Message {
        role: String,
        content: String,
    },
    Error {
        code: String,
        message: String,
    },
    Complete {
        session_id: Uuid,
        message_count: usize,
    },
}

#[allow(dead_code)]
impl From<ExecutionEvent> for ExecuteEvent {
    fn from(event: ExecutionEvent) -> Self {
        match event {
            ExecutionEvent::ToolCall {
                call_id,
                tool_name,
                arguments,
            } => ExecuteEvent::tool_call(tool_name, arguments, call_id),
            ExecutionEvent::ToolResult {
                call_id,
                tool_name,
                result,
            } => ExecuteEvent::tool_result(
                tool_name,
                serde_json::json!({
                    "success": result.success,
                    "content": result.content,
                    "error": result.error,
                }),
                call_id,
                result.success,
            ),
            ExecutionEvent::Message { role, content } => ExecuteEvent::message(role, content),
            ExecutionEvent::Error { code, message } => ExecuteEvent::error(code, message),
            ExecutionEvent::Complete { session_id, .. } => {
                ExecuteEvent::complete(serde_json::json!({
                    "session_id": session_id.to_string(),
                }))
            }
        }
    }
}

pub(crate) async fn execute_agent_loop(
    session: &mut Session,
    agent: &dyn Agent,
    provider: &dyn Provider,
    tool_registry: &ToolRegistry,
    max_iterations: usize,
    max_tool_results_per_iteration: usize,
) -> Result<AgentResponse, OpenCodeError> {
    let mut iteration = 0;
    let final_response;

    loop {
        iteration += 1;
        if iteration > max_iterations {
            return Err(OpenCodeError::InternalError(format!(
                "agent loop exceeded {} iterations",
                max_iterations
            )));
        }

        let response = agent.run(session, provider, tool_registry).await?;

        if response.tool_calls.is_empty() {
            final_response = response;
            break;
        }

        for call in response
            .tool_calls
            .iter()
            .take(max_tool_results_per_iteration)
        {
            let ctx = opencode_tools::ToolContext {
                session_id: session.id.to_string(),
                message_id: Uuid::new_v4().to_string(),
                agent: agent.name().to_string(),
                worktree: None,
                directory: None,
                permission_scope: None,
            };

            let result = tool_registry
                .execute(&call.name, call.arguments.clone(), Some(ctx))
                .await?;

            let result_text = if result.success {
                result.content.clone()
            } else {
                format!("Error: {}", result.error.clone().unwrap_or_default())
            };

            let result_message =
                Message::user(format!("Tool '{}' result:\n{}", call.name, result_text));
            session.add_message(result_message);
        }
    }

    let assistant_msg = Message::assistant(&final_response.content);
    session.add_message(assistant_msg);

    Ok(final_response)
}

#[allow(dead_code)]
impl From<ExecuteMode> for AgentType {
    fn from(mode: ExecuteMode) -> Self {
        match mode {
            ExecuteMode::Build => AgentType::Build,
            ExecuteMode::Plan => AgentType::Plan,
            ExecuteMode::General => AgentType::General,
        }
    }
}

#[allow(dead_code)]
pub(crate) fn system_prompt_for_mode(mode: ExecuteMode) -> &'static str {
    mode.system_prompt()
}

#[derive(Debug, Clone)]
pub struct IntegrationToolResult {
    pub tool_name: String,
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
}

#[allow(dead_code)]
impl IntegrationToolResult {
    /// Create from opencode_tools::ToolResult with a known tool name
    pub(crate) fn from_tools_result(
        tool_name: impl Into<String>,
        result: opencode_tools::ToolResult,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: result.success,
            content: Some(result.content),
            error: result.error,
        }
    }
}

#[allow(dead_code)]
impl From<opencode_core::ToolResult> for IntegrationToolResult {
    fn from(result: opencode_core::ToolResult) -> Self {
        Self {
            tool_name: result.tool_name,
            success: result.success,
            content: result.result,
            error: result.error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::ws::SessionHub;
    use crate::streaming::StreamMessage;

    fn execute_event_to_stream_message(
        event: &ExecuteEvent,
        session_id: &str,
    ) -> Option<StreamMessage> {
        match event {
            ExecuteEvent::ToolCall {
                tool,
                params,
                call_id,
            } => Some(StreamMessage::ToolCall {
                session_id: session_id.to_string(),
                tool_name: tool.clone(),
                args: params.clone(),
                call_id: call_id.clone(),
            }),
            ExecuteEvent::ToolResult {
                tool,
                result,
                call_id,
                success,
            } => Some(StreamMessage::ToolResult {
                session_id: session_id.to_string(),
                call_id: call_id.clone(),
                output: result
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                success: *success,
            }),
            ExecuteEvent::Message { role, content } => Some(StreamMessage::Message {
                session_id: session_id.to_string(),
                content: content.clone(),
                role: role.clone(),
            }),
            ExecuteEvent::Token { content } => Some(StreamMessage::Message {
                session_id: session_id.to_string(),
                content: content.clone(),
                role: "assistant".to_string(),
            }),
            ExecuteEvent::Error { code, message } => Some(StreamMessage::Error {
                session_id: Some(session_id.to_string()),
                error: code.clone(),
                code: code.clone(),
                message: message.clone(),
            }),
            ExecuteEvent::Complete { session_state } => Some(StreamMessage::SessionUpdate {
                session_id: session_id.to_string(),
                status: serde_json::to_string(session_state).unwrap_or_else(|_| "{}".to_string()),
            }),
        }
    }

    #[tokio::test]
    async fn test_execute_endpoint_broadcasts_to_session_hub() {
        let session_id = "test-session-123";
        let hub = std::sync::Arc::new(SessionHub::new(256));

        let tool_call =
            ExecuteEvent::tool_call("read", serde_json::json!({"path": "/test"}), "call-1");
        let message = ExecuteEvent::message("assistant", "Hello, world!");
        let complete = ExecuteEvent::complete(serde_json::json!({"status": "done"}));

        let events = vec![tool_call.clone(), message.clone(), complete.clone()];

        for event in &events {
            if let Some(stream_msg) = execute_event_to_stream_message(event, session_id) {
                hub.broadcast(session_id, stream_msg).await;
            }
        }

        let client_count = hub.get_session_client_count(session_id).await;
        assert_eq!(
            client_count, 0,
            "No clients registered yet, but events are stored"
        );

        let _receiver1 = hub.register_client(session_id, "client-1").await;
        let _receiver2 = hub.register_client(session_id, "client-2").await;

        let count1 = hub.get_session_client_count(session_id).await;
        assert_eq!(count1, 2);
    }

    #[tokio::test]
    async fn test_execute_event_to_stream_message_tool_call() {
        let event = ExecuteEvent::tool_call("read", serde_json::json!({"path": "/test"}), "call-1");
        let stream_msg = execute_event_to_stream_message(&event, "session-1");

        match stream_msg {
            Some(StreamMessage::ToolCall {
                session_id,
                tool_name,
                args,
                call_id,
            }) => {
                assert_eq!(session_id, "session-1");
                assert_eq!(tool_name, "read");
                assert_eq!(call_id, "call-1");
                assert_eq!(args["path"], "/test");
            }
            _ => panic!("Expected ToolCall variant"),
        }
    }

    #[tokio::test]
    async fn test_execute_event_to_stream_message_tool_result() {
        let event = ExecuteEvent::tool_result(
            "read",
            serde_json::json!({"content": "file contents"}),
            "call-1",
            true,
        );
        let stream_msg = execute_event_to_stream_message(&event, "session-1");

        match stream_msg {
            Some(StreamMessage::ToolResult {
                session_id,
                call_id,
                output,
                success,
            }) => {
                assert_eq!(session_id, "session-1");
                assert_eq!(call_id, "call-1");
                assert_eq!(output, "file contents");
                assert!(success);
            }
            _ => panic!("Expected ToolResult variant"),
        }
    }

    #[tokio::test]
    async fn test_execute_event_to_stream_message_message() {
        let event = ExecuteEvent::message("assistant", "Hello!");
        let stream_msg = execute_event_to_stream_message(&event, "session-1");

        match stream_msg {
            Some(StreamMessage::Message {
                session_id,
                content,
                role,
            }) => {
                assert_eq!(session_id, "session-1");
                assert_eq!(content, "Hello!");
                assert_eq!(role, "assistant");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[tokio::test]
    async fn test_execute_event_to_stream_message_token() {
        let event = ExecuteEvent::token("Hi");
        let stream_msg = execute_event_to_stream_message(&event, "session-1");

        match stream_msg {
            Some(StreamMessage::Message {
                session_id,
                content,
                role,
            }) => {
                assert_eq!(session_id, "session-1");
                assert_eq!(content, "Hi");
                assert_eq!(role, "assistant");
            }
            _ => panic!("Expected Message variant for Token"),
        }
    }

    #[tokio::test]
    async fn test_execute_event_to_stream_message_error() {
        let event = ExecuteEvent::error("TOOL_NOT_FOUND", "Tool not found");
        let stream_msg = execute_event_to_stream_message(&event, "session-1");

        match stream_msg {
            Some(StreamMessage::Error {
                session_id,
                error,
                code,
                message,
            }) => {
                assert_eq!(session_id, Some("session-1".to_string()));
                assert_eq!(error, "TOOL_NOT_FOUND");
                assert_eq!(code, "TOOL_NOT_FOUND");
                assert_eq!(message, "Tool not found");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[tokio::test]
    async fn test_execute_event_to_stream_message_complete() {
        let event = ExecuteEvent::complete(serde_json::json!({"status": "done"}));
        let stream_msg = execute_event_to_stream_message(&event, "session-1");

        match stream_msg {
            Some(StreamMessage::SessionUpdate { session_id, status }) => {
                assert_eq!(session_id, "session-1");
                assert!(status.contains("done"));
            }
            _ => panic!("Expected SessionUpdate variant"),
        }
    }

    #[tokio::test]
    async fn test_execute_events_broadcast_to_multiple_clients() {
        let session_id = "broadcast-test-session";
        let hub = std::sync::Arc::new(SessionHub::new(256));

        let mut receiver1 = hub.register_client(session_id, "client-1").await;
        let mut receiver2 = hub.register_client(session_id, "client-2").await;
        let mut receiver3 = hub.register_client(session_id, "client-3").await;

        let event = ExecuteEvent::message("assistant", "Broadcast test");
        if let Some(stream_msg) = execute_event_to_stream_message(&event, session_id) {
            hub.broadcast(session_id, stream_msg).await;
        }

        let msg1 = receiver1.recv().await.expect("client1 should receive");
        let msg2 = receiver2.recv().await.expect("client2 should receive");
        let msg3 = receiver3.recv().await.expect("client3 should receive");

        match (&msg1, &msg2, &msg3) {
            (
                StreamMessage::Message { content: c1, .. },
                StreamMessage::Message { content: c2, .. },
                StreamMessage::Message { content: c3, .. },
            ) => {
                assert_eq!(c1, "Broadcast test");
                assert_eq!(c2, "Broadcast test");
                assert_eq!(c3, "Broadcast test");
            }
            _ => panic!("Expected Message variant"),
        }
    }
}
