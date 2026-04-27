use crate::command::types::*;
use crate::Config;
use crate::OpenCodeError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
    definitions: HashMap<String, CommandDefinition>,
    commands_dir: Option<PathBuf>,
}

#[allow(dead_code)]
impl CommandRegistry {
    pub(crate) fn new() -> Self {
        Self {
            commands: HashMap::new(),
            definitions: HashMap::new(),
            commands_dir: None,
        }
    }

    pub fn with_commands_dir(mut self, dir: PathBuf) -> Self {
        self.commands_dir = Some(dir);
        self
    }

    pub fn register(&mut self, definition: CommandDefinition, command: impl Command + 'static) {
        let name = definition.name.clone();
        self.definitions.insert(name.clone(), definition);
        self.commands.insert(name, Box::new(command));
    }

    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands.get(name).map(|c| c.as_ref())
    }

    pub fn get_definition(&self, name: &str) -> Option<&CommandDefinition> {
        self.definitions.get(name)
    }

    pub fn list(&self) -> Vec<CommandInfo> {
        self.commands
            .values()
            .map(|cmd| CommandInfo {
                name: cmd.name().to_string(),
                description: cmd.description().to_string(),
                usage: cmd.usage().to_string(),
            })
            .collect()
    }

    pub fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    pub fn list_with_usage(&self) -> Vec<CommandInfo> {
        self.list()
    }

    pub fn discover_commands(&mut self) -> Result<(), OpenCodeError> {
        let Some(commands_dir) = &self.commands_dir else {
            return Ok(());
        };

        if !commands_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(commands_dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to read command file {:?}: {}", path, e);
                    continue;
                }
            };

            let Some(definition) = CommandDefinition::from_markdown(&content, path) else {
                tracing::warn!("Failed to parse command from {:?}", path);
                continue;
            };

            let cmd = FileCommand {
                definition: definition.clone(),
            };

            self.register(definition, cmd);
        }

        Ok(())
    }

    pub fn set_runtime_context(
        &self,
        mut ctx: CommandContext,
        session: Option<&crate::Session>,
        config: Option<&Config>,
    ) -> CommandContext {
        if let Some(session) = session {
            let messages: Vec<_> = session.messages.clone();
            ctx.on_clear_session = Some(Box::new(move || messages.len()));
            ctx.on_share_session = Some(Box::new(move || {
                Err("Share requires mutable session access".to_string())
            }));
        }
        if let Some(config) = config {
            let models_cfg = config.clone();
            let agents_cfg = config.clone();
            ctx.on_get_models = Some(Box::new(move || format_models(&models_cfg)));
            ctx.on_get_agents = Some(Box::new(move || format_agents(&agents_cfg)));
        }

        if ctx.on_list_commands.is_none() {
            let commands = self.list_with_usage();
            ctx.on_list_commands = Some(Box::new(move || commands));
        }

        ctx
    }

    pub async fn execute(
        &self,
        name: &str,
        mut ctx: CommandContext,
    ) -> Result<String, OpenCodeError> {
        if ctx.on_list_commands.is_none() {
            let commands = self.list_with_usage();
            ctx.on_list_commands = Some(Box::new(move || commands));
        }

        let command = self
            .get(name)
            .ok_or_else(|| OpenCodeError::Tool(format!("Command not found: {}", name)))?;
        command.execute(ctx).await
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn substitute_command_variables(
    template: &str,
    context: &CommandContext,
) -> Result<String, OpenCodeError> {
    let mut result = Config::substitute_variables(template, Some(Path::new(&context.working_dir)))?;

    result = result.replace("{input}", &context.variables.input);
    result = result.replace("{selection}", &context.variables.selection);

    let file_pattern = regex::Regex::new(r"\{file:([^}]+)\}").ok();
    if let Some(re) = file_pattern {
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let path = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
                let candidate = Path::new(path);
                let resolved = if candidate.is_absolute() {
                    candidate.to_path_buf()
                } else {
                    Path::new(&context.working_dir).join(candidate)
                };

                std::fs::read_to_string(&resolved).unwrap_or_else(|_| format!("{{file:{}}}", path))
            })
            .to_string();
    }

    Ok(result)
}

struct FileCommand {
    definition: CommandDefinition,
}

impl sealed::Sealed for FileCommand {}

impl FileCommand {
    fn name(&self) -> &str {
        &self.definition.name
    }

    fn description(&self) -> &str {
        &self.definition.description
    }

    fn usage(&self) -> &str {
        "/<command>"
    }
}

#[async_trait]
impl Command for FileCommand {
    fn name(&self) -> &str {
        self.name()
    }

    fn description(&self) -> &str {
        self.description()
    }

    fn usage(&self) -> &str {
        self.usage()
    }

    async fn execute(&self, ctx: CommandContext) -> Result<String, OpenCodeError> {
        substitute_command_variables(&self.definition.template, &ctx)
    }
}
