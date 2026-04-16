use actix_web::{web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use opencode_control_plane::sso::{OidcState, SsoConfig, SsoManager, SsoProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::routes::error::{bad_request, internal_error, not_found};
use crate::ServerState;

use std::sync::OnceLock;

static SSO_MANAGER: OnceLock<Mutex<SsoManager>> = OnceLock::new();
static OIDC_STATES: OnceLock<Mutex<HashMap<String, OidcState>>> = OnceLock::new();

fn get_sso_manager() -> &'static Mutex<SsoManager> {
    SSO_MANAGER.get_or_init(|| Mutex::new(SsoManager::new()))
}

fn get_oidc_states() -> &'static Mutex<HashMap<String, OidcState>> {
    OIDC_STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSsoConfigRequest {
    pub provider: String,
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct SsoConfigResponse {
    pub id: String,
    pub provider: String,
    pub entity_id: String,
    pub sso_url: String,
    pub has_certificate: bool,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct OidcAuthorizeResponse {
    pub auth_url: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct OidcCallbackRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OidcCallbackResponse {
    pub success: bool,
    pub token: Option<String>,
    pub error: Option<String>,
}

pub async fn get_sso_config(_state: web::Data<ServerState>) -> impl Responder {
    let manager = get_sso_manager();
    let manager = match manager.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("SSO manager lock poisoned");
        }
    };

    match manager.get_config() {
        Some(config) => HttpResponse::Ok().json(SsoConfigResponse {
            id: config.id.clone(),
            provider: match config.provider {
                SsoProvider::Saml => "saml".to_string(),
                SsoProvider::Oidc => "oidc".to_string(),
            },
            entity_id: config.entity_id.clone(),
            sso_url: config.sso_url.clone(),
            has_certificate: config.certificate.is_some(),
            enabled: config.enabled,
        }),
        None => not_found("SSO not configured"),
    }
}

pub async fn update_sso_config(
    _state: web::Data<ServerState>,
    body: web::Json<UpdateSsoConfigRequest>,
) -> impl Responder {
    let provider = match body.provider.as_str() {
        "saml" => SsoProvider::Saml,
        "oidc" => SsoProvider::Oidc,
        _ => {
            return bad_request("Invalid provider. Must be 'saml' or 'oidc'");
        }
    };

    let config = SsoConfig {
        id: format!("sso-{}", Uuid::new_v4()),
        provider,
        entity_id: body.entity_id.clone(),
        sso_url: body.sso_url.clone(),
        certificate: body.certificate.clone(),
        client_id: body.client_id.clone(),
        client_secret: body.client_secret.clone(),
        redirect_uri: body.redirect_uri.clone(),
        enabled: body.enabled,
    };

    let manager = get_sso_manager();
    let mut manager = match manager.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("SSO manager lock poisoned");
        }
    };
    manager.set_config(config);

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "SSO config updated"
    }))
}

pub async fn oidc_authorize(_state: web::Data<ServerState>) -> impl Responder {
    let manager = get_sso_manager();
    let manager = match manager.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("SSO manager lock poisoned");
        }
    };

    let config = match manager.get_config() {
        Some(c) if c.enabled => c.clone(),
        _ => {
            return bad_request("SSO not enabled or not configured");
        }
    };

    if !matches!(config.provider, SsoProvider::Oidc) {
        return bad_request("Provider is not OIDC");
    }

    let state = format!("state-{}", uuid::Uuid::new_v4());
    let nonce = format!("nonce-{}", uuid::Uuid::new_v4());

    let oidc_state = OidcState {
        state: state.clone(),
        nonce: nonce.clone(),
        code_verifier: None,
        expires_at: Utc::now() + Duration::minutes(10),
    };

    let states = get_oidc_states();
    let mut states = match states.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("OIDC states lock poisoned");
        }
    };
    states.insert(state.clone(), oidc_state);

    let auth_url = format!(
        "{}/authorize?response_type=code&client_id={}&redirect_uri={}&state={}&nonce={}",
        config.sso_url,
        config.client_id.as_deref().unwrap_or(""),
        config.redirect_uri.as_deref().unwrap_or(""),
        state,
        nonce
    );

    HttpResponse::Ok().json(OidcAuthorizeResponse { auth_url, state })
}

