mod cmd;

use clap::{Parser, Subcommand};
use cmd::{
    account::{self, AccountArgs},
    acp::{self, AcpArgs},
    agent::{self, AgentArgs},
    attach::{self, AttachArgs},
    db::{self, DbArgs},
    debug::{self, DebugArgs},
    export::{self, ExportArgs},
    generate::{self, GenerateArgs},
    github::{self, GitHubArgs},
    import::{self, ImportArgs},
    list::{self, ListArgs},
    mcp::{self, McpArgs},
    models::{self, ModelsArgs},
    pr::{self, PrArgs},
    providers::{self, ProvidersArgs},
    run::{self, RunArgs},
    serve::{self, ServeArgs},
    session::{self, SessionArgs},
    stats::{self, StatsArgs},
    thread::{self, ThreadArgs},
    uninstall::{self, UninstallArgs},
    upgrade::{self, UpgradeArgs},
    web::{self, WebArgs},
    workspace_serve::{self, WorkspaceServeArgs},
};
use opencode_core::{Config, Message, ProjectManager, Role, Session, SessionInfo};
use opencode_tui::App;
use serde::Serialize;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opencode-rs")]
#[command(version = "0.1.0")]
#[command(about = "AI coding agent in Rust", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

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
}

fn main() {
    let cli = Cli::parse();

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
        None => {
            println!("No command specified. Use --help for usage.");
        }
    }
}
