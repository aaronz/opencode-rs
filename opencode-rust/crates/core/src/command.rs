use crate::Config;
use crate::OpenCodeError;
use crate::Session;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub template: String,
}

impl CommandDefinition {
    pub fn from_markdown(content: &str, file_path: &Path) -> Option<Self> {
        if !content.starts_with("---") {
            return None;
        }

        let end_idx = content[3..].find("---")?;
        let yaml_part = &content[3..3 + end_idx];
        let body = &content[3 + end_idx + 3..];

        let mut name = None;
        let mut description = None;
        let mut triggers = Vec::new();
        let mut agent = None;
        let mut model = None;

        for line in yaml_part.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "name" => name = Some(value.to_string()),
                    "description" => description = Some(value.to_string()),
                    "triggers" => {
                        triggers = value.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    "agent" => agent = Some(value.to_string()),
                    "model" => model = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        let name = name.or_else(|| {
            file_path
                .file_stem()
                .and_then(|s: &std::ffi::OsStr| s.to_str())
                .map(|s| s.trim_start_matches('/').to_string())
        })?;

        Some(CommandDefinition {
            name,
            description: description.unwrap_or_default(),
            triggers,
            agent,
            model,
            template: body.trim().to_string(),
        })
    }

    pub fn expand(&self, vars: &CommandVariables) -> String {
        let mut result = self.template.clone();

        // Basic variables
        result = result.replace("${file}", &vars.file);
        result = result.replace("${selection}", &vars.selection);
        result = result.replace("${cwd}", &vars.cwd);
        result = result.replace("${git_branch}", &vars.git_branch);
        result = result.replace("${input}", &vars.input);
        result = result.replace("${session_id}", &vars.session_id);
        result = result.replace("${project_path}", &vars.project_path);

        // New variables: cursor position, environment variables
        result = result.replace("${cursor}", &vars.cursor);

        // Environment variables: ${env:VAR_NAME}
        let env_regex = regex::Regex::new(r"\$\{env:([A-Za-z_][A-Za-z0-9_]*)\}").ok();
        if let Some(re) = env_regex {
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    std::env::var(var_name).unwrap_or_default()
                })
                .to_string();
        }

        // File content: {file:path} - read file and insert content
        let file_regex = regex::Regex::new(r"\{file:([^}]+)\}").ok();
        if let Some(re) = file_regex {
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let file_path = &caps[1];
                    std::fs::read_to_string(file_path)
                        .unwrap_or_else(|_| format!("[Cannot read file: {}]", file_path))
                })
                .to_string();
        }

        result
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandVariables {
    pub file: String,
    pub selection: String,
    pub cwd: String,
    pub git_branch: String,
    pub input: String,
    pub session_id: String,
    pub project_path: String,
    pub cursor: String,
}

impl CommandVariables {
    pub fn new(
        file: String,
        selection: String,
        cwd: String,
        git_branch: String,
        input: String,
    ) -> Self {
        Self {
            file,
            selection,
            cwd,
            git_branch,
            input,
            session_id: String::new(),
            project_path: String::new(),
            cursor: String::new(),
        }
    }
}

pub type ClearSessionFn = Box<dyn FnOnce() -> usize + Send + Sync>;
pub type GetModelsFn = Box<dyn FnOnce() -> String + Send + Sync>;
pub type GetAgentsFn = Box<dyn FnOnce() -> String + Send + Sync>;
pub type ShareSessionFn = Box<dyn FnOnce() -> Result<String, String> + Send + Sync>;
pub type CompactFn = Box<dyn FnOnce(usize) -> String + Send + Sync>;
pub type ListCommandsFn = Box<dyn FnOnce() -> Vec<CommandInfo> + Send + Sync>;

pub struct CommandContext {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: String,
    pub variables: CommandVariables,
    pub on_clear_session: Option<ClearSessionFn>,
    pub on_get_models: Option<GetModelsFn>,
    pub on_get_agents: Option<GetAgentsFn>,
    pub on_share_session: Option<ShareSessionFn>,
    pub on_compact: Option<CompactFn>,
    pub on_list_commands: Option<ListCommandsFn>,
}

impl CommandContext {
    pub fn new(args: Vec<String>, env: HashMap<String, String>, working_dir: String) -> Self {
        Self {
            args,
            env,
            working_dir,
            variables: CommandVariables::default(),
            on_clear_session: None,
            on_get_models: None,
            on_get_agents: None,
            on_share_session: None,
            on_compact: None,
            on_list_commands: None,
        }
    }

