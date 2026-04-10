use std::collections::HashMap;
use std::sync::RwLock;

pub const VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiVersion {
    V1,
    V2,
}

impl ApiVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiVersion::V1 => "v1",
            ApiVersion::V2 => "v2",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub plugin_id: String,
    pub app_state: CommandContextState,
}

#[derive(Debug, Clone)]
pub struct CommandContextState {
    pub messages: Vec<CommandMessage>,
    pub work_mode: String,
}

#[derive(Debug, Clone)]
pub struct CommandMessage {
    pub role: String,
    pub content: String,
}

pub trait PluginCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn aliases(&self) -> Vec<String>;
    fn execute(&self, ctx: &CommandContext) -> CommandResult;
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

impl CommandResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegisteredCommand {
    pub plugin_id: String,
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
}

pub struct PluginCommandRegistry {
    commands: RwLock<HashMap<String, RegisteredCommand>>,
    executors: RwLock<HashMap<String, Box<dyn PluginCommand>>>,
}

impl PluginCommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
            executors: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_command<C: PluginCommand + 'static>(
        &self,
        plugin_id: &str,
        command: C,
    ) -> Result<(), PluginCommandError> {
        let name = command.name().to_string();
        let full_name = format!("{}:{}", plugin_id, name);

        let mut commands = self.commands.write().unwrap();
        if commands.contains_key(&full_name) {
            return Err(PluginCommandError::CommandAlreadyRegistered(full_name));
        }

        let registered = RegisteredCommand {
            plugin_id: plugin_id.to_string(),
            name: name.clone(),
            description: command.description().to_string(),
            aliases: command.aliases(),
        };

        commands.insert(full_name.clone(), registered);

        let mut executors = self.executors.write().unwrap();
        executors.insert(full_name, Box::new(command));

        Ok(())
    }

    pub fn unregister_plugin_commands(&self, plugin_id: &str) {
        let prefix = format!("{}:", plugin_id);

        let mut commands = self.commands.write().unwrap();
        let keys_to_remove: Vec<String> = commands
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in keys_to_remove {
            commands.remove(&key);
        }

        let mut executors = self.executors.write().unwrap();
        let exec_keys_to_remove: Vec<String> = executors
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        for key in exec_keys_to_remove {
            executors.remove(&key);
        }
    }

    pub fn get_command(&self, plugin_id: &str, name: &str) -> Option<RegisteredCommand> {
        let full_name = format!("{}:{}", plugin_id, name);
        self.commands.read().unwrap().get(&full_name).cloned()
    }

    pub fn get_by_name(&self, name: &str) -> Option<RegisteredCommand> {
        let commands = self.commands.read().unwrap();
        commands
            .values()
            .find(|c| c.name == name || c.aliases.contains(&name.to_string()))
            .cloned()
    }

    pub fn list_commands(&self) -> Vec<RegisteredCommand> {
        self.commands.read().unwrap().values().cloned().collect()
    }

    pub fn execute(
        &self,
        plugin_id: &str,
        name: &str,
        ctx: &CommandContext,
    ) -> Result<CommandResult, PluginCommandError> {
        let full_name = format!("{}:{}", plugin_id, name);
        let executors = self.executors.read().unwrap();
        let executor = executors
            .get(&full_name)
            .ok_or_else(|| PluginCommandError::CommandNotFound(full_name.clone()))?;

        Ok(executor.execute(ctx))
    }

    pub fn execute_by_name(
        &self,
        name: &str,
        ctx: &CommandContext,
    ) -> Result<CommandResult, PluginCommandError> {
        let cmd = self
            .get_by_name(name)
            .ok_or_else(|| PluginCommandError::CommandNotFound(name.to_string()))?;

        self.execute(&cmd.plugin_id, &cmd.name, ctx)
    }

    pub fn clear(&self) {
        self.commands.write().unwrap().clear();
        self.executors.write().unwrap().clear();
    }
}

impl Default for PluginCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PluginCommandError {
    #[error("command not found: {0}")]
    CommandNotFound(String),
    #[error("command already registered: {0}")]
    CommandAlreadyRegistered(String),
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    #[error("execution error: {0}")]
    ExecutionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommand {
        name: String,
        description: String,
        aliases: Vec<String>,
        should_succeed: bool,
    }

    impl TestCommand {
        fn new(name: &str, description: &str, aliases: Vec<&str>, should_succeed: bool) -> Self {
            Self {
                name: name.to_string(),
                description: description.to_string(),
                aliases: aliases.iter().map(|s| s.to_string()).collect(),
                should_succeed,
            }
        }
    }

