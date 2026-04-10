use crate::{Tool, ToolContext, ToolResult};
use opencode_core::OpenCodeError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Provider ID for filtering tools
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProviderId {
    OpenAI,
    Anthropic,
    OpenCode,
    GitHubCopilot,
    Azure,
    Custom(String),
}

impl ProviderId {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => ProviderId::OpenAI,
            "anthropic" => ProviderId::Anthropic,
            "opencode" => ProviderId::OpenCode,
            "github-copilot" | "github_copilot" => ProviderId::GitHubCopilot,
            "azure" | "azure-cognitive-services" => ProviderId::Azure,
            other => ProviderId::Custom(other.to_string()),
        }
    }

    pub fn is_opencode(&self) -> bool {
        matches!(self, ProviderId::OpenCode)
    }
}

/// Model info for tool filtering
pub struct ModelInfo {
    pub provider_id: ProviderId,
    pub model_id: String,
}

impl ModelInfo {
    /// Check if model should use apply_patch tool (GPT models except GPT-4 and OSS)
    pub fn use_apply_patch(&self) -> bool {
        let model_id = self.model_id.to_lowercase();
        model_id.starts_with("gpt-") && !model_id.contains("gpt-4") && !model_id.contains("oss")
    }
}

pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
    pub ctx: Option<ToolContext>,
}