    pub fn with_variables(mut self, vars: CommandVariables) -> Self {
        self.variables = vars;
        self
    }

    pub fn with_runtime(
        mut self,
        session: Option<&Session>,
        config: Option<&Config>,
        registry: Option<&CommandRegistry>,
    ) -> Self {
        if let Some(session) = session {
            let messages: Vec<_> = session.messages.clone();
            self.on_clear_session = Some(Box::new(move || messages.len()));
            self.on_share_session = Some(Box::new(move || {
                // Share requires mutable access; caller should handle separately
                Err("Share requires mutable session access".to_string())
            }));
        }
        if let Some(config) = config {
            let models_cfg = config.clone();
            let agents_cfg = config.clone();
            self.on_get_models = Some(Box::new(move || format_models(&models_cfg)));
            self.on_get_agents = Some(Box::new(move || format_agents(&agents_cfg)));
        }
        if let Some(registry) = registry {
            let commands = registry.list_with_usage();
            self.on_list_commands = Some(Box::new(move || commands));
        }
        self
    }
}

#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    async fn execute(&self, ctx: CommandContext) -> Result<String, crate::OpenCodeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
}

pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
    definitions: HashMap<String, CommandDefinition>,
    commands_dir: Option<PathBuf>,
}

impl CommandRegistry {
    pub fn new() -> Self {
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

    pub fn set_commands_dir(&mut self, dir: PathBuf) {
        self.commands_dir = Some(dir);
    }

    pub fn register_builtin_commands(&mut self) {
        self.register(Box::new(HelpCommand));
        self.register(Box::new(TestCommand));
        self.register(Box::new(DebugCommand));
        self.register(Box::new(ClearCommand));
        self.register(Box::new(ModelsCommand));
        self.register(Box::new(AgentsCommand));
        self.register(Box::new(ShareCommand));
        self.register(Box::new(CompactCommand));
    }

    pub fn discover(&mut self) -> Result<(), crate::OpenCodeError> {
        let Some(commands_dir) = &self.commands_dir else {
            return Ok(());
        };

        if !commands_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(commands_dir).max_depth(1) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let content = match std::fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Some(def) = CommandDefinition::from_markdown(&content, path) {
                let name = def.name.clone();
                self.definitions.insert(name, def);
            }
        }

        Ok(())
    }

    pub fn register(&mut self, command: Box<dyn Command>) {
        let name = command.name().to_string();
        self.commands.insert(name, command);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands.get(name).map(|c| c.as_ref())
    }

    pub fn get_definition(&self, name: &str) -> Option<&CommandDefinition> {
        self.definitions.get(name)
    }

    pub fn expand_template(&self, name: &str, vars: &CommandVariables) -> Option<String> {
        self.definitions.get(name).map(|def| def.expand(vars))
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        let mut list: Vec<(&str, &str)> = self
            .commands
            .iter()
            .map(|(name, cmd)| (name.as_str(), cmd.description()))
            .collect();

        for (name, def) in &self.definitions {
            list.push((name.as_str(), &def.description));
        }

        list.sort_by(|a, b| a.0.cmp(b.0));
        list
    }

