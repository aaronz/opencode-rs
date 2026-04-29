use std::sync::Arc;

use opencode_permission::PermissionScope;
use tokio::sync::RwLock;

use opencode_agent::{AgentRuntime, AgentType};
use opencode_core::{
    bus::EventBus,
    bus::InternalEvent,
    context::{Context, ContextBudget, ContextItem, ContextLayer, LayerBudgets},
    permission::PermissionManager,
    Session,
};
use opencode_runtime::{
    Runtime, RuntimeCommand, RuntimeContextSummary, RuntimeEvent, RuntimeFacadeError,
    RuntimePermissionAdapter, RuntimePermissionDecision, RuntimeServices, RuntimeSessionStore,
    RuntimeStatus, RuntimeTaskStore, RuntimeToolRouter, SubmitUserInput, TaskControlCommand,
};
use opencode_storage::{
    InMemoryProjectRepository, InMemorySessionRepository, StoragePool, StorageService,
};

fn build_runtime() -> (Runtime, Arc<StorageService>) {
    let event_bus = Arc::new(EventBus::new());
    let permission_manager = Arc::new(RwLock::new(PermissionManager::default()));
    let session_repo = Arc::new(InMemorySessionRepository::default());
    let project_repo = Arc::new(InMemoryProjectRepository::default());
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("runtime-smoke.db");
    let pool = StoragePool::new(&db_path).expect("storage pool");
    let storage = Arc::new(StorageService::new(session_repo, project_repo, pool));
    let agent_runtime = Arc::new(RwLock::new(AgentRuntime::new(
        Session::default(),
        AgentType::Build,
    )));

    std::mem::forget(temp_dir);

    (
        Runtime::new(RuntimeServices::new(
            event_bus,
            permission_manager,
            storage.clone(),
            agent_runtime,
            Arc::new(RuntimeTaskStore::new()),
            Arc::new(RuntimeToolRouter::new(opencode_tools::ToolRegistry::new())),
        )),
        storage,
    )
}

#[tokio::test]
async fn runtime_constructs_and_reports_idle_status() {
    let (runtime, _) = build_runtime();
    assert_eq!(runtime.status().await, RuntimeStatus::Idle);
}

#[tokio::test]
async fn runtime_accepts_submit_command_shape() {
    let (runtime, _) = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::SubmitUserInput(SubmitUserInput {
            session_id: None,
            input: "hello".to_string(),
        }))
        .await
        .expect("submit command should create a turn");

    assert!(result.accepted);
    assert!(result.turn_id.is_some());
    assert_eq!(result.message, "turn created");
}

#[tokio::test]
async fn runtime_task_control_cancel_nonexistent_returns_error() {
    let (runtime, _) = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::TaskControl(TaskControlCommand::Cancel {
            task_id: "00000000-0000-0000-0000-000000000001".to_string(),
        }))
        .await;

    assert!(matches!(result, Err(RuntimeFacadeError::Dependency(_))));
}

#[test]
fn runtime_event_converts_from_internal_message_added() {
    let event = InternalEvent::MessageAdded {
        session_id: "session-1".to_string(),
        message_id: "message-1".to_string(),
    };

    let converted = RuntimeEvent::from_internal_event(&event).expect("runtime event expected");

    assert!(matches!(
        converted,
        RuntimeEvent::MessageAdded {
            session_id,
            message_id,
        } if session_id == "session-1" && message_id == "message-1"
    ));
}

#[test]
fn runtime_event_ignores_unhandled_internal_variants() {
    let event = InternalEvent::AgentStarted {
        session_id: "session-1".to_string(),
        agent: "build".to_string(),
    };

    assert!(RuntimeEvent::from_internal_event(&event).is_none());
}

#[test]
fn runtime_permission_adapter_allows_granted_permissions() {
    let adapter = RuntimePermissionAdapter::default();

    let decision = adapter.check(
        opencode_core::permission::Permission::FileRead,
        "src/lib.rs",
    );

    assert_eq!(decision, RuntimePermissionDecision::Allow);
}

#[test]
fn runtime_permission_adapter_denies_blocked_patterns() {
    use opencode_core::permission::{Permission, PermissionConfig, PermissionManager};

    let mut config = PermissionConfig::default();
    config.always_denied.push(".env".to_string());
    let adapter = RuntimePermissionAdapter::new(
        Arc::new(RwLock::new(PermissionManager::new(config))),
        Arc::new(RwLock::new(opencode_permission::ApprovalQueue::new(
            PermissionScope::default(),
        ))),
        None,
    );

    let decision = adapter.check(Permission::FileRead, ".env");

    assert_eq!(decision, RuntimePermissionDecision::Deny);
}

