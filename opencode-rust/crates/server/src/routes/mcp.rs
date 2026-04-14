use crate::routes::error::json_error;
use crate::ServerState;
use actix_web::{http::StatusCode, web, Error, HttpResponse, Responder};
use opencode_mcp::registry::McpServerConfig;
use opencode_mcp::{
    ConnectionState, JsonRpcRequest, McpManager, McpServer, McpTransport, StdioProcess,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct McpRequestBody {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct McpConnectRequest {
    pub name: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct McpServerInfo {
    pub name: String,
    pub connection_state: String,
}

#[derive(Debug, Serialize)]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
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

pub async fn get_mcp_servers() -> impl Responder {
    let states = McpManager::global().connection_states().await;
    let servers: Vec<McpServerInfo> = states
        .into_iter()
        .map(|(name, state)| McpServerInfo {
            name,
            connection_state: match state {
                ConnectionState::Connected => "connected".to_string(),
                ConnectionState::Connecting => "connecting".to_string(),
                ConnectionState::Disconnected => "disconnected".to_string(),
                ConnectionState::Error(_) => "error".to_string(),
            },
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "items": servers,
        "count": servers.len(),
    }))
}

pub async fn get_mcp_tools() -> impl Responder {
    let tools = McpManager::global().get_tools().await;
    let tool_infos: Vec<McpToolInfo> = tools
        .into_iter()
        .map(|t| McpToolInfo {
            name: t.name,
            description: t.description,
        })
        .collect();

    HttpResponse::Ok().json(serde_json::json!({
        "items": tool_infos,
        "count": tool_infos.len(),
    }))
}

pub async fn connect_mcp_server(body: web::Json<McpConnectRequest>) -> impl Responder {
    let transport = if let Some(url) = &body.url {
        if url.starts_with("http://") || url.starts_with("https://") {
            McpTransport::Sse(url.clone())
        } else {
            return json_error(
                StatusCode::BAD_REQUEST,
                "invalid_transport",
                "URL must start with http:// or https://",
            );
        }
    } else if let Some(cmd) = &body.command {
        McpTransport::Stdio(StdioProcess::new(
            cmd,
            body.args.clone().unwrap_or_default(),
        ))
    } else {
        return json_error(
            StatusCode::BAD_REQUEST,
            "invalid_transport",
            "Either url or command must be provided",
        );
    };

    let config = McpServerConfig::new(transport);
    McpManager::global().add_server(&body.name, config).await;

    match McpManager::global().connect_all().await {
        Ok(connected) => {
            if connected.contains(&body.name) {
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "connected",
                    "server": body.name,
                }))
            } else {
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "registered",
                    "server": body.name,
                    "message": "Server registered but auto_connect is disabled"
                }))
            }
        }
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "connection_failed",
            e.to_string(),
        ),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(mcp_handler))
        .route("/servers", web::get().to(get_mcp_servers))
        .route("/tools", web::get().to(get_mcp_tools))
        .route("/connect", web::post().to(connect_mcp_server));
}
