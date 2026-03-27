use actix_web::{web, HttpResponse, Responder};
use crate::ServerState;
use std::collections::HashSet;

pub async fn get_providers(state: web::Data<ServerState>) -> impl Responder {
    let models = state.models.list();
    let providers: HashSet<String> = models.iter()
        .map(|m| m.provider.to_string())
        .collect();
    
    let mut providers_vec: Vec<String> = providers.into_iter().collect();
    providers_vec.sort();
    
    HttpResponse::Ok().json(providers_vec)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_providers));
}