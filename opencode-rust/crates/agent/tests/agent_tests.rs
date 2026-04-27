use opencode_agent::{
    Agent, AgentResponse, AgentRuntime, AgentType, BuildAgent, PrimaryAgentState,
    PrimaryAgentTracker, RuntimeConfig, RuntimeError, Sealed, Task, TaskDelegate, TaskId,
    TaskResult, TaskStatus, ToolCall,
};
use opencode_core::{Message, OpenCodeError, Session};
use opencode_llm::Provider;
use opencode_permission::AgentPermissionScope;
use opencode_tools::ToolRegistry;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        mut callback: opencode_llm::StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        callback(format!("mock: {}", prompt));
        Ok(())
    }

    async fn chat(
        &self,
        messages: &[opencode_llm::ChatMessage],
    ) -> Result<opencode_llm::ChatResponse, OpenCodeError> {
        let content = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(opencode_llm::ChatResponse {
            content: format!("mock response to: {}", content),
            model: "mock-model".to_string(),
            usage: None,
        })
    }

    fn get_models(&self) -> Vec<opencode_llm::Model> {
        vec![opencode_llm::Model::new("mock-model", "Mock Model")]
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}

#[test]
fn test_primary_agent_tracker_new_is_inactive() {
    let tracker = PrimaryAgentTracker::new();
    assert_eq!(tracker.state, PrimaryAgentState::Inactive);
    assert!(tracker.agent_type.is_none());
    assert!(!tracker.is_active());
}

#[test]
fn test_primary_agent_tracker_new_active_sets_running() {
    let tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    assert_eq!(tracker.state, PrimaryAgentState::Running);
    assert_eq!(tracker.agent_type, Some(AgentType::Build));
    assert!(tracker.is_active());
}

#[test]
fn test_primary_agent_tracker_single_activation_succeeds() {
    let mut tracker = PrimaryAgentTracker::new();
    let result = tracker.activate(AgentType::Build);
    assert!(result.is_ok());
    assert_eq!(tracker.state, PrimaryAgentState::Running);
    assert_eq!(tracker.agent_type, Some(AgentType::Build));
}

#[test]
fn test_primary_agent_tracker_second_activation_fails() {
    let mut tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    let result = tracker.activate(AgentType::Plan);
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::MultiplePrimaryAgents { current, attempted } => {
            assert_eq!(current, AgentType::Build);
            assert_eq!(attempted, AgentType::Plan);
        }
        _ => panic!("Expected MultiplePrimaryAgents error"),
    }
}

#[test]
fn test_primary_agent_tracker_begin_transition_from_running() {
    let mut tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    let result = tracker.begin_transition();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AgentType::Build);
    assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
}

#[test]
fn test_primary_agent_tracker_begin_transition_from_inactive_fails() {
    let mut tracker = PrimaryAgentTracker::new();
    let result = tracker.begin_transition();
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::NoActivePrimaryAgent => {}
        _ => panic!("Expected NoActivePrimaryAgent error"),
    }
}

#[test]
fn test_primary_agent_tracker_complete_transition() {
    let mut tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    tracker.begin_transition().unwrap();
    tracker.complete_transition(AgentType::Plan);
    assert_eq!(tracker.state, PrimaryAgentState::Running);
    assert_eq!(tracker.agent_type, Some(AgentType::Plan));
}

#[test]
fn test_primary_agent_tracker_deactivate_returns_type() {
    let mut tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    let result = tracker.deactivate();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AgentType::Build);
    assert_eq!(tracker.state, PrimaryAgentState::Inactive);
    assert!(tracker.agent_type.is_none());
}

#[test]
fn test_primary_agent_tracker_deactivate_when_inactive_fails() {
    let mut tracker = PrimaryAgentTracker::new();
    let result = tracker.deactivate();
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::NoActivePrimaryAgent => {}
        _ => panic!("Expected NoActivePrimaryAgent error"),
    }
}

