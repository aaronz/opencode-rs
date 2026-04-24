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
        assert_eq!(child.parent_session_id.unwrap(), parent.id.to_string());
        assert_eq!(child.messages.len(), 2);
        assert_eq!(child.id, child_id);
        assert!(child.fork_history.is_empty());
    }

    #[test]
    fn test_session_fork_at_message() {
        use opencode_core::message::Message;
        use opencode_core::session::Session;

        let mut parent = Session::new();
        parent.add_message(Message::user("one"));
        parent.add_message(Message::assistant("two"));
        parent.add_message(Message::user("three"));

        let child = parent.fork_at_message(1).unwrap();
        let parent_id = parent.id.to_string();
        assert_eq!(child.messages.len(), 2);
        assert_eq!(child.messages[1].content, "two");
        assert_eq!(child.parent_session_id.as_deref(), Some(parent_id.as_str()));
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
        use opencode_core::session::Session;

        let session = Session::new();
        let status = session.get_compaction_status();
        assert_eq!(status.trigger, CompactionTrigger::None);
        assert!(!status.needs_attention);
    }

    #[test]
    fn test_command_expand_env_vars() {
        use opencode_core::command::{CommandDefinition, CommandVariables};

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
        let vars = CommandVariables {
            cursor: "line:10,col:5".to_string(),
            ..Default::default()
        };
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
            commands: Some("Ctrl+K".to_string()),
            timeline: Some("Ctrl+T".to_string()),
            settings: None,
            models: None,
            files: None,
            terminal: None,
            custom: None,
        };

        let user = KeybindConfig {
            commands: None,
            timeline: None,
            settings: Some("Ctrl+S".to_string()),
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
            commands: Some("Ctrl+K".to_string()),
            timeline: Some("Ctrl+T".to_string()),
            settings: None,
            models: None,
            files: None,
            terminal: None,
            custom: None,
        };

        let user = KeybindConfig {
            commands: None,
            timeline: None,
            settings: Some("Ctrl+K".to_string()),
            models: None,
            files: None,
            terminal: None,
            custom: Some(HashMap::from([(
                "my_action".to_string(),
                "Ctrl+K".to_string(),
            )])),
        };

        let (_merged, conflicts) = user.merge_with_defaults(&defaults);
        assert!(!conflicts.is_empty());
        assert!(conflicts
            .iter()
            .any(|c| c.contains("Ctrl+K used by both 'commands' and 'settings'")));
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

#[cfg(test)]
mod id_visibility_tests {
    use opencode_core::{IdGenerator, IdParseError, ProjectId, SessionId, UserId};

    #[test]
    fn test_id_generator_public_access() {
        let uuid = IdGenerator::new_uuid();
        assert_eq!(uuid.len(), 36);
    }

    #[test]
    fn test_id_generator_short_public_access() {
        let short = IdGenerator::new_short();
        assert_eq!(short.len(), 8);
    }

    #[test]
    fn test_id_generator_timestamped_public_access() {
        let timestamped = IdGenerator::new_timestamped();
        assert!(timestamped.contains('-'));
    }

    #[test]
    fn test_session_id_public_re_export() {
        let session_id = SessionId::new();
        assert!(!session_id.to_string().is_empty());
        assert!(session_id.to_string().starts_with("session:"));
    }

    #[test]
    fn test_user_id_public_re_export() {
        let user_id = UserId::new();
        assert!(!user_id.to_string().is_empty());
        assert!(user_id.to_string().starts_with("user:"));
    }

    #[test]
    fn test_project_id_public_re_export() {
        let project_id = ProjectId::new();
        assert!(!project_id.to_string().is_empty());
        assert!(project_id.to_string().starts_with("project:"));
    }

    #[test]
    fn test_session_id_roundtrip_public() {
        let session_id = SessionId::new();
        let parsed: SessionId = session_id.to_string().parse().unwrap();
        assert_eq!(session_id, parsed);
    }

    #[test]
    fn test_user_id_roundtrip_public() {
        let user_id = UserId::new();
        let parsed: UserId = user_id.to_string().parse().unwrap();
        assert_eq!(user_id, parsed);
    }

    #[test]
    fn test_project_id_roundtrip_public() {
        let project_id = ProjectId::new();
        let parsed: ProjectId = project_id.to_string().parse().unwrap();
        assert_eq!(project_id, parsed);
    }

    #[test]
    fn test_id_parse_error_public_access() {
        let result: Result<SessionId, IdParseError> = "invalid-uuid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_cross_prefix_rejected_public() {
        let session_id = SessionId::new();
        let result: Result<UserId, IdParseError> = session_id.to_string().parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_id_types_implement_send() {
        fn assert_send<T: Send>() {}
        assert_send::<SessionId>();
        assert_send::<UserId>();
        assert_send::<ProjectId>();
    }

    #[test]
    fn test_id_types_implement_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<SessionId>();
        assert_sync::<UserId>();
        assert_sync::<ProjectId>();
    }

