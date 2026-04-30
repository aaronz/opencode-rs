#![allow(
    unused_imports,
    unused_mut,
    unused_variables,
    clippy::await_holding_lock,
    clippy::io_other_error,
    clippy::len_without_is_empty,
    clippy::manual_contains,
    clippy::redundant_closure
)]

use actix_web::dev::Service;
use actix_web::{middleware as actix_middleware, web, App, HttpResponse, HttpServer, Responder};
use futures::future::{ready, Either};
use futures::FutureExt;
use opencode_agent::{AgentRuntime, AgentType};
use opencode_control_plane::SharedAcpStream;
use opencode_core::bus::SharedEventBus;
use opencode_core::config::ServerConfig;
use opencode_core::{Config, Session};
use opencode_llm::ModelRegistry;
use opencode_permission::{ApprovalQueue, AuditLog, PermissionScope};
use opencode_runtime::{
    RuntimeFacade as OpenCodeRuntime, RuntimeFacadeServices, RuntimeFacadeTaskStore,
    RuntimeFacadeToolRouter,
};
use opencode_storage::{
    InMemoryProjectRepository, InMemorySessionRepository, StoragePool, StorageService,
};
use opencode_tools::ToolRegistry;
use routes::acp_ws::SharedAcpClientRegistry;
use routes::share::ShareServer;
use routes::ws::SessionHub;
use std::sync::Arc;
use std::sync::RwLock;
use streaming::{conn_state::ConnectionMonitor, ReconnectionStore};
use tokio::sync::oneshot;

#[cfg(test)]
mod server_integration_tests;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub mod mdns;
pub mod middleware;
pub mod routes;
pub mod streaming;

pub struct ServerState {
    pub storage: Arc<StorageService>,
    pub runtime: Arc<OpenCodeRuntime>,
    pub models: Arc<ModelRegistry>,
    pub config: Arc<RwLock<Config>>,
    pub event_bus: SharedEventBus,
    pub reconnection_store: ReconnectionStore,
    pub temp_db_dir: Option<std::path::PathBuf>,
    pub connection_monitor: Arc<ConnectionMonitor>,
    pub share_server: Arc<RwLock<ShareServer>>,
    pub acp_enabled: bool,
    pub acp_stream: SharedAcpStream,
    pub acp_client_registry: SharedAcpClientRegistry,
    pub tool_registry: Arc<ToolRegistry>,
    pub session_hub: Arc<SessionHub>,
    pub server_start_time: std::time::SystemTime,
    pub permission_manager: Arc<RwLock<opencode_core::permission::PermissionManager>>,
    pub approval_queue: Arc<RwLock<ApprovalQueue>>,
    pub audit_log: Option<Arc<AuditLog>>,
}

impl Clone for ServerState {
    fn clone(&self) -> Self {
        ServerState {
            storage: self.storage.clone(),
            runtime: self.runtime.clone(),
            models: self.models.clone(),
            config: self.config.clone(),
            event_bus: self.event_bus.clone(),
            reconnection_store: self.reconnection_store.clone(),
            temp_db_dir: self.temp_db_dir.clone(),
            connection_monitor: self.connection_monitor.clone(),
            share_server: self.share_server.clone(),
            acp_enabled: self.acp_enabled,
            acp_stream: self.acp_stream.clone(),
            acp_client_registry: self.acp_client_registry.clone(),
            tool_registry: self.tool_registry.clone(),
            session_hub: self.session_hub.clone(),
            server_start_time: self.server_start_time,
            permission_manager: self.permission_manager.clone(),
            approval_queue: self.approval_queue.clone(),
            audit_log: self.audit_log.clone(),
        }
    }
}

