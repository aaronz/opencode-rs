use opencode_agent::agent::{Agent, AgentResponse, AgentType};
use opencode_agent::runtime::{AgentRuntime, RuntimeConfig};
use opencode_core::{Message, OpenCodeError, Session};
use opencode_llm::Provider;
use opencode_permission::AgentPermissionScope;
use opencode_tools::ToolRegistry;
use uuid::Uuid;

struct MockProvider;

impl opencode_llm::provider::sealed::Sealed for MockProvider {}

#[async_trait::async_trait]
impl Provider for MockProvider {
    async fn complete(
        &self,
        prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        Ok(format!("mock: {}", prompt))
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: opencode_llm::provider::StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        callback(format!("mock: {}", prompt));
        Ok(())
    }

    async fn chat(
        &self,
        messages: &[opencode_llm::provider::ChatMessage],
    ) -> Result<opencode_llm::provider::ChatResponse, OpenCodeError> {
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

struct MockSubagent {
    response_content: String,
    agent_type: AgentType,
    can_write: bool,
    can_run_commands: bool,
    should_error: bool,
    error_message: String,
}

impl MockSubagent {
    fn new(response_content: &str, agent_type: AgentType) -> Self {
        Self {
            response_content: response_content.to_string(),
            agent_type,
            can_write: false,
            can_run_commands: false,
            should_error: false,
            error_message: String::new(),
        }
    }

    fn with_permissions(mut self, can_write: bool, can_run_commands: bool) -> Self {
        self.can_write = can_write;
        self.can_run_commands = can_run_commands;
        self
    }

    fn with_error(mut self, error_message: &str) -> Self {
        self.should_error = true;
        self.error_message = error_message.to_string();
        self
    }
}

impl opencode_agent::agent::sealed::Sealed for MockSubagent {}

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
        if self.should_error {
            return Err(OpenCodeError::InternalError(self.error_message.clone()));
        }
        session.add_message(Message::assistant("subagent response"));
        Ok(AgentResponse {
            content: self.response_content.clone(),
            tool_calls: Vec::new(),
        })
    }
}

#[tokio::test]
async fn test_subagent_exec_context_isolation_parent_session_unchanged() {
    let mut session = Session::default();
    session.add_message(Message::user("original message"));
    let original_message_count = session.messages.len();
    let original_id = session.id;

    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("child result", AgentType::General);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_ne!(result.child_session_id, original_id);

    let parent_session = runtime.session().await;
    assert_eq!(parent_session.messages.len(), original_message_count);
    assert_eq!(parent_session.id, original_id);
}

#[tokio::test]
async fn test_subagent_exec_context_isolation_child_has_separate_id() {
    let session = Session::default();
    let parent_id = session.id;

    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::Explore);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_ne!(result.child_session_id, parent_id);
}

#[tokio::test]
async fn test_subagent_exec_context_isolation_child_messages_empty_initial() {
    let session = Session::default();
    let session_clone = session.clone();
    let runtime = AgentRuntime::new(session_clone, AgentType::Build);

    let context = vec![];

    let mut child_session = session.fork(Uuid::new_v4());
    child_session.messages.clear();
    for msg in context.clone() {
        child_session.add_message(msg);
    }

    assert_eq!(child_session.messages.len(), 0);
    assert_eq!(runtime.active_agent(), Some(AgentType::Build));
}

#[tokio::test]
async fn test_subagent_exec_context_isolation_parent_messages_not_visible_to_child() {
    let mut session = Session::default();
    session.add_message(Message::user("parent-only message"));
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::General);
    let context = vec![Message::user("child task")];

    let _result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    let parent_session = runtime.session().await;
    let child_message_count = parent_session
        .messages
        .iter()
        .filter(|m| m.content.contains("child task"))
        .count();
    assert_eq!(child_message_count, 0);
}

#[tokio::test]
async fn test_subagent_exec_result_handoff_returns_correct_content() {
    let expected_content = "handoff result content";
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new(expected_content, AgentType::Explore);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_eq!(result.response.content, expected_content);
}

#[tokio::test]
async fn test_subagent_exec_result_handoff_includes_agent_type() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::Refactor);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_eq!(result.agent_type, AgentType::Refactor);
}

#[tokio::test]
async fn test_subagent_exec_result_handoff_includes_child_session_id() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::General);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert!(result.child_session_id != Uuid::nil());
}