    impl PluginCommand for TestCommand {
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
            if self.should_succeed {
                CommandResult::success("Test command executed")
            } else {
                CommandResult::error("Test command failed")
            }
        }
    }

    #[test]
    fn test_plugin_command_registry_new() {
        let registry = PluginCommandRegistry::new();
        assert!(registry.list_commands().is_empty());
    }

    #[test]
    fn test_register_command() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("test-cmd", "A test command", vec!["tc"], true);

        registry.register_command("test.plugin", cmd).unwrap();

        let cmds = registry.list_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].name, "test-cmd");
        assert_eq!(cmds[0].plugin_id, "test.plugin");
    }

    #[test]
    fn test_register_duplicate_command() {
        let registry = PluginCommandRegistry::new();
        let cmd1 = TestCommand::new("test-cmd", "First", vec![], true);
        let cmd2 = TestCommand::new("test-cmd", "Second", vec![], true);

        registry.register_command("test.plugin", cmd1).unwrap();

        let result = registry.register_command("test.plugin", cmd2);
        assert!(matches!(
            result,
            Err(PluginCommandError::CommandAlreadyRegistered(_))
        ));
    }

    #[test]
    fn test_unregister_plugin_commands() {
        let registry = PluginCommandRegistry::new();
        let cmd1 = TestCommand::new("cmd1", "First command", vec![], true);
        let cmd2 = TestCommand::new("cmd2", "Second command", vec![], true);

        registry.register_command("test.plugin", cmd1).unwrap();
        registry.register_command("test.plugin", cmd2).unwrap();

        assert_eq!(registry.list_commands().len(), 2);

        registry.unregister_plugin_commands("test.plugin");
        assert!(registry.list_commands().is_empty());
    }

    #[test]
    fn test_get_command() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("get-cmd", "Get test", vec!["gc"], true);

        registry.register_command("get.plugin", cmd).unwrap();

        let found = registry.get_command("get.plugin", "get-cmd");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "get-cmd");
    }

    #[test]
    fn test_get_by_name() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("byname", "By name test", vec!["bn"], true);

        registry.register_command("byname.plugin", cmd).unwrap();

        let found = registry.get_by_name("byname");
        assert!(found.is_some());

        let found_by_alias = registry.get_by_name("bn");
        assert!(found_by_alias.is_some());
    }

    #[test]
    fn test_execute_command() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("exec", "Execute test", vec![], true);

        registry.register_command("exec.plugin", cmd).unwrap();

        let ctx = CommandContext {
            plugin_id: "exec.plugin".to_string(),
            app_state: CommandContextState {
                messages: vec![],
                work_mode: "build".to_string(),
            },
        };

        let result = registry.execute("exec.plugin", "exec", &ctx).unwrap();
        assert!(result.success);
        assert_eq!(result.message, "Test command executed");
    }

    #[test]
    fn test_execute_by_name() {
        let registry = PluginCommandRegistry::new();
        let cmd = TestCommand::new("byname-exec", "By name exec test", vec!["bne"], true);

        registry.register_command("bne.plugin", cmd).unwrap();

        let ctx = CommandContext {
            plugin_id: "bne.plugin".to_string(),
            app_state: CommandContextState {
                messages: vec![],
                work_mode: "build".to_string(),
            },
        };

        let result = registry.execute_by_name("byname-exec", &ctx).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let registry = PluginCommandRegistry::new();
        let ctx = CommandContext {
            plugin_id: "test".to_string(),
            app_state: CommandContextState {
                messages: vec![],
                work_mode: "build".to_string(),
            },
        };

        let result = registry.execute("test.plugin", "nonexistent", &ctx);
        assert!(matches!(
            result,
            Err(PluginCommandError::CommandNotFound(_))
        ));
    }

    #[test]
    fn test_command_result_success() {
        let result = CommandResult::success("OK");
        assert!(result.success);
        assert_eq!(result.message, "OK");
    }

    #[test]
    fn test_command_result_error() {
        let result = CommandResult::error("Failed");
        assert!(!result.success);
        assert_eq!(result.message, "Failed");
    }

    #[test]
    fn test_api_version_as_str() {
        assert_eq!(ApiVersion::V1.as_str(), "v1");
        assert_eq!(ApiVersion::V2.as_str(), "v2");
    }

    #[test]
    fn test_multiple_plugins_same_command_name() {
        let registry = PluginCommandRegistry::new();
        let cmd1 = TestCommand::new("cmd", "From plugin1", vec![], true);
        let cmd2 = TestCommand::new("cmd", "From plugin2", vec![], true);

        registry.register_command("plugin1", cmd1).unwrap();

        let result = registry.register_command("plugin2", cmd2);
        assert!(result.is_ok());
        assert_eq!(registry.list_commands().len(), 2);
    }
}
