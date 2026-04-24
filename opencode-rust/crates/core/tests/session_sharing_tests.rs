#[cfg(test)]
mod session_sharing_tests {
    use opencode_core::bus::EventBus;
    use opencode_core::message::Message;
    
    use opencode_core::session_sharing::{SessionSharing, SharingError};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_sharing() -> (SessionSharing, TempDir) {
        let temp = TempDir::new().unwrap();
        let path = temp.path().to_path_buf();
        let event_bus = Arc::new(EventBus::new());
        let sharing = SessionSharing::new(path, event_bus);
        (sharing, temp)
    }

    #[test]
    fn test_session_sharing_basic_create_and_retrieve() {
        let (sharing, _temp) = create_test_sharing();

        let session = sharing
            .create_session(Some("Test session".to_string()))
            .unwrap();
        assert!(!session.id.is_nil());
        assert_eq!(session.messages.len(), 1);

        let retrieved = sharing.get_session(&session.id).unwrap();
        assert_eq!(retrieved.id, session.id);
    }

    #[test]
    fn test_session_sharing_cross_interface_access() {
        let (sharing1, _temp) = create_test_sharing();
        let mut session = sharing1.create_session(None).unwrap();
        session.add_message(Message::user("Created in interface 1".to_string()));
        sharing1.save_session(&session).unwrap();

        let sharing2 = sharing1.clone();
        let retrieved = sharing2.get_session(&session.id).unwrap();
        assert_eq!(retrieved.messages.len(), 2);
        assert_eq!(retrieved.messages[1].content, "Created in interface 1");
    }

    #[test]
    fn test_session_sharing_state_synchronization() {
        let (sharing, _temp) = create_test_sharing();

        let session = sharing.create_session(None).unwrap();
        let id = session.id;

        let v1 = sharing.get_version(&id).unwrap();

        let mut s = sharing.get_session(&id).unwrap();
        s.add_message(Message::user("Synced message".to_string()));
        sharing.save_session(&s).unwrap();

        let v2 = sharing.get_version(&id).unwrap();
        assert_eq!(v2, v1 + 1);

        let reloaded = sharing.reload_session(&id).unwrap();
        assert_eq!(reloaded.messages.len(), 2);
    }

    #[test]
    fn test_session_sharing_no_conflicts_single_thread() {
        let (sharing, _temp) = create_test_sharing();

        let session = sharing.create_session(None).unwrap();
        let id = session.id;

        let s1 = sharing.get_session(&id).unwrap();
        let mut s1_modified = s1.clone();
        s1_modified.add_message(Message::user("From thread 1".to_string()));
        sharing.save_session(&s1_modified).unwrap();

        let s2 = sharing.get_session(&id).unwrap();
        let mut s2_modified = s2.clone();
        s2_modified.add_message(Message::user("From thread 2".to_string()));
        sharing.save_session(&s2_modified).unwrap();

        let final_session = sharing.get_session(&id).unwrap();
        assert_eq!(final_session.messages.len(), 3);
    }

    #[test]
    fn test_session_sharing_multiple_sessions() {
        let (sharing, _temp) = create_test_sharing();

        let session1 = sharing.create_session(None).unwrap();
        let session2 = sharing.create_session(None).unwrap();
        let session3 = sharing.create_session(None).unwrap();

        let sessions = sharing.list_sessions().unwrap();
        assert_eq!(sessions.len(), 3);

        assert!(sharing.exists(&session1.id));
        assert!(sharing.exists(&session2.id));
        assert!(sharing.exists(&session3.id));

        assert_ne!(session1.id, session2.id);
        assert_ne!(session2.id, session3.id);
        assert_ne!(session1.id, session3.id);
    }

    #[test]
    fn test_session_sharing_unique_ids() {
        let (sharing, _temp) = create_test_sharing();

        let desktop_session = sharing
            .create_session(Some("Desktop session".to_string()))
            .unwrap();
        let web_session = sharing
            .create_session(Some("Web session".to_string()))
            .unwrap();

        assert_ne!(desktop_session.id, web_session.id);

        let desktop_retrieved = sharing.get_session(&desktop_session.id).unwrap();
        let web_retrieved = sharing.get_session(&web_session.id).unwrap();

        assert_eq!(desktop_retrieved.id, desktop_session.id);
        assert_eq!(web_retrieved.id, web_session.id);
    }

    #[test]
    fn test_session_sharing_message_content_preserved() {
        let (sharing, _temp) = create_test_sharing();

        let mut session = sharing.create_session(None).unwrap();
        session.add_message(Message::user("Desktop content".to_string()));
        session.add_message(Message::assistant("Web response".to_string()));
        session.add_message(Message::user("Another user message".to_string()));
        sharing.save_session(&session).unwrap();

        let sharing2 = sharing.clone();
        let retrieved = sharing2.get_session(&session.id).unwrap();

        assert_eq!(retrieved.messages.len(), 4);
        assert!(retrieved.messages[1].content.contains("Desktop"));
        assert!(retrieved.messages[2].content.contains("Web"));
        assert!(retrieved.messages[3].content.contains("Another"));
    }

