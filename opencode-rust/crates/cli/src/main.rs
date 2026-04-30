mod cmd;
mod env_parser;
mod output;

#[cfg(feature = "desktop")]
mod webview;

use clap::{Args, Parser, Subcommand};
use cmd::{
    account::{self, AccountArgs},
    acp::{self, AcpArgs},
    agent::{self, AgentArgs},
    attach::{self, AttachArgs},
    bash::{self, BashArgs},
    completion::{self, CompletionArgs},
    config::{self, ConfigArgs},
    context::{self, ContextArgs},
    db::{self, DbArgs},
    debug::{self, DebugArgs},
    desktop::{self, DesktopArgs},
    export::{self, ExportArgs},
    files::{self, FilesArgs},
    generate::{self, GenerateArgs},
    github::{self, GitHubArgs},
    gitlab::{self, GitLabArgs},
    import::{self, ImportArgs},
    list::{self, ListArgs},
    mcp::{self, McpArgs},
    models::{self, ModelsArgs},
    palette::{self, PaletteArgs},
    permissions::{self, PermissionsArgs},
    plugin::{self, PluginArgs},
    pr::{self, PrArgs},
    project::{self, ProjectArgs},
    prompt::{self, PromptArgs},
    providers::{self, ProvidersArgs},
    quick::{self, QuickArgs},
    run::{self, RunArgs},
    serve::{self, ServeArgs},
    session::{self, SessionArgs},
    shortcuts::{self, ShortcutsArgs},
    stats::{self, StatsArgs},
    terminal::{self, TerminalArgs},
    thread::{self, ThreadArgs},
    ui::{self, UiArgs},
    uninstall::{self, UninstallArgs},
    upgrade::{self, UpgradeArgs},
    web::{self, WebArgs},
    workspace::{self, WorkspaceArgs},
    workspace_serve::{self, WorkspaceServeArgs},
};
use opencode_cli::finalize_tui_run_result;
use opencode_core::Config;
use opencode_llm::ModelRegistry;
use opencode_plugin::PluginManager;
use opencode_tui::App;
use opencode_util::logging::{log_file_path, Logger};
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "opencode-rs")]
#[command(version = "0.1.0")]
#[command(disable_version_flag = true)]
#[command(about = "AI coding agent in Rust", long_about = None)]
struct Cli {
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    // Global flags for TUI
    #[arg(short = 'v', long = "version")]
    pub version: bool,

    #[arg(short = 'V', long = "verbose", global = true)]
    pub verbose: bool,

    #[arg(short = 'c', long = "continue")]
    pub continue_session: Option<String>,

    #[arg(short = 's', long = "session")]
    pub session: Option<String>,

    #[arg(long = "fork")]
    pub fork: bool,

    #[arg(long = "prompt")]
    pub prompt: Option<String>,

    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    #[arg(long = "agent")]
    pub agent: Option<String>,

    #[arg(long = "port")]
    pub port: Option<u16>,

    #[arg(long = "hostname")]
    pub hostname: Option<String>,

    #[arg(long = "log-level")]
    pub log_level: Option<String>,

    #[arg(long = "print-logs")]
    pub print_logs: bool,

    // Positional argument for project path
    #[arg(value_name = "PROJECT")]
    pub project: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Run opencode with optional prompt")]
    Run(RunArgs),

    #[command(about = "Start the opencode server")]
    Serve(ServeArgs),

    #[command(about = "Start desktop mode (TUI + server)")]
    Desktop(DesktopArgs),

    #[command(about = "Manage accounts", alias = "auth")]
    Account(AccountArgs),

    #[command(about = "Show effective config")]
    Config(ConfigArgs),

    #[command(about = "Manage agents")]
    Agent(AgentArgs),

    #[command(about = "Execute shell command")]
    Bash(BashArgs),

    #[command(about = "Generate shell completions")]
    Completion(CompletionArgs),

    #[command(about = "List available models")]
    Models(ModelsArgs),

    #[command(about = "List available providers")]
    Providers(ProvidersArgs),

    #[command(about = "Manage MCP servers")]
    Mcp(McpArgs),

    #[command(about = "Manage sessions")]
    Session(SessionArgs),

    #[command(about = "List sessions")]
    List(ListArgs),

    #[command(about = "Show stats")]
    Stats(StatsArgs),

    #[command(about = "Manage terminal panel")]
    Terminal(TerminalArgs),

    #[command(about = "Manage database")]
    Db(DbArgs),

    #[command(about = "Manage GitHub integration")]
    GitHub(GitHubArgs),

    #[command(about = "Manage GitLab integration")]
    GitLab(GitLabArgs),

    #[command(about = "Manage pull requests")]
    Pr(PrArgs),

