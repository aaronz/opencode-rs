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
use opencode_server::{run_server_with_shutdown, ServerState};
use opencode_storage::{SqliteProjectRepository, SqliteSessionRepository, StorageService};
use opencode_tools::build_default_registry;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::signal;
use tokio::sync::oneshot;

#[cfg(feature = "desktop")]
use crate::webview::WebViewManager;

#[derive(Args, Debug)]
pub(crate) struct DesktopArgs {
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(long)]
    pub hostname: Option<String>,

    #[arg(long)]
    pub no_browser: bool,

    #[arg(long)]
    pub acp_enabled: Option<bool>,
}

pub(crate) fn run(args: DesktopArgs) {
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
        config: config.clone(),
        event_bus: event_bus.clone(),
        reconnection_store,
        temp_db_dir: None,
        connection_monitor,
        share_server,
        acp_enabled,
        acp_stream: SharedAcpStream::default(),
        acp_client_registry: SharedAcpClientRegistry::default(),
        tool_registry,
        session_hub: Arc::new(opencode_server::routes::ws::SessionHub::new(256)),
        server_start_time: std::time::SystemTime::now(),
        permission_manager,
        approval_queue: Arc::new(RwLock::new(ApprovalQueue::default())),
        audit_log: None,
        runtime: opencode_server::build_placeholder_runtime(),
    };

    #[cfg(feature = "desktop")]
    let mut webview_manager = if auto_open_browser && !args.no_browser {
        let url = format!("http://{}:{}", host, port);
        println!("Opening embedded WebView...");
        Some(
            WebViewManager::new(&url, "OpenCode")
                .map_err(|e| format!("Failed to create WebView: {}", e))?,
        )
    } else {
        None
    };

    #[cfg(not(feature = "desktop"))]
    let webview_manager: Option<()> = if auto_open_browser && !args.no_browser {
        let url = format!("http://{}:{}", host, port);
        println!("Desktop WebView not available, opening browser...");
        let _ = open_browser(&url);
        Some(())
    } else {
        None
    };

    let (server_shutdown_tx, server_shutdown_rx) = oneshot::channel::<()>();

    #[cfg(feature = "desktop")]
    let webview_close_rx = webview_manager.as_mut().and_then(|m| m.close_receiver());

    #[cfg(not(feature = "desktop"))]
    let _webview_close_rx: Option<oneshot::Receiver<()>> = None;

    #[cfg(feature = "desktop")]
    let shutdown_result = tokio::select! {
        result = run_server_with_shutdown(Arc::new(state), &host, port, server_shutdown_rx) => {
            result
        }
        _ = async {
            if let Some(rx) = webview_close_rx {
                rx.await.ok();
            }
        } => {
            let _ = server_shutdown_tx.send(());
            Ok::<_, std::io::Error>(())
        }
        _ = signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down...");
            if let Some(ref manager) = webview_manager {
                manager.stop();
            }
            let _ = server_shutdown_tx.send(());
            Ok::<_, std::io::Error>(())
        }
    };

    #[cfg(not(feature = "desktop"))]
    let shutdown_result = tokio::select! {
        result = run_server_with_shutdown(Arc::new(state), &host, port, server_shutdown_rx) => {
            result
        }
        _ = signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down...");
            let _ = server_shutdown_tx.send(());
            Ok::<_, std::io::Error>(())
        }
    };

    if let Err(e) = shutdown_result {
        eprintln!("Shutdown error: {}", e);
    }

    #[cfg(feature = "desktop")]
    {
        println!("Stopping WebView...");
        if let Some(ref mut manager) = webview_manager {
            manager.stop();
            manager.wait_until_stopped();
        }
    }

    #[cfg(not(feature = "desktop"))]
    let _ = webview_manager;

    println!("Desktop mode shutdown complete");
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

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;
    use opencode_core::config::{AcpConfig, DesktopConfig, ServerConfig};
    use tempfile::TempDir;

    #[test]
    fn test_desktop_args_fields_default() {
        let args = DesktopArgs {
            port: None,
            hostname: None,
            no_browser: false,
            acp_enabled: None,
        };
        assert_eq!(args.port, None);
        assert_eq!(args.hostname, None);
        assert!(!args.no_browser);
        assert_eq!(args.acp_enabled, None);
    }

    #[test]
    fn test_desktop_args_with_port() {
        let args = DesktopArgs {
            port: Some(8080),
            hostname: None,
            no_browser: false,
            acp_enabled: None,
        };
        assert_eq!(args.port, Some(8080));
    }

    #[test]
    fn test_desktop_args_with_long_port() {
        let args = DesktopArgs {
            port: Some(9000),
            hostname: None,
            no_browser: false,
            acp_enabled: None,
        };
        assert_eq!(args.port, Some(9000));
    }

    #[test]
    fn test_desktop_args_with_hostname() {
        let args = DesktopArgs {
            port: None,
            hostname: Some("0.0.0.0".to_string()),
            no_browser: false,
            acp_enabled: None,
        };
        assert_eq!(args.hostname, Some("0.0.0.0".to_string()));
    }

    #[test]
    fn test_desktop_args_with_no_browser() {
        let args = DesktopArgs {
            port: None,
            hostname: None,
            no_browser: true,
            acp_enabled: None,
        };
        assert!(args.no_browser);
    }

    #[test]
    fn test_desktop_args_with_acp_enabled() {
        let args = DesktopArgs {
            port: None,
            hostname: None,
            no_browser: false,
            acp_enabled: Some(false),
        };
        assert_eq!(args.acp_enabled, Some(false));
    }

    #[test]
    fn test_desktop_args_with_all_options() {
        let args = DesktopArgs {
            port: Some(3000),
            hostname: Some("localhost".to_string()),
            no_browser: true,
            acp_enabled: Some(true),
        };
        assert_eq!(args.port, Some(3000));
        assert_eq!(args.hostname, Some("localhost".to_string()));
        assert!(args.no_browser);
        assert_eq!(args.acp_enabled, Some(true));
    }

    #[test]
    fn test_desktop_config_loading_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "server": {
                "port": 8080,
                "hostname": "192.168.1.1",
                "desktop": {
                    "enabled": true,
                    "auto_open_browser": false,
                    "port": 9000,
                    "hostname": "desktop-host"
                },
                "acp": {
                    "enabled": false
                }
            }
        }"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load(&config_path).expect("Failed to load config");
        let server_cfg = config.server.expect("Server config missing");

        assert_eq!(server_cfg.port, Some(8080));
        assert_eq!(server_cfg.hostname.as_deref(), Some("192.168.1.1"));

        let desktop_cfg = server_cfg.desktop.expect("Desktop config missing");
        assert_eq!(desktop_cfg.enabled, Some(true));
        assert_eq!(desktop_cfg.auto_open_browser, Some(false));
        assert_eq!(desktop_cfg.port, Some(9000));
        assert_eq!(desktop_cfg.hostname.as_deref(), Some("desktop-host"));
    }

    #[test]
    fn test_desktop_config_loading_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        std::fs::write(&config_path, "{}").unwrap();

        let config = Config::load(&config_path).expect("Failed to load config");
        let server_cfg = config.server.as_ref();

        assert!(server_cfg.is_none());
    }

    #[test]
    fn test_desktop_config_loading_partial_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "server": {
                "port": 3000
            }
        }"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load(&config_path).expect("Failed to load config");
        let server_cfg = config.server.expect("Server config missing");

        assert_eq!(server_cfg.port, Some(3000));
        assert!(server_cfg.desktop.is_none());
    }

    #[test]
    fn test_desktop_config_precedence_cli_over_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        let config_content = r#"{
            "server": {
                "port": 8080,
                "desktop": {
                    "port": 9000
                }
            }
        }"#;

        std::fs::write(&config_path, config_content).unwrap();

        let config = Config::load(&config_path).expect("Failed to load config");
        let default_server_cfg = ServerConfig::default();
        let server_cfg = config.server.as_ref().unwrap_or(&default_server_cfg);
        let desktop_cfg = server_cfg.desktop.as_ref();

        let args_port = Some(7000);
        let resolved_port = args_port
            .or(desktop_cfg.and_then(|d| d.port))
            .or(server_cfg.port)
            .unwrap_or(3000);

        assert_eq!(resolved_port, 7000);
    }

    #[test]
    fn test_desktop_config_precedence_config_over_defaults() {
        let default_server_cfg = ServerConfig::default();
        let desktop_cfg = DesktopConfig {
            enabled: Some(true),
            auto_open_browser: Some(false),
            port: Some(9000),
            hostname: Some("config-host".to_string()),
        };

        let resolved_port = Some(desktop_cfg.port)
            .flatten()
            .or(default_server_cfg.port)
            .unwrap_or(3000);

        assert_eq!(resolved_port, 9000);
    }

    #[test]
    fn test_desktop_config_precedence_default_fallback() {
        let default_server_cfg = ServerConfig::default();

        let resolved_port = Option::<u16>::None
            .or(default_server_cfg.port)
            .unwrap_or(3000);

        assert_eq!(resolved_port, 3000);
    }

    #[test]
    fn test_desktop_config_auto_open_browser_default_true() {
        let desktop_cfg: Option<DesktopConfig> = None;
        let auto_open = desktop_cfg
            .and_then(|d| d.auto_open_browser)
            .unwrap_or(true);
        assert!(auto_open);
    }

    #[test]
    fn test_desktop_config_auto_open_browser_from_config() {
        let desktop_cfg = DesktopConfig {
            enabled: None,
            auto_open_browser: Some(false),
            port: None,
            hostname: None,
        };
        let auto_open = desktop_cfg.auto_open_browser.unwrap_or(true);
        assert!(!auto_open);
    }

    #[test]
    fn test_desktop_config_acp_enabled_default_true() {
        let default_server_cfg = ServerConfig::default();
        let desktop_cfg: Option<DesktopConfig> = None;

        let acp_enabled = desktop_cfg
            .and_then(|d| d.enabled)
            .or(default_server_cfg.acp.as_ref().and_then(|a| a.enabled))
            .unwrap_or(true);

        assert!(acp_enabled);
    }

    #[test]
    fn test_desktop_config_acp_enabled_from_desktop_cfg() {
        let desktop_cfg = DesktopConfig {
            enabled: Some(false),
            auto_open_browser: None,
            port: None,
            hostname: None,
        };

        let acp_enabled = desktop_cfg.enabled.unwrap_or(true);
        assert!(!acp_enabled);
    }

    #[test]
    fn test_desktop_config_acp_enabled_from_server_acp() {
        let server_cfg = ServerConfig {
            acp: Some(AcpConfig {
                enabled: Some(false),
                server_id: None,
                version: None,
                session: None,
            }),
            ..Default::default()
        };
        let desktop_cfg: Option<DesktopConfig> = None;

        let acp_enabled = desktop_cfg
            .and_then(|d| d.enabled)
            .or(server_cfg.acp.as_ref().and_then(|a| a.enabled))
            .unwrap_or(true);

        assert!(!acp_enabled);
    }
}
