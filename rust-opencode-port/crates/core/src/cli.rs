use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub subcommands: Vec<CliCommand>,
    pub args: Vec<CliArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliArg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

pub struct CliRegistry {
    commands: HashMap<String, CliCommand>,
}

impl CliRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, command: CliCommand) {
        self.commands.insert(command.name.clone(), command);
    }

    pub fn get(&self, name: &str) -> Option<&CliCommand> {
        self.commands.get(name)
    }

    pub fn list(&self) -> Vec<&CliCommand> {
        self.commands.values().collect()
    }
}

impl Default for CliRegistry {
    fn default() -> Self {
        Self::new()
    }
}