    #[command(about = "Export data")]
    Export(ExportArgs),

    #[command(about = "Import data")]
    Import(ImportArgs),

    #[command(about = "Generate code")]
    Generate(GenerateArgs),

    #[command(about = "Open web interface")]
    Web(WebArgs),

    #[command(about = "Start TUI in thread mode")]
    Thread(ThreadArgs),

    #[command(about = "Attach to running session")]
    Join(AttachArgs),

    #[command(about = "Uninstall opencode")]
    Uninstall(UninstallArgs),

    #[command(about = "Upgrade opencode")]
    Upgrade(UpgradeArgs),

    #[command(about = "Debug commands")]
    Debug(DebugArgs),

    #[command(about = "Manage ACP protocol")]
    Acp(AcpArgs),

    #[command(about = "Workspace serve")]
    WorkspaceServe(WorkspaceServeArgs),

    #[command(about = "Command palette")]
    Palette(PaletteArgs),

    #[command(about = "Manage plugins")]
    Plugin(PluginArgs),

    #[command(about = "Manage permissions")]
    Permissions(PermissionsArgs),

    #[command(about = "Manage keyboard shortcuts")]
    Shortcuts(ShortcutsArgs),

    #[command(about = "Manage workspace")]
    Workspace(WorkspaceArgs),

    #[command(about = "Manage UI")]
    Ui(UiArgs),

    #[command(about = "Manage projects")]
    Project(ProjectArgs),

    #[command(about = "Manage files")]
    Files(FilesArgs),

    #[command(about = "Prompt commands")]
    Prompt(PromptArgs),

    #[command(about = "Quick actions")]
    Quick(QuickArgs),

    #[command(about = "Start opencode-rs terminal user interface")]
    Tui(TuiArgs),

    #[command(about = "Inspect context")]
    Context(ContextArgs),
}

#[derive(Args, Debug)]
pub struct TuiArgs {
    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub action: Option<String>,

    #[arg(short, long)]
    pub continue_session: Option<String>,

    #[arg(short = 's', long)]
    pub session: Option<String>,

    #[arg(long)]
    pub fork: bool,

    #[arg(long)]
    pub prompt: Option<String>,

    #[arg(short = 'm', long)]
    pub model: Option<String>,

    #[arg(long)]
    pub agent: Option<String>,

    #[arg(long)]
    pub port: Option<u16>,

    #[arg(long)]
    pub hostname: Option<String>,

    #[arg(value_name = "PROJECT")]
    pub project: Option<PathBuf>,
}

fn main() -> ExitCode {
    if let Err(e) = Logger::new().with_file(log_file_path()).init() {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }

    tracing::info!(log_path = %log_file_path().display(), "OpenCode RS starting");

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            err.exit();
        }
    };

    if cli.version {
        println!("opencode-rs 0.1.0");
        return ExitCode::SUCCESS;
    }

    init_plugins();

    match cli.command {
        Some(Commands::Run(args)) => run::run(args),
        Some(Commands::Serve(args)) => serve::run(args),
        Some(Commands::Desktop(args)) => desktop::run(args),
        Some(Commands::Account(args)) => account::run(args),
        Some(Commands::Config(args)) => config::run(args),
        Some(Commands::Agent(args)) => agent::run(args),
        Some(Commands::Bash(args)) => bash::run(args),
        Some(Commands::Completion(args)) => completion::run(args),
        Some(Commands::Models(args)) => models::run(args),
        Some(Commands::Providers(args)) => {
            if let Err(e) = providers::run(args) {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
        }
        Some(Commands::Mcp(args)) => mcp::run(args),
        Some(Commands::Session(args)) => session::run(args),
        Some(Commands::List(args)) => list::run(args),
        Some(Commands::Stats(args)) => stats::run(args),
        Some(Commands::Terminal(args)) => terminal::run(args),
        Some(Commands::Db(args)) => db::run(args),
        Some(Commands::GitHub(args)) => github::run(args),
        Some(Commands::GitLab(args)) => gitlab::run(args),
        Some(Commands::Pr(args)) => pr::run(args),
        Some(Commands::Export(args)) => export::run(args),
        Some(Commands::Import(args)) => import::run(args),
        Some(Commands::Generate(args)) => generate::run(args),
        Some(Commands::Web(args)) => web::run(args),
        Some(Commands::Thread(args)) => thread::run(args),
        Some(Commands::Join(args)) => attach::run(args),
        Some(Commands::Uninstall(args)) => uninstall::run(args),
        Some(Commands::Upgrade(args)) => upgrade::run(args),
        Some(Commands::Debug(args)) => debug::run(args),
        Some(Commands::Acp(args)) => acp::run(args),
        Some(Commands::WorkspaceServe(args)) => workspace_serve::run(args),
        Some(Commands::Palette(args)) => palette::run(args),
        Some(Commands::Plugin(args)) => plugin::run(args),
        Some(Commands::Permissions(args)) => permissions::run(args),
        Some(Commands::Shortcuts(args)) => shortcuts::run(args),
        Some(Commands::Workspace(args)) => workspace::run(args),
        Some(Commands::Ui(args)) => ui::run(args),
        Some(Commands::Project(args)) => project::run(args),
        Some(Commands::Files(args)) => files::run(args),
        Some(Commands::Prompt(args)) => prompt::run(args),
        Some(Commands::Quick(args)) => quick::run(args),
        Some(Commands::Tui(args)) => run_tui(args),
        Some(Commands::Context(args)) => {
            if let Err(e) = context::run_context_command(args) {
                eprintln!("{}", e);
                return ExitCode::FAILURE;
            }
        }
        None => {
            run_tui(TuiArgs {
                json: false,
                action: None,
                continue_session: cli.continue_session,
                session: cli.session,
                fork: cli.fork,
                prompt: cli.prompt,
                model: cli.model,
                agent: cli.agent,
                port: cli.port,
                hostname: cli.hostname,
                project: cli.project,
            });
        }
    }
    ExitCode::SUCCESS
}