    pub fn list_with_usage(&self) -> Vec<CommandInfo> {
        let mut list: Vec<CommandInfo> = self
            .commands
            .iter()
            .map(|(name, cmd)| CommandInfo {
                name: name.clone(),
                description: cmd.description().to_string(),
                usage: cmd.usage().to_string(),
            })
            .collect();

        for (name, def) in &self.definitions {
            list.push(CommandInfo {
                name: name.clone(),
                description: def.description.clone(),
                usage: format!("/{}", name),
            });
        }

        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn list_commands(&self) -> Vec<String> {
        let mut names: Vec<String> = self.commands.keys().cloned().collect();
        names.extend(self.definitions.keys().cloned());
        names.sort();
        names
    }

    pub fn set_runtime_context(
        &self,
        mut ctx: CommandContext,
        session: Option<&Session>,
        config: Option<&Config>,
    ) -> CommandContext {
        let commands = self.list_with_usage();
        ctx.on_list_commands = Some(Box::new(move || commands));

        if let Some(session) = session {
            let msg_count = session.messages.len();
            ctx.on_clear_session = Some(Box::new(move || msg_count));
            let share_id = session.shared_id.clone();
            let share_mode = session.share_mode.clone();
            ctx.on_share_session = Some(Box::new(move || {
                if matches!(share_mode, Some(crate::config::ShareMode::Disabled)) {
                    return Err("sharing is disabled for this session".to_string());
                }
                let id = share_id.unwrap_or_else(|| "new-share".to_string());
                Ok(format!("https://opencode-rs.local/share/{id}"))
            }));
        }

        if let Some(config) = config {
            let models_cfg = config.clone();
            let agents_cfg = config.clone();
            ctx.on_get_models = Some(Box::new(move || format_models(&models_cfg)));
            ctx.on_get_agents = Some(Box::new(move || format_agents(&agents_cfg)));
        }

        ctx
    }

    pub async fn execute(
        &self,
        name: &str,
        mut ctx: CommandContext,
    ) -> Result<String, crate::OpenCodeError> {
        if ctx.on_list_commands.is_none() {
            let commands = self.list_with_usage();
            ctx.on_list_commands = Some(Box::new(move || commands));
        }

        let command = self
            .get(name)
            .ok_or_else(|| crate::OpenCodeError::Tool(format!("Command not found: {}", name)))?;
        command.execute(ctx).await
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn substitute_command_variables(template: &str, context: &CommandContext) -> Result<String, OpenCodeError> {
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

struct HelpCommand;

fn format_help_table(commands: &[CommandInfo]) -> String {
    if commands.is_empty() {
        return "No commands registered.".to_string();
    }

    let width = commands
        .iter()
        .map(|cmd| cmd.name.len() + 1)
        .max()
        .unwrap_or(0)
        .max(6);

    let mut lines = vec!["Available commands:".to_string()];
    for cmd in commands {
        lines.push(format!(
            "/{:<width$} {} (usage: {})",
            cmd.name,
            cmd.description,
            cmd.usage,
            width = width
        ));
    }

    lines.join("\n")
}

fn format_models(config: &Config) -> String {
    let Some(providers) = &config.provider else {
        return "No providers configured.".to_string();
    };

    if providers.is_empty() {
        return "No providers configured.".to_string();
    }

    let mut names: Vec<&String> = providers.keys().collect();
    names.sort();

    let mut lines = vec!["Configured providers and models:".to_string()];
    for provider_name in names {
        let provider = &providers[provider_name];
        let has_options = provider.options.is_some();
        let has_models = provider
            .models
            .as_ref()
            .map(|m| !m.is_empty())
            .unwrap_or(false);
        let has_filters = provider
            .whitelist
            .as_ref()
            .map(|w| !w.is_empty())
            .unwrap_or(false)
            || provider
                .blacklist
                .as_ref()
                .map(|b| !b.is_empty())
                .unwrap_or(false);
        let status = if has_options || has_models || has_filters {
            "configured"
        } else {
            "not configured"
        };

        let models = provider
            .models
            .as_ref()
            .map(|m| {
                let mut model_names: Vec<String> = m
                    .iter()
                    .map(|(key, model_cfg)| {
                        model_cfg
                            .name
                            .clone()
                            .or_else(|| model_cfg.id.clone())
                            .unwrap_or_else(|| key.clone())
                    })
                    .collect();
                model_names.sort();
                model_names
            })
            .filter(|v| !v.is_empty())
            .unwrap_or_default();

        let models_display = if models.is_empty() {
            "(none)".to_string()
        } else {
            models.join(", ")
        };

        lines.push(format!("- {} [{}]", provider_name, status));
        lines.push(format!("  models: {}", models_display));
    }

    lines.join("\n")
}

fn format_agents(config: &Config) -> String {
    let Some(agent_map) = &config.agent else {
        return "No agents configured.".to_string();
    };

    if agent_map.agents.is_empty() {
        return "No agents configured.".to_string();
    }

    let mut names: Vec<&String> = agent_map.agents.keys().collect();
    names.sort();

    let mut lines = vec!["Configured agents:".to_string()];
    for name in names {
        let agent = &agent_map.agents[name];
        let model = agent.model.as_deref().unwrap_or("(default)");
        let description = agent.description.as_deref().unwrap_or("no description");

        let mut capabilities = Vec::new();
        if let Some(steps) = agent.steps.or(agent.max_steps) {
            capabilities.push(format!("steps={steps}"));
        }
        if agent.permission.is_some() {
            capabilities.push("permission=custom".to_string());
        }
        if agent.disable.unwrap_or(false) {
            capabilities.push("disabled=true".to_string());
        }

        let capabilities = if capabilities.is_empty() {
            "default".to_string()
        } else {
            capabilities.join(", ")
        };

        lines.push(format!("- {}", name));
        lines.push(format!("  description: {}", description));
        lines.push(format!("  model: {}", model));
        lines.push(format!("  capabilities: {}", capabilities));
    }

    lines.join("\n")
}

#[async_trait]
impl Command for HelpCommand {
    fn name(&self) -> &str {
        "help"
    }

    fn description(&self) -> &str {
        "Show available commands and their usage"
    }

    fn usage(&self) -> &str {
        "/help [command]"
    }

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        let commands = if let Some(list_commands) = ctx.on_list_commands.take() {
            list_commands()
        } else {
            vec![
                CommandInfo {
                    name: "help".to_string(),
                    description: "Show available commands and their usage".to_string(),
                    usage: "/help [command]".to_string(),
                },
                CommandInfo {
                    name: "test".to_string(),
                    description: "Run test suite or specific test".to_string(),
                    usage: "/test [pattern]".to_string(),
                },
                CommandInfo {
                    name: "debug".to_string(),
                    description: "Show debug information about current session".to_string(),
                    usage: "/debug [section]".to_string(),
                },
            ]
        };

        if let Some(cmd_name) = ctx.args.first() {
            if let Some(command) = commands.iter().find(|c| c.name == *cmd_name) {
                return Ok(format!(
                    "/{name}\nDescription: {description}\nUsage: {usage}",
                    name = command.name,
                    description = command.description,
                    usage = command.usage
                ));
            }
            return Ok(format!("Unknown command: /{}", cmd_name));
        }

        Ok(format_help_table(&commands))
    }
}

struct TestCommand;

#[async_trait]
impl Command for TestCommand {
    fn name(&self) -> &str {
        "test"
    }

    fn description(&self) -> &str {
        "Run test suite or specific test"
    }

    fn usage(&self) -> &str {
        "/test [pattern]"
    }

    async fn execute(&self, ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        let pattern = ctx.args.join(" ");
        Ok(format!(
            "Running tests matching: {}",
            if pattern.is_empty() {
                "all".to_string()
            } else {
                pattern
            }
        ))
    }
}

struct DebugCommand;

#[async_trait]
impl Command for DebugCommand {
    fn name(&self) -> &str {
        "debug"
    }

    fn description(&self) -> &str {
        "Show debug information about current session"
    }

    fn usage(&self) -> &str {
        "/debug [section]"
    }

    async fn execute(&self, ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        let section = ctx.args.first().map(|s| s.as_str()).unwrap_or("all");
        Ok(format!("Debug info for section: {}", section))
    }
}

struct ClearCommand;

#[async_trait]
impl Command for ClearCommand {
    fn name(&self) -> &str {
        "clear"
    }

    fn description(&self) -> &str {
        "Clear current session context"
    }

    fn usage(&self) -> &str {
        "/clear"
    }

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        if let Some(clear_session) = ctx.on_clear_session.take() {
            let cleared = clear_session();
            return Ok(format!(
                "Cleared {} message(s) and reset session state",
                cleared
            ));
        }

        Ok("Session context cleared".to_string())
    }
}

struct ModelsCommand;

#[async_trait]
impl Command for ModelsCommand {
    fn name(&self) -> &str {
        "models"
    }

    fn description(&self) -> &str {
        "List available models from configuration"
    }

    fn usage(&self) -> &str {
        "/models"
    }

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        if let Some(get_models) = ctx.on_get_models.take() {
            return Ok(get_models());
        }

        Ok("Available models: configure providers and models in your config".to_string())
    }
}

struct AgentsCommand;

#[async_trait]
impl Command for AgentsCommand {
    fn name(&self) -> &str {
        "agents"
    }