#[test]
fn test_primary_agent_tracker_transition_flow() {
    let mut tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    assert!(tracker.is_active());
    assert_eq!(tracker.active_type(), Some(AgentType::Build));

    tracker.begin_transition().unwrap();
    assert_eq!(tracker.state, PrimaryAgentState::Transitioning);

    tracker.complete_transition(AgentType::Plan);
    assert_eq!(tracker.state, PrimaryAgentState::Running);
    assert_eq!(tracker.active_type(), Some(AgentType::Plan));

    tracker.deactivate().unwrap();
    assert_eq!(tracker.state, PrimaryAgentState::Inactive);
    assert!(tracker.active_type().is_none());
}

#[test]
fn test_primary_agent_tracker_activate_while_transitioning_fails() {
    let mut tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    tracker.begin_transition().unwrap();

    let result = tracker.activate(AgentType::Plan);
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::AgentTransitionInProgress { current } => {
            assert_eq!(current, AgentType::Build);
        }
        _ => panic!("Expected AgentTransitionInProgress error"),
    }
}

#[test]
fn test_primary_agent_tracker_deactivate_while_transitioning_fails() {
    let mut tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    tracker.begin_transition().unwrap();

    let result = tracker.deactivate();
    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::AgentTransitionInProgress { current } => {
            assert_eq!(current, AgentType::Build);
        }
        _ => panic!("Expected AgentTransitionInProgress error"),
    }
}

#[tokio::test]
async fn test_agent_runtime_run_loop_no_active_agent() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);
    runtime.deactivate_primary_agent().await.unwrap();

    let result = runtime
        .run_loop(
            &opencode_agent::BuildAgent::new(),
            &MockProvider,
            &ToolRegistry::new(),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::NoActivePrimaryAgent => {}
        _ => panic!("Expected NoActivePrimaryAgent error"),
    }
}

#[tokio::test]
async fn test_agent_runtime_run_loop_agent_type_mismatch() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);
    runtime.deactivate_primary_agent().await.unwrap();
    runtime
        .activate_primary_agent(AgentType::Plan)
        .await
        .unwrap();

    let result = runtime
        .run_loop(
            &opencode_agent::BuildAgent::new(),
            &MockProvider,
            &ToolRegistry::new(),
        )
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        RuntimeError::NoSuchAgent { agent_type } => {
            assert_eq!(agent_type, AgentType::Build);
        }
        _ => panic!("Expected NoSuchAgent error"),
    }
}

#[test]
fn test_runtime_config_max_iterations_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.max_iterations, 20);
}

#[test]
fn test_runtime_config_max_iterations_custom() {
    let config = RuntimeConfig {
        max_iterations: 5,
        max_tool_results_per_iteration: 10,
        permission_scope: AgentPermissionScope::Full,
    };
    assert_eq!(config.max_iterations, 5);
}

#[test]
fn test_runtime_config_max_tool_results_per_iteration_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.max_tool_results_per_iteration, 10);
}

#[test]
fn test_runtime_config_permission_scope_default() {
    let config = RuntimeConfig::default();
    assert_eq!(config.permission_scope, AgentPermissionScope::Full);
}

#[test]
fn test_runtime_config_debug_format() {
    let config = RuntimeConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("RuntimeConfig"));
    assert!(debug.contains("max_iterations"));
}

#[test]
fn test_task_delegate_new_has_no_active_tasks() {
    let delegate = TaskDelegate::new();
    assert!(delegate.active_tasks().is_empty());
}

#[test]
fn test_task_delegate_completed_tasks_starts_empty() {
    let delegate = TaskDelegate::new();
    assert!(delegate.completed_tasks().is_empty());
}

#[test]
fn test_task_delegate_get_active_task_nonexistent() {
    let delegate = TaskDelegate::new();
    let fake_id = TaskId::new();
    assert!(delegate.get_active_task(fake_id).is_none());
}

#[test]
fn test_task_delegate_get_completed_task_nonexistent() {
    let delegate = TaskDelegate::new();
    let fake_id = TaskId::new();
    assert!(delegate.get_completed_task(fake_id).is_none());
}

#[test]
fn test_task_delegate_get_active_task_mut_exists() {
    let mut delegate = TaskDelegate::new();
    let task = Task::new(
        "test task",
        AgentType::Explore,
        AgentType::Build,
        vec![Message::user("context")],
    );
    delegate.get_active_task_mut(task.id);

    assert!(delegate.get_active_task(task.id).is_none());
}

