use std::future::Future;
use std::sync::{Arc, Mutex};

use opencode_core::{TokenCounter, ToolDefinition, ToolExecutor, ToolParameter, ToolRegistry};
use serde_json::Value;

use crate::client::{McpClient, McpTool};
use crate::context_cost::SharedContextCostTracker;

/// Delimiter used to separate server name from tool name in qualified names
const TOOL_NAME_DELIMITER: &str = "_";

#[derive(Clone)]
pub struct McpToolAdapter {
    client: Arc<McpClient>,
    tool: McpTool,
    server_name: String,
    token_counter: Option<Arc<Mutex<TokenCounter>>>,
    cost_tracker: Option<SharedContextCostTracker>,
    requires_approval: bool,
}

impl McpToolAdapter {
    pub fn with_token_counter(mut self, counter: Arc<Mutex<TokenCounter>>) -> Self {
        self.token_counter = Some(counter);
        self
    }

    pub fn with_cost_tracker(mut self, tracker: SharedContextCostTracker) -> Self {
        self.cost_tracker = Some(tracker);
        self
    }

    pub fn with_requires_approval(mut self, requires_approval: bool) -> Self {
        self.requires_approval = requires_approval;
        self
    }
}

impl McpToolAdapter {
    pub fn new(client: Arc<McpClient>, tool: McpTool, server_name: &str) -> Self {
        Self {
            client,
            tool,
            server_name: server_name.to_string(),
            token_counter: None,
            cost_tracker: None,
            requires_approval: false,
        }
    }

    pub fn qualified_name(&self) -> String {
        format!(
            "{}{}{}",
            self.server_name, TOOL_NAME_DELIMITER, self.tool.name
        )
    }

    pub fn name(&self) -> &str {
        &self.tool.name
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.qualified_name(),
            description: self.tool.description.clone(),
            parameters: vec![ToolParameter {
                name: "args".to_string(),
                description: "Arguments object for MCP tool invocation".to_string(),
                required: false,
                schema: self.tool.input_schema.clone(),
            }],
            requires_approval: self.requires_approval,
        }
    }

    pub fn executor(&self) -> ToolExecutor {
        let client = self.client.clone();
        let tool_name = self.tool.name.clone();
        let token_counter = self.token_counter.clone();

        Arc::new(move |args| {
            let input_tokens = args.to_string().chars().count().div_ceil(4);

            let result = run_async(async { client.call_tool(&tool_name, &args).await });

            if let Some(counter) = &token_counter {
                if let Ok(output) = &result {
                    let output_tokens = output.content.chars().count().div_ceil(4);
                    if let Ok(mut guard) = counter.lock() {
                        guard.record_tokens("gpt-4o", input_tokens, output_tokens);
                    }
                }
            }

            result
                .map(|result| result.content)
                .map_err(|e| format!("MCP tool call failed: {}", e))
        })
    }

    pub fn executor_with_cost_tracking(&self) -> (ToolExecutor, Option<SharedContextCostTracker>) {
        let client = self.client.clone();
        let tool_name = self.tool.name.clone();
        let server_name = self.server_name.clone();
        let cost_tracker = self.cost_tracker.clone();
        let cost_tracker_for_return = cost_tracker.clone();

        let executor: ToolExecutor = Arc::new(move |args: Value| {
            let input_text = args.to_string();
            let input_tokens = input_text.chars().count().div_ceil(4);

            let result = run_async(async { client.call_tool(&tool_name, &args).await });

            let output_text = result
                .as_ref()
                .map(|r| r.content.clone())
                .unwrap_or_default();
            let output_tokens = output_text.chars().count().div_ceil(4);

            if let Some(ref tracker) = cost_tracker {
                tracker.record_tool_call_with_tokens(
                    &tool_name,
                    &server_name,
                    input_tokens,
                    output_tokens,
                );
                if tracker.should_warn() {
                    if let Some(warning) = tracker.get_warning_message() {
                        tracing::warn!("{}", warning);
                    }
                    tracker.record_warning_shown();
                }
            }

            result
                .map(|result| result.content)
                .map_err(|e| format!("MCP tool call failed: {}", e))
        });

        (executor, cost_tracker_for_return)
    }

    pub fn register_into(&self, registry: &mut ToolRegistry) {
        registry.register(self.definition(), self.executor());
    }

    pub fn register_with_cost_tracking(&self, registry: &mut ToolRegistry) {
        if let Some(_) = &self.cost_tracker {
            let (executor, _) = self.executor_with_cost_tracking();
            registry.register(self.definition(), executor);
        } else {
            registry.register(self.definition(), self.executor());
        }
    }
}

