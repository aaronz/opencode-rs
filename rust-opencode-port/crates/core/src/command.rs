use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: String,
}

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    async fn execute(&self, ctx: CommandContext) -> Result<String, crate::OpenCodeError>;
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, command: Box<dyn Command>) {
        let name = command.name().to_string();
        self.commands.insert(name, command);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands.get(name).map(|c| c.as_ref())
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.commands
            .iter()
            .map(|(name, cmd)| (name.as_str(), cmd.description()))
            .collect()
    }

    pub async fn execute(&self, name: &str, ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        let command = self.get(name)
            .ok_or_else(|| crate::OpenCodeError::Tool(format!("Command not found: {}", name)))?;
        command.execute(ctx).await
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
