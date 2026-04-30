use crate::Config;
use crate::OpenCodeError;
use crate::Session;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod sealed {
    pub trait Sealed {}
}

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
    #[allow(dead_code)]
    pub(crate) fn from_markdown(content: &str, file_path: &Path) -> Option<Self> {
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

        result = result.replace("${file}", &vars.file);
        result = result.replace("${selection}", &vars.selection);
        result = result.replace("${cwd}", &vars.cwd);
        result = result.replace("${git_branch}", &vars.git_branch);
        result = result.replace("${input}", &vars.input);
        result = result.replace("${session_id}", &vars.session_id);
        result = result.replace("${project_path}", &vars.project_path);

        result = result.replace("${cursor}", &vars.cursor);

        let env_regex = regex::Regex::new(r"\$\{env:([A-Za-z_][A-Za-z0-9_]*)\}").ok();
        if let Some(re) = env_regex {
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    std::env::var(var_name).unwrap_or_default()
                })
                .to_string();
        }

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

    #[allow(dead_code)]
    pub(crate) fn with_variables(mut self, vars: CommandVariables) -> Self {
        self.variables = vars;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn with_runtime(
        mut self,
        session: Option<&Session>,
        config: Option<&Config>,
        registry: Option<&crate::command::CommandRegistry>,
    ) -> Self {
        if let Some(session) = session {
            let messages: Vec<_> = session.messages.clone();
            self.on_clear_session = Some(Box::new(move || messages.len()));
            self.on_share_session = Some(Box::new(move || {
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
pub trait Command: Send + Sync + sealed::Sealed {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    async fn execute(&self, ctx: CommandContext) -> Result<String, OpenCodeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
}

#[allow(dead_code)]
pub fn format_models(config: &Config) -> String {
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

#[allow(dead_code)]
pub fn format_agents(config: &Config) -> String {
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

#[allow(dead_code)]
pub fn format_help_table(commands: &[CommandInfo]) -> String {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStep {
    Run {
        command: String,
    },
    AnalyzeFailure,
    InvokeAgent {
        agent_type: String,
    },
    InvokeSkill {
        skill_name: String,
    },
    Summarize {
        include_results: bool,
    },
    If {
        condition: String,
        then_steps: Vec<CommandStep>,
        else_steps: Option<Vec<CommandStep>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandWorkflow {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<CommandStep>,
}

impl CommandWorkflow {
    pub fn new(name: String, steps: Vec<CommandStep>) -> Self {
        Self {
            name,
            description: None,
            steps,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

#[allow(dead_code)]
pub struct WorkflowExecutor<C: CommandContextProvider> {
    context_provider: C,
}

pub trait CommandContextProvider: Send + Sync {
    fn get_session(&self) -> Option<&Session>;
    fn get_config(&self) -> Option<&Config>;
}
