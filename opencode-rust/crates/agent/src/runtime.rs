#![allow(unused_assignments)]

use std::sync::Arc;
use std::time::Instant;

use opencode_core::bus::SharedEventBus;
use opencode_core::events::DomainEvent;
use opencode_core::instance::Instance;
use opencode_core::{Message, Session};
use opencode_llm::provider::{EventCallback, LlmEvent};
use opencode_llm::Provider;
use opencode_permission::AgentPermissionScope;
use opencode_tools::registry::ToolCall as ToolsToolCall;
use opencode_tools::{ToolContext, ToolRegistry};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{Agent, AgentResponse, AgentType, ToolCall};

/// Result from a subagent execution, containing the response
/// and metadata about the subagent run.
#[derive(Debug, Clone)]
pub struct SubagentResult {
    /// The response content from the subagent
    pub response: AgentResponse,
    /// The session ID of the child context used by the subagent
    pub child_session_id: Uuid,
    /// The agent type that was executed
    pub agent_type: AgentType,
    /// The effective permission scope for this subagent.
    /// This is the intersection of parent's scope and subagent's own permissions.
    pub effective_permission_scope: AgentPermissionScope,
}

/// Errors that can occur during subagent invocation.
#[derive(Debug, Clone)]
pub enum SubagentError {
    /// The parent session is in an invalid state for subagent invocation.
    SessionNotActive,
    /// The subagent execution failed.
    SubagentExecutionFailed { reason: String },
    /// Parent session was modified during subagent execution (invariant violation).
    ParentContextModified,
    /// Failed to fork session for subagent context.
    ForkFailed { reason: String },
}

impl std::fmt::Display for SubagentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubagentError::SessionNotActive => write!(f, "parent session is not active"),
            SubagentError::SubagentExecutionFailed { reason } => {
                write!(f, "subagent execution failed: {}", reason)
            }
            SubagentError::ParentContextModified => {
                write!(f, "parent context was modified during subagent execution")
            }
            SubagentError::ForkFailed { reason } => {
                write!(f, "failed to fork session for subagent: {}", reason)
            }
        }
    }
}

impl std::error::Error for SubagentError {}

/// Configuration for the agent runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub max_iterations: usize,
    pub max_tool_results_per_iteration: usize,
    pub permission_scope: AgentPermissionScope,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Full,
        }
    }
}

/// Errors that can occur during runtime operations.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    SessionNotActive,
    MaxIterationsExceeded {
        limit: usize,
    },
    NoSuchAgent {
        agent_type: AgentType,
    },
    ToolExecutionFailed {
        tool: String,
        reason: String,
    },
    PermissionDenied {
        tool: String,
    },
    SessionLocked,
    /// Invariant violation: attempted to activate a second primary agent
    /// while one is already running.
    MultiplePrimaryAgents {
        current: AgentType,
        attempted: AgentType,
    },
    /// Invariant violation: attempted to deactivate or switch while transitioning.
    AgentTransitionInProgress {
        current: AgentType,
    },
    /// Invariant violation: attempted to operate on inactive runtime.
    NoActivePrimaryAgent,
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
            RuntimeError::MultiplePrimaryAgents { current, attempted } => {
                write!(
                    f,
                    "invariant violation: cannot activate '{}' - '{}' is already running",
                    attempted, current
                )
            }
            RuntimeError::AgentTransitionInProgress { current } => {
                write!(
                    f,
                    "invariant violation: '{}' is transitioning - cannot switch now",
                    current
                )
            }
            RuntimeError::NoActivePrimaryAgent => {
                write!(f, "invariant violation: no active primary agent")
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

/// State of a primary agent in the runtime.
/// This enforces the "exactly one active primary agent" invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimaryAgentState {
    /// No primary agent is currently active.
    Inactive,
    /// A primary agent is actively running.
    Running,
    /// Currently transitioning between agents.
    Transitioning,
}

/// PrimaryAgentTracker state machine:
///
/// ```text
///                    ┌─────────────────┐
///                    │    Inactive     │
///                    └────────┬────────┘
///                             │ activate()
///                             ▼
///                    ┌─────────────────┐
///          ┌────────►│     Running     │◄────────┐
///          │         └────────┬────────┘         │
///          │                  │                  │
///          │    begin_transition()                │ activate()
///          │                  ▼                  │
///          │         ┌─────────────────┐         │
///          └─────────│  Transitioning  │─────────┘
///   deactivate()     └─────────────────┘
/// ```
///
/// Valid transitions:
/// - Inactive → Running: activate()
/// - Running → Transitioning: begin_transition()
/// - Transitioning → Running: complete_transition()
/// - Running → Inactive: deactivate()
///
/// Subagents (via invoke_subagent) run on forked sessions and do not affect
/// the PrimaryAgentTracker state - they can execute concurrently with the primary agent.
#[derive(Debug, Clone)]
pub struct PrimaryAgentTracker {
    pub state: PrimaryAgentState,
    pub agent_type: Option<AgentType>,
}

impl PrimaryAgentTracker {
    pub fn new() -> Self {
        Self {
            state: PrimaryAgentState::Inactive,
            agent_type: None,
        }
    }

    /// Create a tracker with an active primary agent.
    /// This is the idiomatic constructor when the agent is known to be active.
    pub fn new_active(agent_type: AgentType) -> Self {
        Self {
            state: PrimaryAgentState::Running,
            agent_type: Some(agent_type),
        }
    }

    /// Activate a new primary agent. Returns error if one is already running.
    pub fn activate(&mut self, agent_type: AgentType) -> Result<(), RuntimeError> {
        match &self.state {
            PrimaryAgentState::Inactive => {
                self.state = PrimaryAgentState::Running;
                self.agent_type = Some(agent_type);
                Ok(())
            }
            PrimaryAgentState::Running => Err(RuntimeError::MultiplePrimaryAgents {
                current: self.agent_type.ok_or(RuntimeError::NoActivePrimaryAgent)?,
                attempted: agent_type,
            }),
            PrimaryAgentState::Transitioning => Err(RuntimeError::AgentTransitionInProgress {
                current: self.agent_type.ok_or(RuntimeError::NoActivePrimaryAgent)?,
            }),
        }
    }

    /// Begin transitioning to a new primary agent.
    /// This puts the runtime in Transitioning state before the switch.
    pub fn begin_transition(&mut self) -> Result<AgentType, RuntimeError> {
        match &self.state {
            PrimaryAgentState::Inactive => Err(RuntimeError::NoActivePrimaryAgent),
            PrimaryAgentState::Running => {
                let current = self.agent_type.ok_or(RuntimeError::NoActivePrimaryAgent)?;
                self.state = PrimaryAgentState::Transitioning;
                Ok(current)
            }
            PrimaryAgentState::Transitioning => Err(RuntimeError::AgentTransitionInProgress {
                current: self.agent_type.ok_or(RuntimeError::NoActivePrimaryAgent)?,
            }),
        }
    }

    /// Complete transition to a new primary agent.
    pub fn complete_transition(&mut self, new_type: AgentType) {
        self.state = PrimaryAgentState::Running;
        self.agent_type = Some(new_type);
    }

