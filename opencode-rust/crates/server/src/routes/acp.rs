use actix_web::{web, HttpResponse, Responder};
use opencode_core::acp::{AcpHandshakeAck, AcpHandshakeRequest, AcpProtocol};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpStatusResponse {
    pub status: String,
    pub version: String,
    pub acp_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpHandshakeResponse {
    pub version: String,
    pub server_id: String,
    pub session_id: String,
    pub accepted: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpConnectRequest {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcpConnectResponse {
    pub status: String,
    pub url: String,
}

pub struct AcpState {
    pub protocol: Arc<RwLock<AcpProtocol>>,
    pub enabled: bool,
}

impl AcpState {
    pub fn new(enabled: bool) -> Self {
        Self {
            protocol: Arc::new(RwLock::new(AcpProtocol::new("server", "1.0"))),
            enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::Responder;

    #[actix_web::test]
    async fn test_acp_status_when_enabled() {
        let state = web::Data::new(AcpState::new(true));
        let req = TestRequest::default().to_http_request();
        let resp = acp_status(state).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_status_when_disabled() {
        let state = web::Data::new(AcpState::new(false));
        let req = TestRequest::default().to_http_request();
        let resp = acp_status(state).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_handshake_when_disabled() {
        let state = web::Data::new(AcpState::new(false));
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: "test-client".to_string(),
            capabilities: vec![],
        });
        let resp = acp_handshake(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_handshake_when_enabled() {
        let state = web::Data::new(AcpState::new(true));
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeRequest {
            version: "1.0".to_string(),
            client_id: "test-client".to_string(),
            capabilities: vec!["chat".to_string()],
        });
        let resp = acp_handshake(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_handshake_version_mismatch() {
        let state = web::Data::new(AcpState::new(true));
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeRequest {
            version: "2.0".to_string(),
            client_id: "test-client".to_string(),
            capabilities: vec![],
        });
        let resp = acp_handshake(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_ack_when_disabled() {
        let state = web::Data::new(AcpState::new(false));
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpHandshakeAck {
            session_id: "test-session".to_string(),
            confirmed: true,
        });
        let resp = acp_ack(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_acp_connect_when_disabled() {
        let state = web::Data::new(AcpState::new(false));
        let req = TestRequest::default().to_http_request();
        let body = web::Json(AcpConnectRequest {
            url: "http://localhost:9000".to_string(),
        });
        let resp = acp_connect(state, body).await.respond_to(&req);
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/status").route(web::get().to(acp_status)));
    cfg.service(web::resource("/handshake").route(web::post().to(acp_handshake)));
    cfg.service(web::resource("/connect").route(web::post().to(acp_connect)));
    cfg.service(web::resource("/ack").route(web::post().to(acp_ack)));
}

async fn acp_status(state: web::Data<AcpState>) -> impl Responder {
    let protocol = state.protocol.read().unwrap();
    HttpResponse::Ok().json(AcpStatusResponse {
        status: if state.enabled { "ready" } else { "disabled" }.to_string(),
        version: protocol.version().to_string(),
        acp_enabled: state.enabled,
    })
}

async fn acp_handshake(
    state: web::Data<AcpState>,
    body: web::Json<AcpHandshakeRequest>,
) -> impl Responder {
    if !state.enabled {
        return HttpResponse::Ok().json(AcpHandshakeResponse {
            version: "1.0".to_string(),
            server_id: String::new(),
            session_id: String::new(),
            accepted: false,
            error: Some("ACP is disabled".to_string()),
        });
    }

    let protocol = state.protocol.read().unwrap();
    let response = protocol.process_handshake(body.into_inner());

    HttpResponse::Ok().json(AcpHandshakeResponse {
        version: response.version,
        server_id: response.server_id,
        session_id: response.session_id,
        accepted: response.accepted,
        error: response.error,
    })
}

async fn acp_connect(
    state: web::Data<AcpState>,
    body: web::Json<AcpConnectRequest>,
) -> impl Responder {
    if !state.enabled {
        return HttpResponse::Ok().json(AcpConnectResponse {
            status: "disabled".to_string(),
            url: body.url.clone(),
        });
    }

    HttpResponse::Ok().json(AcpConnectResponse {
        status: "connecting".to_string(),
        url: body.url.clone(),
    })
}

async fn acp_ack(state: web::Data<AcpState>, body: web::Json<AcpHandshakeAck>) -> impl Responder {
    if !state.enabled {
        return HttpResponse::Ok().json(serde_json::json!({
            "confirmed": false,
            "error": "ACP is disabled"
        }));
    }

    let protocol = state.protocol.read().unwrap();
    let confirmed = protocol.confirm_handshake(body.into_inner());

    HttpResponse::Ok().json(serde_json::json!({
        "confirmed": confirmed
    }))
}
