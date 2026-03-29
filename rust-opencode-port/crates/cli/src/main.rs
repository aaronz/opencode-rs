mod cmd;

use clap::{Args, Parser, Subcommand};
use cmd::{
    account::{self, AccountArgs},
    acp::{self, AcpArgs},
    agent::{self, AgentArgs},
    attach::{self, AttachArgs},
    db::{self, DbArgs},
    debug::{self, DebugArgs},
    export::{self, ExportArgs},
    files::{self, FilesArgs},
    generate::{self, GenerateArgs},
    github::{self, GitHubArgs},
    import::{self, ImportArgs},
    list::{self, ListArgs},
    mcp::{self, McpArgs},
    models::{self, ModelsArgs},
    palette::{self, PaletteArgs},
    pr::{self, PrArgs},
    project::{self, ProjectArgs},
    prompt::{self, PromptArgs},
    providers::{self, ProvidersArgs},
    run::{self, RunArgs},
    serve::{self, ServeArgs},
    session::{self, SessionArgs},
    shortcuts::{self, ShortcutsArgs},
    stats::{self, StatsArgs},
    thread::{self, ThreadArgs},
    ui::{self, UiArgs},
    uninstall::{self, UninstallArgs},
    upgrade::{self, UpgradeArgs},
    web::{self, WebArgs},
    workspace::{self, WorkspaceArgs},
    workspace_serve::{self, WorkspaceServeArgs},
};
use opencode_tui::App;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opencode-rs")]
#[command(version = "0.1.0")]
#[command(about = "AI coding agent in Rust", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    // Global flags for TUI
    #[arg(short = 'v', long = "version")]
    pub version: bool,

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

    #[command(about = "Manage accounts")]
    Account(AccountArgs),

    #[command(about = "Manage agents")]
    Agent(AgentArgs),

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

    #[command(about = "Manage database")]
    Db(DbArgs),

    #[command(about = "Manage GitHub integration")]
    GitHub(GitHubArgs),

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
    Attach(AttachArgs),

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

    #[command(about = "Start opencode-rs terminal user interface")]
    Tui(TuiArgs),
}

#[derive(Args, Debug)]
pub struct TuiArgs {
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

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("opencode-rs 0.1.0");
        return;
    }

    match cli.command {
        Some(Commands::Run(args)) => run::run(args),
        Some(Commands::Serve(args)) => serve::run(args),
        Some(Commands::Account(args)) => account::run(args),
        Some(Commands::Agent(args)) => agent::run(args),
        Some(Commands::Models(args)) => models::run(args),
        Some(Commands::Providers(args)) => providers::run(args),
        Some(Commands::Mcp(args)) => mcp::run(args),
        Some(Commands::Session(args)) => session::run(args),
        Some(Commands::List(args)) => list::run(args),
        Some(Commands::Stats(args)) => stats::run(args),
        Some(Commands::Db(args)) => db::run(args),
        Some(Commands::GitHub(args)) => github::run(args),
        Some(Commands::Pr(args)) => pr::run(args),
        Some(Commands::Export(args)) => export::run(args),
        Some(Commands::Import(args)) => import::run(args),
        Some(Commands::Generate(args)) => generate::run(args),
        Some(Commands::Web(args)) => web::run(args),
        Some(Commands::Thread(args)) => thread::run(args),
        Some(Commands::Attach(args)) => attach::run(args),
        Some(Commands::Uninstall(args)) => uninstall::run(args),
        Some(Commands::Upgrade(args)) => upgrade::run(args),
        Some(Commands::Debug(args)) => debug::run(args),
        Some(Commands::Acp(args)) => acp::run(args),
        Some(Commands::WorkspaceServe(args)) => workspace_serve::run(args),
        Some(Commands::Palette(args)) => palette::run(args),
        Some(Commands::Shortcuts(args)) => shortcuts::run(args),
        Some(Commands::Workspace(args)) => workspace::run(args),
        Some(Commands::Ui(args)) => ui::run(args),
        Some(Commands::Project(args)) => project::run(args),
        Some(Commands::Files(args)) => files::run(args),
        Some(Commands::Prompt(args)) => prompt::run(args),
        Some(Commands::Tui(args)) => run_tui(args),
        None => {
            run_tui(TuiArgs {
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
}

fn run_tui(args: TuiArgs) {
    let mut app = App::new();

    if let Some(prompt) = args.prompt {
        app.input = prompt;
    }

    if let Some(agent) = args.agent {
        app.agent = agent;
    }

    if let Err(e) = app.run() {
        eprintln!("Error running TUI: {}", e);
    }
}
