use clap::Args;
use directories::ProjectDirs;
use opencode_core::bus::SharedEventBus;
use opencode_core::config::ServerConfig;
use opencode_core::Config;
use opencode_llm::ModelRegistry;
use opencode_server::routes::share::ShareServer;
use opencode_server::streaming::{conn_state::ConnectionMonitor, ReconnectionStore};
use opencode_server::{run_server, ServerState};
use opencode_storage::StorageService;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Args, Debug)]
pub struct WebArgs {
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(short, long)]
    pub hostname: Option<String>,
}

pub fn run(args: WebArgs) {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    runtime.block_on(async {
        if let Err(e) = run_web(args).await {
            eprintln!("Web interface error: {}", e);
            std::process::exit(1);
        }
    });
}

async fn run_web(args: WebArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Config::config_path();
    let config = Config::load(&config_path).unwrap_or_default();

    let default_server_cfg = ServerConfig::default();
    let server_cfg = config.server.as_ref().unwrap_or(&default_server_cfg);

    let port = args.port
        .or(server_cfg.port)
        .unwrap_or(3000);
    let host = args.hostname.clone()
        .or_else(|| server_cfg.hostname.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    println!("Starting OpenCode web interface on {}:{}", host, port);
    println!("Open http://{}:{} in your browser", host, port);

    let project_dirs = ProjectDirs::from("ai", "opencode", "opencode-rs")
        .expect("Failed to determine project directories");
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("opencode.db");

    let storage = Arc::new(StorageService::new(
        opencode_storage::database::StoragePool::new(&db_path)?,
    ));

    let models = Arc::new(ModelRegistry::new());
    let config = Arc::new(RwLock::new(config));
    let event_bus = SharedEventBus::default();
    let reconnection_store = ReconnectionStore::default();
    let connection_monitor = Arc::new(ConnectionMonitor::new());
    let share_server = Arc::new(RwLock::new(ShareServer::with_default_config()));

    let state = ServerState {
        storage,
        models,
        config,
        event_bus,
        reconnection_store,
        connection_monitor,
        share_server,
        acp_enabled: true,
    };

    run_server(Arc::new(state), &host, port).await?;

    Ok(())
}
