use actix_web::web;

pub mod acp;
pub mod config;
pub mod error;
pub mod mcp;
pub mod model;
pub mod permission;
pub mod provider;
pub mod run;
pub mod session;
pub mod share;
pub mod sse;
pub mod web_ui;
pub mod ws;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/config").configure(config::init));
    cfg.service(web::scope("/providers").configure(provider::init));
    cfg.service(web::scope("/models").configure(model::init));
    cfg.service(web::scope("/sessions").configure(session::init));
    cfg.service(web::scope("/share").configure(share::init));
    cfg.service(web::scope("/run").configure(run::init));
    cfg.service(web::scope("/permissions").configure(permission::init));
    cfg.service(web::scope("/ws").configure(ws::init));
    cfg.service(web::scope("/sse").configure(sse::init));
    cfg.service(web::scope("/mcp").configure(mcp::init));
    cfg.service(web::scope("/acp").configure(acp::init));
}
