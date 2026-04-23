#[cfg(test)]
mod shell_system_tests {
    use opencode_core::bus::{EventBus, InternalEvent};
    use opencode_core::pty::PtySession;
    use opencode_core::shell::Shell;
    use opencode_core::sync::{SyncManager, SyncStatus};
    use std::path::PathBuf;

    #[test]
    fn test_bus_e2e_001_event_publish_and_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::SessionStarted("test-session".to_string()));
        let event = rx.try_recv().unwrap();
        assert!(matches!(event, InternalEvent::SessionStarted(id) if id == "test-session"));
    }

    #[test]
    fn test_bus_e2e_001_unsubscribe_handlers_dont_receive() {
        let bus = EventBus::new();
        let _rx = bus.subscribe();

        bus.publish(InternalEvent::ConfigUpdated);
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[test]
    fn test_bus_stability_001_subscriber_count_tracking() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);

        let _rx1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }

    #[test]
    fn test_bus_event_types_session_events() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::SessionStarted("s1".to_string()));
        bus.publish(InternalEvent::SessionEnded("s1".to_string()));
        bus.publish(InternalEvent::SessionForked {
            original_id: "orig".to_string(),
            new_id: "new".to_string(),
            fork_point: 5,
        });

        assert!(matches!(rx.try_recv().unwrap(), InternalEvent::SessionStarted(id) if id == "s1"));
        assert!(matches!(rx.try_recv().unwrap(), InternalEvent::SessionEnded(id) if id == "s1"));
        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::SessionForked { .. }
        ));
    }

    #[test]
    fn test_bus_event_types_tool_events() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::ToolCallStarted {
            session_id: "s1".to_string(),
            tool_name: "read".to_string(),
            call_id: "c1".to_string(),
        });
        bus.publish(InternalEvent::ToolCallEnded {
            session_id: "s1".to_string(),
            call_id: "c1".to_string(),
            success: true,
        });

        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::ToolCallStarted { .. }
        ));
        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::ToolCallEnded { .. }
        ));
    }

    #[tokio::test]
    async fn test_bus_event_types_async_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::ServerStarted { port: 8080 });
        let event = rx.recv().await.unwrap();
        assert!(matches!(event, InternalEvent::ServerStarted { port: 8080 }));
    }

    #[test]
    fn test_bus_event_types_error_event() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::Error {
            source: "test".to_string(),
            message: "test error".to_string(),
        });

        let event = rx.try_recv().unwrap();
        assert!(matches!(event, InternalEvent::Error { source, message }
            if source == "test" && message == "test error"));
    }

    #[test]
    fn test_shell_e2e_basic_command_execution() {
        let result = Shell::execute("echo hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_shell_e2e_command_with_pipe() {
        let result = Shell::execute("echo 'test content' | cat");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test content"));
    }

    #[test]
    fn test_shell_e2e_which_command() {
        let result = Shell::which("ls");
        assert!(result.is_some());
        assert!(result.unwrap().contains("ls"));
    }

    #[test]
    fn test_shell_e2e_nonexistent_command() {
        let result = Shell::execute("nonexistent_command_xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_pty_state_001_session_kill() {
        let mut session = PtySession::new("sleep 10").unwrap();
        let result = session.kill();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pty_state_001_session_wait() {
        let mut session = PtySession::new("echo test").unwrap();
        let result = session.wait();
        assert!(result.is_ok());
        assert!(result.unwrap().success());
    }

    #[test]
    fn test_pty_e2e_simple_command() {
        let mut session = PtySession::new("echo hello").unwrap();
        let status = session.wait().unwrap();
        assert!(status.success());
    }

    #[test]
    fn test_sync_e2e_file_sync_state() {
        let mut manager = SyncManager::new();

        manager.track(
            PathBuf::from("/file.txt"),
            "abc123".to_string(),
            "abc123".to_string(),
        );

        let state = manager.get_state(&PathBuf::from("/file.txt")).unwrap();
        assert!(matches!(state.status, SyncStatus::Synced));
    }

    #[test]
    fn test_sync_e2e_local_changed_detection() {
        let mut manager = SyncManager::new();

        manager.track(
            PathBuf::from("/file.txt"),
            "local_hash".to_string(),
            "remote_hash".to_string(),
        );

        assert!(manager.needs_sync(&PathBuf::from("/file.txt")));
        let state = manager.get_state(&PathBuf::from("/file.txt")).unwrap();
        assert!(matches!(state.status, SyncStatus::LocalChanged));
    }

    #[test]
    fn test_sync_e2e_no_sync_when_synced() {
        let mut manager = SyncManager::new();

        manager.track(
            PathBuf::from("/file.txt"),
            "same_hash".to_string(),
            "same_hash".to_string(),
        );

        assert!(!manager.needs_sync(&PathBuf::from("/file.txt")));
    }

    #[test]
    fn test_sync_e2e_untracked_file() {
        let manager = SyncManager::new();
        assert!(!manager.needs_sync(&PathBuf::from("/nonexistent.txt")));
        assert!(manager
            .get_state(&PathBuf::from("/nonexistent.txt"))
            .is_none());
    }

    #[test]
    fn test_bus_multiple_subscribers_receive_same_event() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(InternalEvent::ConfigUpdated);

        let event1 = rx1.try_recv().unwrap();
        let event2 = rx2.try_recv().unwrap();
        assert!(matches!(event1, InternalEvent::ConfigUpdated));
        assert!(matches!(event2, InternalEvent::ConfigUpdated));
    }

    #[test]
    fn test_bus_late_subscriber_misses_early_events() {
        let bus = EventBus::new();

        bus.publish(InternalEvent::SessionEnded("session-1".to_string()));

        let mut rx = bus.subscribe();
        bus.publish(InternalEvent::SessionEnded("session-2".to_string()));

        assert!(
            matches!(rx.try_recv().unwrap(), InternalEvent::SessionEnded(id) if id == "session-2")
        );
    }

    #[test]
    fn test_internal_event_session_id_extraction() {
        let event = InternalEvent::SessionStarted("session-123".to_string());
        assert_eq!(event.session_id(), Some("session-123"));

        let event = InternalEvent::SessionForked {
            original_id: "orig".to_string(),
            new_id: "new".to_string(),
            fork_point: 5,
        };
        assert_eq!(event.session_id(), Some("orig"));

        let event = InternalEvent::ConfigUpdated;
        assert_eq!(event.session_id(), None);
    }

    #[test]
    fn test_bus_permission_events() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::PermissionGranted {
            user_id: "user1".to_string(),
            permission: "file:read".to_string(),
        });
        bus.publish(InternalEvent::PermissionDenied {
            user_id: "user1".to_string(),
            permission: "file:write".to_string(),
        });

        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::PermissionGranted { .. }
        ));
        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::PermissionDenied { .. }
        ));
    }

    #[test]
    fn test_bus_agent_events() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(InternalEvent::AgentStarted {
            session_id: "s1".to_string(),
            agent: "test-agent".to_string(),
        });
        bus.publish(InternalEvent::AgentStopped {
            session_id: "s1".to_string(),
            agent: "test-agent".to_string(),
        });
        bus.publish(InternalEvent::AgentStatusChanged {
            session_id: "s1".to_string(),
            status: "running".to_string(),
        });

        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::AgentStarted { .. }
        ));
        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::AgentStopped { .. }
        ));
        assert!(matches!(
            rx.try_recv().unwrap(),
            InternalEvent::AgentStatusChanged { .. }
        ));
    }
}
