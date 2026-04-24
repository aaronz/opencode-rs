use chrono::Utc;
use opencode_storage::migration::MigrationManager;
use opencode_storage::models::{
    AccountModel, InvocationStatus, ProjectModel, SessionModel, ToolInvocation,
};
use opencode_storage::StoragePool;
use uuid::Uuid;

#[test]
fn test_session_model_serialization() {
    let model = SessionModel {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: r#"{"id":"test","messages":[]}"#.to_string(),
        project_path: None,
    };

    let json = serde_json::to_string(&model).expect("Should serialize");
    let deserialized: SessionModel = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.id, model.id);
}

#[test]
fn test_tool_invocation_new() {
    let session_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();
    let args = serde_json::json!({"path": "/test.txt"});

    let invocation = ToolInvocation::new(session_id, message_id, "read".to_string(), args);

    assert_eq!(invocation.session_id, session_id);
    assert_eq!(invocation.message_id, message_id);
    assert_eq!(invocation.tool_name, "read");
    assert_eq!(invocation.status, InvocationStatus::Running);
    assert!(invocation.completed_at.is_none());
}

#[test]
fn test_tool_invocation_complete() {
    let invocation = ToolInvocation::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "read".to_string(),
        serde_json::json!({"path": "/test.txt"}),
    );

    let mut invocation = invocation;
    let result = serde_json::json!({"content": "file contents", "success": true});
    invocation.complete(result.clone());

    assert_eq!(invocation.status, InvocationStatus::Completed);
    assert!(invocation.completed_at.is_some());
    assert!(invocation.latency_ms.is_some());
    assert!(invocation.result.is_some());
}

#[test]
fn test_tool_invocation_fail() {
    let invocation = ToolInvocation::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "read".to_string(),
        serde_json::json!({"path": "/test.txt"}),
    );

    let mut invocation = invocation;
    invocation.fail();

    assert_eq!(invocation.status, InvocationStatus::Failed);
    assert!(invocation.completed_at.is_some());
    assert!(invocation.result.is_none());
}

#[test]
fn test_invocation_status_display() {
    assert_eq!(InvocationStatus::Running.to_string(), "running");
    assert_eq!(InvocationStatus::Completed.to_string(), "completed");
    assert_eq!(InvocationStatus::Failed.to_string(), "failed");
}

#[test]
fn test_project_model_creation() {
    let project = ProjectModel {
        id: Uuid::new_v4().to_string(),
        path: "/tmp/test_project".to_string(),
        name: Some("Test Project".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        data: None,
    };

    assert_eq!(project.name, Some("Test Project".to_string()));
    assert!(project.data.is_none());
}

#[test]
fn test_account_model_creation() {
    let account = AccountModel {
        id: Uuid::new_v4().to_string(),
        username: "testuser".to_string(),
        email: Some("test@example.com".to_string()),
        password_hash: "hashed_password".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: None,
        is_active: true,
        data: None,
    };

    assert_eq!(account.username, "testuser");
    assert_eq!(account.email, Some("test@example.com".to_string()));
    assert!(account.is_active);
}

#[test]
fn test_account_model_with_last_login() {
    let last_login = Utc::now();
    let account = AccountModel {
        id: Uuid::new_v4().to_string(),
        username: "testuser".to_string(),
        email: None,
        password_hash: "hash".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_login_at: Some(last_login),
        is_active: true,
        data: None,
    };

    assert!(account.last_login_at.is_some());
}

#[tokio::test]
async fn test_storage_pool_creation() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).expect("Should create pool");
    assert!(pool.get().await.is_ok());

    drop(temp_dir);
}

#[tokio::test]
async fn test_migration_manager_new() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let pool = StoragePool::new(&db_path).expect("Should create pool");
    let manager = MigrationManager::new(pool, 3);

    let result = manager.migrate().await;
    assert!(result.is_ok());

    drop(temp_dir);
}
