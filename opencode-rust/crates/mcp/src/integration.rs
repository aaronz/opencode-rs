use std::future::Future;

use opencode_core::ToolRegistry;

use crate::client::McpClient;
use crate::tool_bridge::McpToolAdapter;

pub fn register_mcp_tools(
    client: &McpClient,
    server_name: &str,
    registry: &mut ToolRegistry,
) {
    let client = client.clone();
    let tools = match run_async(async { client.list_tools().await }) {
        Ok(tools) => tools,
        Err(_) => return,
    };

    for tool in tools {
        let adapter =
            McpToolAdapter::new(std::sync::Arc::new(client.clone()), tool, server_name);
        adapter.register_into(registry);
    }
}

fn run_async<T>(future: impl Future<Output = T>) -> T {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| handle.block_on(future))
    } else {
        tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime for mcp tool registration")
            .block_on(future)
    }
}

#[cfg(test)]
mod integration {
    use opencode_core::ToolRegistry;
    use crate::registry::{McpPermission, McpRegistry, McpServerConfig};
    use crate::tool_bridge::McpToolAdapter;
    use crate::{McpTransport, ConnectionState, McpClient, McpTool, protocol::JsonRpcResponse};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;

    type TransportHandler = Arc<
        dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, crate::McpError>
            + Send
            + Sync,
    >;

    #[tokio::test]
    async fn test_local_mcp_connection_end_to_end() {
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
                        'serverInfo': {'name': 'test-local-server', 'version': '1.0.0'}
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
                            {'name': 'echo', 'description': 'Echo back the input', 'inputSchema': {'type': 'object', 'properties': {'value': {'type': 'string'}}}},
                            {'name': 'uppercase', 'description': 'Convert to uppercase', 'inputSchema': {'type': 'object', 'properties': {'text': {'type': 'string'}}}}
                        ]
                    }
                }
                print(json.dumps(response), flush=True)
            elif method == 'tools/call':
                args = request.get('params', {}).get('arguments', {})
                tool_name = request.get('params', {}).get('name', '')
                if tool_name == 'echo':
                    value = args.get('value', 'no value')
                    response = {
                        'jsonrpc': '2.0',
                        'id': id,
                        'result': {
                            'content': [{'type': 'text', 'text': f'echo: {value}'}]
                        }
                    }
                elif tool_name == 'uppercase':
                    text = args.get('text', '')
                    response = {
                        'jsonrpc': '2.0',
                        'id': id,
                        'result': {
                            'content': [{'type': 'text', 'text': text.upper()}]
                        }
                    }
                else:
                    response = {'jsonrpc': '2.0', 'id': id, 'error': {'code': -32601, 'message': f'Unknown tool: {tool_name}'}}
                print(json.dumps(response), flush=True)
            else:
                response = {'jsonrpc': '2.0', 'id': id, 'error': {'code': -32601, 'message': f'Unknown method: {method}'}}
                print(json.dumps(response), flush=True)
        except json.JSONDecodeError:
            continue
        except Exception as e:
            print(json.dumps({'jsonrpc': '2.0', 'id': None, 'error': {'code': -32603, 'message': str(e)}}), flush=True)

if __name__ == '__main__':
    main()
