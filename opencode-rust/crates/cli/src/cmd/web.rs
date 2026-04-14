use clap::Args;
use directories::ProjectDirs;
use opencode_control_plane::SharedAcpStream;
use opencode_core::bus::SharedEventBus;
use opencode_core::config::ServerConfig;
use opencode_core::session_sharing::SessionSharing;
use opencode_core::Config;
use opencode_llm::ModelRegistry;
use opencode_server::routes::acp_ws::SharedAcpClientRegistry;
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

pub struct WebServerState {
    pub storage: Arc<StorageService>,
    #[allow(dead_code)]
    pub session_sharing: Arc<SessionSharing>,
    pub models: Arc<ModelRegistry>,
    pub config: Arc<RwLock<Config>>,
    pub event_bus: SharedEventBus,
    pub reconnection_store: ReconnectionStore,
    pub connection_monitor: Arc<ConnectionMonitor>,
    pub share_server: Arc<RwLock<ShareServer>>,
    pub acp_enabled: bool,
    pub acp_stream: SharedAcpStream,
    pub acp_client_registry: SharedAcpClientRegistry,
}

impl WebServerState {
    pub fn new(
        storage: Arc<StorageService>,
        session_sharing: Arc<SessionSharing>,
        models: Arc<ModelRegistry>,
        config: Arc<RwLock<Config>>,
        event_bus: SharedEventBus,
        share_server: Arc<RwLock<ShareServer>>,
    ) -> Self {
        Self {
            storage,
            session_sharing,
            models,
            config,
            event_bus,
            reconnection_store: ReconnectionStore::default(),
            connection_monitor: Arc::new(ConnectionMonitor::new()),
            share_server,
            acp_enabled: true,
            acp_stream: SharedAcpStream::default(),
            acp_client_registry: SharedAcpClientRegistry::default(),
        }
    }

    pub fn into_server_state(self: Arc<Self>) -> ServerState {
        ServerState {
            storage: self.storage.clone(),
            models: self.models.clone(),
            config: self.config.clone(),
            event_bus: self.event_bus.clone(),
            reconnection_store: self.reconnection_store.clone(),
            connection_monitor: self.connection_monitor.clone(),
            share_server: self.share_server.clone(),
            acp_enabled: self.acp_enabled,
            acp_stream: self.acp_stream.clone(),
            acp_client_registry: self.acp_client_registry.clone(),
        }
    }
}

async fn run_web(args: WebArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Config::config_path();
    let config = Config::load(&config_path).unwrap_or_default();

    let default_server_cfg = ServerConfig::default();
    let server_cfg = config.server.as_ref().unwrap_or(&default_server_cfg);

    let port = args.port.or(server_cfg.port).unwrap_or(3000);
    let host = args
        .hostname
        .clone()
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

    let session_sharing = Arc::new(SessionSharing::with_default_path());
    let models = Arc::new(ModelRegistry::new());
    let config = Arc::new(RwLock::new(config));
    let event_bus = SharedEventBus::default();
    let share_server = Arc::new(RwLock::new(ShareServer::with_default_config()));

    let web_state = Arc::new(WebServerState::new(
        storage,
        session_sharing,
        models,
        config,
        event_bus,
        share_server,
    ));

    let server_state = web_state.clone().into_server_state();
    run_server(Arc::new(server_state), &host, port).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_web_args_default() {
        let args = WebArgs {
            port: None,
            hostname: None,
        };
        assert_eq!(args.port, None);
        assert_eq!(args.hostname, None);
    }

    #[test]
    fn test_web_args_with_port() {
        let args = WebArgs {
            port: Some(8080),
            hostname: None,
        };
        assert_eq!(args.port, Some(8080));
    }

    #[test]
    fn test_web_args_with_hostname() {
        let args = WebArgs {
            port: None,
            hostname: Some("0.0.0.0".to_string()),
        };
        assert_eq!(args.hostname, Some("0.0.0.0".to_string()));
    }

    #[test]
    fn test_web_args_with_all_options() {
        let args = WebArgs {
            port: Some(3000),
            hostname: Some("localhost".to_string()),
        };
        assert_eq!(args.port, Some(3000));
        assert_eq!(args.hostname, Some("localhost".to_string()));
    }

    #[test]
    fn test_web_server_state_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let storage = Arc::new(StorageService::new(
            opencode_storage::database::StoragePool::new(&db_path).unwrap(),
        ));
        let session_sharing = Arc::new(SessionSharing::with_default_path());
        let models = Arc::new(ModelRegistry::new());
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = SharedEventBus::default();
        let share_server = Arc::new(RwLock::new(ShareServer::with_default_config()));

        let web_state = WebServerState::new(
            storage,
            session_sharing,
            models,
            config,
            event_bus,
            share_server,
        );

        assert!(web_state.acp_enabled);
    }

    #[test]
    fn test_web_server_state_into_server_state() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let storage = Arc::new(StorageService::new(
            opencode_storage::database::StoragePool::new(&db_path).unwrap(),
        ));
        let session_sharing = Arc::new(SessionSharing::with_default_path());
        let models = Arc::new(ModelRegistry::new());
        let config = Arc::new(RwLock::new(Config::default()));
        let event_bus = SharedEventBus::default();
        let share_server = Arc::new(RwLock::new(ShareServer::with_default_config()));

        let web_state = Arc::new(WebServerState::new(
            storage,
            session_sharing,
            models,
            config,
            event_bus,
            share_server,
        ));

        let server_state = web_state.clone().into_server_state();
        assert_eq!(server_state.acp_enabled, true);
    }
}
