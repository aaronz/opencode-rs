use crate::ServerState;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ModelQuery {
    pub provider: Option<String>,
}

pub async fn get_models(
    state: web::Data<ServerState>,
    query: web::Query<ModelQuery>,
) -> impl Responder {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_query_empty() {
        let json = r#"{}"#;
        let query: ModelQuery = serde_json::from_str(json).unwrap();
        assert!(query.provider.is_none());
    }

    #[test]
    fn test_model_query_with_provider() {
        let json = r#"{"provider": "openai"}"#;
        let query: ModelQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.provider, Some("openai".to_string()));
    }

    #[test]
    fn test_model_query_preserves_provider_name() {
        let json = r#"{"provider": "anthropic"}"#;
        let query: ModelQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.provider.unwrap(), "anthropic");
    }
}
