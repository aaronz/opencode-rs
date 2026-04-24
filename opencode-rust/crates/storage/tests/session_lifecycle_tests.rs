use opencode_core::checkpoint::CheckpointManager;
use opencode_core::config::ShareMode;
use opencode_core::message::Message;
use opencode_core::revert::RevertManager;
use opencode_core::session::Session;
use opencode_storage::migration::MigrationManager;
use opencode_storage::{SqliteProjectRepository, SqliteSessionRepository, StoragePool};
use tempfile::TempDir;
use uuid::Uuid;

fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

fn create_test_session() -> Session {
    let mut session = Session::new();
    session.add_message(Message::user("Test message 1".to_string()));
    session.add_message(Message::assistant("Test response 1".to_string()));
    session
}

async fn setup_storage_service(temp_dir: &TempDir) -> opencode_storage::StorageService {
    let db_path = temp_dir.path().join("test.db");
    let pool = StoragePool::new(&db_path).expect("Should create pool");
    let manager = MigrationManager::new(pool.clone(), 3);
    manager.migrate().await.expect("Should run migrations");
    let session_repo = std::sync::Arc::new(SqliteSessionRepository::new(pool.clone()));
    let project_repo = std::sync::Arc::new(SqliteProjectRepository::new(pool.clone()));
    opencode_storage::StorageService::new(session_repo, project_repo, pool)
}

#[tokio::test]
async fn test_session_lifecycle_storage_create() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    let session = create_test_session();
    let session_id = session.id;

    service
        .save_session(&session)
        .await
        .expect("Should save session");

    let loaded = service
        .load_session(&session_id.to_string())
        .await
        .expect("Should load session")
        .expect("Session should exist");

    assert_eq!(loaded.id, session_id);
    assert_eq!(loaded.messages.len(), session.messages.len());
    assert_eq!(loaded.messages[0].content, "Test message 1");
}

#[tokio::test]
async fn test_session_lifecycle_storage_fork_child() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    let parent = create_test_session();
    let parent_id = parent.id;

    service
        .save_session(&parent)
        .await
        .expect("Should save parent");

    let child_id = Uuid::new_v4();
    let mut child = parent.fork(child_id);
    child.add_message(Message::user("Child message".to_string()));

    let child_id_str = child.id.to_string();
    service
        .save_session(&child)
        .await
        .expect("Should save child");

    let loaded_parent = service
        .load_session(&parent_id.to_string())
        .await
        .expect("Should load parent")
        .expect("Parent should exist");

    let loaded_child = service
        .load_session(&child_id_str)
        .await
        .expect("Should load child")
        .expect("Child should exist");

    assert_ne!(loaded_child.id, loaded_parent.id);
    assert_eq!(
        loaded_child.parent_session_id.as_deref(),
        Some(parent_id.to_string().as_str())
    );
    assert_eq!(loaded_parent.messages.len(), 2);
    assert_eq!(loaded_child.messages.len(), 3);
}

#[tokio::test]
async fn test_session_lifecycle_storage_share_export() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    let mut session = create_test_session();
    session.set_share_mode(ShareMode::Manual);
    let share_link = session
        .generate_share_link()
        .expect("Should generate share link");

    assert!(share_link.contains("/share/"));
    assert!(session.shared_id.is_some());

    let session_id = session.id.to_string();
    service
        .save_session(&session)
        .await
        .expect("Should save shared session");

    let loaded = service
        .load_session(&session_id)
        .await
        .expect("Should load session")
        .expect("Session should exist");

    assert!(loaded.is_shared());
    assert_eq!(loaded.get_share_id(), session.shared_id.as_deref());
}

#[tokio::test]
async fn test_session_lifecycle_storage_compact() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    let mut session = create_test_session();
    for i in 0..50 {
        let content = format!("Response {} with substantial content to ensure compaction happens and messages need to be compressed properly", i);
        session.add_message(Message::assistant(content));
    }

    let pre_compact_count = session.messages.len();
    assert!(pre_compact_count > 40);

    let result = session.compact_messages(100);
    assert!(
        result.was_compacted,
        "Compaction should occur with many messages and small token budget"
    );
    assert!(
        session.messages.len() < pre_compact_count,
        "Messages should be reduced after compaction"
    );

    let session_id = session.id.to_string();
    service
        .save_session(&session)
        .await
        .expect("Should save compacted session");

    let loaded = service
        .load_session(&session_id)
        .await
        .expect("Should load session")
        .expect("Session should exist");

    assert_eq!(loaded.messages.len(), session.messages.len());
}

