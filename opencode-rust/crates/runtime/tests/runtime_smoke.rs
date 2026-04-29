use std::sync::Arc;

use tokio::sync::RwLock;

use opencode_agent::{AgentRuntime, AgentType};
use opencode_core::{bus::EventBus, bus::InternalEvent, permission::PermissionManager, Session};
use opencode_runtime::{
    Runtime, RuntimeCommand, RuntimeEvent, RuntimeFacadeError, RuntimePermissionAdapter,
    RuntimePermissionDecision, RuntimeServices, RuntimeSessionStore, RuntimeStatus,
    SubmitUserInput, TaskControlCommand,
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
async fn runtime_unimplemented_commands_return_explicit_errors() {
    let (runtime, _) = build_runtime();
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
    let adapter = RuntimePermissionAdapter::new(PermissionManager::new(config));

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