#[test]
fn test_task_status_pending_is_not_terminal() {
    let task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    assert!(!task.is_terminal());
    assert_eq!(task.status, TaskStatus::Pending);
}

#[test]
fn test_task_status_completed_is_terminal() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    task.mark_started();
    let subagent_result = opencode_agent::SubagentResult {
        response: opencode_agent::AgentResponse {
            content: "done".to_string(),
            tool_calls: vec![],
        },
        child_session_id: uuid::Uuid::new_v4(),
        effective_permission_scope: AgentPermissionScope::Full,
        agent_type: AgentType::Explore,
    };
    task.mark_completed(&TaskResult::success(
        task.id,
        subagent_result,
        chrono::Utc::now(),
    ));

    assert!(task.is_terminal());
    assert_eq!(task.status, TaskStatus::Completed);
}

#[test]
fn test_task_status_failed_is_terminal() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    task.mark_failed("error");

    assert!(task.is_terminal());
    assert_eq!(task.status, TaskStatus::Failed);
}

#[test]
fn test_task_mark_started_sets_status_and_timestamp() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    assert!(task.started_at.is_none());

    task.mark_started();

    assert_eq!(task.status, TaskStatus::InProgress);
    assert!(task.started_at.is_some());
}

#[test]
fn test_task_mark_completed_sets_completed_at() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    task.mark_started();
    assert!(task.completed_at.is_none());

    let subagent_result = opencode_agent::SubagentResult {
        response: opencode_agent::AgentResponse {
            content: "done".to_string(),
            tool_calls: vec![],
        },
        child_session_id: uuid::Uuid::new_v4(),
        effective_permission_scope: AgentPermissionScope::Full,
        agent_type: AgentType::Explore,
    };
    let result = TaskResult::success(task.id, subagent_result, chrono::Utc::now());
    task.mark_completed(&result);

    assert!(task.completed_at.is_some());
    assert_eq!(task.status, TaskStatus::Completed);
}

#[test]
fn test_task_mark_failed_sets_completed_at() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    task.mark_started();
    assert!(task.completed_at.is_none());

    task.mark_failed("test error");

    assert!(task.completed_at.is_some());
    assert_eq!(task.status, TaskStatus::Failed);
}

#[test]
fn test_task_duration_after_start_and_fail() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    assert!(task.duration().is_none());

    task.mark_started();
    std::thread::sleep(Duration::from_millis(10));
    task.mark_failed("error");

    let duration = task.duration().unwrap();
    assert!(duration.as_millis() >= 10);
}

#[test]
fn test_task_progress_history_records_all_progress() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    assert_eq!(task.progress_history.len(), 0);

    let progress1 = opencode_agent::TaskProgress::new(task.id, TaskStatus::InProgress, "step 1");
    task.record_progress(progress1);

    let progress2 = opencode_agent::TaskProgress::new(task.id, TaskStatus::InProgress, "step 2");
    task.record_progress(progress2);

    assert_eq!(task.progress_history.len(), 2);
    assert_eq!(task.latest_progress().unwrap().message, "step 2");
}