    fn description(&self) -> &str {
        "List available agents from configuration"
    }

    fn usage(&self) -> &str {
        "/agents"
    }

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        if let Some(get_agents) = ctx.on_get_agents.take() {
            return Ok(get_agents());
        }

        Ok("Available agents: configure agents in your config".to_string())
    }
}

struct ShareCommand;

#[async_trait]
impl Command for ShareCommand {
    fn name(&self) -> &str {
        "share"
    }

    fn description(&self) -> &str {
        "Share current session"
    }

    fn usage(&self) -> &str {
        "/share"
    }

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        if let Some(share_session) = ctx.on_share_session.take() {
            let url = share_session().map_err(crate::OpenCodeError::Session)?;
            return Ok(format!("Session shared: {}", url));
        }

        Ok("Session shared".to_string())
    }
}

struct CompactCommand;

#[async_trait]
impl Command for CompactCommand {
    fn name(&self) -> &str {
        "compact"
    }

    fn description(&self) -> &str {
        "Manually trigger context compaction"
    }

    fn usage(&self) -> &str {
        "/compact"
    }

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        if let Some(compact) = ctx.on_compact.take() {
            let max_tokens = ctx
                .args
                .first()
                .and_then(|arg| arg.parse::<usize>().ok())
                .unwrap_or(10_000);
            return Ok(compact(max_tokens));
        }

