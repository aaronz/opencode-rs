use std::sync::Arc;

use opencode_core::{Message, Session};
use opencode_llm::Provider;
use opencode_tools::registry::ToolCall as ToolsToolCall;
use opencode_tools::ToolRegistry;
use tokio::sync::RwLock;

use crate::{Agent, AgentResponse, AgentType};

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub max_iterations: usize,
    pub max_tool_results_per_iteration: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    SessionNotActive,
    MaxIterationsExceeded { limit: usize },
    NoSuchAgent { agent_type: AgentType },
    ToolExecutionFailed { tool: String, reason: String },
    PermissionDenied { tool: String },
    SessionLocked,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::SessionNotActive => write!(f, "session is not in an active state"),
            RuntimeError::MaxIterationsExceeded { limit } => {
                write!(f, "agent loop exceeded {} iterations", limit)
            }
            RuntimeError::NoSuchAgent { agent_type } => {
                write!(f, "no agent registered for type '{}'", agent_type)
            }
            RuntimeError::ToolExecutionFailed { tool, reason } => {
                write!(f, "tool '{}' failed: {}", tool, reason)
            }
            RuntimeError::PermissionDenied { tool } => {
                write!(f, "permission denied for tool '{}'", tool)
            }
            RuntimeError::SessionLocked => write!(f, "session is locked by another agent"),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub struct AgentRuntime {
    session: Arc<RwLock<Session>>,
    config: RuntimeConfig,
    active_agent: AgentType,
}

impl AgentRuntime {
    pub fn new(session: Session, agent_type: AgentType) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            config: RuntimeConfig::default(),
            active_agent: agent_type,
        }
    }

    pub fn with_config(session: Session, agent_type: AgentType, config: RuntimeConfig) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            config,
            active_agent: agent_type,
        }
    }

    pub async fn run_loop<A: Agent>(
        &self,
        agent: &A,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, RuntimeError> {
        if self.active_agent != agent.agent_type() {
            return Err(RuntimeError::NoSuchAgent {
                agent_type: agent.agent_type(),
            });
        }

        let mut iteration = 0;
        let mut final_response = AgentResponse {
            content: String::new(),
            tool_calls: Vec::new(),
        };

        loop {
            iteration += 1;
            if iteration > self.config.max_iterations {
                return Err(RuntimeError::MaxIterationsExceeded {
                    limit: self.config.max_iterations,
                });
            }

            let response = agent
                .run(&mut *self.session.write().await, provider, tools)
                .await
                .map_err(|e| RuntimeError::ToolExecutionFailed {
                    tool: "agent".to_string(),
                    reason: e.to_string(),
                })?;

            if response.tool_calls.is_empty() {
                final_response = response;
                break;
            }

            for call in response
                .tool_calls
                .iter()
                .take(self.config.max_tool_results_per_iteration)
            {
                let tool_call = ToolsToolCall {
                    name: call.name.clone(),
                    args: call.arguments.clone(),
                    ctx: None,
                };

                let result = tools
                    .execute(&call.name, tool_call.args, None)
                    .await
                    .map_err(|e| RuntimeError::ToolExecutionFailed {
                        tool: call.name.clone(),
                        reason: e.to_string(),
                    })?;

                let result_text = if result.success {
                    result.content.clone()
                } else {
                    format!("Error: {}", result.error.clone().unwrap_or_default())
                };

                let result_message =
                    Message::user(format!("Tool '{}' result:\n{}", call.name, result_text));
                self.session.write().await.add_message(result_message);
            }
        }

        let assistant_msg = Message::assistant(&final_response.content);
        self.session.write().await.add_message(assistant_msg);

        Ok(final_response)
    }

    pub async fn switch_primary_agent(&mut self, new_type: AgentType) {
        self.active_agent = new_type;
    }

    pub fn active_agent(&self) -> AgentType {
        self.active_agent
    }

    pub async fn session(&self) -> Session {
        self.session.read().await.clone()
    }

    pub async fn into_session(self) -> Session {
        Arc::try_unwrap(self.session)
            .map(|lock| lock.into_inner())
            .unwrap_or_else(|_| Session::default())
    }

    pub async fn invoke_subagent<A: Agent>(
        &self,
        _agent: &A,
        _context: Vec<Message>,
        _provider: &dyn Provider,
        _tools: &ToolRegistry,
    ) -> Result<AgentResponse, RuntimeError> {
        todo!("subagent invocation: create subsession, run agent, handoff result")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);
        assert_eq!(runtime.active_agent(), AgentType::Build);
    }

    #[tokio::test]
    async fn test_switch_primary_agent() {
        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);
        runtime.switch_primary_agent(AgentType::Plan).await;
        assert_eq!(runtime.active_agent(), AgentType::Plan);
    }
}