#[tokio::test]
async fn runtime_session_store_round_trips_session() {
    let (_, storage) = build_runtime();
    let store = RuntimeSessionStore::new(storage.clone());
    let session = Session::default();
    let session_id = session.id.to_string();

    store
        .save_session(&session)
        .await
        .expect("session should save through runtime store");

    let loaded = store
        .load_session(&session_id)
        .await
        .expect("session load should succeed")
        .expect("session should exist");

    assert_eq!(loaded.id.to_string(), session_id);
}

#[tokio::test]
async fn runtime_session_store_returns_none_for_missing_session() {
    let (_, storage) = build_runtime();
    let store = RuntimeSessionStore::new(storage);

    let loaded = store
        .load_session("00000000-0000-0000-0000-000000000000")
        .await
        .expect("missing session lookup should succeed");

    assert!(loaded.is_none());
}

#[test]
fn runtime_context_summary_reports_budget_and_counts() {
    let context = Context {
        layers: vec![ContextItem {
            layer: ContextLayer::L0ExplicitInput,
            content: "hello".to_string(),
            token_count: 5,
            source: "explicit".to_string(),
        }],
        file_context: vec!["src/lib.rs".to_string()],
        tool_context: vec!["read: file reader".to_string()],
        session_context: vec!["User: hi".to_string()],
        prompt_messages: vec![],
        budget: ContextBudget {
            total_tokens: 5,
            max_tokens: 100,
            remaining_tokens: 95,
            usage_pct: 0.05,
            layer_breakdown: vec![(ContextLayer::L0ExplicitInput, 5)],
            layer_budgets: LayerBudgets::default(),
            warning_threshold: 0.7,
            compact_threshold: 0.85,
            continuation_threshold: 0.95,
        },
    };

    let summary = RuntimeContextSummary::from_context(&context);

    assert_eq!(summary.total_tokens, 5);
    assert_eq!(summary.remaining_tokens, 95);
    assert_eq!(summary.layer_count, 1);
    assert_eq!(summary.file_count, 1);
    assert_eq!(summary.tool_count, 1);
    assert_eq!(summary.session_count, 1);
}

#[test]
fn runtime_context_summary_preserves_layer_breakdown() {
    let context = Context {
        layers: vec![
            ContextItem {
                layer: ContextLayer::L0ExplicitInput,
                content: "hello".to_string(),
                token_count: 5,
                source: "explicit".to_string(),
            },
            ContextItem {
                layer: ContextLayer::L2ProjectContext,
                content: "src/lib.rs".to_string(),
                token_count: 12,
                source: "project".to_string(),
            },
        ],
        file_context: vec![],
        tool_context: vec![],
        session_context: vec![],
        prompt_messages: vec![],
        budget: ContextBudget {
            total_tokens: 17,
            max_tokens: 100,
            remaining_tokens: 83,
            usage_pct: 0.17,
            layer_breakdown: vec![
                (ContextLayer::L0ExplicitInput, 5),
                (ContextLayer::L2ProjectContext, 12),
            ],
            layer_budgets: LayerBudgets::default(),
            warning_threshold: 0.7,
            compact_threshold: 0.85,
            continuation_threshold: 0.95,
        },
    };

    let summary = RuntimeContextSummary::from_context(&context);

    assert_eq!(summary.layer_breakdown.len(), 2);
    assert_eq!(summary.layer_breakdown[0].1, 5);
    assert_eq!(summary.layer_breakdown[1].1, 12);
}

#[tokio::test]
async fn runtime_submit_with_session_id_persists_turn_to_storage() {
    let (runtime, storage) = build_runtime();
    let session = Session::default();
    let session_id = session.id.to_string();
    storage
        .save_session(&session)
        .await
        .expect("session should save before runtime submit");

    let result = runtime
        .execute(RuntimeCommand::SubmitUserInput(SubmitUserInput {
            session_id: Some(session_id.clone()),
            input: "hello".to_string(),
        }))
        .await
        .expect("submit command should create a persisted turn");

    let stored = storage
        .load_session(&session_id)
        .await
        .expect("session load should succeed")
        .expect("session should exist");

    assert_eq!(result.session_id.as_deref(), Some(session_id.as_str()));
    assert!(result.turn_id.is_some());
    assert_eq!(stored.turns.len(), 1);
    assert_eq!(
        stored.active_turn_id.map(|t| t.0.to_string()),
        result.turn_id
    );
}