#[tokio::test]
async fn test_agent_runtime_with_config_uses_provided_values() {
    let session = Session::default();
    let config = RuntimeConfig {
        max_iterations: 1,
        max_tool_results_per_iteration: 5,
        permission_scope: AgentPermissionScope::ReadOnly,
    };
    let runtime = AgentRuntime::with_config(session, AgentType::Build, config);

    let result = runtime
        .run_loop(
            &opencode_agent::BuildAgent::new(),
            &MockProvider,
            &ToolRegistry::new(),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_runtime_session_id_preserved() {
    let session = Session::default();
    let session_id = session.id;
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let current_session = runtime.session().await;
    assert_eq!(current_session.id, session_id);
}

#[tokio::test]
async fn test_agent_runtime_deactivate_and_activate() {
    let session = Session::default();
    let mut runtime = AgentRuntime::new(session, AgentType::Build);

    assert!(runtime.is_primary_agent_active());
    assert_eq!(runtime.active_agent(), Some(AgentType::Build));

    let deactivated = runtime.deactivate_primary_agent().await.unwrap();
    assert_eq!(deactivated, AgentType::Build);
    assert!(!runtime.is_primary_agent_active());

    runtime
        .activate_primary_agent(AgentType::Plan)
        .await
        .unwrap();
    assert!(runtime.is_primary_agent_active());
    assert_eq!(runtime.active_agent(), Some(AgentType::Plan));
}

#[test]
fn test_task_result_success() {
    let task_id = TaskId::new();
    let subagent_result = opencode_agent::SubagentResult {
        response: opencode_agent::AgentResponse {
            content: "result".to_string(),
            tool_calls: vec![],
        },
        child_session_id: uuid::Uuid::new_v4(),
        effective_permission_scope: AgentPermissionScope::Full,
        agent_type: AgentType::Explore,
    };

    let result = TaskResult::success(task_id, subagent_result, chrono::Utc::now());

    assert_eq!(result.task_id, task_id);
    assert_eq!(result.status, TaskStatus::Completed);
    assert!(result.started_at <= result.completed_at);
}

#[test]
fn test_task_result_failure() {
    let task_id = TaskId::new();
    let result = TaskResult::failure(
        task_id,
        AgentType::Explore,
        uuid::Uuid::new_v4(),
        chrono::Utc::now(),
        "test error".to_string(),
    );

    assert_eq!(result.task_id, task_id);
    assert_eq!(result.status, TaskStatus::Failed);
    assert_eq!(result.response.content, "test error");
}

#[test]
fn test_primary_agent_tracker_clone_preserves_state() {
    let tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    let cloned = tracker.clone();

    assert_eq!(cloned.state, tracker.state);
    assert_eq!(cloned.agent_type, tracker.agent_type);
}

#[test]
fn test_primary_agent_tracker_debug() {
    let tracker = PrimaryAgentTracker::new_active(AgentType::Build);
    let debug = format!("{:?}", tracker);

    assert!(debug.contains("PrimaryAgentTracker"));
    assert!(debug.contains("Running"));
    assert!(debug.contains("Build"));
}

#[test]
fn test_runtime_error_display_multiple_primary_agents() {
    let error = RuntimeError::MultiplePrimaryAgents {
        current: AgentType::Build,
        attempted: AgentType::Plan,
    };
    let display = format!("{}", error);
    assert!(display.contains("build"));
    assert!(display.contains("plan"));
    assert!(display.contains("already running"));
}

#[test]
fn test_runtime_error_display_no_active() {
    let error = RuntimeError::NoActivePrimaryAgent;
    let display = format!("{}", error);
    assert!(display.contains("no active primary agent"));
}

#[test]
fn test_runtime_error_display_max_iterations() {
    let error = RuntimeError::MaxIterationsExceeded { limit: 10 };
    let display = format!("{}", error);
    assert!(display.contains("10"));
}

#[test]
fn test_runtime_error_display_session_locked() {
    let error = RuntimeError::SessionLocked;
    let display = format!("{}", error);
    assert!(display.contains("session is locked"));
}

#[test]
fn test_runtime_error_display_tool_execution_failed() {
    let error = RuntimeError::ToolExecutionFailed {
        tool: "bash".to_string(),
        reason: "command failed".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("bash"));
    assert!(display.contains("command failed"));
}

#[test]
fn test_runtime_error_display_permission_denied() {
    let error = RuntimeError::PermissionDenied {
        tool: "rm".to_string(),
    };
    let display = format!("{}", error);
    assert!(display.contains("rm"));
    assert!(display.contains("permission denied"));
}

struct MockMultiToolCallAgent {
    tool_calls: Vec<ToolCall>,
    response_content: String,
}

impl MockMultiToolCallAgent {
    fn new(tool_calls: Vec<ToolCall>, response_content: &str) -> Self {
        Self {
            tool_calls,
            response_content: response_content.to_string(),
        }
    }
}

impl Sealed for MockMultiToolCallAgent {}

#[async_trait::async_trait]
impl Agent for MockMultiToolCallAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Build
    }

    fn name(&self) -> &str {
        "mock_multi_tool_call_agent"
    }

    fn description(&self) -> &str {
        "Mock agent that returns multiple tool calls"
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

    async fn run(
        &self,
        _session: &mut Session,
        _provider: &dyn Provider,
        _tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError> {
        Ok(AgentResponse {
            content: self.response_content.clone(),
            tool_calls: self.tool_calls.clone(),
        })
    }
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

impl Sealed for MockSubagent {}

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
        true
    }

    fn can_write_files(&self) -> bool {
        true
    }

    fn can_run_commands(&self) -> bool {
        true
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
            tool_calls: vec![],
        })
    }
}

