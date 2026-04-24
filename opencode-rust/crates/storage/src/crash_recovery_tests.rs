#[cfg(test)]
mod crash_recovery {
    use opencode_core::crash_recovery::{CrashRecovery, CrashRecoveryError, PanicHandler};
    use opencode_core::{Message, Session};
    use tempfile::tempdir;

    fn create_test_recovery(tmp: &tempfile::TempDir) -> CrashRecovery {
        CrashRecovery::new().with_dump_dir(tmp.path().join("crashes"))
    }

    fn create_test_session() -> Session {
        let mut session = Session::new();
        session.add_message(Message::user("Hello world".to_string()));
        session.add_message(Message::assistant("Hi there!".to_string()));
        session.add_message(Message::user("How are you?".to_string()));
        session
    }

    fn create_session_with_tool_invocations() -> Session {
        let mut session = create_test_session();
        session
            .tool_invocations
            .push(opencode_core::ToolInvocationRecord {
                id: uuid::Uuid::new_v4(),
                tool_name: "read".to_string(),
                arguments: serde_json::json!({"path": "test.txt"}),
                args_hash: "abc123".to_string(),
                result: Some("file content".to_string()),
                started_at: chrono::Utc::now(),
                completed_at: Some(chrono::Utc::now()),
                latency_ms: Some(100),
            });
        session
    }

    #[test]
    fn crash_recovery_set_and_clear_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(recovery.get_active_session().is_none());

        let session = create_test_session();
        recovery.set_active_session(session);
        assert!(recovery.get_active_session().is_some());