#[tokio::test]
async fn test_session_lifecycle_storage_revert_to_checkpoint() {
    let temp_dir = create_temp_dir();
    let checkpoints_dir = temp_dir.path().join("checkpoints");
    let service = setup_storage_service(&temp_dir).await;

    let mut session = create_test_session();
    let original_message_count = session.messages.len();

    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(10);

    let _checkpoint = checkpoint_manager
        .create(&session, "Initial checkpoint")
        .expect("Should create checkpoint");

    session.add_message(Message::user("Message to be reverted".to_string()));
    session.add_message(Message::assistant("Response to be reverted".to_string()));

    let session_id = session.id.to_string();
    service
        .save_session(&session)
        .await
        .expect("Should save session with extra messages");

    let mut revert_manager = RevertManager::new(5);
    let revert_point =
        revert_manager.create_point(original_message_count, "Before new messages".to_string());

    revert_manager
        .revert_to(&mut session, &revert_point.id)
        .expect("Should revert");

    assert_eq!(session.messages.len(), original_message_count);

    service
        .save_session(&session)
        .await
        .expect("Should save reverted session");

    let loaded = service
        .load_session(&session_id)
        .await
        .expect("Should load session")
        .expect("Session should exist");

    assert_eq!(loaded.messages.len(), original_message_count);
    assert_eq!(loaded.messages[0].content, "Test message 1");
}

#[tokio::test]
async fn test_session_lifecycle_storage_message_history_preserved() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    let mut session = Session::new();
    session.add_message(Message::system("System prompt".to_string()));
    session.add_message(Message::user("User message 1".to_string()));
    session.add_message(Message::assistant("Assistant response 1".to_string()));
    session.add_message(Message::user("User message 2".to_string()));
    session.add_message(Message::assistant("Assistant response 2".to_string()));
    session.add_message(Message::user("User message 3".to_string()));

    let _original_id = session.id;
    let original_messages = session.messages.clone();

    let session_id = session.id.to_string();
    service
        .save_session(&session)
        .await
        .expect("Should save session");

    let child = session.fork(Uuid::new_v4());
    let child_id = child.id.to_string();
    service
        .save_session(&child)
        .await
        .expect("Should save child");

    let mut parent_loaded = service
        .load_session(&session_id)
        .await
        .expect("Should load parent")
        .expect("Parent should exist");

    parent_loaded.add_message(Message::assistant("Parent only message".to_string()));
    service
        .save_session(&parent_loaded)
        .await
        .expect("Should save modified parent");

    let child_loaded = service
        .load_session(&child_id)
        .await
        .expect("Should load child")
        .expect("Child should exist");

    let final_parent = service
        .load_session(&session_id)
        .await
        .expect("Should load final parent")
        .expect("Parent should exist");

    assert_eq!(final_parent.messages.len(), original_messages.len() + 1);
    assert_eq!(
        final_parent
            .messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default(),
        "Parent only message"
    );

    assert_eq!(child_loaded.messages.len(), original_messages.len());
    for (i, msg) in original_messages.iter().enumerate() {
        assert_eq!(child_loaded.messages[i].content, msg.content);
    }
}

