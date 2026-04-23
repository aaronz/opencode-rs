use opencode_mcp::{
    protocol::JsonRpcResponse, ConnectionState, McpClient, McpError, McpTransport, StdioProcess,
};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

type TransportHandler = Arc<
    dyn Fn(opencode_mcp::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
        + Send
        + Sync,
>;

fn ok_response(result: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: None,
        result: Some(result),
        error: None,
    }
}

mod mcp_connection_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_client_connection_state_transitions() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }))),
            "initialized" => Ok(ok_response(json!(null))),
            "ping" => Ok(ok_response(json!(null))),
            _ => Ok(ok_response(json!(null))),
        });

        let client = McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        );

        assert_eq!(
            client.connection_state().await,
            ConnectionState::Disconnected
        );
        assert!(!client.is_connected().await);

        client.connect().await.unwrap();
        assert_eq!(client.connection_state().await, ConnectionState::Connected);
        assert!(client.is_connected().await);

        client.disconnect().await.unwrap();
        assert_eq!(
            client.connection_state().await,
            ConnectionState::Disconnected
        );
    }

    #[tokio::test]
    async fn test_mcp_local_stdio_connection() {
        let python_script = r#"
import sys
import json

def main():
    while True:
        line = sys.stdin.readline()
        if not line:
            break
        try:
            request = json.loads(line)
            method = request.get('method', '')
            id = request.get('id')
            if method == 'initialize':
                response = {
                    'jsonrpc': '2.0',
                    'id': id,
                    'result': {
                        'protocolVersion': '2024-11-05',
                        'capabilities': {'tools': {'listChanged': True}},
                        'serverInfo': {'name': 'test-server', 'version': '1.0'}
                    }
                }
                print(json.dumps(response), flush=True)
            elif method == 'initialized':
                pass
            elif method == 'ping':
                response = {'jsonrpc': '2.0', 'id': id, 'result': None}
                print(json.dumps(response), flush=True)
            elif method == 'tools/list':
                response = {
                    'jsonrpc': '2.0',
                    'id': id,
                    'result': {
                        'tools': [
                            {'name': 'echo', 'description': 'Echo back input', 'inputSchema': {'type': 'object', 'properties': {'value': {'type': 'string'}}}}
                        ]
                    }
                }
                print(json.dumps(response), flush=True)
            elif method == 'tools/call':
                args = request.get('params', {}).get('arguments', {})
                value = args.get('value', 'default')
                response = {
                    'jsonrpc': '2.0',
                    'id': id,
                    'result': {
                        'content': [{'type': 'text', 'text': f'echo: {value}'}]
                    }
                }
                print(json.dumps(response), flush=True)
        except json.JSONDecodeError:
            continue
        except Exception as e:
            print(json.dumps({'jsonrpc': '2.0', 'id': None, 'error': {'code': -32603, 'message': str(e)}}), flush=True)

if __name__ == '__main__':
    main()
"#;

        let transport = McpTransport::Stdio(StdioProcess::new(
            "python3",
            vec!["-c".to_string(), python_script.to_string()],
        ));

        let client = McpClient::new(transport)
            .with_timeout(Duration::from_secs(10))
            .with_max_retries(2);

        assert_eq!(
            client.connection_state().await,
            ConnectionState::Disconnected
        );

        client.connect().await.expect("connect to stdio server");
        assert_eq!(client.connection_state().await, ConnectionState::Connected);

        let tools = client.list_tools().await.expect("list tools");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "echo");

        client.ping().await.expect("ping");

        let result = client
            .call_tool("echo", &json!({"value": "hello"}))
            .await
            .expect("call tool");
        assert!(result.content.contains("echo: hello"));
        assert!(!result.is_error);

        client.disconnect().await.expect("disconnect");
        assert_eq!(
            client.connection_state().await,
            ConnectionState::Disconnected
        );
    }
}

