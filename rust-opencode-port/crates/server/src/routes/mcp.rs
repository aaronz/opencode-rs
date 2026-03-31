use actix_web::{web, HttpResponse, Error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use opencode_mcp::{McpServer, JsonRpcRequest, JsonRpcResponse};
use crate::ServerState;

#[derive(Debug, Deserialize)]
pub struct McpRequestBody {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

pub async fn mcp_handler(
    state: web::Data<ServerState>,
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
