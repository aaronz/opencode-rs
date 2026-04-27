pub mod builtins;
pub mod registry;
pub mod types;

pub use builtins::*;
pub use registry::{substitute_command_variables, CommandRegistry};
pub use types::{
    format_agents, format_help_table, format_models, Command, CommandContext, CommandDefinition,
    CommandInfo, CommandVariables,
};

pub fn register_builtin_commands(registry: &mut CommandRegistry) {
    registry.register(
        CommandDefinition {
            name: "help".to_string(),
            description: "Show available commands and their usage".to_string(),
            triggers: vec!["help".to_string()],
            agent: None,
            model: None,
            template: "Show help".to_string(),
        },
        HelpCommand,
    );

    registry.register(
        CommandDefinition {
            name: "test".to_string(),
            description: "Run test suite or specific test".to_string(),
            triggers: vec!["test".to_string()],
            agent: None,
            model: None,
            template: "Run tests".to_string(),
        },
        TestCommand,
    );

    registry.register(
        CommandDefinition {
            name: "debug".to_string(),
            description: "Show debug information about current session".to_string(),
            triggers: vec!["debug".to_string()],
            agent: None,
            model: None,
            template: "Show debug info".to_string(),
        },
        DebugCommand,
    );

    registry.register(
        CommandDefinition {
            name: "clear".to_string(),
            description: "Clear current session context".to_string(),
            triggers: vec!["clear".to_string()],
            agent: None,
            model: None,
            template: "Clear session".to_string(),
        },
        ClearCommand,
    );

    registry.register(
        CommandDefinition {
            name: "models".to_string(),
            description: "List available models from configuration".to_string(),
            triggers: vec!["models".to_string()],
            agent: None,
            model: None,
            template: "List models".to_string(),
        },
        ModelsCommand,
    );

    registry.register(
        CommandDefinition {
            name: "agents".to_string(),
            description: "List available agents from configuration".to_string(),
            triggers: vec!["agents".to_string()],
            agent: None,
            model: None,
            template: "List agents".to_string(),
        },
        AgentsCommand,
    );

    registry.register(
        CommandDefinition {
            name: "share".to_string(),
            description: "Share current session".to_string(),
            triggers: vec!["share".to_string()],
            agent: None,
            model: None,
            template: "Share session".to_string(),
        },
        ShareCommand,
    );

    registry.register(
        CommandDefinition {
            name: "compact".to_string(),
            description: "Manually trigger context compaction".to_string(),
            triggers: vec!["compact".to_string()],
            agent: None,
            model: None,
            template: "Compact context".to_string(),
        },
        CompactCommand,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentConfig, AgentMapConfig, ModelConfig, ProviderConfig};
    use crate::message::Message;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[test]
    fn test_command_expand() {
        let def = CommandDefinition {
            name: "test".to_string(),
            description: "Test".to_string(),
            triggers: vec![],
            agent: None,
            model: None,
            template: "Run ${file} with ${input}".to_string(),
        };
        let vars = CommandVariables::new(
            "test.rs".to_string(),
            "selected".to_string(),
            "/home".to_string(),
            "main".to_string(),
            "user input".to_string(),
        );
        let expanded = def.expand(&vars);
        assert!(expanded.contains("test.rs"));
        assert!(expanded.contains("user input"));
    }

    #[test]
    fn test_command_registry_list() {
        let registry = CommandRegistry::new();
        let list = registry.list();
        assert!(list.is_empty());
    }

    #[test]
    fn test_register_builtin_commands_has_all_8() {
        let mut registry = CommandRegistry::new();
        register_builtin_commands(&mut registry);
        let mut names = registry.list_commands();
        names.sort();
        assert_eq!(
            names,
            vec![
                "agents".to_string(),
                "clear".to_string(),
                "compact".to_string(),
                "debug".to_string(),
                "help".to_string(),
                "models".to_string(),
                "share".to_string(),
                "test".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn test_builtin_help_command_executes() {
        let cmd = HelpCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert!(out.contains("Available commands"));
    }

    #[tokio::test]
    async fn test_builtin_test_command_executes() {
        let cmd = TestCommand;
        let out = cmd
            .execute(CommandContext::new(
                vec!["unit".to_string()],
                std::collections::HashMap::new(),
                ".".to_string(),
            ))
            .await
            .unwrap();
        assert!(out.contains("unit"));
    }

    #[tokio::test]
    async fn test_builtin_debug_command_executes() {
        let cmd = DebugCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert!(out.contains("all"));
    }

    #[tokio::test]
    async fn test_builtin_clear_command_executes() {
        let cmd = ClearCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert_eq!(out, "Session context cleared");
    }

    #[tokio::test]
    async fn test_builtin_models_command_executes() {
        let cmd = ModelsCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert!(out.contains("models"));
    }

    #[tokio::test]
    async fn test_builtin_agents_command_executes() {
        let cmd = AgentsCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert!(out.contains("agents"));
    }

    #[tokio::test]
    async fn test_builtin_share_command_executes() {
        let cmd = ShareCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert_eq!(out, "Session shared");
    }

    #[tokio::test]
    async fn test_builtin_compact_command_executes() {
        let cmd = CompactCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert_eq!(out, "Context compaction triggered");
    }

    #[tokio::test]
    async fn test_help_command_uses_registry_runtime_context() {
        let mut registry = CommandRegistry::new();
        register_builtin_commands(&mut registry);

        let ctx = registry.set_runtime_context(
            CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()),
            None,
            None,
        );
        let out = registry.execute("help", ctx).await.unwrap();

        assert!(out.contains("Available commands:"));
        assert!(out.contains("/help"));
        assert!(out.contains("usage:"));
    }

    #[tokio::test]
    async fn test_clear_command_clears_session_and_resets_state() {
        let mut registry = CommandRegistry::new();
        register_builtin_commands(&mut registry);

        let session = Arc::new(Mutex::new(crate::Session::new()));
        {
            let mut lock = session.lock().unwrap();
            lock.add_message(Message::user("one".to_string()));
            lock.add_message(Message::assistant("two".to_string()));
        }

        let session_for_clear = Arc::clone(&session);
        let mut ctx = CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string());
        ctx.on_clear_session = Some(Box::new(move || {
            let mut lock = session_for_clear.lock().unwrap();
            let cleared = lock.messages.len();
            lock.messages.clear();
            lock.state = crate::session_state::SessionState::Idle;
            cleared
        }));
        let out = registry.execute("clear", ctx).await.unwrap();

        assert!(out.contains("Cleared 2 message(s)"));
        let lock = session.lock().unwrap();
        assert!(lock.messages.is_empty());
        assert_eq!(lock.state, crate::session_state::SessionState::Idle);
    }

    #[tokio::test]
    async fn test_models_command_lists_configured_models() {
        let mut registry = CommandRegistry::new();
        register_builtin_commands(&mut registry);

        let mut providers = std::collections::HashMap::new();
        let mut model_map = std::collections::HashMap::new();
        model_map.insert(
            "gpt-4o".to_string(),
            ModelConfig {
                name: Some("GPT-4o".to_string()),
                ..Default::default()
            },
        );
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                models: Some(model_map),
                ..Default::default()
            },
        );

        let config = crate::Config {
            provider: Some(providers),
            ..Default::default()
        };

        let ctx = registry.set_runtime_context(
            CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()),
            None,
            Some(&config),
        );
        let out = registry.execute("models", ctx).await.unwrap();

        assert!(out.contains("openai [configured]"));
        assert!(out.contains("GPT-4o"));
    }

    #[tokio::test]
    async fn test_agents_command_lists_agents_with_capabilities() {
        let mut registry = CommandRegistry::new();
        register_builtin_commands(&mut registry);

        let mut agents = std::collections::HashMap::new();
        agents.insert(
            "build".to_string(),
            AgentConfig {
                model: Some("openai/gpt-4o".to_string()),
                description: Some("Build agent".to_string()),
                steps: Some(12),
                ..Default::default()
            },
        );

        let config = crate::Config {
            agent: Some(AgentMapConfig {
                agents,
                default_agent: Some("build".to_string()),
            }),
            ..Default::default()
        };

        let ctx = registry.set_runtime_context(
            CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()),
            None,
            Some(&config),
        );
        let out = registry.execute("agents", ctx).await.unwrap();

        assert!(out.contains("Configured agents:"));
        assert!(out.contains("- build"));
        assert!(out.contains("capabilities: steps=12"));
    }

    #[tokio::test]
    async fn test_share_command_generates_real_link() {
        let mut registry = CommandRegistry::new();
        register_builtin_commands(&mut registry);

        let session = Arc::new(Mutex::new(crate::Session::new()));
        let session_for_share = Arc::clone(&session);
        let mut ctx = CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string());
        ctx.on_share_session = Some(Box::new(move || {
            let mut lock = session_for_share.lock().unwrap();
            lock.generate_share_link().map_err(|e| e.to_string())
        }));
        let out = registry.execute("share", ctx).await.unwrap();

        assert!(out.contains("https://opencode-rs.local/share/"));
        assert!(session.lock().unwrap().shared_id.is_some());
    }

    #[tokio::test]
    async fn test_compact_command_returns_compaction_stats() {
        let mut registry = CommandRegistry::new();
        register_builtin_commands(&mut registry);

        let session = Arc::new(Mutex::new(crate::Session::new()));
        {
            let mut lock = session.lock().unwrap();
            for i in 0..20 {
                lock.add_message(Message::user(format!("Message {} {}", i, "x".repeat(300))));
            }
        }

        let session_for_compact = Arc::clone(&session);
        let mut ctx = CommandContext::new(vec!["50".to_string()], std::collections::HashMap::new(), ".".to_string());
        ctx.on_compact = Some(Box::new(move |max_tokens| {
            let mut lock = session_for_compact.lock().unwrap();
            let result = lock.compact_messages(max_tokens);
            format!(
                "Compaction complete: was_compacted={}, pruned_count={}, summary_inserted={}",
                result.was_compacted, result.pruned_count, result.summary_inserted
            )
        }));
        let out = registry.execute("compact", ctx).await.unwrap();

        assert!(out.contains("was_compacted=true"));
        assert!(out.contains("pruned_count="));
    }

    #[test]
    fn test_substitute_command_variables_input_and_selection() {
        let ctx = CommandContext::new(vec![], std::collections::HashMap::new(), ".".to_string()).with_variables(
            CommandVariables {
                input: "do thing".to_string(),
                selection: "line 1".to_string(),
                ..CommandVariables::default()
            },
        );
        let output = substitute_command_variables("Run: {input} // {selection}", &ctx).unwrap();
        assert_eq!(output, "Run: do thing // line 1");
    }

    #[test]
    fn test_substitute_command_variables_file_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("prompt.txt");
        std::fs::write(&file_path, "file-content").unwrap();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        let ctx = CommandContext::new(vec![], std::collections::HashMap::new(), dir.path().display().to_string());
        let output =
            substitute_command_variables(&format!("before {{file:{}}} after", file_name), &ctx)
                .unwrap();
        assert_eq!(output, "before file-content after");
    }
}
