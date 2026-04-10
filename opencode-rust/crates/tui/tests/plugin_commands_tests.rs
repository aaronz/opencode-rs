//! Tests for TUI plugin commands API
//!
//! # Semantics
//!
//! - Plugins can register commands via API
//! - Registered commands appear in command palette
//! - Commands execute plugin logic when invoked
//!
//! # Test Command
//!
//! ```bash
//! cargo test -p opencode-tui -- plugin_commands
//! ```

use opencode_tui::plugin::{PluginLifecycleState, TuiPluginManager};
use opencode_tui::plugin_api::{
    ApiVersion, CommandContext, CommandContextState, CommandMessage, CommandResult, PluginCommand,
    PluginCommandError, PluginCommandRegistry, VERSION,
};

struct TestPluginCommand {
    name: String,
    description: String,
    aliases: Vec<String>,
    execute_result: CommandResult,
}

impl TestPluginCommand {
    fn new(
        name: &str,
        description: &str,
        aliases: Vec<&str>,
        success: bool,
        message: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            aliases: aliases.iter().map(|s| s.to_string()).collect(),
            execute_result: if success {
                CommandResult::success(message)
            } else {
                CommandResult::error(message)
            },
        }
    }
}

impl PluginCommand for TestPluginCommand {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn aliases(&self) -> Vec<String> {
        self.aliases.clone()
    }

    fn execute(&self, _ctx: &CommandContext) -> CommandResult {
        self.execute_result.clone()
    }
}

fn create_test_context() -> CommandContext {
    CommandContext {
        plugin_id: "test.plugin".to_string(),
        app_state: CommandContextState {
            messages: vec![
                CommandMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
                CommandMessage {
                    role: "assistant".to_string(),
                    content: "Hi there!".to_string(),
                },
            ],
            work_mode: "build".to_string(),
        },
    }
}

#[test]
fn test_version_constant_exists() {
    assert_eq!(VERSION, "1.0.0");
}

#[test]
fn test_api_version_v1() {
    assert_eq!(ApiVersion::V1.as_str(), "v1");
}

#[test]
fn test_api_version_v2() {
    assert_eq!(ApiVersion::V2.as_str(), "v2");
}

#[test]
fn test_plugin_command_trait_implementation() {
    let cmd = TestPluginCommand::new(
        "test-cmd",
        "A test command",
        vec!["tc", "test"],
        true,
        "Executed successfully",
    );

    assert_eq!(cmd.name(), "test-cmd");
    assert_eq!(cmd.description(), "A test command");
    assert_eq!(cmd.aliases(), vec!["tc", "test"]);

    let ctx = create_test_context();
    let result = cmd.execute(&ctx);
    assert!(result.success);
    assert_eq!(result.message, "Executed successfully");
}

#[test]
fn test_command_result_success() {
    let result = CommandResult::success("It works!");
    assert!(result.success);
    assert_eq!(result.message, "It works!");
}

#[test]
fn test_command_result_error() {
    let result = CommandResult::error("Something went wrong");
    assert!(!result.success);
    assert_eq!(result.message, "Something went wrong");
}

#[test]
fn test_command_context_creation() {
    let ctx = create_test_context();
    assert_eq!(ctx.plugin_id, "test.plugin");
    assert_eq!(ctx.app_state.messages.len(), 2);
    assert_eq!(ctx.app_state.work_mode, "build");
}

#[test]
fn test_plugin_command_registry_new() {
    let registry = PluginCommandRegistry::new();
    assert!(registry.list_commands().is_empty());
}

#[test]
fn test_plugin_command_registry_register() {
    let registry = PluginCommandRegistry::new();
    let cmd = TestPluginCommand::new("hello", "Say hello", vec![], true, "Hello!");

    registry.register_command("hello.plugin", cmd).unwrap();

    let commands = registry.list_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].name, "hello");
    assert_eq!(commands[0].plugin_id, "hello.plugin");
}

#[test]
fn test_plugin_command_registry_duplicate() {
    let registry = PluginCommandRegistry::new();
    let cmd1 = TestPluginCommand::new("dup", "First", vec![], true, "First");
    let cmd2 = TestPluginCommand::new("dup", "Second", vec![], true, "Second");

    registry.register_command("dup.plugin", cmd1).unwrap();

    let result = registry.register_command("dup.plugin", cmd2);
    assert!(matches!(
        result,
        Err(PluginCommandError::CommandAlreadyRegistered(_))
    ));
}

