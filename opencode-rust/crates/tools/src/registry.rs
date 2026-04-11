use crate::{Tool, ToolContext, ToolResult};
use opencode_core::OpenCodeError;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

const CACHE_KEY_PREFIX: &str = "tool_cache:";

fn compute_cache_key(tool_name: &str, args: &serde_json::Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(args.to_string().as_bytes());
    let args_hash = format!("{:x}", hasher.finalize());
    format!("{}{}:{}", CACHE_KEY_PREFIX, tool_name, args_hash)
}

/// Source of a tool, used for deterministic collision resolution
/// Lower ordinal = higher priority (can override lower priority tools)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ToolSource {
    /// Built-in tools (highest priority, cannot be overridden)
    #[default]
    Builtin,
    /// Tools from plugins
    Plugin,
    /// Custom tools from project directory (.opencode/tools/)
    CustomProject,
    /// Custom tools from global config directory (~/.config/opencode/tools/)
    CustomGlobal,
}

impl ToolSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolSource::CustomGlobal => "custom_global",
            ToolSource::CustomProject => "custom_project",
            ToolSource::Plugin => "plugin",
            ToolSource::Builtin => "builtin",
        }
    }
}

/// Metadata about a registered tool
struct ToolEntry {
    tool: Box<dyn Tool>,
    source: ToolSource,
}

impl ToolEntry {
    fn new(tool: Box<dyn Tool>, source: ToolSource) -> Self {
        Self { tool, source }
    }
}

impl std::fmt::Debug for ToolEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolEntry")
            .field("source", &self.source)
            .finish()
    }
}