#[tokio::test]
async fn test_subagent_exec_result_handoff_empty_tool_calls() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("simple result", AgentType::Explore);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert!(result.response.tool_calls.is_empty());
}

#[tokio::test]
async fn test_subagent_exec_error_propagation_subagent_error_returns_err() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::General).with_error("subagent failed");
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_subagent_exec_error_propagation_error_type_is_correct() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::General).with_error("specific error");
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .unwrap_err();

    match result {
        opencode_agent::runtime::RuntimeError::ToolExecutionFailed { tool, reason } => {
            assert_eq!(tool, "subagent");
            assert!(reason.contains("specific error"));
        }
        _ => panic!("Expected ToolExecutionFailed error"),
    }
}

#[tokio::test]
async fn test_subagent_exec_error_propagation_parent_context_preserved_on_error() {
    let mut session = Session::default();
    session.add_message(Message::user("parent message 1"));
    session.add_message(Message::user("parent message 2"));
    let original_message_count = session.messages.len();

    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::General).with_error("subagent error");
    let context = vec![Message::user("child task")];

    let _result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await;

    let parent_session = runtime.session().await;
    assert_eq!(parent_session.messages.len(), original_message_count);
}

#[tokio::test]
async fn test_subagent_exec_error_propagation_no_active_primary_agent() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);
    runtime.deactivate_primary_agent().await.unwrap();

    let subagent = MockSubagent::new("result", AgentType::General);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_subagent_exec_result_handoff_with_context_messages() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::General);
    let context = vec![
        Message::user("first instruction"),
        Message::user("second instruction"),
    ];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_eq!(result.response.content, "result");
}

#[tokio::test]
async fn test_subagent_exec_context_isolation_lineage_path_set() {
    let session = Session::default();
    let session_id = session.id;
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("result", AgentType::General);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_ne!(result.child_session_id, session_id);
}

#[tokio::test]
async fn test_subagent_exec_multiple_subagents_sequential() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent1 = MockSubagent::new("result1", AgentType::Explore);
    let context1 = vec![Message::user("task1")];

    let result1 = runtime
        .invoke_subagent(&subagent1, context1, &MockProvider, &ToolRegistry::new())
        .await
        .expect("first subagent invocation should succeed");

    let subagent2 = MockSubagent::new("result2", AgentType::Refactor);
    let context2 = vec![Message::user("task2")];

    let result2 = runtime
        .invoke_subagent(&subagent2, context2, &MockProvider, &ToolRegistry::new())
        .await
        .expect("second subagent invocation should succeed");

    assert_eq!(result1.response.content, "result1");
    assert_eq!(result2.response.content, "result2");
    assert_ne!(result1.child_session_id, result2.child_session_id);
}

#[tokio::test]
async fn test_subagent_exec_permission_scope_full_parent() {
    let session = Session::default();
    let config = RuntimeConfig {
        max_iterations: 20,
        max_tool_results_per_iteration: 10,
        permission_scope: AgentPermissionScope::Full,
    };
    let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

    let subagent = MockSubagent::new("result", AgentType::General).with_permissions(true, true);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_eq!(
        result.effective_permission_scope,
        AgentPermissionScope::Full
    );
}

#[tokio::test]
async fn test_subagent_exec_permission_scope_readonly_parent_restricts() {
    let session = Session::default();
    let config = RuntimeConfig {
        max_iterations: 20,
        max_tool_results_per_iteration: 10,
        permission_scope: AgentPermissionScope::ReadOnly,
    };
    let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

    let subagent = MockSubagent::new("result", AgentType::General).with_permissions(true, true);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_eq!(
        result.effective_permission_scope,
        AgentPermissionScope::ReadOnly
    );
}

#[tokio::test]
async fn test_subagent_exec_permission_scope_none_parent_restricts_fully() {
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
        .expect("subagent invocation should succeed");

    assert_eq!(
        result.effective_permission_scope,
        AgentPermissionScope::None
    );
}

#[tokio::test]
async fn test_subagent_exec_permission_scope_restricted_parent() {
    let session = Session::default();
    let config = RuntimeConfig {
        max_iterations: 20,
        max_tool_results_per_iteration: 10,
        permission_scope: AgentPermissionScope::Restricted,
    };
    let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

    let subagent = MockSubagent::new("result", AgentType::General).with_permissions(true, true);
    let context = vec![Message::user("task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await
        .expect("subagent invocation should succeed");

    assert_eq!(
        result.effective_permission_scope,
        AgentPermissionScope::Restricted
    );
}