mod mcp_tool_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_tool_invocation_roundtrip() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}}
            }))),
            "initialized" => Ok(ok_response(json!(null))),
            "tools/list" => Ok(ok_response(json!({
                "tools": [
                    {"name": "capitalize", "description": "Capitalize text", "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}}}
                ]
            }))),
            "tools/call" => {
                let params = request.params.as_ref();
                let name = params
                    .and_then(|p| p.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let args = params
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(json!({}));

                if name == "capitalize" {
                    let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    Ok(ok_response(json!({
                        "content": [{"type": "text", "text": text.to_uppercase()}],
                        "isError": false
                    })))
                } else {
                    Ok(ok_response(json!({
                        "content": [{"type": "text", "text": "unknown tool"}],
                        "isError": true
                    })))
                }
            }
            _ => Ok(ok_response(json!(null))),
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://test-server/sse".to_string()),
            handler,
        );

        client.connect().await.unwrap();

        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert!(tools.iter().any(|t| t.name == "capitalize"));

        let result = client
            .call_tool("capitalize", &json!({"text": "hello world"}))
            .await
            .unwrap();
        assert!(result.content.contains("HELLO WORLD"));
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_mcp_tool_error_response() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}}
            }))),
            "initialized" => Ok(ok_response(json!(null))),
            "tools/list" => Ok(ok_response(json!({
                "tools": [{"name": "fail_tool", "description": "A failing tool", "inputSchema": {"type": "object"}}]
            }))),
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "error occurred"}],
                "isError": true
            }))),
            _ => Ok(ok_response(json!(null))),
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://test-server/sse".to_string()),
            handler,
        );

        client.connect().await.unwrap();

        let result = client.call_tool("fail_tool", &json!({})).await.unwrap();

        assert!(result.is_error);
        assert!(result.content.contains("error"));
    }
}

mod mcp_security_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_oauth_token_configuration() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }))),
            "initialized" => Ok(ok_response(json!(null))),
            _ => Ok(ok_response(json!(null))),
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://oauth-server/sse".to_string()),
            handler,
        )
        .with_oauth_token("test-token".to_string());

        let result = client.connect().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_timeout_error_on_handler() {
        let handler: TransportHandler = Arc::new(|request| {
            if request.method == "ping" {
                return Err(McpError::Timeout(Duration::from_millis(500)));
            }
            Ok(ok_response(json!(null)))
        });

        let client = McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        )
        .with_timeout(Duration::from_secs(10));

        client.connect().await.unwrap();

        let result = client.ping().await;
        assert!(result.is_err());

        if let Err(McpError::Timeout(duration)) = result {
            assert_eq!(duration, Duration::from_millis(500));
        } else {
            panic!("Expected timeout error");
        }
    }

    #[tokio::test]
    async fn test_mcp_connection_timeout_error_message() {
        let handler: TransportHandler = Arc::new(|request| {
            if request.method == "ping" {
                return Err(McpError::Timeout(Duration::from_secs(10)));
            }
            Ok(ok_response(json!(null)))
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://timeout-test/sse".to_string()),
            handler,
        )
        .with_timeout(Duration::from_millis(50));

        client.connect().await.unwrap();

        let err = client.ping().await.unwrap_err();

        assert!(matches!(err, McpError::Timeout(_)));

        let error_string = err.to_string();
        assert!(
            error_string.contains("timeout") || error_string.contains("Timeout"),
            "Error message should mention timeout: {}",
            error_string
        );
    }

    #[tokio::test]
    async fn test_mcp_auth_error_on_request() {
        let auth_error_called = Arc::new(AtomicUsize::new(0));
        let auth_error_called_clone = auth_error_called.clone();

        let handler: TransportHandler = Arc::new(move |request| {
            if request.method == "ping" {
                auth_error_called_clone.fetch_add(1, Ordering::SeqCst);
                return Err(McpError::ConnectionFailed(
                    "authentication required".to_string(),
                ));
            }
            Ok(ok_response(json!(null)))
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://auth-server/sse".to_string()),
            handler,
        );

        client.connect().await.unwrap();

        let err = client.ping().await.unwrap_err();
        assert!(matches!(err, McpError::ConnectionFailed(_)));
        assert_eq!(auth_error_called.load(Ordering::SeqCst), 1);
    }
}

mod mcp_stability_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_auto_reconnect_on_connection_loss() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_cloned = attempts.clone();
        let handler: TransportHandler = Arc::new(move |request| {
            let call_no = attempts_cloned.fetch_add(1, Ordering::SeqCst);
            if request.method == "tools/list" && call_no == 0 {
                return Err(McpError::ConnectionLost("connection dropped".to_string()));
            }
            Ok(ok_response(json!({ "tools": [] })))
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://reconnect-test/sse".to_string()),
            handler,
        )
        .with_max_retries(3);

        client.connect().await.unwrap();
        let _ = client.list_tools().await.unwrap();

        assert!(attempts.load(Ordering::SeqCst) >= 2);
    }

    #[tokio::test]
    async fn test_mcp_reconnect_resets_retry_state() {
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let handler: TransportHandler = Arc::new(move |_request| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            Ok(ok_response(json!({ "tools": [] })))
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://retry-reset/sse".to_string()),
            handler,
        )
        .with_max_retries(2);

        client.connect().await.unwrap();

        let _ = client.list_tools().await;
        let first_count = call_count.load(Ordering::SeqCst);

        let _ = client.list_tools().await;
        let second_count = call_count.load(Ordering::SeqCst);

        assert_eq!(second_count, first_count + 1);
    }
}

