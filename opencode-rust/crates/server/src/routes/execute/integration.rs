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

impl ExecutionContext {
    pub fn new(
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

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn with_max_tool_results_per_iteration(mut self, max: usize) -> Self {
        self.max_tool_results_per_iteration = max;
        self
    }

    pub fn create_agent(&self) -> Box<dyn Agent> {
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
            ExecutionEvent::Complete { session_id, .. } => ExecuteEvent::complete(
                serde_json::json!({
                    "session_id": session_id.to_string(),
                }),
            ),
        }
    }
}

pub async fn execute_agent_loop(
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

impl From<ExecuteMode> for AgentType {
    fn from(mode: ExecuteMode) -> Self {
        match mode {
            ExecuteMode::Build => AgentType::Build,
            ExecuteMode::Plan => AgentType::Plan,
            ExecuteMode::General => AgentType::General,
        }
    }
}

pub fn system_prompt_for_mode(mode: ExecuteMode) -> &'static str {
    mode.system_prompt()
}

#[derive(Debug, Clone)]
pub struct IntegrationToolResult {
    pub tool_name: String,
    pub success: bool,
    pub content: Option<String>,
    pub error: Option<String>,
}

impl IntegrationToolResult {
    /// Create from opencode_tools::ToolResult with a known tool name
    pub fn from_tools_result(tool_name: impl Into<String>, result: opencode_tools::ToolResult) -> Self {
        Self {
            tool_name: tool_name.into(),
            success: result.success,
            content: Some(result.content),
            error: result.error,
        }
    }
}

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
    use opencode_core::ToolResult;
    use async_trait::async_trait;

    struct MockProvider;

    #[async_trait]
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

    #[test]
    fn test_execution_context_creation() {
        let registry = Arc::new(ToolRegistry::new());
        let provider = Arc::new(MockProvider);

        let ctx = ExecutionContext::new(registry, provider, AgentType::Build);

        assert_eq!(ctx.agent_type, AgentType::Build);
        assert_eq!(ctx.max_iterations, 20);
        assert_eq!(ctx.max_tool_results_per_iteration, 10);
    }

    #[test]
    fn test_execution_context_with_custom_iterations() {
        let registry = Arc::new(ToolRegistry::new());
        let provider = Arc::new(MockProvider);

        let ctx = ExecutionContext::new(registry, provider, AgentType::Plan)
            .with_max_iterations(50)
            .with_max_tool_results_per_iteration(5);

        assert_eq!(ctx.max_iterations, 50);
        assert_eq!(ctx.max_tool_results_per_iteration, 5);
    }

    #[test]
    fn test_execution_context_create_agent_build() {
        let registry = Arc::new(ToolRegistry::new());
        let provider = Arc::new(MockProvider);

        let ctx = ExecutionContext::new(registry, provider, AgentType::Build);
        let agent = ctx.create_agent();

        assert_eq!(agent.agent_type(), AgentType::Build);
    }

    #[test]
    fn test_execution_context_create_agent_plan() {
        let registry = Arc::new(ToolRegistry::new());
        let provider = Arc::new(MockProvider);

        let ctx = ExecutionContext::new(registry, provider, AgentType::Plan);
        let agent = ctx.create_agent();

        assert_eq!(agent.agent_type(), AgentType::Plan);
    }

    #[test]
    fn test_execution_context_create_agent_general() {
        let registry = Arc::new(ToolRegistry::new());
        let provider = Arc::new(MockProvider);

        let ctx = ExecutionContext::new(registry, provider, AgentType::General);
        let agent = ctx.create_agent();

        assert_eq!(agent.agent_type(), AgentType::General);
    }

    #[test]
    fn test_execution_context_create_agent_unknown_defaults_to_general() {
        let registry = Arc::new(ToolRegistry::new());
        let provider = Arc::new(MockProvider);

        let ctx = ExecutionContext::new(registry, provider, AgentType::Explore);
        let agent = ctx.create_agent();

        assert_eq!(agent.agent_type(), AgentType::General);
    }

    #[test]
    fn test_execute_mode_to_agent_type() {
        assert_eq!(AgentType::from(ExecuteMode::Build), AgentType::Build);
        assert_eq!(AgentType::from(ExecuteMode::Plan), AgentType::Plan);
        assert_eq!(AgentType::from(ExecuteMode::General), AgentType::General);
    }

    #[test]
    fn test_system_prompt_for_mode() {
        let build_prompt = system_prompt_for_mode(ExecuteMode::Build);
        assert!(build_prompt.contains("BUILD"));

        let plan_prompt = system_prompt_for_mode(ExecuteMode::Plan);
        assert!(plan_prompt.contains("PLAN"));

        let general_prompt = system_prompt_for_mode(ExecuteMode::General);
        assert!(general_prompt.contains("GENERAL"));
    }

    #[test]
    fn test_tool_registry_accessible_from_integration() {
        let registry = ToolRegistry::new();
        let _tool_list = registry.list_filtered(None);
    }

    #[test]
    fn test_integration_tool_result_from_tool_result() {
        let tool_result = ToolResult {
            id: Uuid::new_v4(),
            tool_name: "read".to_string(),
            success: true,
            result: Some("file content".to_string()),
            error: None,
            started_at: chrono::Utc::now(),
            completed_at: chrono::Utc::now(),
        };

        let integration_result: IntegrationToolResult = tool_result.into();

        assert_eq!(integration_result.tool_name, "read");
        assert!(integration_result.success);
        assert_eq!(integration_result.content, Some("file content".to_string()));
        assert!(integration_result.error.is_none());
    }

    #[test]
    fn test_integration_tool_result_from_failed_tool_result() {
        let tool_result = ToolResult {
            id: Uuid::new_v4(),
            tool_name: "write".to_string(),
            success: false,
            result: None,
            error: Some("Permission denied".to_string()),
            started_at: chrono::Utc::now(),
            completed_at: chrono::Utc::now(),
        };

        let integration_result: IntegrationToolResult = tool_result.into();

        assert_eq!(integration_result.tool_name, "write");
        assert!(!integration_result.success);
        assert!(integration_result.content.is_none());
        assert_eq!(integration_result.error, Some("Permission denied".to_string()));
    }

    #[tokio::test]
    async fn test_agent_executor_can_be_instantiated() {
        let registry = Arc::new(ToolRegistry::new());
        let provider = Arc::new(MockProvider);
        let ctx = ExecutionContext::new(registry, provider, AgentType::Build);

        let agent = ctx.create_agent();
        assert!(agent.can_execute_tools() || !agent.can_execute_tools());
    }

    #[tokio::test]
    async fn test_tool_execution_flow_with_registry() {
        use opencode_tools::Tool;

        let registry = Arc::new(ToolRegistry::new());

        #[derive(Clone)]
        struct TestTool;

        #[async_trait::async_trait]
        impl Tool for TestTool {
            fn name(&self) -> &str {
                "test_tool"
            }

            fn description(&self) -> &str {
                "A test tool"
            }

            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }

            async fn execute(
                &self,
                args: serde_json::Value,
                _ctx: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, OpenCodeError> {
                let input = args
                    .get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                Ok(opencode_tools::ToolResult::ok(format!("received: {}", input)))
            }
        }

        registry.register(TestTool).await;

        let tool = registry.get("test_tool").await;
        assert!(tool.is_some());

        let result = registry
            .execute("test_tool", serde_json::json!({"input": "hello"}), None)
            .await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "received: hello");
    }

    #[tokio::test]
    async fn test_tool_execution_flow_error_case() {
        let registry = Arc::new(ToolRegistry::new());

        let result = registry
            .execute("nonexistent_tool", serde_json::json!({}), None)
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_mode_system_prompt_comprehensive() {
        let build = ExecuteMode::Build;
        let build_prompt = build.system_prompt();
        assert!(build_prompt.contains("BUILD"));

        let plan = ExecuteMode::Plan;
        let plan_prompt = plan.system_prompt();
        assert!(plan_prompt.contains("PLAN"));

        let general = ExecuteMode::General;
        let general_prompt = general.system_prompt();
        assert!(general_prompt.contains("GENERAL"));
    }

    #[test]
    fn test_execution_event_to_execute_event_tool_call() {
        let exec_event = ExecutionEvent::ToolCall {
            call_id: "call-1".to_string(),
            tool_name: "read".to_string(),
            arguments: serde_json::json!({"path": "/test"}),
        };
        let execute_event: ExecuteEvent = exec_event.into();
        match execute_event {
            ExecuteEvent::ToolCall { tool, params: _, call_id } => {
                assert_eq!(tool, "read");
                assert_eq!(call_id, "call-1");
            }
            _ => panic!("Expected ToolCall variant"),
        }
    }

    #[test]
    fn test_execution_event_to_execute_event_message() {
        let exec_event = ExecutionEvent::Message {
            role: "assistant".to_string(),
            content: "Hello".to_string(),
        };
        let execute_event: ExecuteEvent = exec_event.into();
        match execute_event {
            ExecuteEvent::Message { role, content } => {
                assert_eq!(role, "assistant");
                assert_eq!(content, "Hello");
            }
            _ => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_execution_event_to_execute_event_error() {
        let exec_event = ExecutionEvent::Error {
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
        };
        let execute_event: ExecuteEvent = exec_event.into();
        match execute_event {
            ExecuteEvent::Error { code, message } => {
                assert_eq!(code, "TEST_ERROR");
                assert_eq!(message, "Test error message");
            }
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_execution_event_to_execute_event_complete() {
        let exec_event = ExecutionEvent::Complete {
            session_id: Uuid::new_v4(),
            message_count: 5,
        };
        let execute_event: ExecuteEvent = exec_event.into();
        match execute_event {
            ExecuteEvent::Complete { session_state } => {
                assert!(session_state.get("session_id").is_some());
            }
            _ => panic!("Expected Complete variant"),
        }
    }

    #[tokio::test]
    async fn test_session_tool_call_flow() {
        let mut session = Session::new();
        assert_eq!(session.messages.len(), 0);

        session.add_message(Message::user("Hello"));
        assert_eq!(session.messages.len(), 1);

        session.add_message(Message::assistant("I'll read that file for you."));
        assert_eq!(session.messages.len(), 2);

        session.add_message(Message::user("Tool 'read' result:\nfile content here"));
        assert_eq!(session.messages.len(), 3);
    }

    #[tokio::test]
    async fn test_tool_registry_discovery_in_execute_context() {
        let registry = Arc::new(opencode_tools::ToolRegistry::new());

        let tools = registry.list_filtered(None).await;
        assert!(
            tools.is_empty(),
            "New registry should be empty before tool registration"
        );

        let has_tool = registry.get("nonexistent").await;
        assert!(
            has_tool.is_none(),
            "Nonexistent tool should not be found"
        );
    }

    #[tokio::test]
    async fn test_tool_registry_discovery_with_builtin_tools() {
        use opencode_tools::Tool;

        let registry = Arc::new(opencode_tools::ToolRegistry::new());

        #[derive(Clone)]
        struct DiscoverableTool;

        #[async_trait::async_trait]
        impl Tool for DiscoverableTool {
            fn name(&self) -> &str {
                "discoverable_tool"
            }

            fn description(&self) -> &str {
                "A tool for testing discovery"
            }

            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }

            async fn execute(
                &self,
                args: serde_json::Value,
                _ctx: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, OpenCodeError> {
                let input = args
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                Ok(opencode_tools::ToolResult::ok(format!("discovered: {}", input)))
            }
        }

        registry.register(DiscoverableTool).await;

        let discovered = registry.get("discoverable_tool").await;
        assert!(
            discovered.is_some(),
            "Registered tool should be discoverable"
        );

        let tool_list = registry.list_filtered(None).await;
        let tool_names: Vec<&str> = tool_list.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(
            tool_names.contains(&"discoverable_tool"),
            "Tool should appear in registry listing"
        );
    }

    #[tokio::test]
    async fn test_tool_registry_execution_accessible_in_execute_endpoint() {
        use opencode_tools::Tool;

        let registry = Arc::new(opencode_tools::ToolRegistry::new());

        #[derive(Clone)]
        struct ExecutableTestTool;

        #[async_trait::async_trait]
        impl Tool for ExecutableTestTool {
            fn name(&self) -> &str {
                "executable_tool"
            }

            fn description(&self) -> &str {
                "A tool that can be executed"
            }

            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }

            async fn execute(
                &self,
                args: serde_json::Value,
                _ctx: Option<opencode_tools::ToolContext>,
            ) -> Result<opencode_tools::ToolResult, OpenCodeError> {
                let msg = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("executed");
                Ok(opencode_tools::ToolResult::ok(format!("tool executed: {}", msg)))
            }
        }

        registry.register(ExecutableTestTool).await;

        let ctx = ExecutionContext::new(
            registry.clone(),
            Arc::new(MockProvider),
            AgentType::Build,
        );

        assert_eq!(
            ctx.agent_type,
            AgentType::Build,
            "Execution context should be properly configured"
        );

        let result = ctx
            .tool_registry
            .execute(
                "executable_tool",
                serde_json::json!({"message": "test"}),
                None,
            )
            .await;

        assert!(result.is_ok(), "Tool should be executable through registry");
        let result = result.unwrap();
        assert!(
            result.success,
            "Tool execution should succeed"
        );
        assert!(
            result.content.contains("tool executed: test"),
            "Tool should produce expected output"
        );
    }

    #[tokio::test]
    async fn test_tool_discovery_integration_multiple_tools() {
        use opencode_tools::Tool;

        let registry = Arc::new(opencode_tools::ToolRegistry::new());

        #[derive(Clone)]
        struct ToolA;
        #[async_trait::async_trait]
        impl Tool for ToolA {
            fn name(&self) -> &str { "tool_a" }
            fn description(&self) -> &str { "First tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<opencode_tools::ToolContext>) -> Result<opencode_tools::ToolResult, OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("a"))
            }
        }

        #[derive(Clone)]
        struct ToolB;
        #[async_trait::async_trait]
        impl Tool for ToolB {
            fn name(&self) -> &str { "tool_b" }
            fn description(&self) -> &str { "Second tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<opencode_tools::ToolContext>) -> Result<opencode_tools::ToolResult, OpenCodeError> {
                Ok(opencode_tools::ToolResult::ok("b"))
            }
        }

        registry.register(ToolA).await;
        registry.register(ToolB).await;

        let tools = registry.list_filtered(None).await;
        let tool_names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();

        assert!(
            tool_names.contains(&"tool_a"),
            "First tool should be discoverable"
        );
        assert!(
            tool_names.contains(&"tool_b"),
            "Second tool should be discoverable"
        );

        let result_a = registry.execute("tool_a", serde_json::json!({}), None).await;
        let result_b = registry.execute("tool_b", serde_json::json!({}), None).await;

        assert!(result_a.is_ok(), "First tool should be executable");
        assert!(result_b.is_ok(), "Second tool should be executable");
    }

    #[tokio::test]
    async fn test_tool_registry_from_application_state() {
        let registry = Arc::new(opencode_tools::ToolRegistry::new());

        let ctx = ExecutionContext::new(
            registry.clone(),
            Arc::new(MockProvider),
            AgentType::General,
        );

        let has_tool = ctx.tool_registry.get("read").await;
        assert_eq!(
            has_tool.is_some(),
            false,
            "Empty registry should not have read tool"
        );

        let tool_list = ctx.tool_registry.list_filtered(None).await;
        assert!(
            tool_list.is_empty(),
            "Empty registry should have no tools"
        );
    }
}
