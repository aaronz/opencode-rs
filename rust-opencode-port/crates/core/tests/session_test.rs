#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    #[test]
    fn test_session_new() {
        let session = opencode_core::session::Session::new();
        assert!(!session.id.to_string().is_empty());
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_session_add_message() {
        let mut session = opencode_core::session::Session::new();
        session.add_message(opencode_core::message::Message::user("Hello"));
        assert_eq!(session.messages.len(), 1);
    }

    #[test]
    fn test_session_save_load() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("session.json");

        let mut session = opencode_core::session::Session::new();
        session.add_message(opencode_core::message::Message::user("Test message"));
        session.add_message(opencode_core::message::Message::assistant("Test response"));

        session.save(&filepath).unwrap();

        let loaded = opencode_core::session::Session::load(&filepath).unwrap();
        assert_eq!(loaded.messages.len(), 2);
    }

    #[test]
    fn test_message_user() {
        let msg = opencode_core::message::Message::user("test");
        assert!(matches!(msg.role, opencode_core::message::Role::User));
        assert_eq!(msg.content, "test");
    }

    #[test]
    fn test_message_assistant() {
        let msg = opencode_core::message::Message::assistant("response");
        assert!(matches!(msg.role, opencode_core::message::Role::Assistant));
        assert_eq!(msg.content, "response");
    }

    #[test]
    fn test_message_system() {
        let msg = opencode_core::message::Message::system("system prompt");
        assert!(matches!(msg.role, opencode_core::message::Role::System));
        assert_eq!(msg.content, "system prompt");
    }

    #[test]
    fn test_id_new_uuid() {
        let id = opencode_core::id::IdGenerator::new_uuid();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_id_new_short() {
        let id = opencode_core::id::IdGenerator::new_short();
        assert!(!id.is_empty());
        assert!(id.len() < 20);
    }

    #[test]
    fn test_config_default() {
        let _config = opencode_core::config::Config::default();
    }
}
