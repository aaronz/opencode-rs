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
            storage: std::sync::Arc::new(opencode_storage::StorageService::new(
                opencode_storage::database::StoragePool::new(&db_path).unwrap(),
            )),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(opencode_core::Config::default())),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: crate::streaming::ReconnectionStore::default(),
            connection_monitor: std::sync::Arc::new(
                crate::streaming::conn_state::ConnectionMonitor::new(),
            ),
            share_server: std::sync::Arc::new(std::sync::RwLock::new(
                crate::routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: true,
        }
    }

    fn create_test_state_with_api_key(api_key: Option<String>) -> crate::ServerState {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut config = opencode_core::Config::default();
        config.api_key = api_key;
        crate::ServerState {
            storage: std::sync::Arc::new(opencode_storage::StorageService::new(
                opencode_storage::database::StoragePool::new(&db_path).unwrap(),
            )),
            models: std::sync::Arc::new(opencode_llm::ModelRegistry::new()),
            config: std::sync::Arc::new(std::sync::RwLock::new(config)),
            event_bus: opencode_core::bus::SharedEventBus::default(),
            reconnection_store: crate::streaming::ReconnectionStore::default(),
            connection_monitor: std::sync::Arc::new(
                crate::streaming::conn_state::ConnectionMonitor::new(),
            ),
            share_server: std::sync::Arc::new(std::sync::RwLock::new(
                crate::routes::share::ShareServer::with_default_config(),
            )),
            acp_enabled: true,
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
        )
        .await
        .respond_to(&req);

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
        )
        .await
        .respond_to(&req);

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
        )
        .await
        .respond_to(&req);

        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // Auth enforcement tests

    #[actix_web::test]
    async fn test_auth_no_api_key_allows_request() {
        use actix_web::web;

        let state = create_test_state_with_api_key(None);
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(authorized, "Request should be allowed when no API key is configured");
    }

    #[actix_web::test]
    async fn test_auth_missing_header_returns_unauthorized() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("test-api-key".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(!authorized, "Request should be rejected when API key is configured but header is missing");
    }

    #[actix_web::test]
    async fn test_auth_invalid_credentials_returns_unauthorized() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("test-api-key".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .insert_header((actix_web::http::header::HeaderName::from_static("x-api-key"), "wrong-api-key"))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(!authorized, "Request should be rejected when API key is invalid");
    }

    #[actix_web::test]
    async fn test_auth_valid_credentials_allows_request() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("test-api-key".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .insert_header((actix_web::http::header::HeaderName::from_static("x-api-key"), "test-api-key"))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(authorized, "Request should be allowed when API key is valid");
    }

    #[actix_web::test]
    async fn test_auth_empty_api_key_allows_request() {
        use actix_web::web;

        let state = create_test_state_with_api_key(Some("".to_string()));
        let req = TestRequest::default()
            .app_data(web::Data::new(state))
            .to_srv_request();
        let authorized = crate::middleware::is_api_key_authorized(&req);
        assert!(authorized, "Request should be allowed when API key is empty string");
    }
}
