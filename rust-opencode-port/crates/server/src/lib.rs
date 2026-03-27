use std::sync::Arc;
use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use actix_cors::Cors;
use opencode_storage::StorageService;
use opencode_llm::ModelRegistry;
use opencode_core::Config;

pub mod routes;
pub mod middleware;

pub struct ServerState {
    pub storage: Arc<StorageService>,
    pub models: Arc<ModelRegistry>,
    pub config: Arc<Config>,
}

pub async fn run_server(state: Arc<ServerState>, host: &str, port: u16) -> std::io::Result<()> {
    let state_data = web::Data::from(state);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(state_data.clone())
            .wrap(actix_middleware::Logger::default())
            .wrap(cors)
            .service(
                web::scope("/api")
                    .configure(routes::config_routes)
            )
    })
    .bind((host, port))?
    .run()
    .await
}