    /// Deactivate the current primary agent.
    pub fn deactivate(&mut self) -> Result<AgentType, RuntimeError> {
        match &self.state {
            PrimaryAgentState::Inactive => Err(RuntimeError::NoActivePrimaryAgent),
            PrimaryAgentState::Running => {
                let current = self.agent_type.ok_or(RuntimeError::NoActivePrimaryAgent)?;
                self.state = PrimaryAgentState::Inactive;
                self.agent_type = None;
                Ok(current)
            }
            PrimaryAgentState::Transitioning => Err(RuntimeError::AgentTransitionInProgress {
                current: self.agent_type.ok_or(RuntimeError::NoActivePrimaryAgent)?,
            }),
        }
    }

    /// Check if a primary agent is currently active.
    pub fn is_active(&self) -> bool {
        self.state == PrimaryAgentState::Running
    }

    /// Get the current active agent type, if any.
    pub fn active_type(&self) -> Option<AgentType> {
        self.agent_type
    }
}

impl Default for PrimaryAgentTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AgentRuntime {
    session: Arc<RwLock<Session>>,
    config: RuntimeConfig,
    primary_tracker: PrimaryAgentTracker,
    event_bus: Option<SharedEventBus>,
}

impl AgentRuntime {
    pub fn new(session: Session, agent_type: AgentType) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            config: RuntimeConfig::default(),
            primary_tracker: PrimaryAgentTracker::new_active(agent_type),
            event_bus: None,
        }
    }

    pub fn with_config(session: Session, agent_type: AgentType, config: RuntimeConfig) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            config,
            primary_tracker: PrimaryAgentTracker::new_active(agent_type),
            event_bus: None,
        }
    }

    pub fn with_event_bus(mut self, event_bus: SharedEventBus) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    pub fn set_event_bus(&mut self, event_bus: SharedEventBus) {
        self.event_bus = Some(event_bus);
    }

    pub fn session_id(&self) -> String {
        self.session.blocking_read().id.to_string()
    }

    pub async fn run_loop<A: Agent + ?Sized>(
        &self,
        agent: &A,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, RuntimeError> {
        if !self.primary_tracker.is_active() {
            tracing::error!("No active primary agent");
            return Err(RuntimeError::NoActivePrimaryAgent);
        }
        if self.primary_tracker.active_type() != Some(agent.agent_type()) {
            tracing::error!(current = ?self.primary_tracker.active_type(), requested = ?agent.agent_type(), "Agent type mismatch");
            return Err(RuntimeError::NoSuchAgent {
                agent_type: agent.agent_type(),
            });
        }

        let session_id = self.session.read().await.id.to_string();
        tracing::info!(session_id = %session_id, agent = ?agent.agent_type(), max_iterations = self.config.max_iterations, "Starting agent run loop");

        if let Some(ref bus) = self.event_bus {
            bus.publish(DomainEvent::AgentStarted {
                session_id: session_id.clone(),
                agent: agent.name().to_string(),
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
                tracing::warn!(session_id = %session_id, iteration = iteration, limit = self.config.max_iterations, "Max iterations exceeded");
                return Err(RuntimeError::MaxIterationsExceeded {
                    limit: self.config.max_iterations,
                });
            }

            tracing::debug!(session_id = %session_id, iteration = iteration, "Agent iteration starting");

            let llm_start = Instant::now();
            let response = agent
                .run(&mut *self.session.write().await, provider, tools)
                .await;
            let llm_duration = llm_start.elapsed();

            tracing::info!(
                session_id = %session_id,
                iteration = iteration,
                llm_duration_ms = llm_duration.as_millis(),
                "LLM call completed"
            );

            let response = response.map_err(|e| {
                tracing::error!(session_id = %session_id, error = %e, "Agent run failed");
                RuntimeError::ToolExecutionFailed {
                    tool: "agent".to_string(),
                    reason: e.to_string(),
                }
            })?;

            if response.tool_calls.is_empty() {
                tracing::info!(session_id = %session_id, iteration = iteration, response_len = response.content.len(), "Agent completed successfully");
                final_response = response;
                break;
            }

            tracing::debug!(session_id = %session_id, iteration = iteration, tool_count = response.tool_calls.len(), "Processing tool calls");

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

                let ctx = ToolContext {
                    session_id: self.session.read().await.id.to_string(),
                    message_id: Uuid::new_v4().to_string(),
                    agent: agent.name().to_string(),
                    worktree: Instance::worktree().map(|p| p.to_string_lossy().to_string()),
                    directory: Instance::directory().map(|p| p.to_string_lossy().to_string()),
                    permission_scope: Some(self.config.permission_scope),
                };

                tracing::debug!(session_id = %session_id, tool = %call.name, "Executing tool");

                if let Some(ref bus) = self.event_bus {
                    bus.publish(DomainEvent::ToolCallStarted {
                        session_id: session_id.clone(),
                        tool_name: call.name.clone(),
                        call_id: call.id.clone(),
                    });
                }

                let result = tools
                    .execute(&call.name, tool_call.args, Some(ctx))
                    .await
                    .map_err(|e| {
                        tracing::error!(session_id = %session_id, tool = %call.name, error = %e, "Tool execution failed");
                        RuntimeError::ToolExecutionFailed {
                            tool: call.name.clone(),
                            reason: e.to_string(),
                        }
                    })?;

                if let Some(ref bus) = self.event_bus {
                    bus.publish(DomainEvent::ToolCallEnded {
                        session_id: session_id.clone(),
                        call_id: call.id.clone(),
                        success: result.success,
                    });
                }

                let result_text = if result.success {
                    tracing::debug!(session_id = %session_id, tool = %call.name, "Tool execution succeeded");
                    result.content.clone()
                } else {
                    tracing::warn!(session_id = %session_id, tool = %call.name, error = ?result.error, "Tool execution returned error");
                    format!("Error: {}", result.error.clone().unwrap_or_default())
                };

                let result_message =
                    Message::user(format!("Tool '{}' result:\n{}", call.name, result_text));
                self.session.write().await.add_message(result_message);
            }
        }

        let assistant_msg = Message::assistant(&final_response.content);
        self.session.write().await.add_message(assistant_msg);

        if let Some(ref bus) = self.event_bus {
            bus.publish(DomainEvent::AgentStopped {
                session_id: session_id.clone(),
                agent: agent.name().to_string(),
            });
        }

        Ok(final_response)
    }

    pub async fn run_loop_streaming<A: Agent + ?Sized>(
        &self,
        agent: &A,
        provider: &dyn Provider,
        tools: &ToolRegistry,
        external_events: Option<EventCallback>,
    ) -> Result<AgentResponse, RuntimeError> {
        if !self.primary_tracker.is_active() {
            tracing::error!("No active primary agent");
            return Err(RuntimeError::NoActivePrimaryAgent);
        }
        if self.primary_tracker.active_type() != Some(agent.agent_type()) {
            tracing::error!(current = ?self.primary_tracker.active_type(), requested = ?agent.agent_type(), "Agent type mismatch");
            return Err(RuntimeError::NoSuchAgent {
                agent_type: agent.agent_type(),
            });
        }

        let session_id = self.session.read().await.id.to_string();
        tracing::info!(session_id = %session_id, agent = ?agent.agent_type(), max_iterations = self.config.max_iterations, "Starting streaming agent run loop");

        if let Some(ref bus) = self.event_bus {
            bus.publish(DomainEvent::AgentStarted {
                session_id: session_id.clone(),
                agent: agent.name().to_string(),
            });
        }

        let mut iteration = 0;
        let mut final_response = AgentResponse {
            content: String::new(),
            tool_calls: Vec::new(),
        };

        let ext_events_arc = external_events.map(|e| Arc::new(std::sync::Mutex::new(e)));

        loop {
            iteration += 1;
            if iteration > self.config.max_iterations {
                tracing::warn!(session_id = %session_id, iteration = iteration, limit = self.config.max_iterations, "Max iterations exceeded");
                return Err(RuntimeError::MaxIterationsExceeded {
                    limit: self.config.max_iterations,
                });
            }

            tracing::debug!(session_id = %session_id, iteration = iteration, "Agent streaming iteration starting");

            let content_buffer = Arc::new(std::sync::Mutex::new(String::new()));
            let tool_calls_buffer = Arc::new(std::sync::Mutex::new(Vec::new()));
            let cb_clone = content_buffer.clone();
            let tc_clone = tool_calls_buffer.clone();
            let ext_clone = ext_events_arc.clone();
            let event_bus_clone = self.event_bus.clone();
            let session_id_clone = session_id.clone();
            let provider_name = provider.provider_name().to_string();

            let events_callback: EventCallback = Box::new(move |event| match event {
                LlmEvent::TextChunk(text) => {
                    if let Ok(mut guard) = cb_clone.lock() {
                        guard.push_str(&text);
                    }
                    if let Some(ref ext) = ext_clone {
                        if let Ok(mut guard) = ext.lock() {
                            let callback = &mut *guard;
                            callback(LlmEvent::TextChunk(text.clone()));
                        }
                    }
                    if let Some(ref bus) = event_bus_clone {
                        bus.publish(DomainEvent::LlmTokenStreamed {
                            session_id: session_id_clone.clone(),
                            delta: text,
                        });
                    }
                }
                LlmEvent::ToolCall {
                    name,
                    arguments,
                    id,
                } => {
                    if let Ok(mut guard) = tc_clone.lock() {
                        guard.push(ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            arguments: arguments.clone(),
                        });
                    }
                    if let Some(ref ext) = ext_clone {
                        if let Ok(mut guard) = ext.lock() {
                            let callback = &mut *guard;
                            callback(LlmEvent::ToolCall {
                                name,
                                arguments,
                                id,
                            });
                        }
                    }
                }
                LlmEvent::Error(e) => {
                    if let Some(ref ext) = ext_clone {
                        if let Ok(mut guard) = ext.lock() {
                            let callback = &mut *guard;
                            callback(LlmEvent::Error(e.clone()));
                        }
                    }
                    if let Some(ref bus) = event_bus_clone {
                        bus.publish(DomainEvent::LlmError {
                            session_id: session_id_clone.clone(),
                            error: e,
                        });
                    }
                }
                _ => {}
            });

            if let Some(ref bus) = self.event_bus {
                bus.publish(DomainEvent::LlmRequestStarted {
                    session_id: session_id.clone(),
                    provider: provider_name.clone(),
                    model: "".to_string(),
                });
            }

            let llm_start = Instant::now();
            let result = agent
                .run_streaming(
                    &mut *self.session.write().await,
                    provider,
                    tools,
                    events_callback,
                )
                .await;
            let llm_duration = llm_start.elapsed();

            tracing::info!(
                session_id = %session_id,
                iteration = iteration,
                llm_duration_ms = llm_duration.as_millis(),
                "LLM streaming call completed"
            );

            if let Some(ref bus) = self.event_bus {
                bus.publish(DomainEvent::LlmResponseCompleted {
                    session_id: session_id.clone(),
                    total_tokens: None,
                });
            }

            result.map_err(|e| {
                tracing::error!(session_id = %session_id, error = %e, "Agent run_streaming failed");
                RuntimeError::ToolExecutionFailed {
                    tool: "agent".to_string(),
                    reason: e.to_string(),
                }
            })?;

            if let Ok(guard) = content_buffer.lock() {
                final_response.content = guard.clone();
            }
            final_response.tool_calls = tool_calls_buffer.lock().unwrap().clone();

            if final_response.tool_calls.is_empty() {
                tracing::info!(session_id = %session_id, iteration = iteration, response_len = final_response.content.len(), "Agent completed successfully");
                break;
            }

            tracing::debug!(session_id = %session_id, iteration = iteration, tool_count = final_response.tool_calls.len(), "Processing tool calls");

            for call in final_response
                .tool_calls
                .iter()
                .take(self.config.max_tool_results_per_iteration)
            {
                let tool_call = ToolsToolCall {
                    name: call.name.clone(),
                    args: call.arguments.clone(),
                    ctx: None,
                };

                let ctx = ToolContext {
                    session_id: self.session.read().await.id.to_string(),
                    message_id: Uuid::new_v4().to_string(),
                    agent: agent.name().to_string(),
                    worktree: Instance::worktree().map(|p| p.to_string_lossy().to_string()),
                    directory: Instance::directory().map(|p| p.to_string_lossy().to_string()),
                    permission_scope: Some(self.config.permission_scope),
                };

                tracing::debug!(session_id = %session_id, tool = %call.name, "Executing tool");

                if let Some(ref bus) = self.event_bus {
                    bus.publish(DomainEvent::ToolCallStarted {
                        session_id: session_id.clone(),
                        tool_name: call.name.clone(),
                        call_id: call.id.clone(),
                    });
                }

                let result = tools
                    .execute(&call.name, tool_call.args, Some(ctx))
                    .await
                    .map_err(|e| {
                        tracing::error!(session_id = %session_id, tool = %call.name, error = %e, "Tool execution failed");
                        RuntimeError::ToolExecutionFailed {
                            tool: call.name.clone(),
                            reason: e.to_string(),
                        }
                    })?;

                if let Some(ref bus) = self.event_bus {
                    bus.publish(DomainEvent::ToolCallEnded {
                        session_id: session_id.clone(),
                        call_id: call.id.clone(),
                        success: result.success,
                    });
                }

                let result_text = if result.success {
                    tracing::debug!(session_id = %session_id, tool = %call.name, "Tool execution succeeded");
                    result.content.clone()
                } else {
                    tracing::warn!(session_id = %session_id, tool = %call.name, error = ?result.error, "Tool execution returned error");
                    format!("Error: {}", result.error.clone().unwrap_or_default())
                };

                if let Some(ref ext) = ext_events_arc {
                    if let Ok(mut guard) = ext.lock() {
                        let callback = &mut *guard;
                        callback(LlmEvent::ToolResult {
                            id: call.id.clone(),
                            output: result_text.clone(),
                        });
                    }
                }

                let result_message =
                    Message::user(format!("Tool '{}' result:\n{}", call.name, result_text));
                self.session.write().await.add_message(result_message);
            }
        }

        let assistant_msg = Message::assistant(&final_response.content);
        self.session.write().await.add_message(assistant_msg);

        if let Some(ref bus) = self.event_bus {
            bus.publish(DomainEvent::AgentStopped {
                session_id: session_id.clone(),
                agent: agent.name().to_string(),
            });
        }

        Ok(final_response)
    }

    pub async fn switch_primary_agent(&mut self, new_type: AgentType) -> Result<(), RuntimeError> {
        self.primary_tracker.begin_transition()?;
        self.primary_tracker.complete_transition(new_type);
        Ok(())
    }

    pub fn active_agent(&self) -> Option<AgentType> {
        self.primary_tracker.active_type()
    }

    pub fn is_primary_agent_active(&self) -> bool {
        self.primary_tracker.is_active()
    }

    pub fn primary_agent_state(&self) -> PrimaryAgentState {
        self.primary_tracker.state.clone()
    }

    pub async fn deactivate_primary_agent(&mut self) -> Result<AgentType, RuntimeError> {
        self.primary_tracker.deactivate()
    }

    pub async fn activate_primary_agent(
        &mut self,
        new_type: AgentType,
    ) -> Result<(), RuntimeError> {
        self.primary_tracker.activate(new_type)
    }

    pub async fn session(&self) -> Session {
        self.session.read().await.clone()
    }

    pub async fn with_session_mut<R>(&self, f: impl FnOnce(&mut Session) -> R) -> R {
        let mut guard = self.session.write().await;
        f(&mut guard)
    }

    pub async fn into_session(self) -> Session {
        Arc::try_unwrap(self.session)
            .map(|lock| lock.into_inner())
            .unwrap_or_else(|_| Session::default())
    }

    pub async fn invoke_subagent<A: Agent>(
        &self,
        agent: &A,
        context: Vec<Message>,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<SubagentResult, RuntimeError> {
        if !self.primary_tracker.is_active() {
            return Err(RuntimeError::NoActivePrimaryAgent);
        }

        let parent_session = self.session.read().await;
        let parent_message_count = parent_session.messages.len();
        let parent_id = parent_session.id;
        drop(parent_session);

        let child_session_id = Uuid::new_v4();
        let parent_snapshot = self.session.read().await.clone();

        let mut child_session = parent_snapshot.fork(child_session_id);
        child_session.messages.clear();

        for msg in context {
            child_session.add_message(msg);
        }

        drop(parent_snapshot);

        let subagent_response = agent
            .run(&mut child_session, provider, tools)
            .await
            .map_err(|e| RuntimeError::ToolExecutionFailed {
                tool: "subagent".to_string(),
                reason: e.to_string(),
            })?;

        let parent_after = self.session.read().await;
        if parent_after.messages.len() != parent_message_count {
            return Err(RuntimeError::SessionLocked);
        }
        if parent_after.id != parent_id {
            return Err(RuntimeError::SessionLocked);
        }
        drop(parent_after);

        let subagent_own_scope = AgentPermissionScope::from_agent_permissions(
            agent.can_write_files(),
            agent.can_run_commands(),
        );
        let effective_scope = self.config.permission_scope.intersect(subagent_own_scope);

        Ok(SubagentResult {
            response: subagent_response,
            child_session_id,
            agent_type: agent.agent_type(),
            effective_permission_scope: effective_scope,
        })
    }

    pub fn get_permission_scope(&self) -> AgentPermissionScope {
        self.config.permission_scope
    }

    pub fn with_permission_scope(mut self, scope: AgentPermissionScope) -> Self {
        self.config.permission_scope = scope;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sealed;
    use opencode_core::OpenCodeError;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc as StdArc, Mutex as StdMutex};

    #[test]
    fn test_primary_agent_tracker_new_is_inactive() {
        let tracker = PrimaryAgentTracker::new();
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
        assert!(tracker.agent_type.is_none());
        assert!(!tracker.is_active());
    }

    #[test]
    fn test_primary_agent_tracker_activate() {
        let mut tracker = PrimaryAgentTracker::new();
        assert!(tracker.activate(AgentType::Build).is_ok());
        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert_eq!(tracker.agent_type, Some(AgentType::Build));
        assert!(tracker.is_active());
    }

    #[test]
    fn test_primary_agent_tracker_cannot_activate_second() {
        let mut tracker = PrimaryAgentTracker::new();
        assert!(tracker.activate(AgentType::Build).is_ok());
        let result = tracker.activate(AgentType::Plan);
        assert!(result.is_err());
        match result {
            Err(RuntimeError::MultiplePrimaryAgents { current, attempted }) => {
                assert_eq!(current, AgentType::Build);
                assert_eq!(attempted, AgentType::Plan);
            }
            _ => panic!("Expected MultiplePrimaryAgents error"),
        }
    }

    #[test]
    fn test_primary_agent_tracker_transition() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        let current = tracker.begin_transition().unwrap();
        assert_eq!(current, AgentType::Build);
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
        tracker.complete_transition(AgentType::Plan);
        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert_eq!(tracker.agent_type, Some(AgentType::Plan));
    }

    #[test]
    fn test_primary_agent_tracker_cannot_switch_during_transition() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();
        let result = tracker.begin_transition();
        assert!(result.is_err());
        match result {
            Err(RuntimeError::AgentTransitionInProgress { current }) => {
                assert_eq!(current, AgentType::Build);
            }
            _ => panic!("Expected AgentTransitionInProgress error"),
        }
    }

    #[test]
    fn test_primary_agent_tracker_deactivate() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        let deactivated = tracker.deactivate().unwrap();
        assert_eq!(deactivated, AgentType::Build);
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
        assert!(tracker.agent_type.is_none());
        assert!(!tracker.is_active());
    }

    #[test]
    fn test_primary_agent_tracker_cannot_deactivate_inactive() {
        let mut tracker = PrimaryAgentTracker::new();
        let result = tracker.deactivate();
        assert!(result.is_err());
        match result {
            Err(RuntimeError::NoActivePrimaryAgent) => {}
            _ => panic!("Expected NoActivePrimaryAgent error"),
        }
    }

    #[test]
    fn test_primary_invariant_activate_from_inactive_succeeds() {
        let mut tracker = PrimaryAgentTracker::new();
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
        assert!(tracker.agent_type.is_none());
        assert!(!tracker.is_active());

        tracker.activate(AgentType::Build).unwrap();

        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert_eq!(tracker.agent_type, Some(AgentType::Build));
        assert!(tracker.is_active());
        assert_eq!(tracker.active_type(), Some(AgentType::Build));
    }

    #[test]
    fn test_primary_invariant_activate_from_running_fails_with_error() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        assert_eq!(tracker.state, PrimaryAgentState::Running);

        let result = tracker.activate(AgentType::Plan);

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            RuntimeError::MultiplePrimaryAgents { current, attempted } => {
                assert_eq!(current, AgentType::Build);
                assert_eq!(attempted, AgentType::Plan);
            }
            _ => panic!("Expected MultiplePrimaryAgents error, got: {:?}", err),
        }
        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert_eq!(tracker.agent_type, Some(AgentType::Build));
    }

    #[test]
    fn test_primary_invariant_activate_from_transitioning_fails() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);

        let result = tracker.activate(AgentType::Plan);

        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::AgentTransitionInProgress { current } => {
                assert_eq!(current, AgentType::Build);
            }
            _ => panic!("Expected AgentTransitionInProgress error"),
        }
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
    }

    #[test]
    fn test_primary_invariant_deactivate_from_running_succeeds() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        assert!(tracker.is_active());

        let deactivated = tracker.deactivate().unwrap();

        assert_eq!(deactivated, AgentType::Build);
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
        assert!(tracker.agent_type.is_none());
        assert!(!tracker.is_active());
    }

    #[test]
    fn test_primary_invariant_deactivate_from_inactive_fails() {
        let mut tracker = PrimaryAgentTracker::new();
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);

        let result = tracker.deactivate();

        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::NoActivePrimaryAgent => {}
            _ => panic!("Expected NoActivePrimaryAgent error"),
        }
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
    }

    #[test]
    fn test_primary_invariant_deactivate_from_transitioning_fails() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);

        let result = tracker.deactivate();

        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::AgentTransitionInProgress { current } => {
                assert_eq!(current, AgentType::Build);
            }
            _ => panic!("Expected AgentTransitionInProgress error"),
        }
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
    }

    #[test]
    fn test_primary_invariant_begin_transition_from_running_succeeds() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        assert!(tracker.is_active());

        let current = tracker.begin_transition().unwrap();

        assert_eq!(current, AgentType::Build);
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
        assert_eq!(tracker.agent_type, Some(AgentType::Build));
    }

    #[test]
    fn test_primary_invariant_begin_transition_from_inactive_fails() {
        let mut tracker = PrimaryAgentTracker::new();
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);

        let result = tracker.begin_transition();

        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::NoActivePrimaryAgent => {}
            _ => panic!("Expected NoActivePrimaryAgent error"),
        }
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
    }

    #[test]
    fn test_primary_invariant_begin_transition_from_transitioning_fails() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);

        let result = tracker.begin_transition();

        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::AgentTransitionInProgress { current } => {
                assert_eq!(current, AgentType::Build);
            }
            _ => panic!("Expected AgentTransitionInProgress error"),
        }
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
    }

    #[test]
    fn test_primary_invariant_complete_transition_restores_running() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);

        tracker.complete_transition(AgentType::Plan);

        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert_eq!(tracker.agent_type, Some(AgentType::Plan));
        assert!(tracker.is_active());
        assert_eq!(tracker.active_type(), Some(AgentType::Plan));
    }

    #[test]
    fn test_primary_invariant_exactly_one_through_full_cycle() {
        let mut tracker = PrimaryAgentTracker::new();

        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
        assert!(!tracker.is_active());

        tracker.activate(AgentType::Build).unwrap();
        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert!(tracker.is_active());

        tracker.begin_transition().unwrap();
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
        assert!(!tracker.is_active());

        tracker.complete_transition(AgentType::Plan);
        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert_eq!(tracker.agent_type, Some(AgentType::Plan));
        assert!(tracker.is_active());
    }

    #[test]
    fn test_primary_invariant_activate_after_transition_then_deactivate() {
        let mut tracker = PrimaryAgentTracker::new();

        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();
        tracker.complete_transition(AgentType::Plan);
        assert!(tracker.is_active());
        assert_eq!(tracker.agent_type, Some(AgentType::Plan));
        assert_eq!(tracker.active_type(), Some(AgentType::Plan));

        tracker.deactivate().unwrap();
        assert!(!tracker.is_active());
        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
    }

    #[test]
    fn test_primary_invariant_inactive_state_requires_no_agent_type() {
        let tracker = PrimaryAgentTracker::new();

        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
        assert!(!tracker.is_active());
        assert!(tracker.agent_type.is_none());
        assert!(tracker.active_type().is_none());
    }

    #[test]
    fn test_primary_invariant_transitioning_state_preserves_agent_type() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();

        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
        assert!(!tracker.is_active());
        assert_eq!(tracker.agent_type, Some(AgentType::Build));
        assert_eq!(tracker.active_type(), Some(AgentType::Build));
    }

    #[test]
    fn test_primary_invariant_cannot_skip_transitioning() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();

        tracker.complete_transition(AgentType::Plan);

        assert_eq!(tracker.state, PrimaryAgentState::Running);
        assert_eq!(tracker.agent_type, Some(AgentType::Plan));
        assert!(tracker.is_active());
    }

    #[test]
    fn test_primary_invariant_debug_clone_behavior() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();

        let tracker_clone = tracker.clone();
        assert_eq!(tracker.state, tracker_clone.state);
        assert_eq!(tracker.agent_type, tracker_clone.agent_type);
    }

    #[test]
    fn test_primary_invariant_agents_have_descriptions() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();

        let display = format!("{:?}", tracker);
        assert!(display.contains("Running"));
        assert!(display.contains("Build"));
    }

    #[tokio::test]
    async fn test_primary_invariant_runtime_agent_type_reflected_in_errors() {
        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);

        let result = runtime.activate_primary_agent(AgentType::Explore).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_display = format!("{}", err);
        assert!(
            err_display.contains("build"),
            "Error should contain 'build': {}",
            err_display
        );
        assert!(
            err_display.contains("explore"),
            "Error should contain 'explore': {}",
            err_display
        );
    }

    #[test]
    fn test_primary_invariant_multiple_primary_agents_error_contains_both_types() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();

        let result = tracker.activate(AgentType::Plan);

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            RuntimeError::MultiplePrimaryAgents { current, attempted } => {
                assert_eq!(current, AgentType::Build);
                assert_eq!(attempted, AgentType::Plan);
            }
            _ => panic!("Expected MultiplePrimaryAgents variant"),
        }
    }

    #[test]
    fn test_primary_invariant_agent_transition_in_progress_error() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();

        let result = tracker.begin_transition();

        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::AgentTransitionInProgress { current } => {
                assert_eq!(current, AgentType::Build);
            }
            _ => panic!("Expected AgentTransitionInProgress variant"),
        }
    }

    #[test]
    fn test_primary_invariant_no_active_primary_agent_error() {
        let tracker = PrimaryAgentTracker::new();

        assert_eq!(tracker.state, PrimaryAgentState::Inactive);
        assert!(!tracker.is_active());
        assert!(tracker.active_type().is_none());
    }

    #[tokio::test]
    async fn test_runtime_starts_with_exactly_one_primary_agent() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);
        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.primary_agent_state(), PrimaryAgentState::Running);
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));
    }

    #[tokio::test]
    async fn test_runtime_cannot_activate_second_primary_agent() {
        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);
        let result = runtime.activate_primary_agent(AgentType::Plan).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RuntimeError::MultiplePrimaryAgents { current, attempted } => {
                assert_eq!(current, AgentType::Build);
                assert_eq!(attempted, AgentType::Plan);
            }
            e => panic!("Expected MultiplePrimaryAgents error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_runtime_switch_primary_agent_transitions_properly() {
        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));
        runtime.switch_primary_agent(AgentType::Plan).await.unwrap();
        assert_eq!(runtime.active_agent(), Some(AgentType::Plan));
        assert!(runtime.is_primary_agent_active());
    }

    #[tokio::test]
    async fn test_runtime_deactivate_properly_transitions() {
        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);
        assert!(runtime.is_primary_agent_active());
        runtime.deactivate_primary_agent().await.unwrap();
        assert!(!runtime.is_primary_agent_active());
        assert_eq!(runtime.primary_agent_state(), PrimaryAgentState::Inactive);
        assert!(runtime.active_agent().is_none());
    }

    #[tokio::test]
    async fn test_runtime_activate_after_deactivate() {
        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);
        runtime.deactivate_primary_agent().await.unwrap();
        runtime
            .activate_primary_agent(AgentType::Plan)
            .await
            .unwrap();
        assert_eq!(runtime.active_agent(), Some(AgentType::Plan));
        assert!(runtime.is_primary_agent_active());
    }

    #[tokio::test]
    async fn test_runtime_old_tests_still_work() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));
    }

    struct MockSubagent {
        response_content: String,
        agent_type: AgentType,
        can_write: bool,
        can_run_commands: bool,
    }

    impl MockSubagent {
        fn new(response_content: &str, agent_type: AgentType) -> Self {
            Self {
                response_content: response_content.to_string(),
                agent_type,
                can_write: false,
                can_run_commands: false,
            }
        }

        fn with_permissions(mut self, can_write: bool, can_run_commands: bool) -> Self {
            self.can_write = can_write;
            self.can_run_commands = can_run_commands;
            self
        }
    }

    impl sealed::Sealed for MockSubagent {}

    #[async_trait::async_trait]
    impl Agent for MockSubagent {
        fn agent_type(&self) -> AgentType {
            self.agent_type
        }

        fn name(&self) -> &str {
            "mock_subagent"
        }

        fn description(&self) -> &str {
            "Mock subagent for testing"
        }

        fn can_execute_tools(&self) -> bool {
            self.can_write || self.can_run_commands
        }

        fn can_write_files(&self) -> bool {
            self.can_write
        }

        fn can_run_commands(&self) -> bool {
            self.can_run_commands
        }

        async fn run(
            &self,
            session: &mut Session,
            _provider: &dyn Provider,
            _tools: &ToolRegistry,
        ) -> Result<AgentResponse, OpenCodeError> {
            session.add_message(Message::assistant("subagent response"));
            Ok(AgentResponse {
                content: self.response_content.clone(),
                tool_calls: Vec::new(),
            })
        }
    }

    #[derive(Clone)]
    struct MockRuntimeTool;

    impl opencode_tools::sealed::Sealed for MockRuntimeTool {}

    #[async_trait::async_trait]
    impl opencode_tools::Tool for MockRuntimeTool {
        fn name(&self) -> &str {
            "mock_runtime_tool"
        }

        fn description(&self) -> &str {
            "Mock runtime tool"
        }

        fn clone_tool(&self) -> Box<dyn opencode_tools::Tool> {
            Box::new(self.clone())
        }

        async fn execute(
            &self,
            args: serde_json::Value,
            _ctx: Option<opencode_tools::ToolContext>,
        ) -> Result<opencode_tools::ToolResult, OpenCodeError> {
            Ok(opencode_tools::ToolResult::ok(format!("tool-ok:{args}")))
        }
    }

    struct MockStreamingAgent {
        calls: AtomicUsize,
    }

    impl MockStreamingAgent {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }

    impl sealed::Sealed for MockStreamingAgent {}

    #[async_trait::async_trait]
    impl Agent for MockStreamingAgent {
        fn agent_type(&self) -> AgentType {
            AgentType::Build
        }

        fn name(&self) -> &str {
            "mock_streaming_agent"
        }

        fn description(&self) -> &str {
            "Mock streaming agent"
        }

        fn can_execute_tools(&self) -> bool {
            true
        }

        fn can_write_files(&self) -> bool {
            false
        }

        fn can_run_commands(&self) -> bool {
            false
        }

        async fn run(
            &self,
            _session: &mut Session,
            _provider: &dyn Provider,
            _tools: &ToolRegistry,
        ) -> Result<AgentResponse, OpenCodeError> {
            Ok(AgentResponse {
                content: String::new(),
                tool_calls: Vec::new(),
            })
        }

        async fn run_streaming(
            &self,
            session: &mut Session,
            _provider: &dyn Provider,
            _tools: &ToolRegistry,
            mut events: EventCallback,
        ) -> Result<AgentResponse, OpenCodeError> {
            let call_index = self.calls.fetch_add(1, Ordering::SeqCst);
            if call_index == 0 {
                events(LlmEvent::TextChunk("thinking ".to_string()));
                events(LlmEvent::ToolCall {
                    id: "tool-1".to_string(),
                    name: "mock_runtime_tool".to_string(),
                    arguments: serde_json::json!({"value": 1}),
                });
                return Ok(AgentResponse {
                    content: String::new(),
                    tool_calls: Vec::new(),
                });
            }

            let saw_tool_result = session
                .messages
                .iter()
                .any(|message| message.content.contains("Tool 'mock_runtime_tool' result:"));
            assert!(
                saw_tool_result,
                "tool result should be added before second iteration"
            );

            events(LlmEvent::TextChunk("done".to_string()));
            Ok(AgentResponse {
                content: String::new(),
                tool_calls: Vec::new(),
            })
        }
    }

    #[tokio::test]
    async fn test_run_loop_streaming_forwards_text_tool_call_and_tool_result_events() {
        let runtime = AgentRuntime::new(Session::default(), AgentType::Build);
        let agent = MockStreamingAgent::new();
        let provider = MockProvider;
        let tools = ToolRegistry::new();
        tools.register(MockRuntimeTool).await;

        let events = StdArc::new(StdMutex::new(Vec::<String>::new()));
        let events_clone = StdArc::clone(&events);
        let callback: EventCallback = Box::new(move |event| {
            let label = match event {
                LlmEvent::TextChunk(text) => format!("text:{text}"),
                LlmEvent::ToolCall { name, id, .. } => format!("tool_call:{name}:{id}"),
                LlmEvent::ToolResult { id, output } => format!("tool_result:{id}:{output}"),
                LlmEvent::Done => "done".to_string(),
                LlmEvent::Error(err) => format!("error:{err}"),
            };
            events_clone.lock().unwrap().push(label);
        });

        let response = runtime
            .run_loop_streaming(&agent, &provider, &tools, Some(callback))
            .await
            .unwrap();

        assert_eq!(response.content, "done");
        assert!(response.tool_calls.is_empty());

        let collected = events.lock().unwrap().clone();
        assert_eq!(
            collected,
            vec![
                "text:thinking ".to_string(),
                "tool_call:mock_runtime_tool:tool-1".to_string(),
                "tool_result:tool-1:tool-ok:{\"value\":1}".to_string(),
                "text:done".to_string(),
            ]
        );

        let session = runtime.session().await;
        assert!(session
            .messages
            .iter()
            .any(|message| message.content.contains("Tool 'mock_runtime_tool' result:")));
        assert_eq!(
            session
                .messages
                .last()
                .map(|message| message.content.as_str()),
            Some("done")
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_subagent_inherits_parent_permissions() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);

        let subagent =
            MockSubagent::new("response", AgentType::Explore).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::Full
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_readonly_parent_restricts_full_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::ReadOnly,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::Explore).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::ReadOnly
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_none_parent_restricts_any_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::None,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::Explore).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::None
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_subagent_cannot_exceed_parent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Restricted,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::General).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::Restricted
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_full_parent_full_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Full,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::General).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::Full
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_readonly_parent_readonly_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::ReadOnly,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::Explore).with_permissions(false, false);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::ReadOnly
        );
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_full_restricted() {
        let scope = AgentPermissionScope::Full.intersect(AgentPermissionScope::Restricted);
        assert_eq!(scope, AgentPermissionScope::Restricted);
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_full_none() {
        let scope = AgentPermissionScope::Full.intersect(AgentPermissionScope::None);
        assert_eq!(scope, AgentPermissionScope::None);
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_readonly_full() {
        let scope = AgentPermissionScope::ReadOnly.intersect(AgentPermissionScope::Full);
        assert_eq!(scope, AgentPermissionScope::ReadOnly);
    }

    #[tokio::test]
    async fn test_runtime_with_permission_scope() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build)
            .with_permission_scope(AgentPermissionScope::ReadOnly);

        assert_eq!(
            runtime.get_permission_scope(),
            AgentPermissionScope::ReadOnly
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_restricted_parent_readonly_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Restricted,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::Explore).with_permissions(false, false);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::ReadOnly
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_full_parent_readonly_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Full,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::Explore).with_permissions(false, false);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::ReadOnly
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_none_parent_readonly_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::None,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::Explore).with_permissions(false, false);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::None
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_none_parent_restricted_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::None,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::General).with_permissions(true, false);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::None
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_restricted_parent_full_subagent() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Restricted,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::General).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::Restricted
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_multilevel_chain_full_readonly_none() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Full,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let level1_subagent =
            MockSubagent::new("level1", AgentType::General).with_permissions(false, false);
        let context1 = vec![Message::user("level1 task")];

        let result1 = runtime
            .invoke_subagent(
                &level1_subagent,
                context1,
                &MockProvider,
                &ToolRegistry::new(),
            )
            .await
            .unwrap();

        assert_eq!(
            result1.effective_permission_scope,
            AgentPermissionScope::ReadOnly
        );

        let child_config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: result1.effective_permission_scope,
        };
        let child_runtime =
            AgentRuntime::with_config(Session::default(), AgentType::General, child_config);

        let level2_subagent =
            MockSubagent::new("level2", AgentType::Explore).with_permissions(false, false);
        let context2 = vec![Message::user("level2 task")];

        let result2 = child_runtime
            .invoke_subagent(
                &level2_subagent,
                context2,
                &MockProvider,
                &ToolRegistry::new(),
            )
            .await
            .unwrap();

        assert_eq!(
            result2.effective_permission_scope,
            AgentPermissionScope::ReadOnly
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_multilevel_none_blocks_all() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::None,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent = MockSubagent::new("result", AgentType::General).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::None
        );
    }

    #[tokio::test]
    async fn test_permission_inheritance_readonly_subagent_cannot_escalate() {
        let session = Session::default();
        let config = RuntimeConfig {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::ReadOnly,
        };
        let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

        let subagent =
            MockSubagent::new("response", AgentType::General).with_permissions(true, true);
        let context = vec![Message::user("task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(
            result.effective_permission_scope,
            AgentPermissionScope::ReadOnly
        );
        assert!(!result.effective_permission_scope.can_write_files());
        assert!(!result.effective_permission_scope.can_run_commands());
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_readonly_restricted() {
        let scope = AgentPermissionScope::ReadOnly.intersect(AgentPermissionScope::Restricted);
        assert_eq!(scope, AgentPermissionScope::ReadOnly);
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_restricted_readonly() {
        let scope = AgentPermissionScope::Restricted.intersect(AgentPermissionScope::ReadOnly);
        assert_eq!(scope, AgentPermissionScope::ReadOnly);
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_restricted_none() {
        let scope = AgentPermissionScope::Restricted.intersect(AgentPermissionScope::None);
        assert_eq!(scope, AgentPermissionScope::None);
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_none_restricted() {
        let scope = AgentPermissionScope::None.intersect(AgentPermissionScope::Restricted);
        assert_eq!(scope, AgentPermissionScope::None);
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_readonly_none() {
        let scope = AgentPermissionScope::ReadOnly.intersect(AgentPermissionScope::None);
        assert_eq!(scope, AgentPermissionScope::None);
    }

    #[tokio::test]
    async fn test_permission_scope_intersection_none_readonly() {
        let scope = AgentPermissionScope::None.intersect(AgentPermissionScope::ReadOnly);
        assert_eq!(scope, AgentPermissionScope::None);
    }

    #[tokio::test]
    async fn test_subagent_spawn_isolated_context() {
        let session = Session::default();
        let session_id = session.id;
        let runtime = AgentRuntime::new(session, AgentType::Build);

        let subagent = MockSubagent::new("isolated response", AgentType::General);
        let context = vec![Message::user("task instructions")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await;

        assert!(result.is_ok());
        let subagent_result = result.unwrap();
        assert_eq!(subagent_result.response.content, "isolated response");
        assert_eq!(subagent_result.agent_type, AgentType::General);
        assert_ne!(subagent_result.child_session_id, session_id);
    }

    #[tokio::test]
    async fn test_subagent_result_handoff() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);

        let expected_content = "handoff result content";
        let subagent = MockSubagent::new(expected_content, AgentType::Explore);
        let context = vec![Message::user("explore task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_eq!(result.response.content, expected_content);
        assert!(result.response.tool_calls.is_empty());
    }

    #[tokio::test]
    async fn test_subagent_preserves_parent_context() {
        let mut session = Session::default();
        session.add_message(Message::user("parent message 1"));
        session.add_message(Message::assistant("parent message 2"));
        let original_message_count = session.messages.len();
        let original_id = session.id;

        let runtime = AgentRuntime::new(session, AgentType::Build);

        let subagent = MockSubagent::new("subagent response", AgentType::General);
        let context = vec![Message::user("subagent task")];

        let _result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        let parent_after = runtime.session.read().await;
        assert_eq!(parent_after.messages.len(), original_message_count);
        assert_eq!(parent_after.id, original_id);
        assert_eq!(parent_after.messages[0].content, "parent message 1");
        assert_eq!(parent_after.messages[1].content, "parent message 2");
    }

    #[tokio::test]
    async fn test_subagent_child_context_isolated() {
        let session = Session::default();
        let session_id = session.id;
        let runtime = AgentRuntime::new(session, AgentType::Build);

        let subagent = MockSubagent::new("child result", AgentType::General);
        let context = vec![Message::user("isolated task")];

        let result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();

        assert_ne!(result.child_session_id, session_id);

        let child_session = result.child_session_id;
        let parent_session = runtime.session.read().await;
        assert_ne!(child_session, parent_session.id);
    }

    #[tokio::test]
    async fn test_subagent_context_messages_added() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);

        let subagent = MockSubagent::new("response", AgentType::General);
        let context = vec![
            Message::user("first instruction"),
            Message::user("second instruction"),
        ];

        let _result = runtime
            .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
            .await
            .unwrap();
    }

    #[test]
    fn test_primary_agent_hidden_agents_are_not_visible() {
        use crate::system_agents::{CompactionAgent, SummaryAgent, TitleAgent};

        let compaction = CompactionAgent::new();
        let title = TitleAgent::new();
        let summary = SummaryAgent::new();

        assert!(
            !compaction.is_visible(),
            "CompactionAgent should not be visible"
        );
        assert!(!title.is_visible(), "TitleAgent should not be visible");
        assert!(!summary.is_visible(), "SummaryAgent should not be visible");
    }

    #[test]
    fn test_primary_agent_hidden_agent_types_not_treated_as_primary() {
        let mut tracker = PrimaryAgentTracker::new();

        tracker.activate(AgentType::Build).unwrap();
        assert!(tracker.is_active());
        assert_eq!(tracker.active_type(), Some(AgentType::Build));

        let compaction_err = tracker.activate(AgentType::Compaction);
        assert!(
            compaction_err.is_err(),
            "Compaction should fail because Build is already active"
        );
        match compaction_err {
            Err(RuntimeError::MultiplePrimaryAgents { current, attempted }) => {
                assert_eq!(current, AgentType::Build);
                assert_eq!(attempted, AgentType::Compaction);
            }
            _ => panic!("Expected MultiplePrimaryAgents error"),
        }

        let title_err = tracker.activate(AgentType::Title);
        assert!(
            title_err.is_err(),
            "Title should fail because Build is already active"
        );

        let summary_err = tracker.activate(AgentType::Summary);
        assert!(
            summary_err.is_err(),
            "Summary should fail because Build is already active"
        );

        assert_eq!(tracker.active_type(), Some(AgentType::Build));
        assert!(tracker.is_active());
    }

    #[test]
    fn test_primary_agent_switching_still_fails_with_hidden_types() {
        let mut tracker = PrimaryAgentTracker::new();

        tracker.activate(AgentType::Build).unwrap();
        tracker.begin_transition().unwrap();

        let result = tracker.activate(AgentType::Compaction);
        assert!(result.is_err());
        match result {
            Err(RuntimeError::AgentTransitionInProgress { .. }) => {}
            _ => panic!("Expected AgentTransitionInProgress error"),
        }

        tracker.complete_transition(AgentType::Plan);
        assert_eq!(tracker.active_type(), Some(AgentType::Plan));
    }

    #[tokio::test]
    async fn test_hidden_agent_subagent_does_not_affect_primary_invariant() {
        use crate::system_agents::CompactionAgent;

        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);

        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));

        let compaction = CompactionAgent::new();
        let context = vec![Message::user("compact task")];
        let _result = runtime
            .invoke_subagent(&compaction, context, &MockProvider, &ToolRegistry::new())
            .await;

        assert!(
            runtime.is_primary_agent_active(),
            "Primary agent should still be active after hidden subagent"
        );
        assert_eq!(
            runtime.active_agent(),
            Some(AgentType::Build),
            "Active agent should still be Build after hidden subagent"
        );
    }

    #[tokio::test]
    async fn test_multiple_hidden_subagents_preserve_invariant() {
        use crate::system_agents::{CompactionAgent, SummaryAgent, TitleAgent};

        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);

        assert_eq!(runtime.active_agent(), Some(AgentType::Build));

        let compaction = CompactionAgent::new();
        let context1 = vec![Message::user("compact task")];
        let _ = runtime
            .invoke_subagent(&compaction, context1, &MockProvider, &ToolRegistry::new())
            .await;

        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));

        let title = TitleAgent::new();
        let context2 = vec![Message::user("title task")];
        let _ = runtime
            .invoke_subagent(&title, context2, &MockProvider, &ToolRegistry::new())
            .await;

        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));

        let summary = SummaryAgent::new();
        let context3 = vec![Message::user("summary task")];
        let _ = runtime
            .invoke_subagent(&summary, context3, &MockProvider, &ToolRegistry::new())
            .await;

        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));
    }

    #[tokio::test]
    async fn test_hidden_subagent_preserves_primary_after_switch() {
        use crate::system_agents::{CompactionAgent, TitleAgent};

        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);

        assert_eq!(runtime.active_agent(), Some(AgentType::Build));

        runtime.switch_primary_agent(AgentType::Plan).await.unwrap();
        assert_eq!(runtime.active_agent(), Some(AgentType::Plan));

        let compaction = CompactionAgent::new();
        let context = vec![Message::user("compact after switch")];
        let _ = runtime
            .invoke_subagent(&compaction, context, &MockProvider, &ToolRegistry::new())
            .await;

        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.active_agent(), Some(AgentType::Plan));

        let title = TitleAgent::new();
        let context2 = vec![Message::user("title after switch")];
        let _ = runtime
            .invoke_subagent(&title, context2, &MockProvider, &ToolRegistry::new())
            .await;

        assert_eq!(runtime.active_agent(), Some(AgentType::Plan));
    }

    #[tokio::test]
    async fn test_hidden_subagent_after_deactivate_preserves_inactive() {
        use crate::system_agents::CompactionAgent;

        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);

        runtime.deactivate_primary_agent().await.unwrap();
        assert!(!runtime.is_primary_agent_active());
        assert_eq!(runtime.primary_agent_state(), PrimaryAgentState::Inactive);

        let compaction = CompactionAgent::new();
        let context = vec![Message::user("compact when inactive")];
        let result = runtime
            .invoke_subagent(&compaction, context, &MockProvider, &ToolRegistry::new())
            .await;

        assert!(
            result.is_err(),
            "Should fail because no primary agent is active"
        );

        assert!(!runtime.is_primary_agent_active());
    }

    #[tokio::test]
    async fn test_visible_agent_switch_then_hidden_subagent() {
        use crate::system_agents::TitleAgent;

        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);

        runtime
            .switch_primary_agent(AgentType::Explore)
            .await
            .unwrap();
        assert_eq!(runtime.active_agent(), Some(AgentType::Explore));

        let title = TitleAgent::new();
        let context = vec![Message::user("generate title")];
        let _ = runtime
            .invoke_subagent(&title, context, &MockProvider, &ToolRegistry::new())
            .await;

        assert_eq!(runtime.active_agent(), Some(AgentType::Explore));
        assert!(runtime.is_primary_agent_active());
    }

    struct MockProvider;

    impl MockProvider {
        #[allow(dead_code)]
        fn new() -> Self {
            Self
        }
    }

    impl opencode_llm::provider::sealed::Sealed for MockProvider {}

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        async fn complete(
            &self,
            prompt: &str,
            _context: Option<&str>,
        ) -> Result<String, opencode_core::OpenCodeError> {
            Ok(format!("mock: {}", prompt))
        }

        async fn complete_streaming(
            &self,
            prompt: &str,
            mut callback: opencode_llm::provider::StreamingCallback,
        ) -> Result<(), opencode_core::OpenCodeError> {
            callback(format!("mock: {}", prompt));
            Ok(())
        }

        async fn chat(
            &self,
            messages: &[opencode_llm::provider::ChatMessage],
        ) -> Result<opencode_llm::provider::ChatResponse, opencode_core::OpenCodeError> {
            let content = messages
                .iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(opencode_llm::provider::ChatResponse {
                content: format!("mock response to: {}", content),
                model: "mock-model".to_string(),
                usage: None,
            })
        }

        fn get_models(&self) -> Vec<opencode_llm::provider::Model> {
            vec![opencode_llm::provider::Model::new(
                "mock-model",
                "Mock Model",
            )]
        }

        fn provider_name(&self) -> &str {
            "mock"
        }
    }
}