pub fn build_placeholder_runtime() -> Arc<OpenCodeRuntime> {
    let event_bus = Arc::new(opencode_core::bus::EventBus::new());
    let permission_manager = Arc::new(tokio::sync::RwLock::new(
        opencode_core::permission::PermissionManager::default(),
    ));
    let session_repo = Arc::new(InMemorySessionRepository::default());
    let project_repo = Arc::new(InMemoryProjectRepository::default());
    let db_path = std::env::temp_dir().join(format!(
        "opencode-runtime-placeholder-{}.db",
        uuid::Uuid::new_v4()
    ));
    let pool = StoragePool::new(&db_path).expect("placeholder storage pool");
    let storage = Arc::new(StorageService::new(session_repo, project_repo, pool));
    let agent_runtime = Arc::new(tokio::sync::RwLock::new(
        AgentRuntime::new(Session::default(), AgentType::Build).with_event_bus(event_bus.clone()),
    ));

    Arc::new(OpenCodeRuntime::new(RuntimeFacadeServices::new(
        event_bus,
        permission_manager,
        storage,
        agent_runtime,
        Arc::new(RuntimeFacadeTaskStore::new()),
        Arc::new(RuntimeFacadeToolRouter::default()),
        AgentType::Build,
        None,
        None,
    )))
}

pub async fn run_server(state: Arc<ServerState>, host: &str, port: u16) -> std::io::Result<()> {
    validate_port(port)?;

    let server_cfg = state
        .config
        .read()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "server config lock poisoned"))?
        .server
        .clone()
        .unwrap_or_default();
    let cors_origins = server_cfg.cors.clone().unwrap_or_default();

    let mdns_service = if server_cfg.mdns == Some(true) {
        Some(mdns::MdnsService::start(&ServerConfig {
            port: Some(port),
            hostname: Some(host.to_string()),
            mdns: server_cfg.mdns,
            mdns_domain: server_cfg.mdns_domain.clone(),
            cors: server_cfg.cors.clone(),
            desktop: None,
            acp: None,
            password: None,
        })?)
    } else {
        None
    };

    let state_data = web::Data::from(state);

    let result = HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            .wrap(actix_middleware::Logger::default())
            .wrap(middleware::cors_middleware(&cors_origins))
            .route("/", web::get().to(routes::web_ui::index))
            .route("/api/docs", web::get().to(routes::web_ui::api_docs))
            .route(
                "/static/{filename:.*}",
                web::get().to(routes::web_ui::serve_static),
            )
            .route("/health", web::get().to(health_check))
            .route("/api/status", web::get().to(routes::status::get_status))
            .service(
                web::scope("/api")
                    .wrap_fn(|req, srv| {
                        if !middleware::is_api_key_authorized(&req) {
                            let response = routes::error::json_error(
                                actix_web::http::StatusCode::UNAUTHORIZED,
                                "unauthorized",
                                "Missing or invalid x-api-key header",
                            );
                            return Either::Left(ready(Ok(
                                req.into_response(response.map_into_right_body())
                            )));
                        }

                        Either::Right(
                            srv.call(req)
                                .map(|res| res.map(|res| res.map_into_left_body())),
                        )
                    })
                    .configure(routes::config_routes),
            )
    })
    .bind((host, port))?
    .run()
    .await;

    if let Some(mdns) = mdns_service {
        mdns.stop();
    }

    result
}

pub async fn run_server_with_shutdown(
    state: Arc<ServerState>,
    host: &str,
    port: u16,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    validate_port(port)?;

    let server_cfg = state
        .config
        .read()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "server config lock poisoned"))?
        .server
        .clone()
        .unwrap_or_default();
    let cors_origins = server_cfg.cors.clone().unwrap_or_default();

    let mdns_service = if server_cfg.mdns == Some(true) {
        Some(mdns::MdnsService::start(&ServerConfig {
            port: Some(port),
            hostname: Some(host.to_string()),
            mdns: server_cfg.mdns,
            mdns_domain: server_cfg.mdns_domain.clone(),
            cors: server_cfg.cors.clone(),
            desktop: None,
            acp: None,
            password: None,
        })?)
    } else {
        None
    };

    let state_data = web::Data::from(state);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state_data.clone())
            .wrap(actix_middleware::Logger::default())
            .wrap(middleware::cors_middleware(&cors_origins))
            .route("/", web::get().to(routes::web_ui::index))
            .route("/api/docs", web::get().to(routes::web_ui::api_docs))
            .route(
                "/static/{filename:.*}",
                web::get().to(routes::web_ui::serve_static),
            )
            .route("/health", web::get().to(health_check))
            .route("/api/status", web::get().to(routes::status::get_status))
            .service(
                web::scope("/api")
                    .wrap_fn(|req, srv| {
                        if !middleware::is_api_key_authorized(&req) {
                            let response = routes::error::json_error(
                                actix_web::http::StatusCode::UNAUTHORIZED,
                                "unauthorized",
                                "Missing or invalid x-api-key header",
                            );
                            return Either::Left(ready(Ok(
                                req.into_response(response.map_into_right_body())
                            )));
                        }

                        Either::Right(
                            srv.call(req)
                                .map(|res| res.map(|res| res.map_into_left_body())),
                        )
                    })
                    .configure(routes::config_routes),
            )
    })
    .bind((host, port))?
    .run();

    tokio::select! {
        result = server => {
            if let Some(mdns) = mdns_service {
                mdns.stop();
            }
            result
        }
        _ = shutdown_rx => {
            if let Some(mdns) = mdns_service {
                mdns.stop();
            }
            Ok(())
        }
    }
}

