#[cfg(test)]
#[allow(clippy::module_inception)]
mod snapshot_durability_tests {
    use opencode_core::checkpoint::CheckpointManager;
    use opencode_core::revert::RevertManager;
    use opencode_core::{Message, Session};
    use tempfile::TempDir;

    fn create_test_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Hello world".to_string()));
        session.add_message(Message::assistant("Hi there!".to_string()));
        session.add_message(Message::user("How are you?".to_string()));
        session
    }

    fn create_session_with_content() -> Session {
        let mut session = Session::new();
        session.add_message(Message::system("System prompt".to_string()));
        session.add_message(Message::user("User message 1".to_string()));
        session.add_message(Message::assistant("Assistant response 1".to_string()));
        session.add_message(Message::user("User message 2".to_string()));
        session.add_message(Message::assistant("Assistant response 2".to_string()));
        session
    }

    #[test]
    fn snapshot_durability_checkpoint_survives_restart() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let session = create_test_session();
        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);

        manager.create(&session, "Initial checkpoint").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);

        let loaded = manager2.load(&original_id, 0).unwrap();
        assert_eq!(loaded.id, original_id);
        assert_eq!(loaded.messages.len(), 3);
    }

    #[test]
    fn snapshot_durability_multiple_checkpoints_persist() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = create_test_session();
        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(10);

        manager.create(&session, "Checkpoint 1").unwrap();

        session.add_message(Message::user("New message".to_string()));
        manager.create(&session, "Checkpoint 2").unwrap();

        session.add_message(Message::assistant("Another new".to_string()));
        manager.create(&session, "Checkpoint 3").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(10);

        let checkpoints = manager2.list(&original_id).unwrap();
        assert_eq!(checkpoints.len(), 3);

        let loaded0 = manager2.load(&original_id, 0).unwrap();
        assert_eq!(loaded0.messages.len(), 3);

        let loaded1 = manager2.load(&original_id, 1).unwrap();
        assert_eq!(loaded1.messages.len(), 4);

        let loaded2 = manager2.load(&original_id, 2).unwrap();
        assert_eq!(loaded2.messages.len(), 5);
    }

    #[test]
    fn snapshot_durability_revert_operations_reliable() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = create_test_session_with_revert();
        let original_id = session.id;
        let original_message_count = session.messages.len();

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "Before revert").unwrap();

        let mut revert_manager = RevertManager::new(10);
        let point = revert_manager.create_point(2, "Revert point".to_string());

        revert_manager.revert_to(&mut session, &point.id).unwrap();
        assert_eq!(session.messages.len(), 2);

        let loaded = manager.load(&original_id, 0).unwrap();
        assert_eq!(loaded.messages.len(), original_message_count);
    }

    fn create_test_session_with_revert() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));
        session.add_message(Message::user("msg3".to_string()));
        session.add_message(Message::assistant("msg4".to_string()));
        session
    }

    #[test]
    fn snapshot_durability_data_integrity_maintained() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = create_session_with_content();
        let original_id = session.id;

        let original_messages: Vec<_> = session.messages.clone();
        let original_system = original_messages[0].content.clone();
        let original_first_user = original_messages[1].content.clone();

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "Integrity test").unwrap();

        session.add_message(Message::user("Corrupting message".to_string()));
        session.add_message(Message::assistant("More corruption".to_string()));
        session.messages.clear();

        drop(session);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);
        let loaded = manager2.load(&original_id, 0).unwrap();

        assert_eq!(loaded.messages.len(), original_messages.len());
        assert_eq!(loaded.messages[0].content, original_system);
        assert_eq!(loaded.messages[1].content, original_first_user);
    }

    #[test]
    fn snapshot_durability_large_content_survives_restart() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = Session::new();
        session.add_message(Message::user("Small message".to_string()));

        let large_content = "x".repeat(1024 * 1024);
        session.add_message(Message::assistant(large_content.clone()));

        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "Large content").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);
        let loaded = manager2.load(&original_id, 0).unwrap();

        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[1].content.len(), 1024 * 1024);
    }

    #[test]
    fn snapshot_durability_unicode_content_survives_restart() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = Session::new();
        session.add_message(Message::user("Hello 世界 🌍".to_string()));
        session.add_message(Message::assistant("日本語テスト".to_string()));
        session.add_message(Message::user("مرحبا".to_string()));

        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "Unicode").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);
        let loaded = manager2.load(&original_id, 0).unwrap();

        assert_eq!(loaded.messages.len(), 3);
        assert_eq!(loaded.messages[0].content, "Hello 世界 🌍");
        assert_eq!(loaded.messages[1].content, "日本語テスト");
        assert_eq!(loaded.messages[2].content, "مرحبا");
    }

    #[test]
    fn snapshot_durability_tool_invocations_preserved() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = Session::new();
        session.add_message(Message::user("test".to_string()));
        session
            .tool_invocations
            .push(opencode_core::ToolInvocationRecord {
                id: uuid::Uuid::new_v4(),
                tool_name: "read".to_string(),
                arguments: serde_json::json!({"path": "test.txt"}),
                args_hash: "hash123".to_string(),
                result: Some("file content".to_string()),
                started_at: chrono::Utc::now(),
                completed_at: Some(chrono::Utc::now()),
                latency_ms: Some(100),
            });

        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "With tools").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);
        let loaded = manager2.load(&original_id, 0).unwrap();

        assert_eq!(loaded.tool_invocations.len(), 1);
        assert_eq!(loaded.tool_invocations[0].tool_name, "read");
        assert_eq!(loaded.tool_invocations[0].args_hash, "hash123");
    }

    #[test]
    fn snapshot_durability_session_id_preserved() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = Session::new();
        session.add_message(Message::user("test".to_string()));

        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "ID preservation").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);
        let loaded = manager2.load(&original_id, 0).unwrap();

        assert_eq!(loaded.id, original_id);
    }

    #[test]
    fn snapshot_durability_get_latest_returns_most_recent() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = create_test_session();

        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(10);

        manager.create(&session, "First").unwrap();

        session.add_message(Message::user("After first".to_string()));
        manager.create(&session, "Second").unwrap();

        session.add_message(Message::assistant("After second".to_string()));
        manager.create(&session, "Third").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(10);

        let checkpoints = manager2.list(&original_id).unwrap();
        assert_eq!(checkpoints.len(), 3);

        let loaded_2 = manager2.load(&original_id, 2).unwrap();
        assert_eq!(loaded_2.messages.len(), 5);

        let loaded_1 = manager2.load(&original_id, 1).unwrap();
        assert_eq!(loaded_1.messages.len(), 4);
    }

    #[test]
    fn snapshot_durability_revert_then_checkpoint() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = create_test_session();

        let original_id = session.id;

        let mut revert_manager = RevertManager::new(10);
        let point = revert_manager.create_point(2, "Revert here".to_string());

        revert_manager.revert_to(&mut session, &point.id).unwrap();
        assert_eq!(session.messages.len(), 2);

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "After revert").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);

        let loaded = manager2.load(&original_id, 0).unwrap();
        assert_eq!(loaded.messages.len(), 2);
    }

    #[test]
    fn snapshot_durability_empty_session() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let session = Session::new();

        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "Empty").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);

        let loaded = manager2.load(&original_id, 0).unwrap();
        assert!(loaded.messages.is_empty());
    }

    #[test]
    fn snapshot_durability_concurrent_checkpoint_isolation() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();

        let mut session1 = Session::new();
        session1.add_message(Message::user("Session 1 message".to_string()));
        let id1 = session1.id;

        let mut session2 = Session::new();
        session2.add_message(Message::user("Session 2 message".to_string()));
        let id2 = session2.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);

        manager.create(&session1, "S1 checkpoint").unwrap();
        manager.create(&session2, "S2 checkpoint").unwrap();

        session1.add_message(Message::assistant("S1 extra".to_string()));
        manager.create(&session1, "S1 checkpoint 2").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);

        let s1_checkpoints = manager2.list(&id1).unwrap();
        let s2_checkpoints = manager2.list(&id2).unwrap();

        assert_eq!(s1_checkpoints.len(), 2);
        assert_eq!(s2_checkpoints.len(), 1);

        let loaded1 = manager2.load(&id1, 1).unwrap();
        assert_eq!(loaded1.messages.len(), 2);

        let loaded2 = manager2.load(&id2, 0).unwrap();
        assert_eq!(loaded2.messages.len(), 1);
    }

    #[test]
    fn snapshot_durability_revert_point_not_found_after_restart() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = create_test_session();

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);
        manager.create(&session, "Checkpoint").unwrap();

        let mut revert_manager = RevertManager::new(10);
        let point = revert_manager.create_point(1, "Test point".to_string());
        let point_id = point.id.clone();

        drop(revert_manager);

        let revert_manager2 = RevertManager::new(10);
        let result = revert_manager2.revert_to(&mut session, &point_id);
        assert!(result.is_err());
    }

    #[test]
    fn snapshot_durability_message_order_preserved() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = Session::new();

        let roles = vec![
            Message::system("System".to_string()),
            Message::user("User 1".to_string()),
            Message::assistant("Assistant 1".to_string()),
            Message::user("User 2".to_string()),
            Message::assistant("Assistant 2".to_string()),
            Message::user("User 3".to_string()),
        ];

        for msg in roles {
            session.add_message(msg);
        }

        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(5);
        manager.create(&session, "Order test").unwrap();

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(5);

        let loaded = manager2.load(&original_id, 0).unwrap();

        assert_eq!(loaded.messages.len(), 6);
        assert!(loaded.messages[0].role == opencode_core::message::Role::System);
        assert!(loaded.messages[1].role == opencode_core::message::Role::User);
        assert!(loaded.messages[2].role == opencode_core::message::Role::Assistant);
        assert!(loaded.messages[3].role == opencode_core::message::Role::User);
        assert!(loaded.messages[4].role == opencode_core::message::Role::Assistant);
        assert!(loaded.messages[5].role == opencode_core::message::Role::User);
    }

    #[test]
    fn snapshot_durability_checkpoints_pruned_after_limit() {
        let tmp = TempDir::new().unwrap();
        let checkpoints_dir = tmp.path().to_path_buf();
        let mut session = create_test_session();
        let original_id = session.id;

        let manager = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir.clone())
            .with_max_checkpoints(3);

        for i in 0..5 {
            session.add_message(Message::user(format!("msg{}", i)));
            let desc = format!("Checkpoint {}", i);
            manager.create(&session, &desc).unwrap();
        }

        drop(manager);

        let manager2 = CheckpointManager::new()
            .with_checkpoints_dir(checkpoints_dir)
            .with_max_checkpoints(3);

        let checkpoints = manager2.list(&original_id).unwrap();
        assert_eq!(checkpoints.len(), 3);
    }
}
