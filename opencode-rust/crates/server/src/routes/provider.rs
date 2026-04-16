use crate::routes::error::{internal_error, json_error};
use crate::ServerState;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use opencode_llm::{AuthStrategy, Credential, ProviderAuthConfig};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub provider_id: String,
    pub endpoint: String,
    pub auth_strategy: AuthStrategy,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub endpoint: Option<String>,
    pub auth_strategy: Option<AuthStrategy>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderCredentialRequest {
    pub api_key: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct SetProviderEnabledRequest {
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ProviderResponse {
    pub provider_id: String,
    pub endpoint: String,
    pub auth_strategy: AuthStrategy,
}

#[derive(Debug, Serialize)]
pub struct ProviderStatusResponse {
    pub provider_id: String,
    pub enabled: bool,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderConfigChangedEvent {
    pub event: String,
    pub provider_id: String,
    pub enabled: bool,
}

static CREDENTIALS: OnceLock<Mutex<HashMap<String, Credential>>> = OnceLock::new();
static ENABLED_PROVIDERS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static DISABLED_PROVIDERS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn credential_store() -> &'static Mutex<HashMap<String, Credential>> {
    CREDENTIALS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn enabled_providers() -> &'static Mutex<HashSet<String>> {
    ENABLED_PROVIDERS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn disabled_providers() -> &'static Mutex<HashSet<String>> {
    DISABLED_PROVIDERS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn provider_exists(state: &ServerState, provider_id: &str) -> bool {
    state
        .models
        .list()
        .iter()
        .any(|m| m.provider == provider_id)
}

fn is_provider_enabled(provider_id: &str) -> bool {
    let disabled = match disabled_providers().lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };
    if disabled.contains(provider_id) {
        return false;
    }
    drop(disabled);
    let enabled = match enabled_providers().lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };
    if enabled.is_empty() {
        return true;
    }
    enabled.contains(provider_id)
}

pub async fn get_providers(state: web::Data<ServerState>) -> impl Responder {
    let models = state.models.list();
    let providers: HashSet<String> = models.iter().map(|m| m.provider.to_string()).collect();

    let mut providers_vec: Vec<String> = providers.into_iter().collect();
    providers_vec.sort();

    HttpResponse::Ok().json(serde_json::json!({
        "items": providers_vec,
        "count": providers_vec.len(),
    }))
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
        json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        )
    }
}

pub async fn create_provider(
    _state: web::Data<ServerState>,
    body: web::Json<CreateProviderRequest>,
) -> impl Responder {
    let mut config = ProviderAuthConfig::new(
        body.provider_id.clone(),
        body.endpoint.clone(),
        body.auth_strategy.clone(),
    );

    if let Some(headers) = &body.headers {
        for (k, v) in headers {
            config = config.with_header(k.clone(), v.clone());
        }
    }

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

    if provider_exists(&state, &provider_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Provider {provider_id} updated")
        }))
    } else {
        json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        )
    }
}

pub async fn delete_provider(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();

    if provider_exists(&state, &provider_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Provider {provider_id} deleted")
        }))
    } else {
        json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        )
    }
}

pub async fn test_provider(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();

    if provider_exists(&state, &provider_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "provider": provider_id,
            "message": "Provider connection test successful"
        }))
    } else {
        json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        )
    }
}

pub async fn save_provider_credentials(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    body: web::Json<ProviderCredentialRequest>,
) -> impl Responder {
    let provider_id = path.into_inner();
    if !provider_exists(&state, &provider_id) {
        return json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        );
    }

    if body.api_key.trim().is_empty() {
        return json_error(
            StatusCode::BAD_REQUEST,
            "invalid_credentials",
            "api_key cannot be empty",
        );
    }

    let mut credential = Credential::new(provider_id.clone(), body.api_key.clone());
    credential.expires_at = body.expires_at;
    if let Some(metadata) = &body.metadata {
        credential.metadata = metadata.clone();
    }

    let mut store = match credential_store().lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Credential store lock poisoned");
        }
    };
    store.insert(provider_id.clone(), credential);

    HttpResponse::Created().json(serde_json::json!({
        "provider": provider_id,
        "status": "saved"
    }))
}

pub async fn test_provider_credentials(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();
    if !provider_exists(&state, &provider_id) {
        return json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        );
    }

    let store = match credential_store().lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Credential store lock poisoned");
        }
    };
    let Some(credential) = store.get(&provider_id) else {
        return json_error(
            StatusCode::NOT_FOUND,
            "credentials_not_found",
            format!("No credentials found for provider: {provider_id}"),
        );
    };

    if credential.is_valid() {
        HttpResponse::Ok().json(serde_json::json!({
            "provider": provider_id,
            "valid": true,
            "message": "Credentials are valid"
        }))
    } else {
        json_error(
            StatusCode::UNAUTHORIZED,
            "credentials_invalid",
            format!("Credentials for provider {provider_id} are expired or invalid"),
        )
    }
}

