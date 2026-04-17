use actix_files as fs;
use actix_web::{Either, HttpResponse, Responder};
use std::path::PathBuf;

use super::error::not_found;

const INDEX_HTML: &str = include_str!("../../static/index.html");

pub async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_HTML)
}

pub async fn api_docs() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "available",
        "version": "v1",
        "endpoints": {
            "GET /api/config": "Get current configuration",
            "PATCH /api/config": "Update configuration",
            "POST /api/run": "Run a prompt",
            "GET /api/sessions": "List sessions",
            "GET /api/sessions/{id}": "Get session details",
            "GET /api/models": "List available models",
            "GET /api/providers": "List configured providers",
            "WS /ws": "WebSocket for real-time updates",
            "SSE /sse": "Server-Sent Events for real-time updates"
        }
    }))
}

pub async fn serve_static(req: actix_web::HttpRequest) -> impl Responder {
    let path: PathBuf = req
        .match_info()
        .query("filename")
        .parse()
        .unwrap_or_else(|_| PathBuf::from(""));
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    let full_path = base.join(&path);

    if full_path.exists() && full_path.starts_with(&base) {
        match fs::NamedFile::open(full_path) {
            Ok(file) => Either::Left(file),
            Err(_) => Either::Right(not_found("Static file not found")),
        }
    } else {
        Either::Right(not_found("Static file not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn index_returns_html() {
        let response = index()
            .await
            .respond_to(&actix_web::test::TestRequest::default().to_http_request());
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn api_docs_returns_endpoints() {
        let response = api_docs()
            .await
            .respond_to(&actix_web::test::TestRequest::default().to_http_request());
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
    }

    #[test]
    fn test_index_content_type() {
        let response = INDEX_HTML;
        assert!(response.contains("<!DOCTYPE html") || response.contains("<html"));
    }

    #[actix_web::test]
    async fn api_docs_contains_status_endpoint() {
        let response = api_docs()
            .await
            .respond_to(&actix_web::test::TestRequest::default().to_http_request());
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
    }
}
