mod common;

#[test]
fn test_session_sharing_serialization_roundtrip() {
    let mut session = opencode_core::Session::new();
    session.add_message(opencode_core::Message::user("Test message".to_string()));

    let json = serde_json::to_string(&session).unwrap();
    let loaded: opencode_core::Session = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.id, session.id);
    assert_eq!(loaded.messages.len(), 1);
    assert_eq!(loaded.messages[0].content, "Test message");
}

#[test]
fn test_session_sharing_unique_ids() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_path = temp_dir.path().join("opencode.db");

    let desktop_session_id = {
        let mut session = opencode_core::Session::new();
        session.add_message(opencode_core::Message::user(
            "Created on desktop".to_string(),
        ));
        session.id.to_string()
    };

    let web_session_id = {
        let mut session = opencode_core::Session::new();
        session.add_message(opencode_core::Message::user("Created on web".to_string()));
        session.id.to_string()
    };

    assert_ne!(desktop_session_id, web_session_id);
}

#[test]
fn test_session_sharing_multiple_sessions() {
    use opencode_core::{Message, Session};

    let mut session1 = Session::new();
    session1.add_message(Message::user("Session 1".to_string()));

    let mut session2 = Session::new();
    session2.add_message(Message::user("Session 2".to_string()));

    assert_eq!(session1.messages.len(), 1);
    assert_eq!(session2.messages.len(), 1);
    assert_ne!(session1.id, session2.id);
}

#[test]
fn test_session_sharing_message_content_preserved() {
    let mut session = opencode_core::Session::new();
    session.add_message(opencode_core::Message::user(
        "Desktop session content".to_string(),
    ));
    session.add_message(opencode_core::Message::assistant(
        "Web response".to_string(),
    ));

    assert_eq!(session.messages.len(), 2);
    assert!(session.messages[0].content.contains("Desktop"));
    assert!(session.messages[1].content.contains("Web"));

    let json = serde_json::to_string(&session).unwrap();
    let loaded: opencode_core::Session = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.messages.len(), 2);
    assert_eq!(loaded.messages[0].content, "Desktop session content");
    assert_eq!(loaded.messages[1].content, "Web response");
}

#[test]
fn test_session_sharing_state_sync() {
    let mut session = opencode_core::Session::new();
    session.add_message(opencode_core::Message::user("Initial message".to_string()));
    let initial_count = session.messages.len();

    session.add_message(opencode_core::Message::assistant(
        "Response added".to_string(),
    ));
    assert_eq!(session.messages.len(), initial_count + 1);

    let json = serde_json::to_string(&session).unwrap();
    let loaded: opencode_core::Session = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.messages.len(), 2);
    assert!(loaded.messages[1].content.contains("Response"));
}