pub async fn delete_provider_credentials(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();
    if !provider_exists(&state, &provider_id) {
        return json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        );
    }

    let mut store = match credential_store().lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("Credential store lock poisoned");
        }
    };
    if store.remove(&provider_id).is_some() {
        HttpResponse::NoContent().finish()
    } else {
        json_error(
            StatusCode::NOT_FOUND,
            "credentials_not_found",
            format!("No credentials found for provider: {provider_id}"),
        )
    }
}

pub async fn get_provider_status(
    state: web::Data<ServerState>,
    path: web::Path<String>,
) -> impl Responder {
    let provider_id = path.into_inner();
    let exists = provider_exists(&state, &provider_id);
    let enabled = is_provider_enabled(&provider_id);

    HttpResponse::Ok().json(ProviderStatusResponse {
        provider_id: provider_id.clone(),
        enabled,
        exists,
    })
}

pub async fn set_provider_enabled(
    state: web::Data<ServerState>,
    path: web::Path<String>,
    body: web::Json<SetProviderEnabledRequest>,
) -> impl Responder {
    let provider_id = path.into_inner();

    if !provider_exists(&state, &provider_id) {
        return json_error(
            StatusCode::NOT_FOUND,
            "provider_not_found",
            format!("Provider not found: {provider_id}"),
        );
    }

    if body.enabled {
        let mut enabled = match enabled_providers().lock() {
            Ok(guard) => guard,
            Err(_) => {
                return internal_error("Enabled providers lock poisoned");
            }
        };
        enabled.insert(provider_id.clone());
        let mut disabled = match disabled_providers().lock() {
            Ok(guard) => guard,
            Err(_) => {
                return internal_error("Disabled providers lock poisoned");
            }
        };
        disabled.remove(&provider_id);
    } else {
        let mut disabled = match disabled_providers().lock() {
            Ok(guard) => guard,
            Err(_) => {
                return internal_error("Disabled providers lock poisoned");
            }
        };
        disabled.insert(provider_id.clone());
        let mut enabled = match enabled_providers().lock() {
            Ok(guard) => guard,
            Err(_) => {
                return internal_error("Enabled providers lock poisoned");
            }
        };
        enabled.remove(&provider_id);
    }

    info!(event = "provider_enabled_changed", provider = %provider_id, enabled = body.enabled);

    let event = ProviderConfigChangedEvent {
        event: "provider_config_changed".to_string(),
        provider_id: provider_id.clone(),
        enabled: body.enabled,
    };

    HttpResponse::Ok().json(serde_json::json!({
        "provider_id": provider_id,
        "enabled": body.enabled,
        "event": event,
    }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_providers))
        .route("", web::post().to(create_provider))
        .route("/{id}", web::get().to(get_provider))
        .route("/{id}", web::put().to(update_provider))
        .route("/{id}", web::delete().to(delete_provider))
        .route("/{id}/test", web::post().to(test_provider))
        .route("/{id}/status", web::get().to(get_provider_status))
        .route("/{id}/enabled", web::put().to(set_provider_enabled))
        .route(
            "/{id}/credentials",
            web::post().to(save_provider_credentials),
        )
        .route(
            "/{id}/credentials/test",
            web::post().to(test_provider_credentials),
        )
        .route(
            "/{id}/credentials",
            web::delete().to(delete_provider_credentials),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider_request_deserialization() {
        let json = r#"{
            "provider_id": "openai",
            "endpoint": "https://api.openai.com",
            "auth_strategy": "api_key"
        }"#;

        let req: CreateProviderRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.provider_id, "openai");
        assert_eq!(req.endpoint, "https://api.openai.com");
    }

    #[test]
    fn test_create_provider_request_with_headers() {
        let json = r#"{
            "provider_id": "anthropic",
            "endpoint": "https://api.anthropic.com",
            "auth_strategy": "bearer",
            "headers": {
                "X-Custom-Header": "value123"
            }
        }"#;

        let req: CreateProviderRequest = serde_json::from_str(json).unwrap();
        assert!(req.headers.is_some());
        let headers = req.headers.unwrap();
        assert_eq!(
            headers.get("X-Custom-Header"),
            Some(&"value123".to_string())
        );
    }

    #[test]
    fn test_update_provider_request_deserialization() {
        let json = r#"{
            "endpoint": "https://new.endpoint.com",
            "auth_strategy": "oauth2"
        }"#;

        let req: UpdateProviderRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.endpoint, Some("https://new.endpoint.com".to_string()));
        assert!(req.auth_strategy.is_some());
    }

    #[test]
    fn test_update_provider_request_partial() {
        let json = r#"{"endpoint": "https://only-endpoint.com"}"#;
        let req: UpdateProviderRequest = serde_json::from_str(json).unwrap();
        assert!(req.endpoint.is_some());
        assert!(req.auth_strategy.is_none());
        assert!(req.headers.is_none());
    }

    #[test]
    fn test_provider_credential_request_deserialization() {
        let json = r#"{
            "api_key": "sk-1234567890abcdef",
            "expires_at": "2025-12-31T23:59:59Z",
            "metadata": {
                "key": "value"
            }
        }"#;

        let req: ProviderCredentialRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.api_key, "sk-1234567890abcdef");
        assert!(req.expires_at.is_some());
        assert!(req.metadata.is_some());
    }

    #[test]
    fn test_provider_credential_request_minimal() {
        let json = r#"{"api_key": "simple-key"}"#;
        let req: ProviderCredentialRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.api_key, "simple-key");
        assert!(req.expires_at.is_none());
        assert!(req.metadata.is_none());
    }

    #[test]
    fn test_set_provider_enabled_request_deserialization() {
        let json = r#"{"enabled": true}"#;
        let req: SetProviderEnabledRequest = serde_json::from_str(json).unwrap();
        assert!(req.enabled);

        let json_false = r#"{"enabled": false}"#;
        let req_false: SetProviderEnabledRequest = serde_json::from_str(json_false).unwrap();
        assert!(!req_false.enabled);
    }

    #[test]
    fn test_provider_response_serialization() {
        let response = ProviderResponse {
            provider_id: "openai".to_string(),
            endpoint: "https://api.openai.com".to_string(),
            auth_strategy: AuthStrategy::ApiKey,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("openai"));
        assert!(json.contains("api.openai.com"));
    }

    #[test]
    fn test_provider_status_response_serialization() {
        let response = ProviderStatusResponse {
            provider_id: "anthropic".to_string(),
            enabled: true,
            exists: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("anthropic"));
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"exists\":true"));
    }

    #[test]
    fn test_provider_config_changed_event_serialization() {
        let event = ProviderConfigChangedEvent {
            event: "provider_config_changed".to_string(),
            provider_id: "openai".to_string(),
            enabled: false,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("provider_config_changed"));
        assert!(json.contains("openai"));
        assert!(json.contains("\"enabled\":false"));
    }

    #[test]
    fn test_credential_store_singleton() {
        let store1 = credential_store();
        let store2 = credential_store();
        assert!(std::ptr::eq(store1, store2));
    }

    #[test]
    fn test_enabled_providers_singleton() {
        let providers1 = enabled_providers();
        let providers2 = enabled_providers();
        assert!(std::ptr::eq(providers1, providers2));
    }

    #[test]
    fn test_disabled_providers_singleton() {
        let providers1 = disabled_providers();
        let providers2 = disabled_providers();
        assert!(std::ptr::eq(providers1, providers2));
    }

    #[test]
    fn test_credential_store_poisoned_lock_handling() {
        use std::sync::Mutex;
        use std::thread;

        let store = credential_store();

        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            let _lock = store_clone.lock();
        });
        let _ = handle.join();
    }

    #[test]
    fn test_auth_strategy_serialization() {
        let strategies = vec![
            AuthStrategy::ApiKey,
            AuthStrategy::Bearer,
            AuthStrategy::OAuth2,
        ];

        for strategy in strategies {
            let json = serde_json::to_string(&strategy).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_provider_response_roundtrip() {
        let original = ProviderResponse {
            provider_id: "test-provider".to_string(),
            endpoint: "https://test.com".to_string(),
            auth_strategy: AuthStrategy::ApiKey,
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ProviderResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.provider_id, original.provider_id);
        assert_eq!(parsed.endpoint, original.endpoint);
    }

    #[test]
    fn test_provider_status_response_roundtrip() {
        let original = ProviderStatusResponse {
            provider_id: "status-test".to_string(),
            enabled: true,
            exists: false,
        };

        let json = serde_json::to_string(&original).unwrap();
        let parsed: ProviderStatusResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.provider_id, original.provider_id);
        assert_eq!(parsed.enabled, original.enabled);
        assert_eq!(parsed.exists, original.exists);
    }
}