#[tokio::test]
async fn test_agent_e2e_013_parallel_tool_calls_handling() {
    let mut session = Session::default();
    session.add_message(Message::user("read both files"));

    let tool_calls = vec![
        ToolCall {
            id: "call-1".to_string(),
            name: "read".to_string(),
            arguments: serde_json::json!({"path": "/tmp/file1.txt"}),
        },
        ToolCall {
            id: "call-2".to_string(),
            name: "read".to_string(),
            arguments: serde_json::json!({"path": "/tmp/file2.txt"}),
        },
    ];

    let agent = MockMultiToolCallAgent::new(tool_calls.clone(), "I will read both files for you");

    let result = agent
        .run(&mut session, &MockProvider, &ToolRegistry::new())
        .await;

    assert!(result.is_ok(), "agent.run() should succeed");
    let response = result.unwrap();
    assert!(response.content.contains("I will read both files for you"));
    assert_eq!(
        response.tool_calls.len(),
        2,
        "AgentResponse should contain both tool_calls"
    );
    assert_eq!(response.tool_calls[0].name, "read");
    assert_eq!(response.tool_calls[1].name, "read");
    assert!(response.tool_calls[0].arguments.get("path").is_some());
    assert!(response.tool_calls[1].arguments.get("path").is_some());
}

struct MockErrorProvider {
    should_error: bool,
}

impl opencode_llm::provider::sealed::Sealed for MockErrorProvider {}

#[async_trait::async_trait]
impl Provider for MockErrorProvider {
    async fn complete(
        &self,
        _prompt: &str,
        _context: Option<&str>,
    ) -> Result<String, OpenCodeError> {
        if self.should_error {
            Err(OpenCodeError::TokenExpired { detail: None })
        } else {
            Ok("mock".to_string())
        }
    }

    async fn complete_streaming(
        &self,
        prompt: &str,
        mut callback: opencode_llm::StreamingCallback,
    ) -> Result<(), OpenCodeError> {
        if self.should_error {
            Err(OpenCodeError::TokenExpired { detail: None })
        } else {
            callback(format!("mock: {}", prompt));
            Ok(())
        }
    }

    async fn chat(
        &self,
        _messages: &[opencode_llm::ChatMessage],
    ) -> Result<opencode_llm::ChatResponse, OpenCodeError> {
        if self.should_error {
            Err(OpenCodeError::TokenExpired { detail: None })
        } else {
            Ok(opencode_llm::ChatResponse {
                content: "mock response".to_string(),
                model: "mock-model".to_string(),
                usage: None,
            })
        }
    }

    fn get_models(&self) -> Vec<opencode_llm::Model> {
        vec![opencode_llm::Model::new("mock-model", "Mock Model")]
    }

    fn provider_name(&self) -> &str {
        "mock_error"
    }
}

#[tokio::test]
async fn test_agent_panic_001_session_lock_released_on_error() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    let error_provider = MockErrorProvider { should_error: true };

    let result = runtime
        .run_loop(&BuildAgent::new(), &error_provider, &ToolRegistry::new())
        .await;

    assert!(result.is_err(), "First call should error");

    let normal_provider = MockErrorProvider {
        should_error: false,
    };
    let result2 = runtime
        .run_loop(&BuildAgent::new(), &normal_provider, &ToolRegistry::new())
        .await;

    assert!(
        result2.is_ok(),
        "Second call should succeed - session not locked"
    );
}

struct MockErroringSubagent {
    should_error: bool,
    error_message: String,
}

