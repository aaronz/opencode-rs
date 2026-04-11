#[cfg(test)]
mod recovery_tests {
    use crate::compaction::{CompactionManager, ShareabilityVerifier};
    use opencode_core::{checkpoint::CheckpointManager, config::ShareMode, Message, Session};

    fn create_test_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Hello".to_string()));
        session.add_message(Message::assistant("Hi there!".to_string()));
        session.add_message(Message::user("How are you?".to_string()));
        session
    }

    fn create_shareable_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Shareable message".to_string()));
        session.set_share_mode(ShareMode::Manual);
        session.generate_share_link().unwrap();
        session
    }

    #[test]
    fn test_recovery_compaction_maintains_data_integrity() {
        let mut session = create_shareable_session();

        for i in 0..20 {
            session.add_message(Message::assistant(format!(
                "Long response number {} with substantial content to trigger compaction mechanism",
                i
            )));
        }

        let original_message_count = session.messages.len();
        let original_share_id = session.shared_id.clone();

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.compaction_result.was_compacted);
        assert!(result.shareability_preserved);
        assert!(session.messages.len() < original_message_count);
        assert_eq!(session.shared_id, original_share_id);
    }

    #[test]
    fn test_recovery_compaction_idempotent_when_not_needed() {
        let mut session = Session::new();
        session.add_message(Message::user("Short".to_string()));
        session.add_message(Message::assistant("Short".to_string()));

        let result1 =
            CompactionManager::compact_with_shareability_verification(&mut session, 100000);
        assert!(result1.is_ok());
        assert!(!result1.unwrap().compaction_result.was_compacted);

        let result2 =
            CompactionManager::compact_with_shareability_verification(&mut session, 100000);
        assert!(result2.is_ok());
        assert!(!result2.unwrap().compaction_result.was_compacted);
    }

    #[test]
    fn test_recovery_compaction_preserves_system_messages() {
        let mut session = Session::new();
        session.add_message(Message::system("System prompt".to_string()));
        session.add_message(Message::user("User message".to_string()));
        session.add_message(Message::assistant("Assistant response".to_string()));

        for i in 0..30 {
            session.add_message(Message::assistant(format!("Long response {} with lots of content to trigger compaction mechanism and save context", i)));
        }

        let system_message = session.messages[0].content.clone();

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);

        assert!(result.is_ok());
        assert!(result.unwrap().compaction_result.was_compacted);
        assert!(session.messages[0].content == system_message);
    }

    #[test]
    fn test_recovery_compaction_with_very_large_content() {
        let mut session = Session::new();
        session.add_message(Message::user("Start".to_string()));

        let large_content = "x".repeat(5000);
        for i in 0..10 {
            session.add_message(Message::assistant(format!(
                "Large response {}: {}",
                i, large_content
            )));
        }

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.compaction_result.was_compacted);
        assert!(session.messages.len() < 12);
    }

    #[test]
    fn test_recovery_compaction_non_shareable_session() {
        let mut session = Session::new();
        session.add_message(Message::user("Non-shareable session".to_string()));

        for i in 0..25 {
            session.add_message(Message::assistant(format!("This is a longer response number {} with more content to ensure compaction happens properly", i)));
        }

        let original_count = session.messages.len();

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.compaction_result.was_compacted);
        assert!(session.messages.len() < original_count);
        assert!(!result.original_was_shareable);
    }

    #[test]
    fn test_recovery_checkpoint_creates_and_loads_session() {
        let mut session = create_test_session();
        let original_id = session.id;
        let original_message_count = session.messages.len();

        let manager = CheckpointManager::new().with_max_checkpoints(5);
        let checkpoint = manager.create(&session, "Test checkpoint").unwrap();

        assert_eq!(checkpoint.session_id, original_id);
        assert_eq!(checkpoint.sequence_number, 0);

        let loaded = manager.load(&session.id, 0).unwrap();
        assert_eq!(loaded.id, original_id);
        assert_eq!(loaded.messages.len(), original_message_count);
    }

    #[test]
    fn test_recovery_checkpoint_preserves_message_integrity() {
        let mut session = create_test_session();
        session.add_message(Message::assistant("Response content".to_string()));

        let manager = CheckpointManager::new().with_max_checkpoints(5);
        manager.create(&session, "Before recovery point").unwrap();

        session.add_message(Message::user("New message after checkpoint".to_string()));
        assert_eq!(session.messages.len(), 5);

        let loaded = manager.load(&session.id, 0).unwrap();
        assert_eq!(loaded.messages.len(), 4);
        assert_eq!(loaded.messages.last().unwrap().content, "Response content");
    }

    #[test]
    fn test_recovery_checkpoint_preserves_all_message_types() {
        let mut session = Session::new();
        session.add_message(Message::system("System message".to_string()));
        session.add_message(Message::user("User message".to_string()));
        session.add_message(Message::assistant("Assistant message".to_string()));
        session.add_message(Message::user("Another user".to_string()));
        session.add_message(Message::assistant("Another assistant".to_string()));

        let manager = CheckpointManager::new().with_max_checkpoints(5);
        manager.create(&session, "All message types").unwrap();

        let loaded = manager.load(&session.id, 0).unwrap();

        assert_eq!(loaded.messages.len(), 5);
        assert!(loaded.messages[0].role == opencode_core::message::Role::System);
        assert!(loaded.messages[1].role == opencode_core::message::Role::User);
        assert!(loaded.messages[2].role == opencode_core::message::Role::Assistant);
    }

    #[test]
    fn test_recovery_checkpoint_preserves_session_id() {
        let mut session = create_test_session();
        let original_id = session.id;

        let manager = CheckpointManager::new().with_max_checkpoints(5);
        manager.create(&session, "ID preservation test").unwrap();

        let loaded = manager.load(&session.id, 0).unwrap();
        assert_eq!(loaded.id, original_id);
    }

    #[test]
    fn test_recovery_checkpoint_empty_session() {
        let session = Session::new();
        let manager = CheckpointManager::new().with_max_checkpoints(5);

        let checkpoint = manager
            .create(&session, "Empty session checkpoint")
            .unwrap();
        assert_eq!(checkpoint.session_id, session.id);

        let loaded = manager.load(&session.id, 0).unwrap();
        assert!(loaded.messages.is_empty());
    }

    #[test]
    fn test_recovery_checkpoint_get_latest_none_when_empty() {
        let session = create_test_session();
        let manager = CheckpointManager::new().with_max_checkpoints(5);

        let latest = manager.get_latest(&session.id).unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn test_recovery_checkpoint_list_empty() {
        let session = create_test_session();
        let manager = CheckpointManager::new().with_max_checkpoints(5);

        let checkpoints = manager.list(&session.id).unwrap();
        assert!(checkpoints.is_empty());
    }

    #[test]
    fn test_recovery_compaction_shareability_auto_mode() {
        let mut session = Session::new();
        session.add_message(Message::user("Test".to_string()));
        session.set_share_mode(ShareMode::Auto);
        session.generate_share_link().unwrap();

        for i in 0..20 {
            session.add_message(Message::assistant(format!("Response {} with content", i)));
        }

        let result = CompactionManager::compact_with_shareability_verification(&mut session, 100);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.shareability_preserved);
        assert_eq!(result.verification.share_mode, Some("Auto".to_string()));
    }

    #[test]
    fn test_recovery_compaction_does_not_compact_when_not_needed() {
        let mut session = Session::new();
        session.add_message(Message::user("Short".to_string()));
        session.add_message(Message::assistant("Short".to_string()));

        let original_len = session.messages.len();

        let result =
            CompactionManager::compact_with_shareability_verification(&mut session, 100000);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.compaction_result.was_compacted);
        assert_eq!(session.messages.len(), original_len);
    }

    #[test]
    fn test_recovery_compaction_verification_auto_mode() {
        let mut session = Session::new();
        session.add_message(Message::user("Test".to_string()));
        session.set_share_mode(ShareMode::Auto);
        session.generate_share_link().unwrap();

        let verification = ShareabilityVerifier::verify(&session);

        assert!(verification.is_shareable);
        assert_eq!(verification.share_mode, Some("Auto".to_string()));
    }
}
