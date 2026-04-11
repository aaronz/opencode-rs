use actix_web::{web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use opencode_control_plane::sso::{OidcState, SsoConfig, SsoManager, SsoProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

use crate::routes::error::{bad_request, not_found};
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
    let manager = manager.lock().unwrap();

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
    let mut manager = manager.lock().unwrap();
    manager.set_config(config);

    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "SSO config updated"
    }))
}

pub async fn oidc_authorize(_state: web::Data<ServerState>) -> impl Responder {
    let manager = get_sso_manager();
    let manager = manager.lock().unwrap();

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
    let mut states = states.lock().unwrap();
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
    let mut states = states.lock().unwrap();

    let oidc_state = match states.remove(&body.state) {
        Some(s) => s,
        None => {
            return HttpResponse::BadRequest().json(OidcCallbackResponse {
                success: false,
                token: None,
                error: Some("Invalid state".to_string()),
            });
        }
    };

    if oidc_state.expires_at < Utc::now() {
        return HttpResponse::BadRequest().json(OidcCallbackResponse {
            success: false,
            token: None,
            error: Some("State expired".to_string()),
        });
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
}
