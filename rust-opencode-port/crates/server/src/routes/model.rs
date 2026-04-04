use actix_web::{web, HttpResponse, Responder};
use crate::ServerState;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ModelQuery {
    pub provider: Option<String>,
}

pub async fn get_models(state: web::Data<ServerState>, query: web::Query<ModelQuery>) -> impl Responder {
    let mut models = state.models.list();
    if let Some(provider) = &query.provider {
        models.retain(|model| model.provider == *provider);
    }
    HttpResponse::Ok().json(serde_json::json!({
        "items": models,
        "count": models.len(),
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_models));
}