fn validate_port(port: u16) -> std::io::Result<()> {
    if port < 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid server port {}: must be in range 1024-65535", port),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_port_rejects_privileged_ports() {
        assert!(validate_port(80).is_err());
        assert!(validate_port(1023).is_err());
    }

    #[test]
    fn validate_port_accepts_non_privileged_ports() {
        assert!(validate_port(1024).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn server_state_clone_preserves_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();
        let session_repo = Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
        let project_repo = Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));
        let state = ServerState {
            storage: Arc::new(opencode_storage::StorageService::new(
                session_repo,
                project_repo,
                pool,
            )),
            models: Arc::new(opencode_llm::ModelRegistry::new()),
            config: Arc::new(RwLock::new(Config::default())),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: streaming::ReconnectionStore::default(),
            temp_db_dir: None,
            connection_monitor: Arc::new(streaming::conn_state::ConnectionMonitor::new()),
            share_server: Arc::new(RwLock::new(
                routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: true,
            acp_stream: opencode_control_plane::AcpEventStream::new().into(),
            acp_client_registry: Arc::new(tokio::sync::RwLock::new(
                routes::acp_ws::AcpClientRegistry::new(),
            )),
            tool_registry: Arc::new(opencode_tools::ToolRegistry::new()),
            session_hub: Arc::new(routes::ws::SessionHub::new(256)),
            server_start_time: std::time::SystemTime::now(),
            permission_manager: Arc::new(RwLock::new(opencode_core::PermissionManager::default())),
            approval_queue: Arc::new(RwLock::new(opencode_permission::ApprovalQueue::new(
                opencode_permission::PermissionScope::Full,
            ))),
            audit_log: None,
            runtime: crate::build_placeholder_runtime(),
        };

        let cloned = state.clone();
        assert_eq!(state.acp_enabled, cloned.acp_enabled);
        assert_eq!(
            state
                .server_start_time
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap(),
            cloned
                .server_start_time
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
        );
    }

    #[test]
    fn server_state_all_fields_initialized() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = opencode_storage::database::StoragePool::new(&db_path).unwrap();
        let session_repo = Arc::new(opencode_storage::SqliteSessionRepository::new(pool.clone()));
        let project_repo = Arc::new(opencode_storage::SqliteProjectRepository::new(pool.clone()));
        let _state = ServerState {
            storage: Arc::new(opencode_storage::StorageService::new(
                session_repo,
                project_repo,
                pool,
            )),
            models: Arc::new(opencode_llm::ModelRegistry::new()),
            config: Arc::new(RwLock::new(Config::default())),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: streaming::ReconnectionStore::default(),
            temp_db_dir: None,
            connection_monitor: Arc::new(streaming::conn_state::ConnectionMonitor::new()),
            share_server: Arc::new(RwLock::new(
                routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: false,
            acp_stream: opencode_control_plane::AcpEventStream::new().into(),
            acp_client_registry: Arc::new(tokio::sync::RwLock::new(
                routes::acp_ws::AcpClientRegistry::new(),
            )),
            tool_registry: Arc::new(opencode_tools::ToolRegistry::new()),
            session_hub: Arc::new(routes::ws::SessionHub::new(256)),
            server_start_time: std::time::SystemTime::now(),
            permission_manager: Arc::new(RwLock::new(opencode_core::PermissionManager::default())),
            approval_queue: Arc::new(RwLock::new(opencode_permission::ApprovalQueue::new(
                opencode_permission::PermissionScope::Full,
            ))),
            audit_log: None,
            runtime: crate::build_placeholder_runtime(),
        };
    }
}