fn run_async<T>(future: impl Future<Output = T>) -> T {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(future)
    } else {
        tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime for MCP tool adapter")
            .block_on(future)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::client::{McpError, McpTransport, StdioProcess};
    use crate::protocol::JsonRpcResponse;

    fn ok_response(result: serde_json::Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: None,
            result: Some(result),
            error: None,
        }
    }

    #[test]
    fn test_tool_adapter_definition_and_execution() {
        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(
            |request: crate::protocol::JsonRpcRequest| match request.method.as_str() {
                "tools/call" => Ok(ok_response(json!({
                    "content": [{"type": "text", "text": "adapter-ok"}],
                    "isError": false
                }))),
                _ => Err(McpError::Other("unexpected method".to_string())),
            },
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        ));
        rt.block_on(client.connect()).unwrap();

        let adapter = McpToolAdapter::new(
            client,
            McpTool {
                name: "echo".to_string(),
                description: "Echo tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "mock_server",
        );

        let def = adapter.definition();
        assert_eq!(def.name, "mock_server_echo");
        assert_eq!(def.parameters.len(), 1);

        let mut registry = ToolRegistry::new();
        adapter.register_into(&mut registry);

        let exec = registry.get_executor("mock_server_echo").unwrap();
        let out = exec(json!({"message": "hi"})).unwrap();
        assert_eq!(out, "adapter-ok");
    }

    #[test]
    fn test_mcp_tool_qualification() {
        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "qualified-test-ok"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        ));
        rt.block_on(client.connect()).unwrap();

        let adapter = McpToolAdapter::new(
            client,
            McpTool {
                name: "search".to_string(),
                description: "Search tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "docs_server",
        )
        .with_requires_approval(true);

        assert_eq!(adapter.qualified_name(), "docs_server_search");
        assert_eq!(adapter.name(), "search");
        assert_eq!(adapter.server_name(), "docs_server");

        let def = adapter.definition();
        assert_eq!(def.name, "docs_server_search");
        assert_eq!(def.description, "Search tool");
        assert!(def.requires_approval);
    }

    #[test]
    fn test_mcp_tool_qualification_no_conflicts() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let search_handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "search-result"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let docs_client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock-docs", vec![])),
            search_handler.clone(),
        ));
        rt.block_on(docs_client.connect()).unwrap();

        let web_client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock-web", vec![])),
            search_handler,
        ));
        rt.block_on(web_client.connect()).unwrap();

        let docs_adapter = McpToolAdapter::new(
            docs_client,
            McpTool {
                name: "search".to_string(),
                description: "Search docs".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "docs_server",
        );

        let web_adapter = McpToolAdapter::new(
            web_client,
            McpTool {
                name: "search".to_string(),
                description: "Search web".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "web_server",
        );

        assert_eq!(docs_adapter.qualified_name(), "docs_server_search");
        assert_eq!(web_adapter.qualified_name(), "web_server_search");
        assert_ne!(docs_adapter.qualified_name(), web_adapter.qualified_name());

        let mut registry = ToolRegistry::new();
        docs_adapter.register_into(&mut registry);
        web_adapter.register_into(&mut registry);

        assert!(registry.contains("docs_server_search"));
        assert!(registry.contains("web_server_search"));
        assert!(!registry.contains("search"));
    }

    #[test]
    fn test_mcp_tool_naming_convention() {
        let tool = McpTool {
            name: "myTool".to_string(),
            description: "Test tool".to_string(),
            input_schema: json!({"type": "object"}),
        };

        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            Arc::new(|_| Err(McpError::Other("not used".to_string()))),
        ));

        let adapter = McpToolAdapter::new(client, tool, "my-server");

        let qualified = adapter.qualified_name();
        assert!(qualified.starts_with("my-server_"));
        assert!(qualified.contains("myTool"));
        assert!(!qualified.contains(" "));
        assert!(!qualified.contains("/"));
    }

    #[test]
    fn test_mcp_tool_qualification_collision_handling() {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handler1: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "server1-result"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let handler2: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(|request| match request.method.as_str() {
            "tools/call" => Ok(ok_response(json!({
                "content": [{"type": "text", "text": "server2-result"}],
                "isError": false
            }))),
            _ => Err(McpError::Other("unexpected method".to_string())),
        });

        let client1 = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock1", vec![])),
            handler1,
        ));
        let client2 = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock2", vec![])),
            handler2,
        ));
        rt.block_on(client1.connect()).unwrap();
        rt.block_on(client2.connect()).unwrap();

        let adapter1 = McpToolAdapter::new(
            client1,
            McpTool {
                name: "echo".to_string(),
                description: "Echo from server1".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "server1",
        );

        let adapter2 = McpToolAdapter::new(
            client2,
            McpTool {
                name: "echo".to_string(),
                description: "Echo from server2".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "server2",
        );

        assert_eq!(adapter1.qualified_name(), "server1_echo");
        assert_eq!(adapter2.qualified_name(), "server2_echo");
        assert_ne!(adapter1.qualified_name(), adapter2.qualified_name());

        let mut registry = ToolRegistry::new();
        adapter1.register_into(&mut registry);
        adapter2.register_into(&mut registry);

        let exec1 = registry.get_executor("server1_echo").unwrap();
        let exec2 = registry.get_executor("server2_echo").unwrap();
        assert!(exec1(json!({})).is_ok());
        assert!(exec2(json!({})).is_ok());
    }

    #[test]
    fn test_context_cost_tracker_integration() {
        use crate::context_cost::SharedContextCostTracker;

        let tracker = SharedContextCostTracker::new();

        let record = tracker.record_tool_call(
            "search_docs",
            "docs_server",
            "query about rust programming",
            "Found 42 results about rust programming language",
        );

        assert_eq!(record.tool_name, "search_docs");
        assert_eq!(record.server_name, "docs_server");
        assert!(record.input_tokens > 0);
        assert!(record.output_tokens > 0);
        assert_eq!(tracker.stats().tool_call_count, 1);
    }

    #[test]
    fn test_context_cost_warnings() {
        use crate::context_cost::{ContextCostTracker, CostLimits};

        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(100));

        tracker.record_tool_call("tool1", "server", "input", "output");
        assert!(!tracker.should_warn());

        for _ in 0..10 {
            tracker.record_tool_call(
                "tool",
                "server",
                "some input text here that has meaningful content for token counting purposes",
                "some output text here that has meaningful content for token counting purposes",
            );
        }

        assert!(tracker.should_warn());
        let warning_message = tracker.get_warning_message();
        assert!(warning_message.is_some());

        let msg = warning_message.unwrap();
        assert!(msg.contains("tokens"), "Warning should contain token count");
        assert!(msg.contains("compact"), "Warning should suggest /compact command");
    }

    #[test]
    fn test_cost_calculation_accuracy() {
        use crate::context_cost::{ContextCostTracker, CostLimits};

        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(10000));

        let input = "This is a test input string for token counting";
        let output = "This is a test output string for token counting";

        let record = tracker.record_tool_call("test_tool", "test_server", input, output);

        assert!(record.input_tokens > 0);
        assert!(record.output_tokens > 0);
        assert_eq!(
            record.total_tokens,
            record.input_tokens + record.output_tokens
        );

        let stats = tracker.stats();
        assert_eq!(stats.total_input_tokens, record.input_tokens);
        assert_eq!(stats.total_output_tokens, record.output_tokens);
        assert_eq!(stats.total_tokens, record.total_tokens);
    }

    #[test]
    fn test_cost_level_thresholds() {
        use crate::context_cost::{ContextCostTracker, CostLevel, CostLimits};

        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(1000));

        assert_eq!(tracker.calculate_cost_level(0), CostLevel::Normal);
        assert_eq!(tracker.calculate_cost_level(400), CostLevel::Normal);
        assert_eq!(tracker.calculate_cost_level(500), CostLevel::Warning);
        assert_eq!(tracker.calculate_cost_level(800), CostLevel::Critical);
        assert_eq!(tracker.calculate_cost_level(950), CostLevel::LimitExceeded);
    }

    #[test]
    fn test_shared_tracker_thread_safe() {
        use crate::context_cost::SharedContextCostTracker;

        let tracker = SharedContextCostTracker::new();

        tracker.record_tool_call("tool1", "server1", "input1", "output1");
        tracker.record_tool_call("tool2", "server2", "input2", "output2");

        assert_eq!(tracker.stats().tool_call_count, 2);
        assert!(tracker.total_tokens() > 0);
        assert!(tracker.remaining_tokens() < 128000);
    }

    #[test]
    fn test_cost_tracker_would_exceed_limit() {
        use crate::context_cost::{ContextCostTracker, CostLimits};

        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(100));

        assert!(!tracker.would_exceed_limit(50));
        assert!(tracker.would_exceed_limit(100));
        assert!(tracker.would_exceed_limit(101));
    }

    #[test]
    fn test_cost_records_preserved() {
        use crate::context_cost::{ContextCostTracker, CostLimits};

        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(10000));

        tracker.record_tool_call("tool1", "server", "input1", "output1");
        tracker.record_tool_call("tool2", "server", "input2", "output2");

        let records = tracker.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].tool_name, "tool1");
        assert_eq!(records[1].tool_name, "tool2");
    }

    #[test]
    fn test_cost_tracker_clear() {
        use crate::context_cost::{ContextCostTracker, CostLimits};

        let mut tracker = ContextCostTracker::new();
        tracker.update_limits(CostLimits::new(1000));

        tracker.record_tool_call("tool", "server", "input", "output");
        assert_eq!(tracker.tool_call_count(), 1);

        tracker.clear();
        assert_eq!(tracker.tool_call_count(), 0);
        assert_eq!(tracker.total_tokens(), 0);
        assert!(!tracker.should_warn());
    }

    #[test]
    fn test_mcp_tool_adapter_with_cost_tracker() {
        let handler: Arc<
            dyn Fn(crate::protocol::JsonRpcRequest) -> Result<JsonRpcResponse, McpError>
                + Send
                + Sync,
        > = Arc::new(
            |request: crate::protocol::JsonRpcRequest| match request.method.as_str() {
                "tools/call" => Ok(ok_response(json!({
                    "content": [{"type": "text", "text": "cost-tracked-result"}],
                    "isError": false
                }))),
                _ => Err(McpError::Other("unexpected method".to_string())),
            },
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = Arc::new(McpClient::with_handler(
            McpTransport::Stdio(StdioProcess::new("mock", vec![])),
            handler,
        ));
        rt.block_on(client.connect()).unwrap();

        let cost_tracker = crate::context_cost::SharedContextCostTracker::new();

        let adapter = McpToolAdapter::new(
            client,
            McpTool {
                name: "costly_tool".to_string(),
                description: "A tool with cost tracking".to_string(),
                input_schema: json!({"type": "object"}),
            },
            "cost_server",
        )
        .with_cost_tracker(cost_tracker.clone());

        let mut registry = ToolRegistry::new();
        adapter.register_with_cost_tracking(&mut registry);

        let exec = registry.get_executor("cost_server_costly_tool").unwrap();
        let out = exec(json!({"param": "value"})).unwrap();
        assert_eq!(out, "cost-tracked-result");

        let stats = cost_tracker.stats();
        assert_eq!(stats.tool_call_count, 1);
    }
}
