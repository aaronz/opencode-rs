#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::Responder;

    #[actix_web::test]
    async fn test_health_check() {
        let req = TestRequest::default().to_http_request();
        let resp = crate::health_check().await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    fn create_test_state() -> crate::ServerState {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        crate::ServerState {
            storage: std::sync::Arc::new(
                opencode_storage::StorageService::new(
                    opencode_storage::database::StoragePool::new(&db_path).unwrap()
                )
            ),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(opencode_core::Config::default())),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: crate::streaming::ReconnectionStore::default(),
        }
    }

    #[actix_web::test]
    async fn test_permission_reply_allows_allow_decision() {
        use actix_web::web;

        let req = TestRequest::default().to_http_request();
        let resp = crate::routes::session::permission_reply(
            web::Data::new(create_test_state()),
            web::Path::from(("test-session".to_string(), "test-req".to_string())),
            web::Json(crate::routes::session::PermissionReplyRequest {
                decision: "allow".to_string(),
            }),
        ).await.respond_to(&req);

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_permission_reply_allows_deny_decision() {
        use actix_web::web;

        let req = TestRequest::default().to_http_request();
        let resp = crate::routes::session::permission_reply(
            web::Data::new(create_test_state()),
            web::Path::from(("test-session".to_string(), "test-req".to_string())),
            web::Json(crate::routes::session::PermissionReplyRequest {
                decision: "deny".to_string(),
            }),
        ).await.respond_to(&req);

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_permission_reply_rejects_invalid_decision() {
        use actix_web::web;

        let req = TestRequest::default().to_http_request();
        let resp = crate::routes::session::permission_reply(
            web::Data::new(create_test_state()),
            web::Path::from(("test-session".to_string(), "test-req".to_string())),
            web::Json(crate::routes::session::PermissionReplyRequest {
                decision: "invalid".to_string(),
            }),
        ).await.respond_to(&req);

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
