#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub name: String,
    pub aliases: Vec<String>,
    pub description: String,
    pub action: CommandAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandAction {
    /// Toggle plan/build mode
    SetMode(String),
    /// Clear messages
    Clear,
    /// Open timeline view
    OpenTimeline,
    /// Open fork dialog
    OpenFork,
    /// Toggle metadata display
    ToggleMetadata,
    /// Open settings dialog
    OpenSettings,
    /// Open model selection dialog
    OpenModels,
    /// Open provider management dialog
    OpenProviders,
    OpenConnect,
    /// Toggle file tree
    ToggleFiles,
    /// Open release notes dialog
    OpenReleaseNotes,
    /// Compact session
    Compact,
    /// Summarize session
    Summarize,
    /// Export session
    Export,
    /// Undo last file changes
    Undo,
    /// Toggle tool details
    ToggleDetails,
    /// List themes
    ListThemes,
    /// Switch theme
    SwitchTheme,
    /// Exit application
    Exit,
    OpenSessions,
    NewSession,
    Custom(String),
}

pub struct CommandRegistry {
    commands: Vec<Command>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let commands = vec![
            Command {
                name: "plan".to_string(),
                aliases: vec!["p".to_string()],
                description: "Switch to plan agent (read-only)".to_string(),
                action: CommandAction::SetMode("plan".to_string()),
            },
            Command {
                name: "build".to_string(),
                aliases: vec!["b".to_string()],
                description: "Switch to build agent (full access)".to_string(),
                action: CommandAction::SetMode("build".to_string()),
            },
            Command {
                name: "clear".to_string(),
                aliases: vec![],
                description: "Clear all messages".to_string(),
                action: CommandAction::Clear,
            },
            Command {
                name: "help".to_string(),
                aliases: vec!["h".to_string(), "?".to_string()],
                description: "Show all available commands".to_string(),
                action: CommandAction::Custom("help".to_string()),
            },
            Command {
                name: "timeline".to_string(),
                aliases: vec!["t".to_string()],
                description: "Open timeline view".to_string(),
                action: CommandAction::OpenTimeline,
            },
            Command {
                name: "fork".to_string(),
                aliases: vec!["f".to_string()],
                description: "Fork session at current message".to_string(),
                action: CommandAction::OpenFork,
            },
            Command {
                name: "meta".to_string(),
                aliases: vec!["m".to_string()],
                description: "Toggle metadata display".to_string(),
                action: CommandAction::ToggleMetadata,
            },
            Command {
                name: "settings".to_string(),
                aliases: vec![",".to_string()],
                description: "Open settings dialog".to_string(),
                action: CommandAction::OpenSettings,
            },
            Command {
                name: "models".to_string(),
                aliases: vec!["model".to_string()],
                description: "Open model selection".to_string(),
                action: CommandAction::OpenModels,
            },
            Command {
                name: "providers".to_string(),
                aliases: vec!["provider".to_string()],
                description: "Open provider management".to_string(),
                action: CommandAction::OpenProviders,
            },
            Command {
                name: "connect".to_string(),
                aliases: vec![],
                description: "Connect a provider".to_string(),
                action: CommandAction::OpenConnect,
            },
            Command {
                name: "files".to_string(),
                aliases: vec!["file".to_string()],
                description: "Toggle file tree panel".to_string(),
                action: CommandAction::ToggleFiles,
            },
            Command {
                name: "release-notes".to_string(),
                aliases: vec!["notes".to_string(), "rn".to_string()],
                description: "Open release notes dialog".to_string(),
                action: CommandAction::OpenReleaseNotes,
            },
            Command {
                name: "compact".to_string(),
                aliases: vec!["c".to_string()],
                description: "Compact session to save tokens".to_string(),
                action: CommandAction::Compact,
            },
            Command {
                name: "summarize".to_string(),
                aliases: vec!["s".to_string()],
                description: "Summarize current conversation".to_string(),
                action: CommandAction::Summarize,
            },
            Command {
                name: "export".to_string(),
                aliases: vec!["e".to_string()],
                description: "Export session to markdown file".to_string(),
                action: CommandAction::Export,
            },
            Command {
                name: "undo".to_string(),
                aliases: vec!["u".to_string()],
                description: "Undo last file changes".to_string(),
                action: CommandAction::Undo,
            },
            Command {
                name: "sessions".to_string(),
                aliases: vec!["session".to_string(), "ses".to_string()],
                description: "List and manage sessions".to_string(),
                action: CommandAction::OpenSessions,
            },
            Command {
                name: "new".to_string(),
                aliases: vec![],
                description: "Create a new session".to_string(),
                action: CommandAction::NewSession,
            },
            Command {
                name: "details".to_string(),
                aliases: vec!["d".to_string()],
                description: "Toggle tool execution details".to_string(),
                action: CommandAction::ToggleDetails,
            },
            Command {
                name: "themes".to_string(),
                aliases: vec!["theme-list".to_string()],
                description: "List available themes".to_string(),
                action: CommandAction::ListThemes,
            },
            Command {
                name: "theme".to_string(),
                aliases: vec![],
                description: "Switch to next theme".to_string(),
                action: CommandAction::SwitchTheme,
            },
            Command {
                name: "exit".to_string(),
                aliases: vec!["quit".to_string(), "q".to_string()],
                description: "Exit the application".to_string(),
                action: CommandAction::Exit,
            },
        ];

        Self { commands }
    }

    pub fn find(&self, query: &str) -> Vec<&Command> {
        let query_lower = query.to_lowercase();
        self.commands
            .iter()
            .filter(|cmd| {
                cmd.name.starts_with(&query_lower)
                    || cmd
                        .aliases
                        .iter()
                        .any(|alias| alias.starts_with(&query_lower))
            })
            .collect()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Command> {
        self.commands
            .iter()
            .find(|cmd| cmd.name == name || cmd.aliases.contains(&name.to_string()))
    }

    pub fn all(&self) -> &[Command] {
        &self.commands
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_find() {
        let registry = CommandRegistry::new();
        let results = registry.find("p");
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.name == "plan"));
    }

    #[test]
    fn test_command_registry_get_by_name() {
        let registry = CommandRegistry::new();
        let cmd = registry.get_by_name("plan");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "plan");
    }

    #[test]
    fn test_command_registry_aliases() {
        let registry = CommandRegistry::new();
        let cmd = registry.get_by_name("h");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "help");
    }

    #[test]
    fn test_command_registry_connect_command() {
        let registry = CommandRegistry::new();
        let cmd = registry.get_by_name("connect");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().description, "Connect a provider");
    }
}
