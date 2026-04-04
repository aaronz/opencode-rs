use actix_web::{HttpResponse, Responder};

/// Reserved Web UI entrypoint contract.
///
/// This route is intentionally minimal in TASK-3.6 and serves as a stable
/// reservation for future SPA/static hosting integration.
///
/// Contract:
/// - Method: GET
/// - Path: /
/// - Response: placeholder HTML page
pub async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            "<!doctype html><html><head><title>OpenCode Web UI</title></head><body><h1>OpenCode Web UI (Reserved)</h1><p>This endpoint is reserved for the upcoming web interface.</p></body></html>",
        )
}

/// Reserved Web API documentation contract.
///
/// Contract:
/// - Method: GET
/// - Path: /api/docs
/// - Response: JSON placeholder describing reserved docs URL and state
pub async fn api_docs() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "reserved",
        "message": "API docs endpoint is reserved for future OpenAPI/Swagger publication.",
        "version": "v0",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn index_returns_html() {
        let response = index().await.respond_to(&actix_web::test::TestRequest::default().to_http_request());
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
    }
}
