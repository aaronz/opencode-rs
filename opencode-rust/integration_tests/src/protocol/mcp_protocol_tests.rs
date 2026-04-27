use opencode_mcp::protocol::McpPermission;
use opencode_mcp::protocol::{McpServerConfig, McpServerType};
use opencode_mcp::{
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    ServerCapabilities, ServerInfo, ToolContent, ToolDefinition, ToolResult,
};

#[test]
fn test_mcp_protocol_request_format() {
    let request = JsonRpcRequest::new("tools/list", None);

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "tools/list");
}

#[test]
fn test_mcp_protocol_request_with_id() {
    let request = JsonRpcRequest::new("tools/list", Some(serde_json::json!({})))
        .with_id(serde_json::json!(42));

    assert_eq!(request.id, Some(serde_json::json!(42)));
}

#[test]
fn test_mcp_protocol_request_serialization_roundtrip() {
    let request = JsonRpcRequest::new(
        "tools/call",
        Some(serde_json::json!({"name": "echo", "arguments": {"v": 1}})),
    )
    .with_id(serde_json::json!(1));

    let encoded = serde_json::to_string(&request).unwrap();
    let decoded: JsonRpcRequest = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded.jsonrpc, "2.0");
    assert_eq!(decoded.method, "tools/call");
    assert_eq!(decoded.id, Some(serde_json::json!(1)));
}

#[test]
fn test_mcp_protocol_tools_list_request() {
    let request = JsonRpcRequest::new("tools/list", Some(serde_json::json!({})));

    let json = serde_json::to_string(&request).expect("Should serialize");
    assert!(json.contains("\"method\":\"tools/list\""));
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
}

#[test]
fn test_mcp_protocol_initialize_request() {
    let params = serde_json::json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": "test",
            "version": "1.0.0"
        }
    });

    let request = JsonRpcRequest::new("initialize", Some(params));

    assert_eq!(request.method, "initialize");
    assert!(request.params.is_some());
}

#[test]
fn test_mcp_protocol_tool_call_request() {
    let params = serde_json::json!({
        "name": "read",
        "arguments": {"path": "/test.txt"}
    });

    let request = JsonRpcRequest::new("tools/call", Some(params));

    assert_eq!(request.method, "tools/call");
    let call_params = request.params.unwrap();
    assert_eq!(call_params.get("name").unwrap(), "read");
}

#[test]
fn test_mcp_protocol_resources_list_request() {
    let request = JsonRpcRequest::new("resources/list", Some(serde_json::json!({})));

    assert_eq!(request.method, "resources/list");
}

#[test]
fn test_mcp_protocol_notification_format() {
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "initialized".to_string(),
        params: None,
    };

    assert_eq!(notification.jsonrpc, "2.0");
    assert_eq!(notification.method, "initialized");
}

#[test]
fn test_mcp_protocol_notification_with_params() {
    let notification = JsonRpcNotification {
        jsonrpc: "2.0".to_string(),
        method: "textDocument/publishDiagnostics".to_string(),
        params: Some(serde_json::json!({
            "uri": "file:///test.rs",
            "diagnostics": []
        })),
    };

    assert_eq!(notification.method, "textDocument/publishDiagnostics");
    assert!(notification.params.is_some());
}

#[test]
fn test_mcp_protocol_response_success() {
    let response =
        JsonRpcResponse::success(Some(serde_json::json!(1)), serde_json::json!({"tools": []}));

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_mcp_protocol_response_success_with_tools() {
    let tools = vec![
        ToolDefinition {
            name: "read".to_string(),
            description: "Read a file".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        },
        ToolDefinition {
            name: "write".to_string(),
            description: "Write a file".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        },
    ];

    let response = JsonRpcResponse::success(
        Some(serde_json::json!(1)),
        serde_json::json!({"tools": tools}),
    );

    assert!(response.result.is_some());
    let result = response.result.unwrap();
    assert!(result.get("tools").is_some());
}

#[test]
fn test_mcp_protocol_response_error() {
    let response = JsonRpcResponse::error(
        Some(serde_json::json!(1)),
        -32602,
        "Invalid params".to_string(),
    );

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_none());
    assert!(response.error.is_some());
    assert_eq!(response.error.as_ref().unwrap().code, -32602);
}

#[test]
fn test_mcp_protocol_response_error_with_data() {
    let response = JsonRpcResponse::error(
        Some(serde_json::json!(1)),
        -32603,
        "Internal error".to_string(),
    );

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.error.is_some());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32603);
    assert_eq!(error.message, "Internal error");
}

#[test]
fn test_mcp_protocol_server_info_serialization() {
    let info = ServerInfo {
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
    };

    let encoded = serde_json::to_string(&info).unwrap();
    let decoded: ServerInfo = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded.name, "test-server");
    assert_eq!(decoded.version, "1.0.0");
}

