use crate::ServerState;
use actix_web::{web, Error, HttpResponse};
use opencode_mcp::{JsonRpcRequest, McpServer};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct McpRequestBody {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

pub async fn mcp_handler(
    _state: web::Data<ServerState>,
    body: web::Json<McpRequestBody>,
) -> Result<HttpResponse, Error> {
    let request = JsonRpcRequest {
        jsonrpc: body.jsonrpc.clone(),
        id: body.id.clone(),
        method: body.method.clone(),
        params: body.params.clone(),
    };

    let server = McpServer::new("opencode-rs", "0.1.0");
    let response = server.handle_request(request).await;

    Ok(HttpResponse::Ok().json(response))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(mcp_handler));
}