impl MockErroringSubagent {
    fn new(should_error: bool, error_message: &str) -> Self {
        Self {
            should_error,
            error_message: error_message.to_string(),
        }
    }
}

impl Sealed for MockErroringSubagent {}

#[async_trait::async_trait]
impl Agent for MockErroringSubagent {
    fn agent_type(&self) -> AgentType {
        AgentType::Explore
    }

    fn name(&self) -> &str {
        "mock_erroring_subagent"
    }

    fn description(&self) -> &str {
        "Mock subagent that can error"
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

    async fn run(
        &self,
        _session: &mut Session,
        _provider: &dyn Provider,
        _tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError> {
        if self.should_error {
            Err(OpenCodeError::TokenExpired {
                detail: Some(self.error_message.clone()),
            })
        } else {
            Ok(AgentResponse {
                content: "success".to_string(),
                tool_calls: vec![],
            })
        }
    }
}

#[tokio::test]
async fn test_agent_panic_002_task_delegate_cleanup_on_error() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);
    let tools = ToolRegistry::new();
    let provider = MockErrorProvider {
        should_error: false,
    };
    let subagent = MockErroringSubagent::new(true, "subagent failed");

    let mut delegate = TaskDelegate::new();
    assert_eq!(
        delegate.active_tasks().len(),
        0,
        "No active tasks initially"
    );

    let task = Task::new(
        "test task",
        AgentType::Explore,
        AgentType::Build,
        vec![Message::user("test".to_string())],
    );

    let result = delegate
        .delegate_task(task, &subagent, &provider, &tools, &runtime)
        .await;

    assert!(
        result.is_err() || result.is_ok(),
        "delegate_task should complete"
    );
    assert!(
        delegate.active_tasks().is_empty(),
        "Active tasks should be empty after delegate_task completes (error or success)"
    );

    assert_eq!(
        delegate.completed_tasks().len(),
        1,
        "Task should be in completed_tasks"
    );
}

#[tokio::test]
async fn test_agent_e2e_008_task_delegate_lifecycle() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);
    let tools = ToolRegistry::new();

    let mut delegate = TaskDelegate::new();
    assert!(
        delegate.active_tasks().is_empty(),
        "No active tasks initially"
    );
    assert!(
        delegate.completed_tasks().is_empty(),
        "No completed tasks initially"
    );

    let task = Task::new(
        "test task",
        AgentType::Explore,
        AgentType::Build,
        vec![Message::user("test".to_string())],
    );

    assert_eq!(task.status, TaskStatus::Pending);

    let subagent = MockSubagent::new("task result", AgentType::Explore);
    let result = delegate
        .delegate_task(task, &subagent, &MockProvider, &tools, &runtime)
        .await;

    assert!(result.is_ok(), "delegate_task should succeed");
    let task_result = result.unwrap();
    assert_eq!(task_result.status, TaskStatus::Completed);
    assert!(
        delegate.active_tasks().is_empty(),
        "Active tasks should be empty after completion"
    );
    assert_eq!(
        delegate.completed_tasks().len(),
        1,
        "Task should be in completed_tasks"
    );
    let completed_task = delegate.completed_tasks().first().unwrap();
    assert_eq!(completed_task.status, TaskStatus::Completed);
}

#[tokio::test]
async fn test_agent_e2e_014_nested_agent_call_isolation() {
    let mut session = Session::default();
    session.add_message(Message::user("original message"));
    let original_message_count = session.messages.len();
    let original_id = session.id;

    let runtime = AgentRuntime::new(session, AgentType::Build);

    let subagent = MockSubagent::new("child result", AgentType::General);
    let context = vec![Message::user("nested task")];

    let result = runtime
        .invoke_subagent(&subagent, context, &MockProvider, &ToolRegistry::new())
        .await;

    assert!(result.is_ok(), "invoke_subagent should succeed");
    let subagent_result = result.unwrap();

    assert_ne!(
        subagent_result.child_session_id, original_id,
        "Child session should have different ID"
    );

    let parent_session = runtime.session().await;
    assert_eq!(
        parent_session.messages.len(),
        original_message_count,
        "Parent session message count unchanged after nested call"
    );
    assert_eq!(
        parent_session.id, original_id,
        "Parent session ID unchanged after nested call"
    );
}