        Ok("Context compaction triggered".to_string())
    }
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
        registry.register_builtin_commands();
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
            .execute(CommandContext::new(vec![], HashMap::new(), ".".to_string()))
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
                HashMap::new(),
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
            .execute(CommandContext::new(vec![], HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert!(out.contains("all"));
    }

    #[tokio::test]
    async fn test_builtin_clear_command_executes() {
        let cmd = ClearCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert_eq!(out, "Session context cleared");
    }

    #[tokio::test]
    async fn test_builtin_models_command_executes() {
        let cmd = ModelsCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert!(out.contains("models"));
    }

    #[tokio::test]
    async fn test_builtin_agents_command_executes() {
        let cmd = AgentsCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert!(out.contains("agents"));
    }

    #[tokio::test]
    async fn test_builtin_share_command_executes() {
        let cmd = ShareCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert_eq!(out, "Session shared");
    }

    #[tokio::test]
    async fn test_builtin_compact_command_executes() {
        let cmd = CompactCommand;
        let out = cmd
            .execute(CommandContext::new(vec![], HashMap::new(), ".".to_string()))
            .await
            .unwrap();
        assert_eq!(out, "Context compaction triggered");
    }

    #[tokio::test]
    async fn test_help_command_uses_registry_runtime_context() {
        let mut registry = CommandRegistry::new();
        registry.register_builtin_commands();

        let ctx = registry.set_runtime_context(
            CommandContext::new(vec![], HashMap::new(), ".".to_string()),
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
        registry.register_builtin_commands();

        let session = Arc::new(Mutex::new(Session::new()));
        {
            let mut lock = session.lock().unwrap();
            lock.add_message(Message::user("one".to_string()));
            lock.add_message(Message::assistant("two".to_string()));
        }

        let session_for_clear = Arc::clone(&session);
        let mut ctx = CommandContext::new(vec![], HashMap::new(), ".".to_string());
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
        registry.register_builtin_commands();

        let mut providers = HashMap::new();
        let mut model_map = HashMap::new();
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

        let config = Config {
            provider: Some(providers),
            ..Default::default()
        };

        let ctx = registry.set_runtime_context(
            CommandContext::new(vec![], HashMap::new(), ".".to_string()),
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
        registry.register_builtin_commands();

        let mut agents = HashMap::new();
        agents.insert(
            "build".to_string(),
            AgentConfig {
                model: Some("openai/gpt-4o".to_string()),
                description: Some("Build agent".to_string()),
                steps: Some(12),
                ..Default::default()
            },
        );

        let config = Config {
            agent: Some(AgentMapConfig {
                agents,
                default_agent: Some("build".to_string()),
            }),
            ..Default::default()
        };

        let ctx = registry.set_runtime_context(
            CommandContext::new(vec![], HashMap::new(), ".".to_string()),
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
        registry.register_builtin_commands();

        let session = Arc::new(Mutex::new(Session::new()));
        let session_for_share = Arc::clone(&session);
        let mut ctx = CommandContext::new(vec![], HashMap::new(), ".".to_string());
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
        registry.register_builtin_commands();

        let session = Arc::new(Mutex::new(Session::new()));
        {
            let mut lock = session.lock().unwrap();
            for i in 0..20 {
                lock.add_message(Message::user(format!("Message {} {}", i, "x".repeat(300))));
            }
        }

        let session_for_compact = Arc::clone(&session);
        let mut ctx = CommandContext::new(vec!["50".to_string()], HashMap::new(), ".".to_string());
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
        let ctx = CommandContext::new(vec![], HashMap::new(), ".".to_string()).with_variables(
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

        let ctx = CommandContext::new(vec![], HashMap::new(), dir.path().display().to_string());
        let output =
            substitute_command_variables(&format!("before {{file:{}}} after", file_name), &ctx).unwrap();
        assert_eq!(output, "before file-content after");
    }
}