impl Clone for ToolEntry {
    fn clone(&self) -> Self {
        Self {
            tool: self.tool.clone_tool(),
            source: self.source,
        }
    }
}

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
    tools: Arc<RwLock<HashMap<String, ToolEntry>>>,
    disabled: HashSet<String>,
    cache: Arc<RwLock<HashMap<String, ToolResult>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            disabled: HashSet::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_disabled(&mut self, tools: HashSet<String>) {
        self.disabled = tools;
    }

    pub fn is_disabled(&self, name: &str) -> bool {
        self.disabled.contains(name)
    }

    pub async fn invalidate_cache_for_tool(&self, tool_name: &str) {
        let mut cache = self.cache.write().await;
        let prefix = format!("{}{}:", CACHE_KEY_PREFIX, tool_name);
        cache.retain(|key, _| !key.starts_with(&prefix));
    }

    pub async fn invalidate_all_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn get_cached_result(&self, tool_name: &str, args: &serde_json::Value) -> Option<ToolResult> {
        let cache = self.cache.read().await;
        let key = compute_cache_key(tool_name, args);
        cache.get(&key).cloned()
    }

    async fn cache_result(&self, tool_name: &str, args: &serde_json::Value, result: ToolResult) {
        let mut cache = self.cache.write().await;
        let key = compute_cache_key(tool_name, args);
        cache.insert(key, result);
    }

    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        self.register_with_source(tool, ToolSource::Builtin).await;
    }

    pub async fn register_with_source<T: Tool + 'static>(&self, tool: T, source: ToolSource) {
        let mut tools = self.tools.write().await;
        let name = tool.name().to_string();

        if let Some(existing) = tools.get(&name) {
            if source < existing.source {
                tracing::debug!(
                    tool = %name,
                    existing_source = %existing.source.as_str(),
                    new_source = %source.as_str(),
                    "tool collision resolved: new higher-priority tool replaces existing"
                );
            } else {
                tracing::debug!(
                    tool = %name,
                    existing_source = %existing.source.as_str(),
                    new_source = %source.as_str(),
                    "tool collision resolved: existing tool wins"
                );
                return;
            }
        }

        tracing::debug!(
            tool = %name,
            source = %source.as_str(),
            "registering tool"
        );
        tools.insert(name, ToolEntry::new(Box::new(tool), source));
    }

    pub async fn register_plugin_tools(&self, tools: Vec<Box<dyn Tool>>) {
        self.register_tools_with_source(tools, ToolSource::Plugin).await;
    }

    pub async fn register_tools_with_source(&self, tools: Vec<Box<dyn Tool>>, source: ToolSource) {
        let mut registry = self.tools.write().await;
        for tool in tools {
            let name = tool.name().to_string();

            if let Some(existing) = registry.get(&name) {
                if source < existing.source {
                    tracing::debug!(
                        tool = %name,
                        existing_source = %existing.source.as_str(),
                        new_source = %source.as_str(),
                        "tool collision resolved: new higher-priority tool replaces existing"
                    );
                } else {
                    tracing::debug!(
                        tool = %name,
                        existing_source = %existing.source.as_str(),
                        new_source = %source.as_str(),
                        "tool collision resolved: existing tool wins"
                    );
                    continue;
                }
            }

            tracing::debug!(
                tool = %name,
                source = %source.as_str(),
                "registering tool"
            );
            registry.insert(name, ToolEntry::new(tool, source));
        }
    }

    pub async fn get(&self, name: &str) -> Option<Box<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).map(|t| t.tool.clone_tool())
    }

    pub async fn list_filtered(&self, model: Option<&ModelInfo>) -> Vec<(String, String, bool)> {
        let tools = self.tools.read().await;

        let Some(model_info) = model else {
            return tools
                .iter()
                .map(|(name, entry)| {
                    (
                        name.clone(),
                        entry.tool.description().to_string(),
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
            .map(|(name, entry)| {
                (
                    name.clone(),
                    entry.tool.description().to_string(),
                    self.is_disabled(name),
                )
            })
            .collect()
    }

    pub async fn get_with_status(&self, name: &str) -> Option<(Box<dyn Tool>, bool)> {
        let tools = self.tools.read().await;
        tools
            .get(name)
            .map(|entry| (entry.tool.clone_tool(), self.is_disabled(name)))
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

        let is_safe = tool.is_safe();

        if is_safe {
            if let Some(cached) = self.get_cached_result(name, &args).await {
                tracing::debug!(tool = %name, "returning cached result");
                return Ok(cached);
            }
        }

        let result = tool.execute(args.clone(), ctx).await?;

        if is_safe && result.success {
            self.cache_result(name, &args, result.clone()).await;
        }

        Ok(result)
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
                    tools.get(&name).map(|t| t.tool.clone_tool())
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

    #[tokio::test]
    async fn test_collision_resolution_builtin_overrides_custom() {
        #[derive(Clone)]
        struct CustomTool;
        #[async_trait]
        impl Tool for CustomTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Custom tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("custom"))
            }
        }

        #[derive(Clone)]
        struct BuiltinTool;
        #[async_trait]
        impl Tool for BuiltinTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Builtin tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("builtin"))
            }
        }

        let registry = ToolRegistry::new();

        registry.register_with_source(CustomTool, ToolSource::CustomGlobal).await;
        registry.register_with_source(BuiltinTool, ToolSource::Builtin).await;

        let result = registry
            .execute("collision_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "builtin");
    }

    #[tokio::test]
    async fn test_collision_resolution_plugin_overrides_custom_global() {
        #[derive(Clone)]
        struct GlobalTool;
        #[async_trait]
        impl Tool for GlobalTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Global tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("global"))
            }
        }

        #[derive(Clone)]
        struct PluginTool;
        #[async_trait]
        impl Tool for PluginTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Plugin tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("plugin"))
            }
        }

        let registry = ToolRegistry::new();

        registry.register_with_source(GlobalTool, ToolSource::CustomGlobal).await;
        registry.register_with_source(PluginTool, ToolSource::Plugin).await;

        let result = registry
            .execute("collision_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "plugin");
    }

    #[tokio::test]
    async fn test_collision_resolution_plugin_overrides_custom_project() {
        #[derive(Clone)]
        struct ProjectTool;
        #[async_trait]
        impl Tool for ProjectTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Project tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("project"))
            }
        }

        #[derive(Clone)]
        struct PluginTool;
        #[async_trait]
        impl Tool for PluginTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Plugin tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("plugin"))
            }
        }

        let registry = ToolRegistry::new();

        registry.register_with_source(ProjectTool, ToolSource::CustomProject).await;
        registry.register_with_source(PluginTool, ToolSource::Plugin).await;

        let result = registry
            .execute("collision_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "plugin");
    }

    #[tokio::test]
    async fn test_collision_resolution_custom_project_overrides_custom_global() {
        #[derive(Clone)]
        struct GlobalTool;
        #[async_trait]
        impl Tool for GlobalTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Global tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("global"))
            }
        }

        #[derive(Clone)]
        struct ProjectTool;
        #[async_trait]
        impl Tool for ProjectTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Project tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("project"))
            }
        }

        let registry = ToolRegistry::new();

        registry.register_with_source(GlobalTool, ToolSource::CustomGlobal).await;
        registry.register_with_source(ProjectTool, ToolSource::CustomProject).await;

        let result = registry
            .execute("collision_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "project");
    }

    #[tokio::test]
    async fn test_collision_resolution_deterministic_first_registers_wins() {
        #[derive(Clone)]
        struct FirstTool;
        #[async_trait]
        impl Tool for FirstTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "First tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("first"))
            }
        }

        #[derive(Clone)]
        struct SecondTool;
        #[async_trait]
        impl Tool for SecondTool {
            fn name(&self) -> &str { "collision_tool" }
            fn description(&self) -> &str { "Second tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("second"))
            }
        }

        let registry = ToolRegistry::new();

        registry.register_with_source(FirstTool, ToolSource::CustomProject).await;
        registry.register_with_source(SecondTool, ToolSource::CustomProject).await;

        let result = registry
            .execute("collision_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "first");
    }

    #[tokio::test]
    async fn test_collision_resolution_same_collision_always_resolves_same_way() {
        #[derive(Clone)]
        struct ToolA;
        #[async_trait]
        impl Tool for ToolA {
            fn name(&self) -> &str { "test_tool" }
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
            fn name(&self) -> &str { "test_tool" }
            fn description(&self) -> &str { "Tool B" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("b"))
            }
        }

        let registry1 = ToolRegistry::new();
        registry1.register_with_source(ToolA, ToolSource::Builtin).await;
        registry1.register_with_source(ToolB, ToolSource::CustomGlobal).await;

        let result1 = registry1.execute("test_tool", serde_json::json!({}), None).await.unwrap();

        let registry2 = ToolRegistry::new();
        registry2.register_with_source(ToolB, ToolSource::CustomGlobal).await;
        registry2.register_with_source(ToolA, ToolSource::Builtin).await;

        let result2 = registry2.execute("test_tool", serde_json::json!({}), None).await.unwrap();

        assert_eq!(result1.content, "a");
        assert_eq!(result2.content, "a");
    }

    #[tokio::test]
    async fn test_builtin_tool_cannot_be_overridden_by_lower_priority() {
        #[derive(Clone)]
        struct BuiltinTool;
        #[async_trait]
        impl Tool for BuiltinTool {
            fn name(&self) -> &str { "my_tool" }
            fn description(&self) -> &str { "Builtin" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("builtin"))
            }
        }

        #[derive(Clone)]
        struct CustomTool;
        #[async_trait]
        impl Tool for CustomTool {
            fn name(&self) -> &str { "my_tool" }
            fn description(&self) -> &str { "Custom" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(self.clone()) }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("custom"))
            }
        }

        let registry = ToolRegistry::new();

        registry.register_with_source(BuiltinTool, ToolSource::Builtin).await;
        registry.register_with_source(CustomTool, ToolSource::CustomGlobal).await;

        let result = registry.execute("my_tool", serde_json::json!({}), None).await.unwrap();
        assert_eq!(result.content, "builtin");

        let registry2 = ToolRegistry::new();

        registry2.register_with_source(CustomTool, ToolSource::CustomGlobal).await;
        registry2.register_with_source(BuiltinTool, ToolSource::Builtin).await;

        let result2 = registry2.execute("my_tool", serde_json::json!({}), None).await.unwrap();
        assert_eq!(result2.content, "builtin");
    }

    #[tokio::test]
    async fn test_tool_source_ordering() {
        assert!(ToolSource::Builtin < ToolSource::Plugin);
        assert!(ToolSource::Plugin < ToolSource::CustomProject);
        assert!(ToolSource::CustomProject < ToolSource::CustomGlobal);

        assert!(ToolSource::Builtin <= ToolSource::Builtin);
        assert!(ToolSource::CustomGlobal >= ToolSource::CustomGlobal);
    }

    #[derive(Clone)]
    struct SafeTestTool {
        call_count: std::sync::Arc<std::sync::Mutex<u32>>,
    }

    impl SafeTestTool {
        fn new() -> Self {
            Self {
                call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    #[async_trait]
    impl Tool for SafeTestTool {
        fn name(&self) -> &str {
            "safe_test_tool"
        }

        fn description(&self) -> &str {
            "Safe test tool"
        }

        fn clone_tool(&self) -> Box<dyn Tool> {
            let count = std::sync::Arc::clone(&self.call_count);
            Box::new(SafeTestTool { call_count: count })
        }

        fn is_safe(&self) -> bool {
            true
        }

        async fn execute(
            &self,
            args: serde_json::Value,
            _ctx: Option<ToolContext>,
        ) -> Result<ToolResult, OpenCodeError> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("default");
            Ok(ToolResult::ok(format!("called {} times: {}", *count, input)))
        }
    }

    #[derive(Clone)]
    struct UnsafeTestTool {
        call_count: std::sync::Arc<std::sync::Mutex<u32>>,
    }

    impl UnsafeTestTool {
        fn new() -> Self {
            Self {
                call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
            }
        }
    }

    #[async_trait]
    impl Tool for UnsafeTestTool {
        fn name(&self) -> &str {
            "unsafe_test_tool"
        }

        fn description(&self) -> &str {
            "Unsafe test tool"
        }

        fn clone_tool(&self) -> Box<dyn Tool> {
            let count = std::sync::Arc::clone(&self.call_count);
            Box::new(UnsafeTestTool { call_count: count })
        }

        fn is_safe(&self) -> bool {
            false
        }

        async fn execute(
            &self,
            args: serde_json::Value,
            _ctx: Option<ToolContext>,
        ) -> Result<ToolResult, OpenCodeError> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("default");
            Ok(ToolResult::ok(format!("called {} times: {}", *count, input)))
        }
    }

    #[tokio::test]
    async fn test_result_caching_safe_tools_cached() {
        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry.execute("safe_test_tool", args.clone(), None).await.unwrap();
        let result2 = registry.execute("safe_test_tool", args.clone(), None).await.unwrap();

        assert_eq!(result1.content, result2.content);
        assert!(result1.content.contains("called 1 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_unsafe_tools_not_cached() {
        let registry = ToolRegistry::new();
        registry.register(UnsafeTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry.execute("unsafe_test_tool", args.clone(), None).await.unwrap();
        let result2 = registry.execute("unsafe_test_tool", args.clone(), None).await.unwrap();

        assert!(result1.content.contains("called 1 times:"));
        assert!(result2.content.contains("called 2 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_different_args_not_cached() {
        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;

        let args1 = serde_json::json!({"input": "test1"});
        let args2 = serde_json::json!({"input": "test2"});

        let result1 = registry.execute("safe_test_tool", args1.clone(), None).await.unwrap();
        let result2 = registry.execute("safe_test_tool", args2.clone(), None).await.unwrap();

        assert!(result1.content.contains("called 1 times:"));
        assert!(result2.content.contains("called 2 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_cache_invalidation() {
        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry.execute("safe_test_tool", args.clone(), None).await.unwrap();
        assert!(result1.content.contains("called 1 times:"));

        registry.invalidate_cache_for_tool("safe_test_tool").await;

        let result2 = registry.execute("safe_test_tool", args.clone(), None).await.unwrap();
        assert!(result2.content.contains("called 2 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_invalidate_all() {
        #[derive(Clone)]
        struct AnotherSafeTool;
        #[async_trait]
        impl Tool for AnotherSafeTool {
            fn name(&self) -> &str { "another_safe_tool" }
            fn description(&self) -> &str { "Another safe tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(AnotherSafeTool) }
            fn is_safe(&self) -> bool { true }
            async fn execute(&self, args: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                let input = args.get("input").and_then(|v| v.as_str()).unwrap_or("default");
                Ok(ToolResult::ok(format!("result: {}", input)))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;
        registry.register(AnotherSafeTool).await;

        registry.execute("safe_test_tool", serde_json::json!({"input": "test"}), None).await.unwrap();
        registry.execute("another_safe_tool", serde_json::json!({"input": "test"}), None).await.unwrap();

        registry.invalidate_all_cache().await;

        let result = registry.execute("safe_test_tool", serde_json::json!({"input": "test"}), None).await.unwrap();
        assert!(result.content.contains("called"));
    }

    #[tokio::test]
    async fn test_result_caching_failure_not_cached() {
        #[derive(Clone)]
        struct SafeFailingTool;
        #[async_trait]
        impl Tool for SafeFailingTool {
            fn name(&self) -> &str { "safe_failing_tool" }
            fn description(&self) -> &str { "Safe failing tool" }
            fn clone_tool(&self) -> Box<dyn Tool> { Box::new(SafeFailingTool) }
            fn is_safe(&self) -> bool { true }
            async fn execute(&self, _: serde_json::Value, _: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::err("intentional failure"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(SafeFailingTool).await;

        let args = serde_json::json!({});

        let result1 = registry.execute("safe_failing_tool", args.clone(), None).await.unwrap();
        assert!(!result1.success);

        let result2 = registry.execute("safe_failing_tool", args.clone(), None).await.unwrap();
        assert!(!result2.success);
        assert!(result2.error.unwrap().contains("intentional failure"));
    }
}