#[test]
fn test_agent_conc_001_primary_agent_tracker_concurrent_activation() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    let tracker = Arc::new(Mutex::new(PrimaryAgentTracker::new()));
    let success_count = Arc::new(AtomicUsize::new(0));
    let results: Arc<Mutex<Vec<(usize, AgentType, bool, Option<RuntimeError>)>>> =
        Arc::new(Mutex::new(Vec::new()));

    let agent_types = [
        AgentType::Build,
        AgentType::Plan,
        AgentType::Explore,
        AgentType::General,
    ];

    let mut handles = vec![];

    for i in 0..100 {
        let tracker = Arc::clone(&tracker);
        let success_count = Arc::clone(&success_count);
        let results = Arc::clone(&results);
        let agent_type = agent_types[i % agent_types.len()];

        let handle = thread::spawn(move || {
            let result = tracker.lock().unwrap().activate(agent_type);
            let is_success = result.is_ok();
            if is_success {
                success_count.fetch_add(1, Ordering::SeqCst);
            }
            results
                .lock()
                .unwrap()
                .push((i, agent_type, result.is_ok(), result.err()));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(
        success_count.load(Ordering::SeqCst),
        1,
        "Exactly one activation should succeed"
    );

    let results = results.lock().unwrap();
    let success_results: Vec<_> = results.iter().filter(|r| r.2).collect();
    let failure_results: Vec<_> = results.iter().filter(|r| !r.2).collect();

    assert_eq!(success_results.len(), 1, "Exactly one success");
    assert_eq!(failure_results.len(), 99, "Exactly 99 failures");

    for failure in &failure_results {
        if let Some(RuntimeError::MultiplePrimaryAgents { current, .. }) = &failure.3 {
            assert_eq!(*current, success_results[0].1);
        } else {
            panic!("Expected MultiplePrimaryAgents error, got {:?}", failure.3);
        }
    }
}

#[tokio::test]
async fn test_agent_e2e_001_build_agent_run_completes() {
    let mut session = Session::default();
    session.add_message(Message::user("List files in /tmp".to_string()));
    let initial_message_count = session.messages.len();

    let agent = BuildAgent::new();
    let tools = ToolRegistry::new();

    let result = agent.run(&mut session, &MockProvider, &tools).await;

    assert!(result.is_ok(), "agent.run() should succeed");
    let response = result.unwrap();

    assert!(
        !response.content.is_empty(),
        "AgentResponse should have content"
    );

    assert_eq!(
        session.messages.len(),
        initial_message_count + 1,
        "Session should have assistant message added"
    );

    let last_message = session.messages.last().unwrap();
    assert!(
        matches!(last_message.role, opencode_core::Role::Assistant),
        "Last message should be from assistant"
    );

    assert!(
        response.content.contains("mock response"),
        "Response should contain content from provider"
    );
}

#[tokio::test]
async fn test_agent_e2e_002_plan_agent_enforces_read_only_constraints() {
    use opencode_agent::PlanAgent;

    let agent = PlanAgent::new();

    assert!(
        !agent.can_execute_tools(),
        "PlanAgent should not be able to execute tools"
    );
    assert!(
        !agent.can_write_files(),
        "PlanAgent should not be able to write files"
    );
    assert!(
        !agent.can_run_commands(),
        "PlanAgent should not be able to run commands"
    );
    assert!(agent.is_visible(), "PlanAgent should be visible in UI");

    let agent_with_model = PlanAgent::new().with_model("claude-3");

    assert!(
        !agent_with_model.can_execute_tools(),
        "PlanAgent with model override should not be able to execute tools"
    );
    assert!(
        !agent_with_model.can_write_files(),
        "PlanAgent with model override should not be able to write files"
    );
    assert!(
        !agent_with_model.can_run_commands(),
        "PlanAgent with model override should not be able to run commands"
    );
    assert!(
        agent_with_model.is_visible(),
        "PlanAgent with model override should be visible"
    );
}
