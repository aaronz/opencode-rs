use clap::Args;
use directories::ProjectDirs;
use opencode_control_plane::SharedAcpStream;
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

#[cfg(feature = "desktop")]
use crate::webview::spawn_webview_thread;

#[derive(Args, Debug)]
pub struct DesktopArgs {
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(long)]
    pub hostname: Option<String>,

    #[arg(long)]
    pub no_browser: bool,

    #[arg(long)]
    pub acp_enabled: Option<bool>,
}

pub fn run(args: DesktopArgs) {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    runtime.block_on(async {
        if let Err(e) = run_desktop(args).await {
            eprintln!("Desktop mode error: {}", e);
            std::process::exit(1);
        }
    });
}

async fn run_desktop(args: DesktopArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Config::config_path();
    let config = Config::load(&config_path).unwrap_or_default();

    let default_server_cfg = ServerConfig::default();
    let server_cfg = config.server.as_ref().unwrap_or(&default_server_cfg);
    let desktop_cfg = server_cfg.desktop.as_ref();

    let port = args
        .port
        .or(desktop_cfg.and_then(|d| d.port))
        .or(server_cfg.port)
        .unwrap_or(3000);
    let host = args
        .hostname
        .clone()
        .or_else(|| desktop_cfg.and_then(|d| d.hostname.clone()))
        .or_else(|| server_cfg.hostname.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());
    let acp_enabled = args
        .acp_enabled
        .or(desktop_cfg.and_then(|d| d.enabled))
        .or(server_cfg.acp.as_ref().and_then(|a| a.enabled))
        .unwrap_or(true);
    let auto_open_browser = desktop_cfg
        .and_then(|d| d.auto_open_browser)
        .unwrap_or(true);

    println!("Starting OpenCode desktop mode on {}:{}", host, port);

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
        config: config.clone(),
        event_bus: event_bus.clone(),
        reconnection_store,
        connection_monitor,
        share_server,
        acp_enabled,
        acp_stream: SharedAcpStream::default(),
    };

    if auto_open_browser && !args.no_browser {
        let url = format!("http://{}:{}", host, port);
        
        #[cfg(feature = "desktop")]
        {
            println!("Opening embedded WebView...");
            let _webview_handle = spawn_webview_thread(url, "OpenCode".to_string())
                .map_err(|e| format!("Failed to spawn WebView: {}", e))?;
        }

        #[cfg(not(feature = "desktop"))]
        {
            let _ = open_browser(&url);
        }
    }

    run_server(Arc::new(state), &host, port).await?;

    Ok(())
}

#[cfg(not(feature = "desktop"))]
use std::process::Command;

#[cfg(not(feature = "desktop"))]
#[cfg(target_os = "macos")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("open").arg(url).spawn()?;
    Ok(())
}

#[cfg(not(feature = "desktop"))]
#[cfg(target_os = "windows")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("cmd").args(["/c", "start", url]).spawn()?;
    Ok(())
}

#[cfg(not(feature = "desktop"))]
#[cfg(target_os = "linux")]
fn open_browser(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

#[cfg(not(feature = "desktop"))]
#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn open_browser(_url: &str) -> std::io::Result<()> {
    Ok(())
}