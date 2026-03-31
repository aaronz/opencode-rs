use actix_web::web;

pub mod config;
pub mod provider;
pub mod model;
pub mod session;
pub mod run;
pub mod permission;
pub mod ws;
pub mod sse;
pub mod mcp;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/config")
            .configure(config::init)
    );
    cfg.service(
        web::scope("/providers")
            .configure(provider::init)
    );
    cfg.service(
        web::scope("/models")
            .configure(model::init)
    );
    cfg.service(
        web::scope("/sessions")
            .configure(session::init)
    );
    cfg.service(
        web::scope("/run")
            .configure(run::init)
    );
    cfg.service(
        web::scope("/permissions")
            .configure(permission::init)
    );
    cfg.service(
        web::scope("/ws")
            .configure(ws::init)
    );
    cfg.service(
        web::scope("/sse")
            .configure(sse::init)
    );
    cfg.service(
        web::scope("/mcp")
            .configure(mcp::init)
    );
}