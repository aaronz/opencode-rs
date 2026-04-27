use crate::command::types::*;
use crate::OpenCodeError;
use async_trait::async_trait;

pub struct HelpCommand;

impl sealed::Sealed for HelpCommand {}
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

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, OpenCodeError> {
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

pub struct TestCommand;

impl sealed::Sealed for TestCommand {}
#[async_trait]
#[allow(dead_code)]
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

    async fn execute(&self, ctx: CommandContext) -> Result<String, OpenCodeError> {
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

pub struct DebugCommand;

impl sealed::Sealed for DebugCommand {}
#[async_trait]
#[allow(dead_code)]
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

    async fn execute(&self, ctx: CommandContext) -> Result<String, OpenCodeError> {
        let section = ctx.args.first().map(|s| s.as_str()).unwrap_or("all");
        Ok(format!("Debug info for section: {}", section))
    }
}

pub struct ClearCommand;

impl sealed::Sealed for ClearCommand {}
#[async_trait]
#[allow(dead_code)]
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

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, OpenCodeError> {
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

pub struct ModelsCommand;

impl sealed::Sealed for ModelsCommand {}
#[async_trait]
#[allow(dead_code)]
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

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, OpenCodeError> {
        if let Some(get_models) = ctx.on_get_models.take() {
            return Ok(get_models());
        }

        Ok("Available models: configure providers and models in your config".to_string())
    }
}

pub struct AgentsCommand;

impl sealed::Sealed for AgentsCommand {}
#[async_trait]
#[allow(dead_code)]
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

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, OpenCodeError> {
        if let Some(get_agents) = ctx.on_get_agents.take() {
            return Ok(get_agents());
        }

        Ok("Available agents: configure agents in your config".to_string())
    }
}

pub struct ShareCommand;

impl sealed::Sealed for ShareCommand {}
#[async_trait]
#[allow(dead_code)]
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

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, OpenCodeError> {
        if let Some(share_session) = ctx.on_share_session.take() {
            let url = share_session().map_err(OpenCodeError::Session)?;
            return Ok(format!("Session shared: {}", url));
        }

        Ok("Session shared".to_string())
    }
}

pub struct CompactCommand;

impl sealed::Sealed for CompactCommand {}
#[async_trait]
#[allow(dead_code)]
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

    async fn execute(&self, mut ctx: CommandContext) -> Result<String, OpenCodeError> {
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
