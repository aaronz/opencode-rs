use clap::Args;
use directories::ProjectDirs;
use opencode_control_plane::SharedAcpStream;
use opencode_core::bus::SharedEventBus;
use opencode_core::config::ServerConfig;
use opencode_core::permission::PermissionManager;
use opencode_core::Config;
use opencode_llm::ModelRegistry;
use opencode_permission::ApprovalQueue;
use opencode_server::routes::acp_ws::SharedAcpClientRegistry;
use opencode_server::routes::share::ShareServer;
use opencode_server::streaming::{conn_state::ConnectionMonitor, ReconnectionStore};
use opencode_server::{run_server, ServerState};
use opencode_storage::{SqliteProjectRepository, SqliteSessionRepository, StorageService};
use opencode_tools::build_default_registry;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Args, Debug)]
pub struct ServeArgs {
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(short, long)]
    pub hostname: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serve_args_default() {
        let args = ServeArgs {
            port: None,
            hostname: None,
        };
        assert!(args.port.is_none());
        assert!(args.hostname.is_none());
    }

    #[test]
    fn test_serve_args_with_port() {
        let args = ServeArgs {
            port: Some(8080),
            hostname: None,
        };
        assert_eq!(args.port, Some(8080));
    }

    #[test]
    fn test_serve_args_with_hostname() {
        let args = ServeArgs {
            port: None,
            hostname: Some("0.0.0.0".to_string()),
        };
        assert_eq!(args.hostname.as_deref(), Some("0.0.0.0"));
    }

    #[test]
    fn test_serve_args_full() {
        let args = ServeArgs {
            port: Some(3000),
            hostname: Some("localhost".to_string()),
        };
        assert_eq!(args.port, Some(3000));
        assert_eq!(args.hostname.as_deref(), Some("localhost"));
    }
}

pub fn run(args: ServeArgs) {
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    runtime.block_on(async {
        if let Err(e) = run_serve(args).await {
            eprintln!("Server error: {}", e);
            std::process::exit(1);
        }
    });
}

async fn run_serve(args: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = Config::config_path();
    let config = Config::load(&config_path).unwrap_or_default();

    let default_server_cfg = ServerConfig::default();
    let server_cfg = config.server.as_ref().unwrap_or(&default_server_cfg);

    let port = args.port.or(server_cfg.port).unwrap_or(8080);
    let host = args
        .hostname
        .clone()
        .or_else(|| server_cfg.hostname.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    println!("Starting OpenCode server on {}:{}", host, port);

    let project_dirs = ProjectDirs::from("ai", "opencode", "opencode-rs")
        .expect("Failed to determine project directories");
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;
    let db_path = data_dir.join("opencode.db");

    let pool = opencode_storage::database::StoragePool::new(&db_path)?;
    let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
    let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));
    let storage = Arc::new(StorageService::new(session_repo, project_repo, pool));

    let models = Arc::new(ModelRegistry::new());
    let config = Arc::new(RwLock::new(config));
    let event_bus = SharedEventBus::default();
    let reconnection_store = ReconnectionStore::default();
    let connection_monitor = Arc::new(ConnectionMonitor::new());
    let share_server = Arc::new(RwLock::new(ShareServer::with_default_config()));
    let tool_registry = Arc::new(build_default_registry(None).await);
    let permission_manager = Arc::new(RwLock::new(PermissionManager::default()));

    let state = ServerState {
        storage,
        models,
        config,
        event_bus,
        reconnection_store,
        temp_db_dir: None,
        connection_monitor,
        share_server,
        acp_enabled: true,
        acp_stream: SharedAcpStream::default(),
        acp_client_registry: SharedAcpClientRegistry::default(),
        tool_registry,
        session_hub: Arc::new(opencode_server::routes::ws::SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
        permission_manager,
        approval_queue: Arc::new(RwLock::new(ApprovalQueue::default())),
        audit_log: None,
    };

    run_server(Arc::new(state), &host, port).await?;

    Ok(())
}
