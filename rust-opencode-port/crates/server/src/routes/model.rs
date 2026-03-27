use actix_web::{web, HttpResponse, Responder};
use crate::ServerState;

pub async fn get_models(state: web::Data<ServerState>) -> impl Responder {
    let models = state.models.list();
    HttpResponse::Ok().json(models)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_models));
}