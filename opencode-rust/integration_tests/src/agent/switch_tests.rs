use opencode_agent::{
    AgentRuntime, Agent, AgentType, PrimaryAgentState,
};
use opencode_core::{Message, Session};
use opencode_llm::Provider;
use opencode_tools::ToolRegistry;

struct MockProvider;

impl opencode_llm::provider::sealed::Sealed for MockProvider {}

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

struct MockVisibleAgent {
    agent_type: AgentType,
    response_content: String,
}

impl MockVisibleAgent {
    fn new(agent_type: AgentType, response_content: &str) -> Self {
        Self {
            agent_type,
            response_content: response_content.to_string(),
        }
    }
}

impl opencode_agent::Sealed for MockVisibleAgent {}

#[async_trait::async_trait]
impl Agent for MockVisibleAgent {
    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    fn name(&self) -> &str {
        match self.agent_type {
            AgentType::Build => "build",
            AgentType::Plan => "plan",
            AgentType::General => "general",
            AgentType::Explore => "explore",
            _ => "mock",
        }
    }

    fn description(&self) -> &str {
        "Mock visible agent for testing"
    }

    fn can_execute_tools(&self) -> bool {
        true
    }

    fn can_write_files(&self) -> bool {
        true
    }

    fn can_run_commands(&self) -> bool {
        true
    }

    fn is_visible(&self) -> bool {
        true
    }

    async fn run(
        &self,
        session: &mut Session,
        _provider: &dyn Provider,
        _tools: &ToolRegistry,
    ) -> Result<opencode_agent::AgentResponse, opencode_core::OpenCodeError> {
        session.add_message(opencode_core::Message::assistant(
            self.response_content.clone(),
        ));
        Ok(opencode_agent::AgentResponse {
            content: self.response_content.clone(),
            tool_calls: Vec::new(),
        })
    }
}

#[tokio::test]
async fn test_agent_switch_basic() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    assert_eq!(runtime.active_agent(), Some(AgentType::Build));
    assert!(runtime.is_primary_agent_active());

    runtime.switch_primary_agent(AgentType::Plan).await.unwrap();

    assert_eq!(runtime.active_agent(), Some(AgentType::Plan));
    assert!(runtime.is_primary_agent_active());
}

#[tokio::test]
async fn test_agent_switch_multiple_times() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    assert_eq!(runtime.active_agent(), Some(AgentType::Build));

    runtime.switch_primary_agent(AgentType::Plan).await.unwrap();
    assert_eq!(runtime.active_agent(), Some(AgentType::Plan));

    runtime
        .switch_primary_agent(AgentType::Explore)
        .await
        .unwrap();
    assert_eq!(runtime.active_agent(), Some(AgentType::Explore));

    runtime
        .switch_primary_agent(AgentType::General)
        .await
        .unwrap();
    assert_eq!(runtime.active_agent(), Some(AgentType::General));

    runtime
        .switch_primary_agent(AgentType::Build)
        .await
        .unwrap();
    assert_eq!(runtime.active_agent(), Some(AgentType::Build));
}

#[tokio::test]
async fn test_agent_switch_invariant_always_one_active() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    for agent_type in [
        AgentType::Plan,
        AgentType::Explore,
        AgentType::General,
        AgentType::Review,
        AgentType::Refactor,
    ] {
        runtime.switch_primary_agent(agent_type).await.unwrap();
        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.primary_agent_state(), PrimaryAgentState::Running);
        assert_eq!(runtime.active_agent(), Some(agent_type));
    }
}

#[tokio::test]
async fn test_agent_switch_deactivate_activate() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    assert!(runtime.is_primary_agent_active());

    runtime.deactivate_primary_agent().await.unwrap();
    assert!(!runtime.is_primary_agent_active());
    assert_eq!(runtime.primary_agent_state(), PrimaryAgentState::Inactive);

    runtime
        .activate_primary_agent(AgentType::Plan)
        .await
        .unwrap();
    assert!(runtime.is_primary_agent_active());
    assert_eq!(runtime.active_agent(), Some(AgentType::Plan));
}

#[tokio::test]
async fn test_agent_switch_cannot_activate_when_inactive_no_error() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    runtime.deactivate_primary_agent().await.unwrap();

    let result = runtime.activate_primary_agent(AgentType::Explore).await;
    assert!(result.is_ok());
    assert_eq!(runtime.active_agent(), Some(AgentType::Explore));
}

#[tokio::test]
async fn test_agent_switch_reactivation_after_deactivate() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    runtime.deactivate_primary_agent().await.unwrap();
    assert!(!runtime.is_primary_agent_active());

    runtime
        .activate_primary_agent(AgentType::Review)
        .await
        .unwrap();
    assert!(runtime.is_primary_agent_active());
    assert_eq!(runtime.active_agent(), Some(AgentType::Review));

    runtime
        .switch_primary_agent(AgentType::Refactor)
        .await
        .unwrap();
    assert_eq!(runtime.active_agent(), Some(AgentType::Refactor));
}

#[tokio::test]
async fn test_agent_switch_preserves_session_messages() {
    let mut session = Session::default();
    session.add_message(Message::user("Hello"));
    session.add_message(Message::assistant("Hi there"));

    let runtime = AgentRuntime::new(session, AgentType::Build);

    let session_after = runtime.session().await;
    assert_eq!(session_after.messages.len(), 2);
}

#[tokio::test]
async fn test_agent_switch_invalid_switch_still_maintains_current() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    let original_agent = runtime.active_agent();

    let result = runtime.activate_primary_agent(AgentType::Plan).await;
    assert!(result.is_err());

    assert_eq!(runtime.active_agent(), original_agent);
    assert!(runtime.is_primary_agent_active());
}

#[tokio::test]
async fn test_agent_switch_cycle_maintains_invariant() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    let agent_types = vec![
        AgentType::Build,
        AgentType::Plan,
        AgentType::Explore,
        AgentType::General,
        AgentType::Review,
    ];

    for _ in 0..3 {
        for agent_type in &agent_types {
            runtime.switch_primary_agent(*agent_type).await.unwrap();
            assert!(runtime.is_primary_agent_active());
            assert_eq!(runtime.active_agent(), Some(*agent_type));
        }
    }
}

#[tokio::test]
async fn test_agent_switch_reject_duplicate_activation() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    let result = runtime.activate_primary_agent(AgentType::Build).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        opencode_agent::RuntimeError::MultiplePrimaryAgents { current, attempted } => {
            assert_eq!(current, AgentType::Build);
            assert_eq!(attempted, AgentType::Build);
        }
        _ => panic!("Expected MultiplePrimaryAgents error"),
    }

    assert_eq!(runtime.active_agent(), Some(AgentType::Build));
}

#[tokio::test]
async fn test_session_agent_switch_preserves_id() {
    let mut session = Session::default();
    session.add_message(Message::user("test"));
    let original_id = session.id;

    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    runtime.switch_primary_agent(AgentType::Plan).await.unwrap();

    let session_after = runtime.session().await;
    assert_eq!(session_after.id, original_id);
}
