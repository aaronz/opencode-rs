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

    #[test]
    fn test_session_fork() {
        use opencode_core::message::Message;
        use opencode_core::session::Session;

        let mut parent = Session::new();
        parent.add_message(Message::user("Hello"));
        parent.add_message(Message::assistant("Hi there"));

        let child_id = uuid::Uuid::new_v4();
        let child = parent.fork(child_id);

        assert!(child.parent_session_id.is_some());
        assert_eq!(child.parent_session_id.unwrap(), parent.id);
        assert_eq!(child.messages.len(), 2);
        assert_eq!(child.id, child_id);
        assert!(child.fork_history.is_empty());
    }

    #[test]
    fn test_session_estimate_tokens() {
        use opencode_core::message::Message;
        use opencode_core::session::Session;

        let mut session = Session::new();
        session.add_message(Message::user("hello world"));
        let tokens = session.estimate_token_count();
        assert!(tokens > 0);
    }

    #[test]
    fn test_session_compaction_status() {
        use opencode_core::compaction::CompactionTrigger;
        use opencode_core::message::Message;
        use opencode_core::session::Session;

        let mut session = Session::new();
        let status = session.get_compaction_status();
        assert_eq!(status.trigger, CompactionTrigger::None);
        assert!(!status.needs_attention);
    }

    #[test]
    fn test_command_expand_env_vars() {
        use opencode_core::command::{CommandDefinition, CommandVariables};
        use std::path::Path;

        std::env::set_var("TEST_CMD_VAR", "hello_from_env");
        let def = CommandDefinition {
            name: "test".to_string(),
            description: "Test".to_string(),
            triggers: vec![],
            agent: None,
            model: None,
            template: "Run with ${env:TEST_CMD_VAR}".to_string(),
        };
        let vars = CommandVariables::default();
        let expanded = def.expand(&vars);
        assert!(expanded.contains("hello_from_env"));
        std::env::remove_var("TEST_CMD_VAR");
    }

    #[test]
    fn test_command_expand_cursor() {
        use opencode_core::command::{CommandDefinition, CommandVariables};

        let def = CommandDefinition {
            name: "test".to_string(),
            description: "Test".to_string(),
            triggers: vec![],
            agent: None,
            model: None,
            template: "At cursor ${cursor}".to_string(),
        };
        let mut vars = CommandVariables::default();
        vars.cursor = "line:10,col:5".to_string();
        let expanded = def.expand(&vars);
        assert!(expanded.contains("line:10,col:5"));
    }

    #[test]
    fn test_scroll_acceleration_deserialize_both_formats() {
        use opencode_core::config::ScrollAccelerationConfig;

        let legacy: ScrollAccelerationConfig = serde_json::from_str("2.5").unwrap();
        assert!(legacy.enabled);
        assert_eq!(legacy.speed, Some(2.5));

        let new_fmt: ScrollAccelerationConfig =
            serde_json::from_str(r#"{"enabled":true,"speed":3.0}"#).unwrap();
        assert!(new_fmt.enabled);
        assert_eq!(new_fmt.speed, Some(3.0));
    }

    #[test]
    fn test_keybind_merge_no_conflict() {
        use opencode_core::config::KeybindConfig;
        use std::collections::HashMap;

        let defaults = KeybindConfig {
            commands: Some("Ctrl+K"),
            timeline: Some("Ctrl+T"),
            settings: None,
            models: None,
            files: None,
            terminal: None,
            custom: None,
        };

        let user = KeybindConfig {
            commands: None,
            timeline: None,
            settings: Some("Ctrl+S"),
            models: None,
            files: None,
            terminal: None,
            custom: Some(HashMap::from([(
                "custom_action".to_string(),
                "Ctrl+X".to_string(),
            )])),
        };

        let (merged, conflicts) = user.merge_with_defaults(&defaults);
        assert!(conflicts.is_empty());
        assert_eq!(merged.commands, Some("Ctrl+K".to_string()));
        assert_eq!(merged.settings, Some("Ctrl+S".to_string()));
    }

    #[test]
    fn test_keybind_merge_with_conflict() {
        use opencode_core::config::KeybindConfig;
        use std::collections::HashMap;

        let defaults = KeybindConfig {
            commands: Some("Ctrl+K"),
            timeline: Some("Ctrl+T"),
            settings: None,
            models: None,
            files: None,
            terminal: None,
            custom: None,
        };

        let user = KeybindConfig {
            commands: None,
            timeline: None,
            settings: None,
            models: None,
            files: None,
            terminal: None,
            custom: Some(HashMap::from([(
                "commands".to_string(),
                "Ctrl+J".to_string(),
            )])),
        };

        let (_merged, conflicts) = user.merge_with_defaults(&defaults);
        assert!(!conflicts.is_empty());
        assert!(conflicts[0].contains("commands"));
    }

    #[test]
    fn test_share_export_json() {
        use opencode_core::message::Message;
        use opencode_core::session::Session;
        use opencode_core::share::{ExportFormat, ExportOptions, ShareManager};

        let mut session = Session::new();
        session.add_message(Message::user("Hello"));

        let mgr = ShareManager::new();
        let opts = ExportOptions {
            include_metadata: false,
            sanitize_sensitive: false,
            format: ExportFormat::Json,
        };
        let output = mgr.export_session(&session, &opts);
        assert!(output.contains("Hello"));
    }

    #[test]
    fn test_share_export_markdown() {
        use opencode_core::message::Message;
        use opencode_core::session::Session;
        use opencode_core::share::{ExportFormat, ExportOptions, ShareManager};

        let mut session = Session::new();
        session.add_message(Message::user("Test message"));

        let mgr = ShareManager::new();
        let opts = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: false,
            format: ExportFormat::Markdown,
        };
        let output = mgr.export_session(&session, &opts);
        assert!(output.contains("# Session"));
        assert!(output.contains("**User**"));
        assert!(output.contains("Test message"));
    }

    #[test]
    fn test_share_sanitize_sensitive() {
        use opencode_core::message::Message;
        use opencode_core::session::Session;
        use opencode_core::share::{ExportFormat, ExportOptions, ShareManager};

        let mut session = Session::new();
        session.add_message(Message::user("My API key is sk-abc123secret"));

        let mgr = ShareManager::new();
        let opts = ExportOptions {
            include_metadata: true,
            sanitize_sensitive: true,
            format: ExportFormat::Json,
        };
        let output = mgr.export_session(&session, &opts);
        assert!(!output.contains("abc123secret"));
        assert!(output.contains("[REDACTED]"));
    }
}
