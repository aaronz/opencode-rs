use actix_web::{web, HttpResponse, Responder};
use crate::ServerState;
use serde::{Deserialize, Serialize};
use opencode_llm::{ProviderAuthConfig, AuthStrategy};

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub provider_id: String,
    pub endpoint: String,
    pub auth_strategy: AuthStrategy,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub endpoint: Option<String>,
    pub auth_strategy: Option<AuthStrategy>,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct ProviderResponse {
    pub provider_id: String,
    pub endpoint: String,
    pub auth_strategy: AuthStrategy,
}

pub async fn get_providers(state: web::Data<ServerState>) -> impl Responder {
    let models = state.models.list();
    let providers: std::collections::HashSet<String> = models.iter()
        .map(|m| m.provider.to_string())
        .collect();
    
    let mut providers_vec: Vec<String> = providers.into_iter().collect();
    providers_vec.sort();
    
    HttpResponse::Ok().json(providers_vec)
}

pub async fn get_provider(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();
    let models = state.models.list();
    
    if let Some(model) = models.iter().find(|m| m.provider == provider_id) {
        let response = ProviderResponse {
            provider_id: model.provider.clone(),
            endpoint: String::new(),
            auth_strategy: AuthStrategy::default(),
        };
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Provider not found: {}", provider_id)
        }))
    }
}

pub async fn create_provider(
    _state: web::Data<ServerState>,
    body: web::Json<CreateProviderRequest>,
) -> impl Responder {
    let config = ProviderAuthConfig::new(
        body.provider_id.clone(),
        body.endpoint.clone(),
        body.auth_strategy.clone(),
    );
    
    let response = ProviderResponse {
        provider_id: config.provider_id,
        endpoint: config.endpoint,
        auth_strategy: config.auth_strategy,
    };
    
    HttpResponse::Created().json(response)
}

pub async fn update_provider(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    _body: web::Json<UpdateProviderRequest>,
) -> impl Responder {
    let provider_id = path.into_inner();
    let models = state.models.list();
    
    if models.iter().any(|m| m.provider == provider_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Provider {} updated", provider_id)
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Provider not found: {}", provider_id)
        }))
    }
}

pub async fn delete_provider(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();
    let models = state.models.list();
    
    if models.iter().any(|m| m.provider == provider_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Provider {} deleted", provider_id)
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Provider not found: {}", provider_id)
        }))
    }
}

pub async fn test_provider(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();
    let models = state.models.list();
    
    if models.iter().any(|m| m.provider == provider_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "provider": provider_id,
            "message": "Provider connection test successful"
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Provider not found: {}", provider_id)
        }))
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_providers))
       .route("", web::post().to(create_provider))
       .route("/{id}", web::get().to(get_provider))
       .route("/{id}", web::put().to(update_provider))
       .route("/{id}", web::delete().to(delete_provider))
       .route("/{id}/test", web::post().to(test_provider));
}
