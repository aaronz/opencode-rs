use actix_web::{web, HttpResponse, Responder};
use opencode_core::share::{ExportFormat, ExportOptions, ShareManager};
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
    pub include_metadata: bool,
    #[serde(default = "default_sanitize")]
    pub sanitize_sensitive: bool,
}

impl ExportQuery {
    pub fn new(include_metadata: bool, sanitize_sensitive: bool) -> Self {
        Self {
            include_metadata,
            sanitize_sensitive,
        }
    }
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

    HttpResponse::Ok().content_type("text/plain").body(content)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/export/sessions")
            .route("/{id}", web::get().to(export_session_json))
            .route("/{id}/transcript", web::get().to(export_session_markdown))
            .route("/{id}/patch", web::get().to(export_session_patch)),
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

    #[test]
    fn test_export_response_all_fields() {
        let response = ExportResponse {
            session_id: "session-abc".to_string(),
            format: "markdown".to_string(),
            content: "# Session Export\n\nContent here".to_string(),
            content_type: "text/markdown".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("session-abc"));
        assert!(json.contains("markdown"));
        assert!(json.contains("text/markdown"));
    }

    #[test]
    fn test_export_response_json_format() {
        let response = ExportResponse {
            session_id: "json-session".to_string(),
            format: "json".to_string(),
            content: r#"{"messages": []}"#.to_string(),
            content_type: "application/json".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["session_id"], "json-session");
        assert_eq!(parsed["format"], "json");
        assert_eq!(parsed["content_type"], "application/json");
    }

    #[test]
    fn test_export_response_markdown_content() {
        let response = ExportResponse {
            session_id: "md-session".to_string(),
            format: "markdown".to_string(),
            content: "# Title\n\nSome content".to_string(),
            content_type: "text/markdown".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("md-session"));
        assert!(json.contains("markdown"));
    }

    #[test]
    fn test_export_query_defaults() {
        let query = ExportQuery::new(true, true);
        assert!(query.include_metadata);
        assert!(query.sanitize_sensitive);
    }

    #[test]
    fn test_export_query_with_metadata_disabled() {
        let query = ExportQuery::new(false, true);
        assert!(!query.include_metadata);
        assert!(query.sanitize_sensitive);
    }

    #[test]
    fn test_export_query_with_sanitize_disabled() {
        let query = ExportQuery::new(true, false);
        assert!(query.include_metadata);
        assert!(!query.sanitize_sensitive);
    }

    #[test]
    fn test_export_query_both_disabled() {
        let query = ExportQuery::new(false, false);
        assert!(!query.include_metadata);
        assert!(!query.sanitize_sensitive);
    }

    #[test]
    fn test_export_query_deserialization_with_defaults() {
        let json = r#"{}"#;
        let query: ExportQuery = serde_json::from_str(json).unwrap();
        assert!(query.include_metadata);
        assert!(query.sanitize_sensitive);
    }

    #[test]
    fn test_export_query_deserialization_explicit() {
        let json = r#"{"include_metadata": false, "sanitize_sensitive": false}"#;
        let query: ExportQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_metadata);
        assert!(!query.sanitize_sensitive);
    }

    #[test]
    fn test_export_query_deserialization_partial() {
        let json = r#"{"include_metadata": false}"#;
        let query: ExportQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_metadata);
        assert!(query.sanitize_sensitive);
    }

    #[test]
    fn test_default_include_metadata() {
        assert!(default_include_metadata());
    }

    #[test]
    fn test_default_sanitize() {
        assert!(default_sanitize());
    }

    #[test]
    fn test_export_response_patch_bundle_format() {
        let response = ExportResponse {
            session_id: "patch-session".to_string(),
            format: "patch_bundle".to_string(),
            content: "diff --git a/file.txt b/file.txt".to_string(),
            content_type: "text/plain".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("patch-session"));
        assert!(json.contains("patch_bundle"));
    }

    #[test]
    fn test_export_response_unicode_content() {
        let response = ExportResponse {
            session_id: "unicode-session".to_string(),
            format: "json".to_string(),
            content: "Hello 世界 🌍".to_string(),
            content_type: "application/json".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["content"].as_str().unwrap().contains("世界"));
    }
}