fn init_plugins() {
    let mut plugin_manager = PluginManager::new();
    if let Err(error) = plugin_manager.discover_default_dirs() {
        tracing::warn!("Plugin discovery failed: {}", error);
        return;
    }

    if let Err(error) = plugin_manager.init_all() {
        tracing::warn!("Plugin initialization failed: {}", error);
    }
}

#[expect(
    clippy::expect_used,
    reason = "CLI entry point where failure should panic with clear error messages"
)]
fn run_tui(args: TuiArgs) {
    if let Some(action) = args.action.as_deref() {
        match action {
            "open-model-dialog" => {
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "dialog": "model-selection",
                        }))
                        .expect("failed to serialize JSON output")
                    );
                } else {
                    println!("model-selection");
                }
                return;
            }
            "close-model-dialog" => {
                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "dialog": "model-selection",
                            "status": "closed",
                        }))
                        .expect("failed to serialize JSON output")
                    );
                } else {
                    println!("closed model-selection");
                }
                return;
            }
            "confirm-model-switch" => {
                let model_id = match args.model.clone() {
                    Some(model_id) => model_id,
                    None => {
                        eprintln!("Missing --model for confirm-model-switch");
                        std::process::exit(1);
                    }
                };

                let registry = ModelRegistry::default();
                if registry.get(&model_id).is_none() {
                    eprintln!("Unknown model: {}", model_id);
                    std::process::exit(1);
                }

                let path = Config::config_path();
                let mut config = Config::load(&path).unwrap_or_default();
                config.model = Some(model_id.clone());
                if let Err(error) = config.save(&path) {
                    eprintln!("Failed to save config: {}", error);
                    std::process::exit(1);
                }

                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "active_model": model_id,
                            "status": "confirmed",
                        }))
                        .expect("failed to serialize JSON output")
                    );
                } else {
                    println!("confirmed model switch to {}", model_id);
                }
                return;
            }
            "cancel-model-switch" => {
                let path = Config::config_path();
                let config = Config::load(&path).unwrap_or_default();
                let active_model = config.model;

                if args.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "active_model": active_model,
                            "status": "cancelled",
                        }))
                        .expect("failed to serialize JSON output")
                    );
                } else {
                    println!("cancelled model switch");
                }
                return;
            }
            other => {
                eprintln!("Unknown tui action: {}", other);
                std::process::exit(1);
            }
        }
    }

    let env_config = env_parser::EnvVarConfig::parse();

    tracing::debug!(
        auto_share = ?env_config.auto_share,
        config_path = ?env_config.config_path,
        config_dir = ?env_config.config_dir,
        disable_autoupdate = ?env_config.disable_autoupdate,
        enable_exa = ?env_config.enable_exa,
        "Environment variable configuration parsed"
    );

    let mut app = App::new();

    // Initialize LLM provider
    if let Err(e) = app.init_llm_provider() {
        tracing::warn!(error = %e, "LLM provider not initialized before TUI run");
    }

    if let Some(prompt) = args.prompt {
        app.input = prompt;
    }

    if let Some(agent) = args.agent {
        app.agent = agent;
    }

    if let Err(message) = finalize_tui_run_result(app.run(), App::restore_terminal_after_error) {
        tracing::error!(error = %message, "TUI run failed");
    }
}