#[test]
fn test_plugin_command_registry_get_by_name() {
    let registry = PluginCommandRegistry::new();
    let cmd = TestPluginCommand::new("getme", "Get me test", vec!["gm"], true, "Got it!");

    registry.register_command("get.plugin", cmd).unwrap();

    let found = registry.get_by_name("getme").unwrap();
    assert_eq!(found.name, "getme");

    let found_by_alias = registry.get_by_name("gm").unwrap();
    assert_eq!(found_by_alias.name, "getme");
}

#[test]
fn test_plugin_command_registry_execute() {
    let registry = PluginCommandRegistry::new();
    let cmd = TestPluginCommand::new("exec", "Execute test", vec![], true, "Executed!");

    registry.register_command("exec.plugin", cmd).unwrap();

    let ctx = create_test_context();
    let result = registry.execute("exec.plugin", "exec", &ctx).unwrap();
    assert!(result.success);
    assert_eq!(result.message, "Executed!");
}

#[test]
fn test_plugin_command_registry_execute_by_name() {
    let registry = PluginCommandRegistry::new();
    let cmd = TestPluginCommand::new("byname", "By name test", vec![], true, "By name!");

    registry.register_command("bn.plugin", cmd).unwrap();

    let ctx = create_test_context();
    let result = registry.execute_by_name("byname", &ctx).unwrap();
    assert!(result.success);
}

#[test]
fn test_plugin_command_registry_execute_not_found() {
    let registry = PluginCommandRegistry::new();
    let ctx = create_test_context();

    let result = registry.execute("nonexistent.plugin", "cmd", &ctx);
    assert!(matches!(
        result,
        Err(PluginCommandError::CommandNotFound(_))
    ));
}

#[test]
fn test_plugin_command_registry_unregister_plugin() {
    let registry = PluginCommandRegistry::new();
    let cmd1 = TestPluginCommand::new("cmd1", "First", vec![], true, "One");
    let cmd2 = TestPluginCommand::new("cmd2", "Second", vec![], true, "Two");

    registry.register_command("my.plugin", cmd1).unwrap();
    registry.register_command("my.plugin", cmd2).unwrap();
    registry
        .register_command(
            "other.plugin",
            TestPluginCommand::new("cmd3", "Third", vec![], true, "Three"),
        )
        .unwrap();

    assert_eq!(registry.list_commands().len(), 3);

    registry.unregister_plugin_commands("my.plugin");
    assert_eq!(registry.list_commands().len(), 1);
}

#[test]
fn test_plugin_command_registry_clear() {
    let registry = PluginCommandRegistry::new();
    let cmd = TestPluginCommand::new("clear", "Clear test", vec![], true, "Cleared");

    registry.register_command("clear.plugin", cmd).unwrap();
    assert!(!registry.list_commands().is_empty());

    registry.clear();
    assert!(registry.list_commands().is_empty());
}

#[test]
fn test_plugin_manager_with_commands() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "cmd.plugin".to_string(),
            "npm:cmd.plugin".to_string(),
            "@cmd/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let cmd = TestPluginCommand::new("greet", "Greet the user", vec!["g"], true, "Hello, user!");
    manager.register_plugin_command("cmd.plugin", cmd).unwrap();

    let commands = manager.list_plugin_commands();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].name, "greet");

    let ctx = create_test_context();
    let result = manager
        .execute_plugin_command("cmd.plugin", "greet", &ctx)
        .unwrap();
    assert!(result.success);
    assert_eq!(result.message, "Hello, user!");
}

#[test]
fn test_plugin_manager_command_registry_access() {
    let manager = TuiPluginManager::new();
    let registry = manager.command_registry();

    let cmd = TestPluginCommand::new("access", "Access test", vec![], true, "Accessed!");
    registry.register_command("access.plugin", cmd).unwrap();

    assert_eq!(registry.list_commands().len(), 1);
}