#[tokio::test]
async fn runtime_submit_creates_task() {
    let (runtime, _) = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::SubmitUserInput(SubmitUserInput {
            session_id: None,
            input: "fix the bug".to_string(),
        }))
        .await
        .expect("submit should succeed");

    assert!(result.accepted);
    let task_count = runtime.task_store().active_count().await;
    assert_eq!(task_count, 1, "one active task should exist after submit");
}

#[tokio::test]
async fn runtime_task_cancel_requests_cancellation() {
    let (runtime, _) = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::SubmitUserInput(SubmitUserInput {
            session_id: None,
            input: "do work".to_string(),
        }))
        .await
        .expect("submit should succeed");

    let task_id = runtime.task_store().list_active_tasks().await[0]
        .id
        .0
        .to_string();

    let cancel_result = runtime
        .execute(RuntimeCommand::TaskControl(TaskControlCommand::Cancel {
            task_id: task_id.clone(),
        }))
        .await
        .expect("cancel should succeed");

    assert!(cancel_result.accepted);
    assert_eq!(cancel_result.session_id, result.session_id);
    assert_eq!(cancel_result.turn_id, result.turn_id);
}

#[tokio::test]
async fn runtime_task_cancel_nonexistent_returns_error() {
    let (runtime, _) = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::TaskControl(TaskControlCommand::Cancel {
            task_id: "00000000-0000-0000-0000-000000000999".to_string(),
        }))
        .await;

    assert!(matches!(result, Err(RuntimeFacadeError::Dependency(_))));
}

#[tokio::test]
async fn runtime_task_events_published_after_submit() {
    let event_bus = Arc::new(EventBus::new());
    let permission_manager = Arc::new(RwLock::new(PermissionManager::default()));
    let session_repo = Arc::new(InMemorySessionRepository::default());
    let project_repo = Arc::new(InMemoryProjectRepository::default());
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("runtime-task-events.db");
    let pool = StoragePool::new(&db_path).expect("storage pool");
    let storage = Arc::new(StorageService::new(session_repo, project_repo, pool));
    let agent_runtime = Arc::new(RwLock::new(AgentRuntime::new(
        Session::default(),
        AgentType::Build,
    )));
    std::mem::forget(temp_dir);

    let mut rx = event_bus.subscribe();

    let runtime = Runtime::new(RuntimeServices::new(
        event_bus,
        permission_manager,
        storage,
        agent_runtime,
        Arc::new(RuntimeTaskStore::new()),
        Arc::new(RuntimeToolRouter::new(opencode_tools::ToolRegistry::new())),
    ));

    runtime
        .execute(RuntimeCommand::SubmitUserInput(SubmitUserInput {
            session_id: None,
            input: "hello".to_string(),
        }))
        .await
        .expect("submit should succeed");

    let mut found_task_started = false;
    while let Ok(event) = rx.recv().await {
        if matches!(event, InternalEvent::TaskStarted { .. }) {
            found_task_started = true;
            break;
        }
    }
    assert!(
        found_task_started,
        "TaskStarted event should be published after submit"
    );
}

#[test]
fn runtime_task_status_state_machine() {
    use opencode_runtime::{RuntimeTask, RuntimeTaskStatus, TaskKind};
    use uuid::Uuid;

    let mut task = RuntimeTask::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskKind::Agent,
        "test task".to_string(),
        None,
    );

    assert_eq!(task.status, RuntimeTaskStatus::Pending);
    assert!(!task.is_terminal());
    assert!(task.can_cancel());

    task.mark_preparing();
    assert_eq!(task.status, RuntimeTaskStatus::Preparing);
    assert!(task.can_cancel());

    task.mark_started();
    assert_eq!(task.status, RuntimeTaskStatus::Running);
    assert!(task.can_cancel());

    task.mark_waiting_for_permission();
    assert_eq!(task.status, RuntimeTaskStatus::WaitingForPermission);
    assert!(task.can_cancel());

    task.mark_started();
    task.mark_completed();
    assert_eq!(task.status, RuntimeTaskStatus::Completed);
    assert!(task.is_terminal());
    assert!(!task.can_cancel());

    let mut task2 = RuntimeTask::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskKind::Agent,
        "test task 2".to_string(),
        None,
    );
    task2.mark_started();
    task2.mark_failed();
    assert_eq!(task2.status, RuntimeTaskStatus::Failed);
    assert!(task2.is_terminal());

    let mut task3 = RuntimeTask::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        TaskKind::Agent,
        "test task 3".to_string(),
        None,
    );
    task3.mark_cancelling();
    assert_eq!(task3.status, RuntimeTaskStatus::Cancelling);
    task3.mark_cancelled();
    assert_eq!(task3.status, RuntimeTaskStatus::Cancelled);
    assert!(task3.is_terminal());
}

