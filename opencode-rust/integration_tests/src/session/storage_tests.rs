use crate::common::TempProject;
use opencode_core::{Message, Session};

#[test]
fn test_session_create_and_save() {
    let project = TempProject::new();
    let session_path = project.path().join("sessions");

    let mut session = Session::new();
    session.add_message(Message::user("Hello".to_string()));
    session.add_message(Message::assistant("Hi there!".to_string()));

    let save_path = session_path.join("test_session.json");
    session.save(&save_path).expect("Session should save");

    assert!(save_path.exists());
}

#[test]
fn test_session_load() {
    let project = TempProject::new();
    let session_path = project.path().join("sessions");

    let mut original = Session::new();
    original.add_message(Message::user("Test message".to_string()));
    original.add_message(Message::assistant("Response".to_string()));

    let save_path = session_path.join("load_test.json");
    original.save(&save_path).expect("Session should save");

    let loaded = Session::load(&save_path).expect("Session should load");

    assert_eq!(loaded.id, original.id);
    assert_eq!(loaded.messages.len(), original.messages.len());
    assert_eq!(loaded.messages[0].content, "Test message");
    assert_eq!(loaded.messages[1].content, "Response");
}

#[test]
fn test_session_delete() {
    let project = TempProject::new();
    let save_path = project.path().join("delete_test.json");

    let mut session = Session::new();
    session.add_message(Message::user("To be deleted".to_string()));
    session.save(&save_path).expect("Session should save");
    assert!(save_path.exists());

    std::fs::remove_file(&save_path).expect("Should delete file");
    assert!(!save_path.exists());
}

#[test]
fn test_session_list_returns_result() {
    let sessions = Session::list();
    assert!(sessions.is_ok());
}

#[test]
fn test_session_list_empty_when_no_sessions() {
    let sessions = Session::list().unwrap_or_default();
    let initial_count = sessions.len();

    let project = TempProject::new();
    let mut session = Session::new();
    session.add_message(Message::user("Test".to_string()));
    session
        .save(&project.path().join("outside_default.json"))
        .ok();

    let sessions_after = Session::list().unwrap_or_default();
    assert_eq!(sessions_after.len(), initial_count);
}

#[test]
fn test_session_fork() {
    let project = TempProject::new();

    let mut parent = Session::new();
    parent.add_message(Message::user("Parent message".to_string()));
    parent.add_message(Message::assistant("Parent response".to_string()));

    let save_path = project.path().join("parent.json");
    parent.save(&save_path).expect("Should save");

    let child = parent.fork_at_message(0).expect("Should fork at message 0");

    assert_ne!(child.id, parent.id);
    assert_eq!(child.messages.len(), 1);
    assert_eq!(child.parent_session_id, Some(parent.id.to_string()));
}

#[test]
fn test_session_undo_redo() {
    let mut session = Session::new();

    session.add_message(Message::user("First".to_string()));
    assert_eq!(session.messages.len(), 1);

    session.add_message(Message::assistant("Second".to_string()));
    assert_eq!(session.messages.len(), 2);

    session.undo(1).expect("Should undo");
    assert_eq!(session.messages.len(), 1);

    session.redo(1).expect("Should redo");
    assert_eq!(session.messages.len(), 2);
}

#[test]
fn test_session_truncate() {
    let mut session = Session::new();
    session.add_message(Message::user("A".repeat(100)));
    session.add_message(Message::assistant("B".repeat(100)));

    let original_len = session.messages.len();
    session.truncate_for_context(10);

    assert!(session.messages.len() < original_len);
}

#[test]
fn test_session_export_json() {
    let mut session = Session::new();
    session.add_message(Message::user("Export me".to_string()));
    session.add_message(Message::assistant("Exported!".to_string()));

    let json = session.export_json().expect("Should export JSON");

    assert!(json.contains("Export me"));
    assert!(json.contains("Exported!"));
    assert!(json.contains("User"));
    assert!(json.contains("Assistant"));
}

#[test]
fn test_session_export_markdown() {
    let mut session = Session::new();
    session.add_message(Message::user("Export me".to_string()));
    session.add_message(Message::assistant("Exported!".to_string()));

    let md = session.export_markdown().expect("Should export Markdown");

    assert!(md.contains("Export me"));
    assert!(md.contains("Exported!"));
    assert!(md.contains("**User**"));
    assert!(md.contains("**Assistant**"));
}

#[test]
fn test_session_sanitize_content_in_export() {
    let mut session = Session::new();
    let content_with_key = "My API key is sk-1234567890abcdefghij";
    session.add_message(Message::user(content_with_key.to_string()));

    let json = session.export_json().expect("Should export JSON");

    assert!(!json.contains("sk-1234567890"));
    assert!(json.contains("[REDACTED"));
}

#[test]
fn test_session_id_persistence() {
    let project = TempProject::new();

    let mut session = Session::new();
    let original_id = session.id;
    session.add_message(Message::user("Test".to_string()));

    let save_path = project.path().join("id_persist.json");
    session.save(&save_path).expect("Should save");

    let loaded = Session::load(&save_path).expect("Should load");
    assert_eq!(loaded.id, original_id);
}

#[test]
fn test_multiple_session_save_load_cycle() {
    let project = TempProject::new();

    for i in 0..5 {
        let mut session = Session::new();
        session.add_message(Message::user(format!("Message {}", i)));
        let path = project.path().join(format!("session_{}.json", i));
        session.save(&path).expect("Should save");
    }

    for i in 0..5 {
        let path = project.path().join(format!("session_{}.json", i));
        let loaded = Session::load(&path).expect("Should load");
        assert_eq!(loaded.messages[0].content, format!("Message {}", i));
    }
}