"#;

        let transport = McpTransport::Stdio(
            crate::client::StdioProcess::new("python3", vec!["-c".to_string(), python_script.to_string()])
        );

        let client = McpClient::new(transport)
            .with_timeout(Duration::from_secs(10))
            .with_max_retries(2);

        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
        
        client.connect().await.expect("connect to local stdio server");
        assert_eq!(client.connection_state().await, ConnectionState::Connected);

        let tools = client.list_tools().await.expect("list tools");
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "echo"));
        assert!(tools.iter().any(|t| t.name == "uppercase"));

        client.ping().await.expect("ping");

        let result = client
            .call_tool("echo", &json!({"value": "hello local"}))
            .await
            .expect("call echo tool");
        assert!(result.content.contains("echo: hello local"));
        assert!(!result.is_error);

        let result = client
            .call_tool("uppercase", &json!({"text": "hello"}))
            .await
            .expect("call uppercase tool");
        assert!(result.content.contains("HELLO"));
        assert!(!result.is_error);

        client.disconnect().await.expect("disconnect");
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_remote_mcp_connection_end_to_end() {
        let handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {"tools": {"listChanged": true}},
                        "serverInfo": {"name": "test-remote-server", "version": "1.0.0"}
                    }),
                )),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "ping" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"tools": [
                        {"name": "search", "description": "Search remote resources", "inputSchema": {"type": "object", "properties": {"query": {"type": "string"}}}},
                        {"name": "fetch", "description": "Fetch remote resource", "inputSchema": {"type": "object", "properties": {"url": {"type": "string"}}}}
                    ]}),
                )),
                "tools/call" => {
                    let tool_name = request.params.as_ref()
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    let args = request.params.as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned()
                        .unwrap_or(json!({}));
                    
                    if tool_name == "search" {
                        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                        Ok(JsonRpcResponse::success(
                            request.id.clone(),
                            json!({"content": [{"type": "text", "text": format!("search results for: {}", query)}], "isError": false}),
                        ))
                    } else if tool_name == "fetch" {
                        let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
                        Ok(JsonRpcResponse::success(
                            request.id.clone(),
                            json!({"content": [{"type": "text", "text": format!("fetched: {}", url)}], "isError": false}),
                        ))
                    } else {
                        Ok(JsonRpcResponse::error(
                            request.id.clone(),
                            -32601,
                            format!("Unknown tool: {}", tool_name),
                        ))
                    }
                },
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://mock-remote-server:8080/sse".to_string()),
            handler,
        );

        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
        
        client.connect().await.expect("connect with SSE handler");
        assert_eq!(client.connection_state().await, ConnectionState::Connected);

        let tools = client.list_tools().await.expect("list remote tools");
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().any(|t| t.name == "search"));
        assert!(tools.iter().any(|t| t.name == "fetch"));

        client.ping().await.expect("ping remote server");

        let result = client
            .call_tool("search", &json!({"query": "rust programming"}))
            .await
            .expect("call search tool");
        assert!(result.content.contains("search results for: rust programming"));
        assert!(!result.is_error);

        let result = client
            .call_tool("fetch", &json!({"url": "https://example.com"}))
            .await
            .expect("call fetch tool");
        assert!(result.content.contains("fetched: https://example.com"));
        assert!(!result.is_error);

        client.disconnect().await.expect("disconnect");
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_tool_discovery_integration() {
        let local_handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "local", "version": "1.0"}}),
                )),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"tools": [
                        {"name": "local_tool_a", "description": "A local tool", "inputSchema": {"type": "object"}},
                        {"name": "local_tool_b", "description": "Another local tool", "inputSchema": {"type": "object"}}
                    ]}),
                )),
                "tools/call" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"content": [{"type": "text", "text": "local-result"}], "isError": false}),
                )),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let remote_handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "remote", "version": "1.0"}}),
                )),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"tools": [
                        {"name": "remote_tool_x", "description": "A remote tool", "inputSchema": {"type": "object"}}
                    ]}),
                )),
                "tools/call" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"content": [{"type": "text", "text": "remote-result"}], "isError": false}),
                )),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let local_client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(crate::client::StdioProcess::new("mock-local", vec![])),
            local_handler,
        ));
        
        let remote_client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://mock-remote/sse".to_string()),
            remote_handler,
        ));

        local_client.connect().await.unwrap();
        remote_client.connect().await.unwrap();

        let mut registry = McpRegistry::new();
        registry.clients.insert("local_server".to_string(), local_client.clone());
        registry.clients.insert("remote_server".to_string(), remote_client.clone());
        
        registry.discovered_tools.insert(
            "local_server".to_string(),
            local_client.list_tools().await.unwrap(),
        );
        registry.discovered_tools.insert(
            "remote_server".to_string(),
            remote_client.list_tools().await.unwrap(),
        );

        let local_tools = registry.tools_for_server("local_server");
        assert!(local_tools.is_some());
        let local_tools = local_tools.unwrap();
        assert_eq!(local_tools.len(), 2);
        assert!(local_tools.iter().any(|t| t.name == "local_tool_a"));
        assert!(local_tools.iter().any(|t| t.name == "local_tool_b"));

        let remote_tools = registry.tools_for_server("remote_server");
        assert!(remote_tools.is_some());
        let remote_tools = remote_tools.unwrap();
        assert_eq!(remote_tools.len(), 1);
        assert!(remote_tools.iter().any(|t| t.name == "remote_tool_x"));

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        assert!(tool_registry.contains("local_server_local_tool_a"));
        assert!(tool_registry.contains("local_server_local_tool_b"));
        assert!(tool_registry.contains("remote_server_remote_tool_x"));
        assert!(!tool_registry.contains("local_tool_a"));
        assert!(!tool_registry.contains("remote_tool_x"));

        let all_tools: Vec<&McpTool> = registry
            .discovered_tools
            .values()
            .flat_map(|tools| tools.iter())
            .collect();
        assert_eq!(all_tools.len(), 3);
    }

    #[test]
    fn test_mcp_registry_bridges_tools_correctly() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        let handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "bridge-test", "version": "1.0"}}),
                )),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"tools": [
                        {"name": "bridged_tool", "description": "A tool to be bridged", "inputSchema": {"type": "object", "properties": {"input": {"type": "string"}}}}
                    ]}),
                )),
                "tools/call" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"content": [{"type": "text", "text": "bridged-call-success"}], "isError": false}),
                )),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://bridge-test/sse".to_string()),
            handler,
        ));
        rt.block_on(client.connect()).unwrap();

        let mut registry = McpRegistry::new();
        registry.clients.insert("bridge_server".to_string(), client.clone());
        registry.discovered_tools.insert(
            "bridge_server".to_string(),
            rt.block_on(client.list_tools()).unwrap(),
        );

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        let qualified_name = "bridge_server_bridged_tool";
        assert!(tool_registry.contains(qualified_name));

        let executor = tool_registry.get_executor(qualified_name);
        assert!(executor.is_some());

        let result = executor.unwrap()(json!({"input": "test"}));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "bridged-call-success");
    }

    #[tokio::test]
    async fn test_mcp_client_resources_end_to_end() {
        let handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"protocolVersion": "2024-11-05", "capabilities": {"resources": {}}, "serverInfo": {"name": "resources-test", "version": "1.0"}}),
                )),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "resources/list" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"resources": [
                        {"uri": "file:///test/README.md", "name": "README", "description": "Project README", "mimeType": "text/markdown"},
                        {"uri": "file:///test/config.json", "name": "Config", "description": "Configuration file", "mimeType": "application/json"}
                    ]}),
                )),
                "resources/read" => {
                    let uri = request.params.as_ref()
                        .and_then(|p| p.get("uri"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let content = if uri.contains("README") {
                        "# Test Project\n\nWelcome to the test project."
                    } else {
                        r#"{"setting": "value"}"#
                    };
                    Ok(JsonRpcResponse::success(
                        request.id.clone(),
                        json!({"contents": [{"uri": uri, "text": content, "mimeType": "text/plain"}]}),
                    ))
                },
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://resources-test/sse".to_string()),
            handler,
        );

        client.connect().await.unwrap();

        let resources = client.list_resources().await.unwrap();
        assert_eq!(resources.len(), 2);
        assert!(resources.iter().any(|r| r.name == "README"));
        assert!(resources.iter().any(|r| r.name == "Config"));

        let readme_content = client.read_resource("file:///test/README.md").await.unwrap();
        assert!(readme_content.contains("Test Project"));

        let config_content = client.read_resource("file:///test/config.json").await.unwrap();
        assert!(config_content.contains("setting"));
    }

    #[tokio::test]
    async fn test_mcp_connection_state_transitions() {
        let handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "state-test", "version": "1.0"}}),
                )),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "ping" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client = McpClient::with_handler(
            McpTransport::Sse("http://state-test/sse".to_string()),
            handler,
        );

        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
        assert!(!client.is_connected().await);

        client.connect().await.unwrap();
        assert_eq!(client.connection_state().await, ConnectionState::Connected);
        assert!(client.is_connected().await);

        client.ping().await.unwrap();

        client.disconnect().await.unwrap();
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
        assert!(!client.is_connected().await);

        client.connect().await.unwrap();
        assert_eq!(client.connection_state().await, ConnectionState::Connected);

        client.disconnect().await.unwrap();
        assert_eq!(client.connection_state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_multiple_servers_tool_discovery() {
        let server1_handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "server1", "version": "1.0"}}))),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"tools": [{"name": "tool1", "description": "From server 1", "inputSchema": {"type": "object"}}]}))),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let server2_handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "server2", "version": "1.0"}}))),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"tools": [{"name": "tool2", "description": "From server 2", "inputSchema": {"type": "object"}}]}))),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let server3_handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "server3", "version": "1.0"}}))),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"tools": [{"name": "tool3", "description": "From server 3", "inputSchema": {"type": "object"}}]}))),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client1 = Arc::new(McpClient::with_handler(McpTransport::Sse("http://server1/sse".to_string()), server1_handler));
        let client2 = Arc::new(McpClient::with_handler(McpTransport::Sse("http://server2/sse".to_string()), server2_handler));
        let client3 = Arc::new(McpClient::with_handler(McpTransport::Sse("http://server3/sse".to_string()), server3_handler));

        client1.connect().await.unwrap();
        client2.connect().await.unwrap();
        client3.connect().await.unwrap();

        let mut registry = McpRegistry::new();
        registry.clients.insert("server1".to_string(), client1.clone());
        registry.clients.insert("server2".to_string(), client2.clone());
        registry.clients.insert("server3".to_string(), client3.clone());

        registry.discovered_tools.insert("server1".to_string(), client1.list_tools().await.unwrap());
        registry.discovered_tools.insert("server2".to_string(), client2.list_tools().await.unwrap());
        registry.discovered_tools.insert("server3".to_string(), client3.list_tools().await.unwrap());

        let all_tools: Vec<&McpTool> = registry
            .discovered_tools
            .values()
            .flat_map(|tools: &Vec<McpTool>| tools.iter())
            .collect();
        assert_eq!(all_tools.len(), 3);
        assert!(all_tools.iter().any(|t| t.name == "tool1"));
        assert!(all_tools.iter().any(|t| t.name == "tool2"));
        assert!(all_tools.iter().any(|t| t.name == "tool3"));

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        assert!(tool_registry.contains("server1_tool1"));
        assert!(tool_registry.contains("server2_tool2"));
        assert!(tool_registry.contains("server3_tool3"));
    }

    #[tokio::test]
    async fn test_mcp_tool_adapter_qualified_naming() {
        let handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "adapter-test", "version": "1.0"}}))),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/call" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"content": [{"type": "text", "text": "adapter-test-result"}], "isError": false}))),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://adapter-test/sse".to_string()),
            handler,
        ));
        client.connect().await.unwrap();

        let adapter = McpToolAdapter::new(
            client,
            McpTool {
                name: "myTool".to_string(),
                description: "Test tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "myServer",
        );

        assert_eq!(adapter.qualified_name(), "myServer_myTool");
        assert_eq!(adapter.name(), "myTool");
        assert_eq!(adapter.server_name(), "myServer");

        let def = adapter.definition();
        assert_eq!(def.name, "myServer_myTool");
        assert!(def.name.contains("myServer_"));
    }

    #[tokio::test]
    async fn test_mcp_registry_server_permissions() {
        let handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "perms-test", "version": "1.0"}}))),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"tools": [{"name": "perm_tool", "description": "Test tool", "inputSchema": {"type": "object"}}]}))),
                "tools/call" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"content": [{"type": "text", "text": "perm-ok"}], "isError": false}))),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://perms-test/sse".to_string()),
            handler,
        ));
        client.connect().await.unwrap();

        let mut registry = McpRegistry::new();
        registry.add_server("allow_server", McpServerConfig::new(McpTransport::Sse("http://allow/sse".to_string())).with_permission(McpPermission::Allow));
        registry.add_server("ask_server", McpServerConfig::new(McpTransport::Sse("http://ask/sse".to_string())).with_permission(McpPermission::Ask));
        registry.add_server("deny_server", McpServerConfig::new(McpTransport::Sse("http://deny/sse".to_string())).with_permission(McpPermission::Deny));

        registry.clients.insert("allow_server".to_string(), client.clone());
        registry.clients.insert("ask_server".to_string(), client.clone());
        registry.clients.insert("deny_server".to_string(), client.clone());

        registry.discovered_tools.insert("allow_server".to_string(), client.list_tools().await.unwrap());
        registry.discovered_tools.insert("ask_server".to_string(), client.list_tools().await.unwrap());
        registry.discovered_tools.insert("deny_server".to_string(), client.list_tools().await.unwrap());

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        let allow_def = tool_registry.get("allow_server_perm_tool");
        assert!(allow_def.is_some());
        assert!(!allow_def.unwrap().requires_approval, "Allow permission should not require approval");

        let ask_def = tool_registry.get("ask_server_perm_tool");
        assert!(ask_def.is_some());
        assert!(ask_def.unwrap().requires_approval, "Ask permission should require approval");
    }

    #[test]
    fn test_tool_execution_after_discovery() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        let handler: TransportHandler = Arc::new(|request| {
            match request.method.as_str() {
                "initialize" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"protocolVersion": "2024-11-05", "capabilities": {}, "serverInfo": {"name": "exec-test", "version": "1.0"}}))),
                "initialized" => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
                "tools/list" => Ok(JsonRpcResponse::success(request.id.clone(), json!({"tools": [{"name": "exec_tool", "description": "Execution test", "inputSchema": {"type": "object"}}]}))),
                "tools/call" => Ok(JsonRpcResponse::success(
                    request.id.clone(),
                    json!({"content": [{"type": "text", "text": "exec-result"}], "isError": false}),
                )),
                _ => Ok(JsonRpcResponse::success(request.id.clone(), json!(null))),
            }
        });

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Sse("http://exec-test/sse".to_string()),
            handler,
        ));
        rt.block_on(client.connect()).unwrap();

        let mut registry = McpRegistry::new();
        registry.clients.insert("exec_server".to_string(), client.clone());
        registry.discovered_tools.insert("exec_server".to_string(), rt.block_on(client.list_tools()).unwrap());

        let mut tool_registry = ToolRegistry::new();
        registry.bridge_to_tool_registry(&mut tool_registry);

        let executor1 = tool_registry.get_executor("exec_server_exec_tool").unwrap();
        let result1 = executor1(json!({})).unwrap();
        assert_eq!(result1, "exec-result");

        let executor2 = tool_registry.get_executor("exec_server_exec_tool").unwrap();
        let result2 = executor2(json!({})).unwrap();
        assert_eq!(result2, "exec-result");

        let executor3 = tool_registry.get_executor("exec_server_exec_tool").unwrap();
        let result3 = executor3(json!({})).unwrap();
        assert_eq!(result3, "exec-result");
    }
}