    #[test]
    fn test_session_id_default_public() {
        let id: SessionId = Default::default();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn test_user_id_default_public() {
        let id: UserId = Default::default();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn test_project_id_default_public() {
        let id: ProjectId = Default::default();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn test_session_e2e_006_tool_invocation_recording() {
        use opencode_core::session::{Session, ToolInvocationRecord};
        use sha2::{Digest, Sha256};

        let mut session = Session::new();

        let args = serde_json::json!({"path": "/tmp/test.txt"});
        let args_str = args.to_string();

        let mut hasher = Sha256::new();
        hasher.update(args_str.as_bytes());
        let expected_hash = format!("{:x}", hasher.finalize());

        let started_at = chrono::Utc::now();
        let completed_at = started_at + chrono::Duration::milliseconds(150);

        let record = ToolInvocationRecord {
            id: uuid::Uuid::new_v4(),
            tool_name: "read".to_string(),
            arguments: args,
            args_hash: expected_hash.clone(),
            result: Some("file content".to_string()),
            started_at,
            completed_at: Some(completed_at),
            latency_ms: Some(150),
        };

        session.tool_invocations.push(record.clone());

        assert_eq!(session.tool_invocations.len(), 1);
        assert_eq!(session.tool_invocations[0].tool_name, "read");
        assert_eq!(session.tool_invocations[0].args_hash, expected_hash);
        assert_eq!(session.tool_invocations[0].latency_ms, Some(150));

        let empty_args = serde_json::json!({});
        let mut hasher2 = Sha256::new();
        hasher2.update(empty_args.to_string().as_bytes());
        let empty_hash = format!("{:x}", hasher2.finalize());
        assert!(!empty_hash.is_empty());

        let large_latency_record = ToolInvocationRecord {
            id: uuid::Uuid::new_v4(),
            tool_name: "bash".to_string(),
            arguments: serde_json::json!({"cmd": "sleep 1000"}),
            args_hash: empty_hash.clone(),
            result: None,
            started_at,
            completed_at: None,
            latency_ms: Some(u64::MAX),
        };
        session.tool_invocations.push(large_latency_record);
        assert_eq!(session.tool_invocations.len(), 2);
        assert_eq!(session.tool_invocations[1].latency_ms, Some(u64::MAX));
    }

    #[test]
    fn test_session_conc_001_fork_and_undo_operations() {
        use opencode_core::session::Session;

        let mut session = Session::new();

        for i in 0..10 {
            session.add_message(opencode_core::message::Message::user(format!(
                "message {}",
                i
            )));
        }

        assert_eq!(session.messages.len(), 10);

        let child = session.fork_at_message(5).unwrap();
        assert_eq!(child.messages.len(), 6);
        assert_eq!(session.messages.len(), 10);

        let undo_result = session.undo(3);
        assert!(undo_result.is_ok());
        assert_eq!(undo_result.unwrap(), 3);
        assert_eq!(session.messages.len(), 7);

        let redo_result = session.redo(1);
        assert!(redo_result.is_ok());
        assert_eq!(session.messages.len(), 8);

        session.add_message(opencode_core::message::Message::user(
            "after concurrent ops".to_string(),
        ));
        assert_eq!(session.messages.len(), 9);

        let another_fork = session.fork_at_message(4).unwrap();
        assert_eq!(another_fork.messages.len(), 5);
    }

    #[test]
    fn test_session_state_002_undo_does_not_cross_fork_boundary() {
        use opencode_core::session::Session;

        let mut parent = Session::new();
        for i in 0..5 {
            parent.add_message(opencode_core::message::Message::user(format!(
                "parent message {}",
                i
            )));
        }
        assert_eq!(parent.messages.len(), 5);
        let parent_message_count = parent.messages.len();
        let parent_undo_len = parent.undo_history.len();

        let mut child = parent.fork_at_message(3).unwrap();
        assert_eq!(child.messages.len(), 4);
        assert_eq!(
            child.parent_session_id.as_deref(),
            Some(parent.id.to_string().as_str())
        );
        assert!(child.undo_history.is_empty());

        child.add_message(opencode_core::message::Message::user(
            "child message 1".to_string(),
        ));
        child.add_message(opencode_core::message::Message::user(
            "child message 2".to_string(),
        ));
        assert_eq!(child.messages.len(), 6);
        assert_eq!(child.undo_history.len(), 2);

        let undo_result = child.undo(10);
        assert!(undo_result.is_ok());
        let undone_count = undo_result.unwrap();
        assert_eq!(undone_count, 2);

        assert_eq!(
            child.messages.len(),
            4,
            "Messages should be restored to fork point"
        );

        assert_eq!(parent.messages.len(), parent_message_count);
        assert_eq!(parent.undo_history.len(), parent_undo_len);

        parent.add_message(opencode_core::message::Message::user(
            "new message".to_string(),
        ));
        assert_eq!(parent.messages.len(), 6);

        let child2 = parent.fork_at_message(3).unwrap();
        assert_eq!(child2.messages.len(), 4);
        assert_eq!(
            child2.parent_session_id.as_deref(),
            Some(parent.id.to_string().as_str())
        );
    }

    #[test]
    fn test_session_state_003_storage_corruption_recovery() {
        use opencode_core::session::Session;
        use std::io::Write;

        let tmp = tempfile::tempdir().unwrap();
        let filepath = tmp.path().join("corrupted_session.json");

        {
            let mut file = std::fs::File::create(&filepath).unwrap();
            file.write_all(b"invalid json content {{{{").unwrap();
        }

        let result = Session::load(&filepath);
        assert!(
            result.is_err(),
            "Loading corrupted session should return error"
        );

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            !err_msg.is_empty() && err_msg.len() > 10,
            "Error message should be meaningful, got: {}",
            err_msg
        );

        std::fs::remove_file(&filepath).ok();

        let valid_result = Session::load(&filepath);
        assert!(
            valid_result.is_err(),
            "Loading non-existent session should return error"
        );
    }

    #[tokio::test]
    async fn test_session_state_004_message_ordering_after_fork_and_undo() {
        use opencode_core::session::Session;

        let mut parent = Session::new();
        parent.add_message(opencode_core::message::Message::user("A"));
        parent.add_message(opencode_core::message::Message::user("B"));
        parent.add_message(opencode_core::message::Message::user("C"));
        parent.add_message(opencode_core::message::Message::user("D"));
        parent.add_message(opencode_core::message::Message::user("E"));

        let mut child = parent.fork_at_message(2).unwrap();

        child.add_message(opencode_core::message::Message::user("F"));
        child.add_message(opencode_core::message::Message::user("G"));

        child.undo(1).unwrap();

        let child_messages: Vec<String> =
            child.messages.iter().map(|m| m.content.clone()).collect();
        assert_eq!(child_messages, vec!["A", "B", "C", "F"]);

        let parent_messages: Vec<String> =
            parent.messages.iter().map(|m| m.content.clone()).collect();
        assert_eq!(parent_messages, vec!["A", "B", "C", "D", "E"]);
    }

    #[tokio::test]
    async fn test_session_state_005_concurrent_forks_from_same_parent() {
        use opencode_core::session::Session;
        use std::sync::Arc;

        let mut parent = Session::new();
        for c in ['A', 'B', 'C', 'D', 'E'] {
            parent.add_message(opencode_core::message::Message::user(c.to_string()));
        }

        let parent = Arc::new(parent);
        let mut handles = vec![];

        for _ in 0..10 {
            let parent = parent.clone();
            let handle = tokio::spawn(async move { parent.fork_at_message(3).unwrap() });
            handles.push(handle);
        }

        let mut children = Vec::new();
        for handle in handles {
            children.push(handle.await.unwrap());
        }

        let child_ids: Vec<_> = children.iter().map(|c| c.id).collect();
        let unique_ids: std::collections::HashSet<_> = child_ids.iter().collect();
        assert_eq!(unique_ids.len(), 10, "All child IDs should be unique");

        for child in &children {
            assert_eq!(child.messages.len(), 4);
            let parent_msg_count = parent.messages.len();
            assert_eq!(parent_msg_count, 5, "Parent should still have 5 messages");
        }
    }

    #[test]
    fn test_session_export_001_redaction_completeness() {
        use opencode_core::message::Message;
        use opencode_core::session::Session;
        use opencode_core::share::{ExportFormat, ExportOptions, ShareManager};

        let mut session = Session::new();
        session.add_message(Message::user("My API key is sk-abc123secret"));
        session.add_message(Message::user("Token: op-xyz789secret"));
        session.add_message(Message::user("Bearer sk-def456token"));
        session.add_message(Message::user("api_key=sk-xyz789value"));

        let mgr = ShareManager::new();
        let opts = ExportOptions {
            include_metadata: false,
            sanitize_sensitive: true,
            format: ExportFormat::Json,
        };
        let output = mgr.export_session(&session, &opts);

        let sensitive_patterns = ["sk-abc123secret", "sk-def456token", "sk-xyz789value"];

        for pattern in sensitive_patterns {
            assert!(
                !output.contains(pattern),
                "Sensitive pattern '{}' should be redacted but found in output",
                pattern
            );
        }

        assert!(
            output.contains("[REDACTED]"),
            "Output should contain [REDACTED] marker"
        );
    }
}