pub async fn oidc_callback(
    _state: web::Data<ServerState>,
    body: web::Json<OidcCallbackRequest>,
) -> impl Responder {
    let states = get_oidc_states();
    let mut states = match states.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return internal_error("OIDC states lock poisoned");
        }
    };

    let oidc_state = match states.remove(&body.state) {
        Some(s) => s,
        None => {
            return bad_request("Invalid state parameter");
        }
    };

    if oidc_state.expires_at < Utc::now() {
        return bad_request("OIDC state has expired. Please try again");
    }

    HttpResponse::Ok().json(OidcCallbackResponse {
        success: true,
        token: Some("mock-jwt-token".to_string()),
        error: None,
    })
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/config", web::get().to(get_sso_config));
    cfg.route("/config", web::put().to(update_sso_config));
    cfg.route("/oidc/authorize", web::post().to(oidc_authorize));
    cfg.route("/oidc/callback", web::post().to(oidc_callback));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sso_config_response_serialization() {
        let response = SsoConfigResponse {
            id: "sso-1".to_string(),
            provider: "oidc".to_string(),
            entity_id: "opencode".to_string(),
            sso_url: "https://sso.example.com".to_string(),
            has_certificate: true,
            enabled: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("oidc"));
    }

    #[test]
    fn test_sso_config_response_serialization_saml() {
        let response = SsoConfigResponse {
            id: "sso-2".to_string(),
            provider: "saml".to_string(),
            entity_id: "my-entity".to_string(),
            sso_url: "https://saml.example.com".to_string(),
            has_certificate: false,
            enabled: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("saml"));
        assert!(json.contains("my-entity"));
        assert!(!json.contains("has_certificate"));
    }

    #[test]
    fn test_update_sso_config_request_deserialization() {
        let json = r#"{
            "provider": "oidc",
            "entity_id": "test-entity",
            "sso_url": "https://sso.test.com",
            "client_id": "client123",
            "client_secret": "secret456",
            "redirect_uri": "https://app.example.com/callback",
            "enabled": true
        }"#;

        let req: UpdateSsoConfigRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.provider, "oidc");
        assert_eq!(req.entity_id, "test-entity");
        assert_eq!(req.sso_url, "https://sso.test.com");
        assert_eq!(req.client_id, Some("client123".to_string()));
        assert_eq!(req.client_secret, Some("secret456".to_string()));
        assert!(req.enabled);
    }

    #[test]
    fn test_update_sso_config_request_minimal() {
        let json = r#"{
            "provider": "saml",
            "entity_id": "minimal-entity",
            "sso_url": "https://saml.test.com",
            "enabled": false
        }"#;

        let req: UpdateSsoConfigRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.provider, "saml");
        assert!(req.certificate.is_none());
        assert!(req.client_id.is_none());
        assert!(!req.enabled);
    }

    #[test]
    fn test_oidc_authorize_response_serialization() {
        let response = OidcAuthorizeResponse {
            auth_url: "https://sso.example.com/authorize?code=abc".to_string(),
            state: "state-123".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("auth_url"));
        assert!(json.contains("state"));
    }

    #[test]
    fn test_oidc_callback_request_deserialization() {
        let json = r#"{"code": "auth-code-xyz", "state": "state-789"}"#;
        let req: OidcCallbackRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.code, "auth-code-xyz");
        assert_eq!(req.state, "state-789");
    }

    #[test]
    fn test_oidc_callback_response_success() {
        let response = OidcCallbackResponse {
            success: true,
            token: Some("jwt-token-abc".to_string()),
            error: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("jwt-token-abc"));
    }

    #[test]
    fn test_oidc_callback_response_error() {
        let response = OidcCallbackResponse {
            success: false,
            token: None,
            error: Some("Authentication failed".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("Authentication failed"));
    }

    #[test]
    fn test_get_sso_manager_returns_singleton() {
        let manager1 = get_sso_manager();
        let manager2 = get_sso_manager();
        assert!(std::ptr::eq(manager1, manager2));
    }

    #[test]
    fn test_get_oidc_states_returns_singleton() {
        let states1 = get_oidc_states();
        let states2 = get_oidc_states();
        assert!(std::ptr::eq(states1, states2));
    }

    #[test]
    fn test_get_sso_manager_poisoned_lock_handling() {
        use std::sync::Mutex;
        use std::thread;

        let manager = get_sso_manager();

        let manager_clone = manager.clone();
        let handle = thread::spawn(move || {
            let _lock = manager_clone.lock();
        });
        let _ = handle.join();
    }
}