    #[test]
    fn test_session_sharing_delete_and_verify() {
        let (sharing, _temp) = create_test_sharing();

        let session = sharing.create_session(None).unwrap();
        let id = session.id;

        assert!(sharing.exists(&id));

        sharing.delete_session(&id).unwrap();

        assert!(!sharing.exists(&id));

        let result = sharing.get_session(&id);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SharingError::SessionNotFound(_)
        ));
    }

    #[test]
    fn test_session_sharing_fork_inheritance() {
        let (sharing, _temp) = create_test_sharing();

        let mut parent = sharing.create_session(None).unwrap();
        parent.add_message(Message::user("Parent message".to_string()));
        sharing.save_session(&parent).unwrap();

        let child_id = uuid::Uuid::new_v4();
        let child = sharing.fork_session(&parent.id, child_id).unwrap();

        assert_ne!(child.id, parent.id);
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent.id.to_string().as_str())
        );
        assert_eq!(child.messages.len(), 2);
        assert_eq!(child.messages[1].content, "Parent message");

        let child_retrieved = sharing.get_session(&child.id).unwrap();
        assert_eq!(child_retrieved.parent_session_id, child.parent_session_id);
    }

    #[test]
    fn test_session_sharing_event_bus_integration() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().to_path_buf();
        let event_bus = Arc::new(EventBus::new());
        let sharing = SessionSharing::new(path, event_bus.clone());

        let mut rx = event_bus.subscribe();
        let session = sharing.create_session(None).unwrap();
        let session_id = session.id.to_string();

        drop(session);

        let event = rx.try_recv();
        assert!(event.is_ok());
        match event.unwrap() {
            opencode_core::bus::InternalEvent::SessionStarted(id) => {
                assert_eq!(id, session_id);
            }
            _ => panic!("Expected SessionStarted event"),
        }
    }

    #[test]
    fn test_session_sharing_reload_persists_changes() {
        let (sharing, _temp) = create_test_sharing();

        let session = sharing.create_session(None).unwrap();
        let id = session.id;

        let mut s = sharing.get_session(&id).unwrap();
        s.add_message(Message::user("Persistent data".to_string()));
        sharing.save_session(&s).unwrap();

        sharing.clear_cache().unwrap();

        assert!(sharing.exists(&id));

        let reloaded = sharing.reload_session(&id).unwrap();
        assert_eq!(reloaded.messages.len(), 2);
        assert_eq!(reloaded.messages[1].content, "Persistent data");
    }

    #[test]
    fn test_session_sharing_concurrent_access_same_version() {
        let (sharing, _temp) = create_test_sharing();

        let session = sharing.create_session(None).unwrap();
        let id = session.id;

        let version_before = sharing.get_version(&id).unwrap();

        let s1 = sharing.get_session(&id).unwrap();
        let s2 = sharing.get_session(&id).unwrap();

        assert_eq!(sharing.get_version(&id).unwrap(), version_before);

        drop(s1);
        drop(s2);

        assert_eq!(sharing.get_version(&id).unwrap(), version_before);
    }

    #[test]
    fn test_session_sharing_cloned_sharing_independent_cache() {
        let (sharing1, _temp) = create_test_sharing();
        let session = sharing1.create_session(None).unwrap();

        sharing1
            .get_session(&session.id)
            .unwrap()
            .add_message(Message::user("From sharing1".to_string()));

        let sharing2 = sharing1.clone();
        let from_sharing2 = sharing2.get_session(&session.id).unwrap();
        assert_eq!(from_sharing2.messages.len(), 1);
    }

    #[test]
    fn test_session_sharing_list_sorted_by_updated() {
        let (sharing, _temp) = create_test_sharing();

        let session1 = sharing.create_session(None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let mut session2 = sharing.create_session(None).unwrap();
        session2.add_message(Message::user("Newer".to_string()));
        sharing.save_session(&session2).unwrap();

        let sessions = sharing.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, session2.id);
        assert_eq!(sessions[1].id, session1.id);
    }

    #[test]
    fn test_session_sharing_session_path_storage() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().to_path_buf();
        let event_bus = Arc::new(EventBus::new());
        let sharing = SessionSharing::new(path.clone(), event_bus);

        let session = sharing.create_session(None).unwrap();

        let stored_path = sharing.session_path(&session.id);
        assert!(stored_path.starts_with(&path));
        assert!(stored_path.exists());
    }
}
