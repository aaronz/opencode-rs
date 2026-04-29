use std::sync::Arc;

use tokio::sync::RwLock;

use opencode_agent::{AgentRuntime, AgentType};
use opencode_core::{bus::EventBus, bus::InternalEvent, permission::PermissionManager, Session};
use opencode_runtime::{
    Runtime, RuntimeCommand, RuntimeEvent, RuntimeFacadeError, RuntimeServices, RuntimeStatus,
    SubmitUserInput, TaskControlCommand,
};
use opencode_storage::{
    InMemoryProjectRepository, InMemorySessionRepository, StoragePool, StorageService,
};

fn build_runtime() -> Runtime {
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

    Runtime::new(RuntimeServices::new(
        event_bus,
        permission_manager,
        storage,
        agent_runtime,
    ))
}

#[tokio::test]
async fn runtime_constructs_and_reports_idle_status() {
    let runtime = build_runtime();
    assert_eq!(runtime.status().await, RuntimeStatus::Idle);
}

#[tokio::test]
async fn runtime_accepts_submit_command_shape() {
    let runtime = build_runtime();
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
async fn runtime_unimplemented_commands_return_explicit_errors() {
    let runtime = build_runtime();
    let result = runtime
        .execute(RuntimeCommand::TaskControl(TaskControlCommand::Cancel {
            task_id: "task-1".to_string(),
        }))
        .await;

    assert!(matches!(
        result,
        Err(RuntimeFacadeError::NotImplemented("task control"))
    ));
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