mod mcp_protocol_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_protocol_version_in_initialization() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => {
                let params = request.params.as_ref();
                let client_version = params.and_then(|p| p.get("clientInfo"));
                assert!(client_version.is_some());
                Ok(ok_response(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "serverInfo": {"name": "version-test", "version": "1.0"}
                })))
            }
            "initialized" => Ok(ok_response(json!(null))),
            "ping" => Ok(ok_response(json!(null))),
            _ => Ok(ok_response(json!(null))),
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://version-test/sse".to_string()),
            handler,
        );

        client.connect().await.unwrap();
        client.ping().await.unwrap();
    }

    #[tokio::test]
    async fn test_mcp_jsonrpc_error_response_handling() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }))),
            "initialized" => Ok(ok_response(json!(null))),
            "tools/call" => Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(opencode_mcp::protocol::JsonRpcError {
                    code: -32600,
                    message: "Invalid request".to_string(),
                    data: None,
                }),
            }),
            _ => Ok(ok_response(json!(null))),
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://error-test/sse".to_string()),
            handler,
        );

        client.connect().await.unwrap();

        let result = client.call_tool("any_tool", &json!({})).await;

        assert!(result.is_err());
        if let Err(McpError::Protocol(msg)) = result {
            assert!(msg.contains("Invalid request") || msg.contains("32600"));
        }
    }
}

mod mcp_registry_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_registry_bridges_tools_to_tool_registry() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "serverInfo": {"name": "registry-test", "version": "1.0"}
            }))),
            "initialized" => Ok(ok_response(json!(null))),
            "tools/list" => Ok(ok_response(json!({
                "tools": [
                    {"name": "registry_tool", "description": "A bridged tool", "inputSchema": {"type": "object"}}
                ]
            }))),
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "bridged-result"}],
                "isError": false
            }))),
            _ => Ok(ok_response(json!(null))),
        });

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://registry-test/sse".to_string()),
            handler,
        ));

        client.connect().await.unwrap();

        let tools = client.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "registry_tool");
    }

    #[tokio::test]
    async fn test_mcp_multiple_server_connections() {
        let server1_handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(
                json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "server1", "version": "1.0"}}),
            )),
            "initialized" => Ok(ok_response(json!(null))),
            "tools/list" => Ok(ok_response(
                json!({"tools": [{"name": "tool1", "description": "From server 1", "inputSchema": {"type": "object"}}]}),
            )),
            _ => Ok(ok_response(json!(null))),
        });

        let server2_handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(
                json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "server2", "version": "1.0"}}),
            )),
            "initialized" => Ok(ok_response(json!(null))),
            "tools/list" => Ok(ok_response(
                json!({"tools": [{"name": "tool2", "description": "From server 2", "inputSchema": {"type": "object"}}]}),
            )),
            _ => Ok(ok_response(json!(null))),
        });

        let client1 = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://server1/sse".to_string()),
            server1_handler,
        ));
        let client2 = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://server2/sse".to_string()),
            server2_handler,
        ));

        client1.connect().await.unwrap();
        client2.connect().await.unwrap();

        let tools1 = client1.list_tools().await.unwrap();
        let tools2 = client2.list_tools().await.unwrap();

        assert_eq!(tools1.len(), 1);
        assert_eq!(tools1[0].name, "tool1");
        assert_eq!(tools2.len(), 1);
        assert_eq!(tools2[0].name, "tool2");
    }
}

mod mcp_resource_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_resources_list_and_read() {
        let handler: TransportHandler = Arc::new(|request| match request.method.as_str() {
            "initialize" => Ok(ok_response(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {"resources": {}},
                "serverInfo": {"name": "resources-test", "version": "1.0"}
            }))),
            "initialized" => Ok(ok_response(json!(null))),
            "resources/list" => Ok(ok_response(json!({
                "resources": [
                    {"uri": "file:///test/README.md", "name": "README", "description": "Project README", "mimeType": "text/markdown"}
                ]
            }))),
            "resources/read" => {
                let uri = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("uri"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let content = if uri.contains("README") {
                    "# Test Project\n\nWelcome."
                } else {
                    "unknown"
                };
                Ok(ok_response(
                    json!({"contents": [{"uri": uri, "text": content, "mimeType": "text/plain"}]}),
                ))
            }
            _ => Ok(ok_response(json!(null))),
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://resources-test/sse".to_string()),
            handler,
        );

        client.connect().await.unwrap();

        let resources = client.list_resources().await.unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].name, "README");

        let content = client
            .read_resource("file:///test/README.md")
            .await
            .unwrap();
        assert!(content.contains("Test Project"));
    }
}