#[test]
fn test_plugin_commands_integrated_with_lifecycle() {
    let manager = TuiPluginManager::new();

    manager
        .register_plugin(
            "lifecycle.plugin".to_string(),
            "npm:lifecycle.plugin".to_string(),
            "@lifecycle/plugin@1.0.0".to_string(),
            true,
        )
        .unwrap();

    let cmd = TestPluginCommand::new(
        "lifecycle-cmd",
        "Lifecycle command",
        vec!["lc"],
        true,
        "Lifecycle works!",
    );
    manager
        .register_plugin_command("lifecycle.plugin", cmd)
        .unwrap();

    assert_eq!(manager.list_plugin_commands().len(), 1);

    manager.activate("lifecycle.plugin").unwrap();
    let entry = manager.get_plugin("lifecycle.plugin").unwrap();
    assert_eq!(entry.state, PluginLifecycleState::Active);

    manager.deactivate("lifecycle.plugin").unwrap();
    assert_eq!(manager.list_plugin_commands().len(), 1);

    manager.unregister_plugin_commands("lifecycle.plugin");
    assert_eq!(manager.list_plugin_commands().len(), 0);
}

#[test]
fn test_command_not_found_when_plugin_not_registered() {
    let manager = TuiPluginManager::new();
    let cmd = TestPluginCommand::new("orphan", "Orphan command", vec![], true, "Orphaned");

    let result = manager.register_plugin_command("nonexistent.plugin", cmd);
    assert!(matches!(result, Err(PluginCommandError::PluginNotFound(_))));
}

#[test]
fn test_multiple_plugins_same_command_name() {
    let registry = PluginCommandRegistry::new();

    let cmd1 = TestPluginCommand::new("share", "From plugin1", vec![], true, "Plugin1");
    let cmd2 = TestPluginCommand::new("share", "From plugin2", vec![], true, "Plugin2");

    registry.register_command("plugin1", cmd1).unwrap();
    registry.register_command("plugin2", cmd2).unwrap();

    let commands = registry.list_commands();
    assert_eq!(commands.len(), 2);

    let ctx = create_test_context();

    let result1 = registry.execute("plugin1", "share", &ctx).unwrap();
    assert_eq!(result1.message, "Plugin1");

    let result2 = registry.execute("plugin2", "share", &ctx).unwrap();
    assert_eq!(result2.message, "Plugin2");
}

#[test]
fn test_command_execution_with_context() {
    let registry = PluginCommandRegistry::new();

    struct ContextCheckCommand {
        expected_plugin_id: String,
        expected_work_mode: String,
    }

    impl ContextCheckCommand {
        fn new(plugin_id: &str, work_mode: &str) -> Self {
            Self {
                expected_plugin_id: plugin_id.to_string(),
                expected_work_mode: work_mode.to_string(),
            }
        }
    }

    impl PluginCommand for ContextCheckCommand {
        fn name(&self) -> &str {
            "ctx-check"
        }

        fn description(&self) -> &str {
            "Check context"
        }

        fn aliases(&self) -> Vec<String> {
            vec![]
        }

        fn execute(&self, ctx: &CommandContext) -> CommandResult {
            if ctx.plugin_id != self.expected_plugin_id {
                return CommandResult::error(format!(
                    "Expected plugin_id {} but got {}",
                    self.expected_plugin_id, ctx.plugin_id
                ));
            }
            if ctx.app_state.work_mode != self.expected_work_mode {
                return CommandResult::error(format!(
                    "Expected work_mode {} but got {}",
                    self.expected_work_mode, ctx.app_state.work_mode
                ));
            }
            CommandResult::success("Context verified")
        }
    }

    let cmd = ContextCheckCommand::new("ctx.plugin", "plan");
    registry.register_command("ctx.plugin", cmd).unwrap();

    let ctx = CommandContext {
        plugin_id: "ctx.plugin".to_string(),
        app_state: CommandContextState {
            messages: vec![],
            work_mode: "plan".to_string(),
        },
    };

    let result = registry.execute("ctx.plugin", "ctx-check", &ctx).unwrap();
    assert!(result.success);
}

#[test]
fn test_registered_command_contains_all_fields() {
    let registry = PluginCommandRegistry::new();
    let cmd = TestPluginCommand::new(
        "full-cmd",
        "Full command description",
        vec!["fc", "full"],
        true,
        "Full command executed",
    );

    registry.register_command("full.plugin", cmd).unwrap();

    let registered = registry.get_command("full.plugin", "full-cmd").unwrap();
    assert_eq!(registered.plugin_id, "full.plugin");
    assert_eq!(registered.name, "full-cmd");
    assert_eq!(registered.description, "Full command description");
    assert_eq!(registered.aliases, vec!["fc", "full"]);
}
