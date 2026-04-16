use crate::routes::error::json_error;
use crate::ServerState;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use opencode_permission::{ApprovalQueue, PermissionScope};
use serde::Serialize;
use std::collections::HashSet;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderStatus {
    pub name: String,
    pub status: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginStatus {
    pub name: String,
    pub version: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub version: String,
    pub rustc_version: String,
    pub build_timestamp: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub providers: Vec<ProviderStatus>,
    pub plugins: Vec<PluginStatus>,
}

pub async fn get_status(state: web::Data<ServerState>) -> impl Responder {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let rustc_version = built_info::RUSTC_VERSION.to_string();
    let build_timestamp = built_info::BUILT_TIME_UTC.to_string();
    let status = "running".to_string();

    let uptime_seconds = std::time::SystemTime::now()
        .duration_since(state.server_start_time)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let active_sessions = state.session_hub.session_count().await;

    let total_sessions = state.storage.count_sessions().await.unwrap_or(0);

    let models = state.models.list();
    let mut providers_set: HashSet<String> = HashSet::new();
    let providers: Vec<ProviderStatus> = models
        .iter()
        .filter(|m| providers_set.insert(m.provider.clone()))
        .map(|m| ProviderStatus {
            name: m.provider.clone(),
            status: "ready".to_string(),
            model: m.name.clone(),
        })
        .collect();

    let tools = state.tool_registry.list_filtered(None).await;
    let plugins: Vec<PluginStatus> = tools
        .iter()
        .take(10)
        .map(|(name, _, _)| PluginStatus {
            name: name.clone(),
            version: "1.0.0".to_string(),
            status: "loaded".to_string(),
        })
        .collect();

    HttpResponse::Ok().json(StatusResponse {
        version,
        rustc_version,
        build_timestamp,
        status,
        uptime_seconds,
        active_sessions,
        total_sessions,
        providers,
        plugins,
    })
}

pub async fn get_status_simple() -> impl Responder {
    json_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "status_unavailable",
        "Server state not available",
    )
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_status));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty(), "CARGO_PKG_VERSION should not be empty");

        let rustc_version = built_info::RUSTC_VERSION;
        assert!(
            rustc_version.contains("rustc"),
            "RUSTC_VERSION should contain 'rustc', got: {}",
            rustc_version
        );

        let build_timestamp = built_info::BUILT_TIME_UTC;
        assert!(
            !build_timestamp.is_empty(),
            "Build timestamp should not be empty"
        );
    }

    #[test]
    fn status_response_serializes_to_json_correctly() {
        let response = StatusResponse {
            version: "1.0.0".to_string(),
            rustc_version: "rustc 1.75.0".to_string(),
            build_timestamp: "2024-01-15T12:00:00Z".to_string(),
            status: "running".to_string(),
            uptime_seconds: 3600,
            active_sessions: 5,
            total_sessions: 142,
            providers: vec![
                ProviderStatus {
                    name: "openai".to_string(),
                    status: "ready".to_string(),
                    model: "gpt-4".to_string(),
                },
                ProviderStatus {
                    name: "anthropic".to_string(),
                    status: "ready".to_string(),
                    model: "claude-3-opus".to_string(),
                },
            ],
            plugins: vec![PluginStatus {
                name: "example-plugin".to_string(),
                version: "1.0.0".to_string(),
                status: "loaded".to_string(),
            }],
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize StatusResponse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");

        assert_eq!(parsed["version"], "1.0.0");
        assert_eq!(parsed["rustc_version"], "rustc 1.75.0");
        assert_eq!(parsed["build_timestamp"], "2024-01-15T12:00:00Z");
        assert_eq!(parsed["status"], "running");
        assert_eq!(parsed["uptime_seconds"], 3600);
        assert_eq!(parsed["active_sessions"], 5);
        assert_eq!(parsed["total_sessions"], 142);

        assert_eq!(parsed["providers"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["providers"][0]["name"], "openai");
        assert_eq!(parsed["providers"][0]["status"], "ready");
        assert_eq!(parsed["providers"][0]["model"], "gpt-4");
        assert_eq!(parsed["providers"][1]["name"], "anthropic");
        assert_eq!(parsed["providers"][1]["status"], "ready");
        assert_eq!(parsed["providers"][1]["model"], "claude-3-opus");

        assert_eq!(parsed["plugins"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["plugins"][0]["name"], "example-plugin");
        assert_eq!(parsed["plugins"][0]["version"], "1.0.0");
        assert_eq!(parsed["plugins"][0]["status"], "loaded");
    }

    #[test]
    fn status_response_with_empty_providers_and_plugins() {
        let response = StatusResponse {
            version: "1.0.0".to_string(),
            rustc_version: "rustc 1.75.0".to_string(),
            build_timestamp: "2024-01-15T12:00:00Z".to_string(),
            status: "running".to_string(),
            uptime_seconds: 0,
            active_sessions: 0,
            total_sessions: 0,
            providers: vec![],
            plugins: vec![],
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize StatusResponse");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");

        assert_eq!(parsed["version"], "1.0.0");
        assert_eq!(parsed["providers"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["plugins"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn provider_status_serialization() {
        let provider = ProviderStatus {
            name: "test".to_string(),
            status: "ready".to_string(),
            model: "model-x".to_string(),
        };

        let json = serde_json::to_string(&provider).expect("Should serialize");
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"status\":\"ready\""));
        assert!(json.contains("\"model\":\"model-x\""));
    }

    #[test]
    fn plugin_status_serialization() {
        let plugin = PluginStatus {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            status: "loaded".to_string(),
        };

        let json = serde_json::to_string(&plugin).expect("Should serialize");
        assert!(json.contains("\"name\":\"test-plugin\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"status\":\"loaded\""));
    }

    #[test]
    fn uptime_calculation_logic() {
        let start_time = std::time::SystemTime::now() - std::time::Duration::from_secs(100);
        let now = std::time::SystemTime::now();

        let uptime = now
            .duration_since(start_time)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        assert!(
            uptime >= 100 && uptime <= 101,
            "Uptime should be approximately 100 seconds, got: {}",
            uptime
        );
    }

    #[test]
    fn uptime_calculated_correctly_from_server_start_time() {
        use crate::routes::acp_ws::AcpClientRegistry;
        use crate::routes::share::ShareServer;
        use crate::routes::ws::SessionHub;
        use crate::streaming::conn_state::ConnectionMonitor;
        use crate::streaming::ReconnectionStore;
        use opencode_control_plane::AcpEventStream;
        use opencode_core::bus::SharedEventBus;
        use opencode_core::Config;
        use opencode_llm::ModelRegistry;
        use opencode_storage::database::StoragePool;
        use opencode_storage::sqlite_repository::{
            SqliteProjectRepository, SqliteSessionRepository,
        };
        use opencode_storage::StorageService;
        use opencode_tools::ToolRegistry;
        use std::sync::Arc;
        use std::sync::RwLock;
        use tempfile::tempdir;

        let start_time = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);

        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let pool = StoragePool::new(&db_path).unwrap();
        let session_repo = Arc::new(SqliteSessionRepository::new(pool.clone()));
        let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));

        let server_state = ServerState {
            storage: Arc::new(StorageService::new(session_repo, project_repo, pool)),
            models: Arc::new(ModelRegistry::new()),
            config: Arc::new(RwLock::new(Config::default())),
            event_bus: SharedEventBus::default(),
            reconnection_store: ReconnectionStore::default(),
            temp_db_dir: None,
            connection_monitor: Arc::new(ConnectionMonitor::new()),
            share_server: Arc::new(RwLock::new(ShareServer::with_default_config())),
            acp_enabled: true,
            acp_stream: AcpEventStream::new().into(),
            acp_client_registry: Arc::new(tokio::sync::RwLock::new(AcpClientRegistry::new())),
            tool_registry: Arc::new(ToolRegistry::new()),
            session_hub: Arc::new(SessionHub::new(256)),
            server_start_time: start_time,
            permission_manager: Arc::new(RwLock::new(opencode_core::PermissionManager::default())),
            approval_queue: Arc::new(RwLock::new(ApprovalQueue::new(PermissionScope::Full))),
            audit_log: None,
        };

        let calculated_uptime = std::time::SystemTime::now()
            .duration_since(server_state.server_start_time)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        assert!(
            calculated_uptime >= 3599 && calculated_uptime <= 3601,
            "Uptime should be approximately 3600 seconds, got: {}",
            calculated_uptime
        );
    }
}
