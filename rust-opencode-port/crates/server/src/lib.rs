use std::sync::Arc;
use std::sync::RwLock;
use actix_web::dev::Service;
use actix_web::{web, App, HttpServer, middleware as actix_middleware, HttpResponse, Responder};
use futures::future::{Either, ready};
use futures::FutureExt;
use opencode_storage::StorageService;
use opencode_llm::ModelRegistry;
use opencode_core::Config;
use opencode_core::bus::SharedEventBus;
use opencode_core::config::ServerConfig;
use streaming::ReconnectionStore;

#[cfg(test)]
mod server_integration_tests;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub mod routes;
pub mod middleware;
pub mod mdns;
pub mod streaming;

pub struct ServerState {
    pub storage: Arc<StorageService>,
    pub models: Arc<ModelRegistry>,
    pub config: Arc<RwLock<Config>>,
    pub event_bus: SharedEventBus,
    pub reconnection_store: ReconnectionStore,
}

pub async fn run_server(state: Arc<ServerState>, host: &str, port: u16) -> std::io::Result<()> {
    validate_port(port)?;

    let server_cfg = state
        .config
        .read()
        .expect("server config lock poisoned")
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
            .route("/static/{filename:.*}", web::get().to(routes::web_ui::serve_static))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api")
                    .wrap_fn(|req, srv| {
                        if !middleware::is_api_key_authorized(&req) {
                            let response = routes::error::json_error(
                                actix_web::http::StatusCode::UNAUTHORIZED,
                                "unauthorized",
                                "Missing or invalid x-api-key header",
                            );
                            return Either::Left(ready(Ok(req.into_response(response.map_into_right_body()))));
                        }

                        Either::Right(srv.call(req).map(|res| res.map(|res| res.map_into_left_body())))
                    })
                    .configure(routes::config_routes)
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
    use super::validate_port;

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
}
