use std::collections::HashMap;
use std::path::{Path, PathBuf};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;
use crate::Config;

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
                        triggers = value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect();
                    }
                    "agent" => agent = Some(value.to_string()),
                    "model" => model = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        let name = name.or_else(|| {
            file_path.file_stem().and_then(|s: &std::ffi::OsStr| s.to_str()).map(|s| s.trim_start_matches('/').to_string())
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
            result = re.replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                std::env::var(var_name).unwrap_or_default()
            }).to_string();
        }
        
        // File content: {file:path} - read file and insert content
        let file_regex = regex::Regex::new(r"\{file:([^}]+)\}").ok();
        if let Some(re) = file_regex {
            result = re.replace_all(&result, |caps: &regex::Captures| {
                let file_path = &caps[1];
                std::fs::read_to_string(file_path).unwrap_or_else(|_| format!("[Cannot read file: {}]", file_path))
            }).to_string();
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

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: String,
    pub variables: CommandVariables,
}

impl CommandContext {
    pub fn new(args: Vec<String>, env: HashMap<String, String>, working_dir: String) -> Self {
        Self {
            args,
            env,
            working_dir,
            variables: CommandVariables::default(),
        }
    }

    pub fn with_variables(mut self, vars: CommandVariables) -> Self {
        self.variables = vars;
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
        let mut list: Vec<(&str, &str)> = self.commands
            .iter()
            .map(|(name, cmd)| (name.as_str(), cmd.description()))
            .collect();
        
        for (name, def) in &self.definitions {
            list.push((name.as_str(), &def.description));
        }
        
        list.sort_by(|a, b| a.0.cmp(b.0));
        list
    }

    pub fn list_commands(&self) -> Vec<String> {
        let mut names: Vec<String> = self.commands.keys().cloned().collect();
        names.extend(self.definitions.keys().cloned());
        names.sort();
        names
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

pub fn substitute_command_variables(template: &str, context: &CommandContext) -> String {
    let mut result = Config::substitute_variables(template, Some(Path::new(&context.working_dir)));

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

                std::fs::read_to_string(&resolved)
                    .unwrap_or_else(|_| format!("{{file:{}}}", path))
            })
            .to_string();
    }

    result
}

struct HelpCommand;

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

    async fn execute(&self, ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        if let Some(cmd_name) = ctx.args.first() {
            Ok(format!("Help for command: {}", cmd_name))
        } else {
            Ok("Available commands: help, test, debug".to_string())
        }
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
        Ok(format!("Running tests matching: {}", if pattern.is_empty() { "all".to_string() } else { pattern }))
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

    async fn execute(&self, _ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
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

    async fn execute(&self, _ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
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

    async fn execute(&self, _ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
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

    async fn execute(&self, _ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
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

    async fn execute(&self, _ctx: CommandContext) -> Result<String, crate::OpenCodeError> {
        Ok("Context compaction triggered".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            .execute(CommandContext::new(vec!["unit".to_string()], HashMap::new(), ".".to_string()))
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

    #[test]
    fn test_substitute_command_variables_input_and_selection() {
        let ctx = CommandContext::new(vec![], HashMap::new(), ".".to_string()).with_variables(CommandVariables {
            input: "do thing".to_string(),
            selection: "line 1".to_string(),
            ..CommandVariables::default()
        });
        let output = substitute_command_variables("Run: {input} // {selection}", &ctx);
        assert_eq!(output, "Run: do thing // line 1");
    }

    #[test]
    fn test_substitute_command_variables_file_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("prompt.txt");
        std::fs::write(&file_path, "file-content").unwrap();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        let ctx = CommandContext::new(vec![], HashMap::new(), dir.path().display().to_string());
        let output = substitute_command_variables(&format!("before {{file:{}}} after", file_name), &ctx);
        assert_eq!(output, "before file-content after");
    }
}