#[tokio::test]
async fn runtime_trace_store_begin_and_end_trace() {
    use opencode_runtime::RuntimeTraceStore;
    use uuid::Uuid;

    let store = RuntimeTraceStore::new();
    let session_id = Uuid::new_v4();

    let trace_id = store
        .begin_trace(session_id, None, None)
        .await
        .expect("trace begins");
    assert!(trace_id.0 != Uuid::nil());

    store
        .end_trace(trace_id, true, None)
        .await
        .expect("trace ends");

    let retrieved = store.get_trace(&trace_id).await.expect("trace retrieved");
    assert!(retrieved.is_some());
    let trace = retrieved.unwrap();
    assert!(trace.ended_at.is_some());
    assert!(trace.success);
    assert!(trace.error.is_none());
}

#[tokio::test]
async fn runtime_trace_store_records_tool_calls() {
    use opencode_runtime::RuntimeTraceStore;
    use uuid::Uuid;

    let store = RuntimeTraceStore::new();
    let session_id = Uuid::new_v4();

    let trace_id = store
        .begin_trace(session_id, None, None)
        .await
        .expect("trace begins");

    store
        .record_tool_call(trace_id, "read")
        .await
        .expect("tool call 1");
    store
        .record_tool_call(trace_id, "write")
        .await
        .expect("tool call 2");
    store
        .record_tool_call(trace_id, "grep")
        .await
        .expect("tool call 3");

    let retrieved = store.get_trace(&trace_id).await.expect("trace retrieved");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().tool_call_count, 3);
}

#[tokio::test]
async fn runtime_trace_store_list_session_traces() {
    use opencode_runtime::RuntimeTraceStore;
    use uuid::Uuid;

    let store = RuntimeTraceStore::new();
    let session_id = Uuid::new_v4();

    let trace1 = store
        .begin_trace(session_id, None, None)
        .await
        .expect("trace 1");
    store
        .end_trace(trace1, true, None)
        .await
        .expect("end trace 1");

    let trace2 = store
        .begin_trace(session_id, None, None)
        .await
        .expect("trace 2");
    store
        .end_trace(trace2, false, Some("error".to_string()))
        .await
        .expect("end trace 2");

    let summaries = store
        .list_session_traces(&session_id)
        .await
        .expect("list traces");
    assert_eq!(summaries.len(), 2);
}

#[tokio::test]
async fn runtime_checkpoint_store_save_and_load() {
    use opencode_runtime::{
        Checkpoint, CheckpointStore, RuntimeCheckpointStore, RuntimeTaskId, RuntimeTaskStatus,
    };
    use uuid::Uuid;

    let store = RuntimeCheckpointStore::new();
    let task_id = RuntimeTaskId::new();
    let session_id = Uuid::new_v4();
    let turn_id = Uuid::new_v4();

    let checkpoint = Checkpoint::new(
        task_id,
        session_id,
        turn_id,
        RuntimeTaskStatus::Running,
        "test task".to_string(),
        "step 1".to_string(),
        serde_json::json!({"key": "value"}),
    );
    let checkpoint_id = checkpoint.id;

    store
        .save_checkpoint(&checkpoint)
        .await
        .expect("checkpoint saves");

    let loaded = store.load_latest(&task_id).await.expect("checkpoint loads");
    assert!(loaded.is_some());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.id, checkpoint_id);
    assert_eq!(loaded.task_description, "test task");
    assert_eq!(loaded.current_step, "step 1");
}

#[tokio::test]
async fn runtime_checkpoint_store_multiple_per_task() {
    use opencode_runtime::{
        Checkpoint, CheckpointStore, RuntimeCheckpointStore, RuntimeTaskId, RuntimeTaskStatus,
    };
    use uuid::Uuid;

    let store = RuntimeCheckpointStore::new();
    let task_id = RuntimeTaskId::new();
    let session_id = Uuid::new_v4();
    let turn_id = Uuid::new_v4();

    let cp1 = Checkpoint::new(
        task_id,
        session_id,
        turn_id,
        RuntimeTaskStatus::Running,
        "task".to_string(),
        "step 1".to_string(),
        serde_json::json!({}),
    );
    let cp2 = Checkpoint::new(
        task_id,
        session_id,
        turn_id,
        RuntimeTaskStatus::Running,
        "task".to_string(),
        "step 2".to_string(),
        serde_json::json!({}),
    );

    store.save_checkpoint(&cp1).await.expect("cp1 saves");
    store.save_checkpoint(&cp2).await.expect("cp2 saves");

    let loaded = store.load_latest(&task_id).await.expect("loads latest");
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().id, cp2.id);
}