pub struct ToolCallResult {
    pub name: String,
    pub result: Result<ToolResult, OpenCodeError>,
}

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn Tool>>>>,
    disabled: HashSet<String>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            disabled: HashSet::new(),
        }
    }

    pub fn set_disabled(&mut self, tools: HashSet<String>) {
        self.disabled = tools;
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled.contains(name)
    }

    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub async fn register_plugin_tools(&self, tools: Vec<Box<dyn Tool>>) {
        let mut registry = self.tools.write().await;
        for tool in tools {
            registry.insert(tool.name().to_string(), tool);
        }
    }

    pub async fn get(&self, name: &str) -> Option<Box<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).map(|t| t.clone_tool())
    }

    pub async fn list_filtered(&self, model: Option<&ModelInfo>) -> Vec<(String, String, bool)> {
        let tools = self.tools.read().await;

        let Some(model_info) = model else {
            return tools
                .iter()
                .map(|(name, tool)| {
                    (
                        name.clone(),
                        tool.description().to_string(),
                        self.is_disabled(name),
                    )
                })
                .collect();
        };

        tools
            .iter()
            .filter(|(name, _)| match name.as_str() {
                "codesearch" | "websearch" => model_info.provider_id.is_opencode(),
                "apply_patch" => model_info.use_apply_patch(),
                "edit" | "write" => !model_info.use_apply_patch(),
                _ => true,
            })
            .map(|(name, tool)| {
                (
                    name.clone(),
                    tool.description().to_string(),
                    self.is_disabled(name),
                )
            })
            .collect()
    }

    pub async fn get_with_status(&self, name: &str) -> Option<(Box<dyn Tool>, bool)> {
        let tools = self.tools.read().await;
        tools
            .get(name)
            .map(|tool| (tool.clone_tool(), self.is_disabled(name)))
    }

    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        if self.is_disabled(name) {
            return Err(OpenCodeError::Tool(format!("Tool '{}' is disabled", name)));
        }

        let tool = self
            .get(name)
            .await
            .ok_or_else(|| OpenCodeError::Tool(format!("Tool '{}' not found", name)))?;
        tool.execute(args, ctx).await
    }

    pub async fn execute_parallel(&self, calls: Vec<ToolCall>) -> Vec<ToolCallResult> {
        let mut handles = Vec::new();

        for call in calls {
            let registry = Arc::clone(&self.tools);
            let disabled = self.disabled.clone();
            let name = call.name.clone();
            let args = call.args;
            let ctx = call.ctx;

            handles.push(tokio::spawn(async move {
                if disabled.contains(&name) {
                    return ToolCallResult {
                        name: name.clone(),
                        result: Err(OpenCodeError::Tool(format!("Tool '{}' is disabled", name))),
                    };
                }

                let tool = {
                    let tools = registry.read().await;
                    tools.get(&name).map(|t| t.clone_tool())
                };

                let result = match tool {
                    Some(t) => t.execute(args, ctx).await,
                    None => Err(OpenCodeError::Tool(format!("Tool '{}' not found", name))),
                };

                ToolCallResult { name, result }
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        results
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct TestTool;

    #[async_trait]
    impl Tool for TestTool {
        fn name(&self) -> &str {
            "test_tool"
        }

        fn description(&self) -> &str {
            "Test tool"
        }

        fn clone_tool(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }

        async fn execute(
            &self,
            _args: serde_json::Value,
            _ctx: Option<ToolContext>,
        ) -> Result<ToolResult, OpenCodeError> {
            Ok(ToolResult::ok("ok"))
        }
    }

    #[tokio::test]
    async fn execute_returns_error_for_disabled_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool).await;
        registry.set_disabled(HashSet::from(["test_tool".to_string()]));

        let result = registry
            .execute("test_tool", serde_json::json!({}), None)
            .await;

        match result {
            Err(OpenCodeError::Tool(message)) => {
                assert!(message.contains("disabled"));
            }
            other => panic!("expected disabled tool error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_with_status_marks_disabled_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool).await;
        registry.set_disabled(HashSet::from(["test_tool".to_string()]));

        let tools = registry.list_filtered(None).await;
        let entry = tools
            .iter()
            .find(|(name, _, _)| name == "test_tool")
            .expect("tool should be listed");

        assert!(entry.2);
    }

    #[tokio::test]
    async fn empty_disabled_set_keeps_tools_enabled() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool).await;
        registry.set_disabled(HashSet::new());

        let exec = registry
            .execute("test_tool", serde_json::json!({}), None)
            .await;
        assert!(exec.is_ok());

        let tools = registry.list_filtered(None).await;
        let entry = tools
            .iter()
            .find(|(name, _, _)| name == "test_tool")
            .expect("tool should be listed");
        assert!(!entry.2);
    }

    #[derive(Clone)]
    struct TestToolWithArgs;

    #[async_trait]
    impl Tool for TestToolWithArgs {
        fn name(&self) -> &str {
            "test_tool_with_args"
        }

        fn description(&self) -> &str {
            "Test tool with arguments"
        }

        fn clone_tool(&self) -> Box<dyn Tool> {
            Box::new(self.clone())
        }

        async fn execute(
            &self,
            args: serde_json::Value,
            _ctx: Option<ToolContext>,
        ) -> Result<ToolResult, OpenCodeError> {
            let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolResult::ok(format!("received: {}", input)))
        }
    }

    #[tokio::test]
    async fn test_register_and_lookup_tool() {
        let registry = ToolRegistry::new();
        registry.register(TestTool).await;

        let tool = registry.get("test_tool").await;
        assert!(tool.is_some(), "Registered tool should be findable by name");

        let tool = registry.get("nonexistent").await;
        assert!(tool.is_none(), "Nonexistent tool should return None");
    }

    #[tokio::test]
    async fn test_register_multiple_unique_tools() {
        #[derive(Clone)]
        struct ToolA;
        #[async_trait]
        impl Tool for ToolA {
            fn name(&self) -> &str { "tool_a" }
            fn description(&self) -> &str { "Tool A" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("a"))
            }
        }

        #[derive(Clone)]
        struct ToolB;
        #[async_trait]
        impl Tool for ToolB {
            fn name(&self) -> &str { "tool_b" }
            fn description(&self) -> &str { "Tool B" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("b"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(ToolA).await;
        registry.register(ToolB).await;

        let tools = registry.list_filtered(None).await;
        let names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(names.contains(&"tool_a"), "tool_a should be registered");
        assert!(names.contains(&"tool_b"), "tool_b should be registered");
    }

    #[tokio::test]
    async fn test_tool_execution_with_arguments() {
        let registry = ToolRegistry::new();
        registry.register(TestToolWithArgs).await;

        let result = registry
            .execute("test_tool_with_args", serde_json::json!({"input": "hello"}), None)
            .await;

        assert!(result.is_ok(), "Tool should execute successfully");
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "received: hello");
    }

    #[tokio::test]
    async fn test_tool_lifecycle_full() {
        let mut registry = ToolRegistry::new();

        // Register
        registry.register(TestTool).await;

        // Lookup
        let tool = registry.get("test_tool").await;
        assert!(tool.is_some(), "Tool should be findable after registration");

        // Execute
        let result = registry
            .execute("test_tool", serde_json::json!({}), None)
            .await;
        assert!(result.is_ok(), "Tool should execute after registration");

        // Disable
        registry.set_disabled(HashSet::from(["test_tool".to_string()]));

        // Verify disabled
        let result = registry
            .execute("test_tool", serde_json::json!({}), None)
            .await;
        assert!(result.is_err(), "Disabled tool should return error");
    }

    #[tokio::test]
    async fn test_get_with_status() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool).await;

        let Some((_, disabled)) = registry.get_with_status("test_tool").await else {
            panic!("Tool should be returned");
        };
        assert!(!disabled, "Tool should not be disabled by default");

        registry.set_disabled(HashSet::from(["test_tool".to_string()]));
        let Some((_, disabled)) = registry.get_with_status("test_tool").await else {
            panic!("Tool should still be returned");
        };
        assert!(disabled, "Tool should be marked as disabled");
    }

    #[tokio::test]
    async fn test_tool_not_found_error() {
        let registry = ToolRegistry::new();

        let result = registry
            .execute("nonexistent_tool", serde_json::json!({}), None)
            .await;

        match result {
            Err(OpenCodeError::Tool(msg)) => {
                assert!(msg.contains("not found"), "Error should indicate tool not found");
            }
            other => panic!("Expected tool not found error, got {:?}", other),
        }
    }
}