#[tokio::test]
async fn test_session_lifecycle_storage_full_sequence() {
    let temp_dir = create_temp_dir();
    let checkpoints_dir = temp_dir.path().join("checkpoints");
    let service = setup_storage_service(&temp_dir).await;

    let mut session = Session::new();
    session.add_message(Message::system("You are a helpful assistant".to_string()));
    session.add_message(Message::user("Start a task".to_string()));
    session.add_message(Message::assistant("I'll help".to_string()));

    let original_id = session.id;
    let session_id_str = original_id.to_string();

    service
        .save_session(&session)
        .await
        .expect("Phase 1: Save initial session");

    let checkpoint_manager = CheckpointManager::new()
        .with_checkpoints_dir(checkpoints_dir.clone())
        .with_max_checkpoints(5);

    let _checkpoint = checkpoint_manager
        .create(&session, "Initial checkpoint")
        .expect("Phase 2: Create checkpoint");

    let mut child = session.fork(Uuid::new_v4());
    let child_id_str = child.id.to_string();
    child.add_message(Message::user("Child task".to_string()));
    service
        .save_session(&child)
        .await
        .expect("Phase 3: Save forked child");

    let share_path = temp_dir.path().join("temp_session.json");
    session.save(&share_path).expect("Save for sharing");
    let mut shareable = Session::load(&share_path).expect("Load for sharing");
    shareable.set_share_mode(ShareMode::Manual);
    let _share_link = shareable
        .generate_share_link()
        .expect("Phase 4: Generate share link");

    for i in 0..25 {
        shareable.add_message(Message::assistant(format!("Additional response {}", i)));
    }

    let _pre_compact_count = shareable.messages.len();
    shareable.compact_messages(100);
    service
        .save_session(&shareable)
        .await
        .expect("Phase 5: Save compacted session");

    let mut revert_manager = RevertManager::new(5);
    let revert_point = revert_manager.create_point(2, "Before additional".to_string());

    let mut to_revert = service
        .load_session(&session_id_str)
        .await
        .expect("Should load for revert")
        .expect("Session should exist");
    to_revert.add_message(Message::user("Extra message".to_string()));

    revert_manager
        .revert_to(&mut to_revert, &revert_point.id)
        .expect("Phase 6: Revert");

    service
        .save_session(&to_revert)
        .await
        .expect("Phase 6: Save reverted session");

    let checkpoints = checkpoint_manager
        .list(&original_id)
        .expect("Should list checkpoints");
    assert!(
        !checkpoints.is_empty(),
        "Checkpoints should exist after full sequence"
    );

    let final_session = service
        .load_session(&session_id_str)
        .await
        .expect("Should load final session")
        .expect("Final session should exist");
    assert_eq!(final_session.id, original_id);

    let final_child = service
        .load_session(&child_id_str)
        .await
        .expect("Should load child")
        .expect("Child should exist");
    assert_eq!(
        final_child.parent_session_id.as_deref(),
        Some(original_id.to_string().as_str())
    );
    assert!(final_child
        .messages
        .iter()
        .any(|m| m.content == "Child task"));
}

#[tokio::test]
async fn test_session_model_persists_all_fields() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    let mut session = Session::new();
    session.add_message(Message::user("Test".to_string()));
    session.set_share_mode(ShareMode::Auto);
    let _ = session.generate_share_link();

    let session_id = session.id.to_string();
    service
        .save_session(&session)
        .await
        .expect("Should save session");

    let loaded = service
        .load_session(&session_id)
        .await
        .expect("Should load session")
        .expect("Session should exist");

    assert_eq!(loaded.id, session.id);
    assert_eq!(loaded.messages.len(), session.messages.len());
    assert_eq!(loaded.share_mode, session.share_mode);
    assert_eq!(loaded.shared_id, session.shared_id);
}

#[tokio::test]
async fn test_session_lifecycle_storage_delete() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    let session = create_test_session();
    let session_id = session.id.to_string();

    service
        .save_session(&session)
        .await
        .expect("Should save session");

    let loaded = service
        .load_session(&session_id)
        .await
        .expect("Should load session")
        .expect("Session should exist");
    assert_eq!(loaded.id, session.id);

    service
        .delete_session(&session_id)
        .await
        .expect("Should delete session");

    let loaded = service
        .load_session(&session_id)
        .await
        .expect("Should load session");
    assert!(loaded.is_none(), "Session should be deleted");
}

#[tokio::test]
async fn test_session_lifecycle_storage_list_sessions() {
    let temp_dir = create_temp_dir();
    let service = setup_storage_service(&temp_dir).await;

    for i in 0..3 {
        let mut session = Session::new();
        session.add_message(Message::user(format!("Message {}", i)));
        service
            .save_session(&session)
            .await
            .expect("Should save session");
    }

    let sessions = service
        .list_sessions(10, 0)
        .await
        .expect("Should list sessions");
    assert_eq!(sessions.len(), 3);
}
