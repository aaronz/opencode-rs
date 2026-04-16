use crate::routes::error::json_error;
use crate::ServerState;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
pub struct ProviderStatus {
    pub name: String,
    pub status: String,
    pub model: String,
}

#[derive(Debug, Serialize)]
pub struct PluginStatus {
    pub name: String,
    pub version: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub version: String,
    pub status: String,
    pub uptime_seconds: u64,
    pub active_sessions: usize,
    pub total_sessions: usize,
    pub providers: Vec<ProviderStatus>,
    pub plugins: Vec<PluginStatus>,
}

pub async fn get_status(state: web::Data<ServerState>) -> impl Responder {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let status = "running".to_string();
    let uptime_seconds: u64 = 0;

    let active_sessions = state.session_hub.session_count().await;

    let total_sessions = match state.storage.list_sessions(usize::MAX, 0).await {
        Ok(sessions) => sessions.len(),
        Err(_) => 0,
    };

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
