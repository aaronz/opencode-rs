use opencode_mcp::{JsonRpcRequest, JsonRpcResponse};

#[test]
fn test_mcp_protocol_request_format() {
    let request = JsonRpcRequest::new("tools/list", None);

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "tools/list");
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
fn test_mcp_protocol_response_success() {
    let response =
        JsonRpcResponse::success(Some(serde_json::json!(1)), serde_json::json!({"tools": []}));

    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.error.is_none());
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
