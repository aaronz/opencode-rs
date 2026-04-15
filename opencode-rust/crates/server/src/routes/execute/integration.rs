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