        recovery.clear_active_session();
        assert!(recovery.get_active_session().is_none());
    }

    #[test]
    fn crash_recovery_save_dump_requires_active_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let result = recovery.save_crash_dump(Some("test panic".to_string()), None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CrashRecoveryError::NoActiveSession
        ));
    }

    #[test]
    fn crash_recovery_save_and_load_dump() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_test_session();
        session.id = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(
                Some("test panic".to_string()),
                Some("main.rs:100".to_string()),
            )
            .unwrap();

        assert!(path.exists());

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.session_id, "550e8400-e29b-41d4-a716-446655440000");
        assert!(loaded.panic_message.is_some());
        assert!(loaded.stack_trace.is_some());
        assert!(!loaded.messages_summary.is_empty());
    }

    #[test]
    fn crash_recovery_preserves_data_integrity() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let session = create_session_with_tool_invocations();
        let original_id = session.id;
        let original_message_count = session.messages.len();
        let original_tool_count = session.tool_invocations.len();

        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.session_id, original_id.to_string());
        assert_eq!(loaded.messages_summary.len(), original_message_count);
        assert_eq!(loaded.tool_invocations_summary.len(), original_tool_count);
    }

    #[test]
    fn crash_recovery_system_state_consistent_after_recovery() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_test_session();
        session.add_message(Message::system("System prompt".to_string()));
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("crash".to_string()), None)
            .unwrap();

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.state, "Idle");
        assert!(loaded.messages_summary.iter().any(|m| m.role == "System"));
    }

    #[test]
    fn crash_recovery_no_data_loss_multiple_messages() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = Session::new();
        for i in 0..10 {
            session.add_message(Message::user(format!("User message {}", i)));
            session.add_message(Message::assistant(format!("Assistant response {}", i)));
        }

        let original_count = session.messages.len();
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.messages_summary.len(), original_count);
    }

    #[test]
    fn crash_recovery_find_crash_dumps() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap();
        recovery.set_active_session(session2);
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let dumps1 = recovery.find_crash_dumps("11111111-1111-1111-1111-111111111111");
        assert_eq!(dumps1.len(), 1);

        let dumps2 = recovery.find_crash_dumps("22222222-2222-2222-2222-222222222222");
        assert_eq!(dumps2.len(), 1);
    }

    #[test]
    fn crash_recovery_list_recent_crashes() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("aaaaaaa1-1111-1111-1111-111111111111").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic A".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("aaaaaaa2-2222-2222-2222-222222222222").unwrap();
        recovery.set_active_session(session2);
        recovery
            .save_crash_dump(Some("panic B".to_string()), None)
            .unwrap();

        let recent = recovery.list_recent_crashes(10);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn crash_recovery_delete_crash_dump() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_test_session();
        session.id = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddddd01").unwrap();
        recovery.set_active_session(session);
        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        assert!(path.exists());
        recovery.delete_crash_dump(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn crash_recovery_cleanup_session_crashes() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("ccccccc1-cccc-cccc-cccc-cccccccccc01").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("ccccccc2-cccc-cccc-cccc-cccccccccc02").unwrap();
        recovery.set_active_session(session2);
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let count = recovery
            .cleanup_session_crashes("ccccccc1-cccc-cccc-cccc-cccccccccc01")
            .unwrap();
        assert_eq!(count, 1);
        assert!(recovery
            .find_crash_dumps("ccccccc1-cccc-cccc-cccc-cccccccccc01")
            .is_empty());
        assert_eq!(
            recovery
                .find_crash_dumps("ccccccc2-cccc-cccc-cccc-cccccccccc02")
                .len(),
            1
        );
    }

    #[test]
    fn crash_recovery_has_recoverable_crash() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(!recovery.has_recoverable_crash("session-none"));

        let mut session = create_test_session();
        session.id = uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        recovery.set_active_session(session);
        recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        assert!(recovery.has_recoverable_crash("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"));
    }

    #[test]
    fn crash_recovery_recover_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = create_session_with_tool_invocations();
        session.id = uuid::Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbb01").unwrap();
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let recovered = recovery.recover_session(&path).unwrap();
        assert_eq!(recovered.messages.len(), 3);
        assert!(!recovered.tool_invocations.is_empty());
    }

    #[test]
    fn crash_recovery_recover_session_latest() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddd0001").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("dddddddd-dddd-dddd-dddd-dddddddd0002").unwrap();
        recovery.set_active_session(session2);
        std::thread::sleep(std::time::Duration::from_millis(10));
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let recovered = recovery
            .recover_session_latest("dddddddd-dddd-dddd-dddd-dddddddd0002")
            .unwrap();
        assert!(recovered.is_some());
        assert_eq!(recovered.unwrap().messages.len(), 3);
    }

    #[test]
    fn crash_recovery_get_active_session() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        assert!(recovery.get_active_session().is_none());

        let session = create_test_session();
        recovery.set_active_session(session.clone());

        let active = recovery.get_active_session().unwrap();
        assert_eq!(active.messages.len(), 3);
    }

    #[test]
    fn crash_recovery_panic_handler_start_stop() {
        let tmp = tempdir().unwrap();
        let mut handler = PanicHandler::with_crash_recovery(
            CrashRecovery::new().with_dump_dir(tmp.path().join("crashes")),
        );

        handler.start();
        let _ = std::panic::take_hook();
        handler.stop();
    }

    #[test]
    fn crash_recovery_captures_message_previews() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = Session::new();
        session.add_message(Message::user("Short message".to_string()));
        session.add_message(Message::assistant(
            "This is a very long assistant response that definitely exceeds 200 characters and should be truncated during crash dump capture to save space and maintain reasonable dump sizes for proper testing and verification".to_string(),
        ));
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.messages_summary.len(), 2);
        assert_eq!(loaded.messages_summary[1].content_preview, "Short message");
        assert!(loaded.messages_summary[0].content_preview.ends_with("..."));
        assert!(loaded.messages_summary[0].content_preview.len() <= 203);
    }

    #[test]
    fn crash_recovery_tool_invocation_summary() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session = Session::new();
        session.add_message(Message::user("test".to_string()));
        session
            .tool_invocations
            .push(opencode_core::ToolInvocationRecord {
                id: uuid::Uuid::new_v4(),
                tool_name: "read".to_string(),
                arguments: serde_json::json!({"path": "a.txt"}),
                args_hash: "hash1".to_string(),
                result: None,
                started_at: chrono::Utc::now(),
                completed_at: None,
                latency_ms: None,
            });
        session
            .tool_invocations
            .push(opencode_core::ToolInvocationRecord {
                id: uuid::Uuid::new_v4(),
                tool_name: "write".to_string(),
                arguments: serde_json::json!({"path": "b.txt", "content": "hi"}),
                args_hash: "hash2".to_string(),
                result: Some("ok".to_string()),
                started_at: chrono::Utc::now(),
                completed_at: Some(chrono::Utc::now()),
                latency_ms: Some(50),
            });
        recovery.set_active_session(session);

        let path = recovery
            .save_crash_dump(Some("panic".to_string()), None)
            .unwrap();

        let loaded = recovery.load_crash_dump(&path).unwrap();
        assert_eq!(loaded.tool_invocations_summary.len(), 2);
        assert_eq!(loaded.tool_invocations_summary[0].tool_name, "write");
        assert!(loaded.tool_invocations_summary[0].completed_at.is_some());
        assert_eq!(loaded.tool_invocations_summary[1].tool_name, "read");
        assert!(loaded.tool_invocations_summary[1].completed_at.is_none());
    }

    #[test]
    fn crash_recovery_different_session_ids_isolated() {
        let tmp = tempdir().unwrap();
        let recovery = create_test_recovery(&tmp);

        let mut session1 = create_test_session();
        session1.id = uuid::Uuid::parse_str("aaaa0000-0000-0000-0000-000000000001").unwrap();
        recovery.set_active_session(session1);
        recovery
            .save_crash_dump(Some("panic 1".to_string()), None)
            .unwrap();

        let mut session2 = create_test_session();
        session2.id = uuid::Uuid::parse_str("aaaa0000-0000-0000-0000-000000000002").unwrap();
        recovery.set_active_session(session2);
        recovery
            .save_crash_dump(Some("panic 2".to_string()), None)
            .unwrap();

        let dumps1 = recovery.find_crash_dumps("aaaa0000-0000-0000-0000-000000000001");
        let dumps2 = recovery.find_crash_dumps("aaaa0000-0000-0000-0000-000000000002");

        assert_eq!(dumps1.len(), 1);
        assert_eq!(dumps2.len(), 1);
        assert_ne!(dumps1[0], dumps2[0]);
    }
}
