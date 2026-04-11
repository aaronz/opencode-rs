use actix_web::{web, HttpResponse, Responder};
use opencode_core::share::{ShareManager, ExportFormat, ExportOptions};
use serde::{Deserialize, Serialize};

use crate::routes::error::not_found;
use crate::ServerState;

/// Response for export endpoints
#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub session_id: String,
    pub format: String,
    pub content: String,
    pub content_type: String,
}

/// Query params for export
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    #[serde(default = "default_include_metadata")]
    include_metadata: bool,
    #[serde(default = "default_sanitize")]
    sanitize_sensitive: bool,
}

fn default_include_metadata() -> bool {
    true
}

fn default_sanitize() -> bool {
    true
}

/// GET /export/sessions/{id} - JSON export
pub async fn export_session_json(
    state: web::Data<ServerState>,
    id: web::Path<String>,
    query: web::Query<ExportQuery>,
) -> impl Responder {
    let session_id = id.into_inner();
    
    let session = match state.storage.load_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return not_found("Session not found");
        }
        Err(e) => {
            return not_found(format!("Session not found: {}", e));
        }
    };
    
    let options = ExportOptions {
        include_metadata: query.include_metadata,
        sanitize_sensitive: query.sanitize_sensitive,
        format: ExportFormat::Json,
    };
    
    let share_manager = ShareManager::new();
    let content = share_manager.export_session(&session, &options);
    
    HttpResponse::Ok()
        .content_type("application/json")
        .json(ExportResponse {
            session_id,
            format: "json".to_string(),
            content,
            content_type: "application/json".to_string(),
        })
}

/// GET /export/sessions/{id}/transcript - Markdown export
pub async fn export_session_markdown(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    let session_id = id.into_inner();
    
    let session = match state.storage.load_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return not_found("Session not found");
        }
        Err(e) => {
            return not_found(format!("Session not found: {}", e));
        }
    };
    
    let options = ExportOptions {
        include_metadata: true,
        sanitize_sensitive: true,
        format: ExportFormat::Markdown,
    };
    
    let share_manager = ShareManager::new();
    let content = share_manager.export_session(&session, &options);
    
    HttpResponse::Ok()
        .content_type("text/markdown")
        .body(content)
}

/// GET /export/sessions/{id}/patch - Patch bundle export
pub async fn export_session_patch(
    state: web::Data<ServerState>,
    id: web::Path<String>,
) -> impl Responder {
    let session_id = id.into_inner();
    
    let session = match state.storage.load_session(&session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return not_found("Session not found");
        }
        Err(e) => {
            return not_found(format!("Session not found: {}", e));
        }
    };
    
    let options = ExportOptions {
        include_metadata: true,
        sanitize_sensitive: true,
        format: ExportFormat::PatchBundle,
    };
    
    let share_manager = ShareManager::new();
    let content = share_manager.export_session(&session, &options);
    
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(content)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/export/sessions")
            .route("/{id}", web::get().to(export_session_json))
            .route("/{id}/transcript", web::get().to(export_session_markdown))
            .route("/{id}/patch", web::get().to(export_session_patch))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_export_response_serialization() {
        let response = ExportResponse {
            session_id: "test-123".to_string(),
            format: "json".to_string(),
            content: "{}".to_string(),
            content_type: "application/json".to_string(),
        };
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-123"));
    }
}