#[test]
fn test_mcp_protocol_server_capabilities() {
    let capabilities = ServerCapabilities {
        tools: Some(opencode_mcp::ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(opencode_mcp::ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(false),
        }),
    };

    let encoded = serde_json::to_string(&capabilities).unwrap();
    let decoded: ServerCapabilities = serde_json::from_str(&encoded).unwrap();

    assert!(decoded.tools.is_some());
    assert!(decoded.resources.is_some());
}

#[test]
fn test_mcp_protocol_tool_definition() {
    let tool = ToolDefinition {
        name: "search".to_string(),
        description: "Search for files".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            },
            "required": ["query"]
        }),
    };

    let encoded = serde_json::to_string(&tool).unwrap();
    let decoded: ToolDefinition = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded.name, "search");
    assert!(decoded.input_schema.get("properties").is_some());
}

#[test]
fn test_mcp_protocol_tool_result_text_content() {
    let result = ToolResult {
        content: vec![ToolContent::Text {
            text: "Hello, World!".to_string(),
        }],
        is_error: Some(false),
    };

    let encoded = serde_json::to_string(&result).unwrap();
    let decoded: ToolResult = serde_json::from_str(&encoded).unwrap();

    match decoded.content[0] {
        ToolContent::Text { ref text } => assert_eq!(text, "Hello, World!"),
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_mcp_protocol_tool_result_error() {
    let result = ToolResult {
        content: vec![ToolContent::Text {
            text: "Error occurred".to_string(),
        }],
        is_error: Some(true),
    };

    assert!(result.is_error.is_some() && result.is_error.unwrap());
}

#[test]
fn test_mcp_protocol_server_config_local() {
    let config = McpServerConfig::local(
        "local-server".to_string(),
        "npx".to_string(),
        vec![
            "-y".to_string(),
            "@modelcontextprotocol/server-everything".to_string(),
        ],
    );

    assert_eq!(config.name, "local-server");
    assert_eq!(config.server_type, McpServerType::Local);
    assert!(config.command.is_some());
    assert!(config.url.is_none());
    assert!(config.enabled);
}

#[test]
fn test_mcp_protocol_server_config_remote() {
    let config = McpServerConfig::remote(
        "remote-server".to_string(),
        "https://mcp.example.com/mcp".to_string(),
    );

    assert_eq!(config.name, "remote-server");
    assert_eq!(config.server_type, McpServerType::Remote);
    assert!(config.command.is_none());
    assert!(config.url.is_some());
}

#[test]
fn test_mcp_protocol_server_config_with_permission() {
    let config =
        McpServerConfig::local("permission-server".to_string(), "echo".to_string(), vec![])
            .with_permission(McpPermission::Deny);

    assert_eq!(config.permission, McpPermission::Deny);
}

#[test]
fn test_mcp_protocol_full_roundtrip_initialize_flow() {
    let init_request = JsonRpcRequest::new(
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "opencode-test",
                "version": "1.0.0"
            }
        })),
    )
    .with_id(serde_json::json!(1));

    let request_json = serde_json::to_string(&init_request).unwrap();
    let parsed_request: JsonRpcRequest = serde_json::from_str(&request_json).unwrap();

    assert_eq!(parsed_request.method, "initialize");
    assert_eq!(parsed_request.id, Some(serde_json::json!(1)));

    let init_response = JsonRpcResponse::success(
        Some(serde_json::json!(1)),
        serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {"listChanged": true}
            },
            "serverInfo": {
                "name": "test-server",
                "version": "1.0.0"
            }
        }),
    );

    let response_json = serde_json::to_string(&init_response).unwrap();
    let parsed_response: JsonRpcResponse = serde_json::from_str(&response_json).unwrap();

    assert!(parsed_response.result.is_some());
    assert!(parsed_response.error.is_none());
}

#[test]
fn test_mcp_protocol_tool_call_execution_flow() {
    let tool_call_request = JsonRpcRequest::new(
        "tools/call",
        Some(serde_json::json!({
            "name": "read",
            "arguments": {"path": "/test.txt"}
        })),
    )
    .with_id(serde_json::json!(2));

    let request_json = serde_json::to_string(&tool_call_request).unwrap();
    let parsed: JsonRpcRequest = serde_json::from_str(&request_json).unwrap();

    assert_eq!(parsed.method, "tools/call");
    let params = parsed.params.unwrap();
    assert_eq!(params.get("name").unwrap(), "read");

    let tool_result = ToolResult {
        content: vec![ToolContent::Text {
            text: "File content here".to_string(),
        }],
        is_error: Some(false),
    };

    let response =
        JsonRpcResponse::success(Some(serde_json::json!(2)), serde_json::json!(tool_result));
    let response_json = serde_json::to_string(&response).unwrap();
    let parsed_response: JsonRpcResponse = serde_json::from_str(&response_json).unwrap();

    assert!(parsed_response.result.is_some());
}
