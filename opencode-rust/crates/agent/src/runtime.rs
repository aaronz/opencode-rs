use std::sync::Arc;

use opencode_core::{Message, Session};
use opencode_llm::Provider;
use opencode_tools::registry::ToolCall as ToolsToolCall;
use opencode_tools::ToolRegistry;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{Agent, AgentResponse, AgentType};

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
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
        }
    }
}

/// Errors that can occur during runtime operations.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    SessionNotActive,
    MaxIterationsExceeded { limit: usize },
    NoSuchAgent { agent_type: AgentType },
    ToolExecutionFailed { tool: String, reason: String },
    PermissionDenied { tool: String },
    SessionLocked,
    /// Invariant violation: attempted to activate a second primary agent
    /// while one is already running.
    MultiplePrimaryAgents {
        current: AgentType,
        attempted: AgentType,
    },
    /// Invariant violation: attempted to deactivate or switch while transitioning.
    AgentTransitionInProgress { current: AgentType },
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

/// Tracks the primary agent and its state.
/// The invariant is: exactly one primary agent is active at any given time.
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

    /// Activate a new primary agent. Returns error if one is already running.
    pub fn activate(&mut self, agent_type: AgentType) -> Result<(), RuntimeError> {
        match &self.state {
            PrimaryAgentState::Inactive => {
                self.state = PrimaryAgentState::Running;
                self.agent_type = Some(agent_type);
                Ok(())
            }
            PrimaryAgentState::Running => {
                Err(RuntimeError::MultiplePrimaryAgents {
                    current: self.agent_type.unwrap(),
                    attempted: agent_type,
                })
            }
            PrimaryAgentState::Transitioning => {
                Err(RuntimeError::AgentTransitionInProgress {
                    current: self.agent_type.unwrap(),
                })
            }
        }
    }

    /// Begin transitioning to a new primary agent.
    /// This puts the runtime in Transitioning state before the switch.
    pub fn begin_transition(&mut self) -> Result<AgentType, RuntimeError> {
        match &self.state {
            PrimaryAgentState::Inactive => Err(RuntimeError::NoActivePrimaryAgent),
            PrimaryAgentState::Running => {
                let current = self.agent_type.unwrap();
                self.state = PrimaryAgentState::Transitioning;
                Ok(current)
            }
            PrimaryAgentState::Transitioning => {
                Err(RuntimeError::AgentTransitionInProgress {
                    current: self.agent_type.unwrap(),
                })
            }
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
                let current = self.agent_type.unwrap();
                self.state = PrimaryAgentState::Inactive;
                self.agent_type = None;
                Ok(current)
            }
            PrimaryAgentState::Transitioning => {
                Err(RuntimeError::AgentTransitionInProgress {
                    current: self.agent_type.unwrap(),
                })
            }
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
}

impl AgentRuntime {
    pub fn new(session: Session, agent_type: AgentType) -> Self {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(agent_type).expect(
            "Failed to activate initial agent - this indicates a programming error",
        );
        Self {
            session: Arc::new(RwLock::new(session)),
            config: RuntimeConfig::default(),
            primary_tracker: tracker,
        }
    }

    pub fn with_config(session: Session, agent_type: AgentType, config: RuntimeConfig) -> Self {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(agent_type).expect(
            "Failed to activate initial agent - this indicates a programming error",
        );
        Self {
            session: Arc::new(RwLock::new(session)),
            config,
            primary_tracker: tracker,
        }
    }

    pub async fn run_loop<A: Agent>(
        &self,
        agent: &A,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, RuntimeError> {
        if !self.primary_tracker.is_active() {
            return Err(RuntimeError::NoActivePrimaryAgent);
        }
        if self.primary_tracker.active_type() != Some(agent.agent_type()) {
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

    pub async fn activate_primary_agent(&mut self, new_type: AgentType) -> Result<(), RuntimeError> {
        self.primary_tracker.activate(new_type)
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

        Ok(SubagentResult {
            response: subagent_response,
            child_session_id,
            agent_type: agent.agent_type(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencode_core::OpenCodeError;

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
        runtime.activate_primary_agent(AgentType::Plan).await.unwrap();
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
    }

    impl MockSubagent {
        fn new(response_content: &str, agent_type: AgentType) -> Self {
            Self {
                response_content: response_content.to_string(),
                agent_type,
            }
        }
    }

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
            false
        }

        fn can_write_files(&self) -> bool {
            false
        }

        fn can_run_commands(&self) -> bool {
            false
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

    struct MockProvider;

    impl MockProvider {
        fn new() -> Self {
            Self
        }
    }

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
