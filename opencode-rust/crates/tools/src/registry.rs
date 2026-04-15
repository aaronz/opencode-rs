use crate::sealed;
use crate::{Tool, ToolContext, ToolResult};
use opencode_core::OpenCodeError;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const CACHE_KEY_PREFIX: &str = "tool_cache:";
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

struct CachedToolResult {
    result: ToolResult,
    cached_at: Instant,
    ttl: Duration,
    dependencies: HashSet<PathBuf>,
}

impl CachedToolResult {
    fn new(result: ToolResult, ttl: Duration, dependencies: HashSet<PathBuf>) -> Self {
        Self {
            result,
            cached_at: Instant::now(),
            ttl,
            dependencies,
        }
    }

    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }

    fn has_stale_dependencies(&self) -> bool {
        let cached_duration = self.cached_at.elapsed();
        for dep in &self.dependencies {
            if let Ok(metadata) = std::fs::metadata(dep) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(modified_duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                        if modified_duration > cached_duration {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn is_valid(&self) -> bool {
        !self.is_expired() && !self.has_stale_dependencies()
    }
}

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
    pub(crate) fn as_str(&self) -> &'static str {
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
    #[allow(dead_code)]
    pub(crate) fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => ProviderId::OpenAI,
            "anthropic" => ProviderId::Anthropic,
            "opencode" => ProviderId::OpenCode,
            "github-copilot" | "github_copilot" => ProviderId::GitHubCopilot,
            "azure" | "azure-cognitive-services" => ProviderId::Azure,
            other => ProviderId::Custom(other.to_string()),
        }
    }

    pub(crate) fn is_opencode(&self) -> bool {
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
    pub(crate) fn use_apply_patch(&self) -> bool {
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
    cache: Arc<RwLock<HashMap<String, CachedToolResult>>>,
    default_ttl: Duration,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            disabled: HashSet::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: DEFAULT_CACHE_TTL,
        }
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            disabled: HashSet::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: ttl,
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

    pub async fn invalidate_cache_for_file(&self, file_path: &PathBuf) {
        let mut cache = self.cache.write().await;
        cache.retain(|_key, cached| !cached.dependencies.contains(file_path));
    }

    pub async fn invalidate_cache_for_files(&self, file_paths: &HashSet<PathBuf>) {
        let mut cache = self.cache.write().await;
        cache.retain(|_key, cached| {
            !cached
                .dependencies
                .iter()
                .any(|dep| file_paths.contains(dep))
        });
    }

    pub async fn get_cached_result(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
    ) -> Option<ToolResult> {
        let key = compute_cache_key(tool_name, args);
        let mut cache = self.cache.write().await;

        if let Some(cached) = cache.get(&key) {
            if cached.is_valid() {
                tracing::debug!(tool = %tool_name, "returning valid cached result");
                return Some(cached.result.clone());
            } else {
                tracing::debug!(tool = %tool_name, "cached result expired or has stale dependencies, removing");
                cache.remove(&key);
            }
        }
        None
    }

    async fn cache_result(
        &self,
        tool_name: &str,
        args: &serde_json::Value,
        result: ToolResult,
        dependencies: HashSet<PathBuf>,
    ) {
        let mut cache = self.cache.write().await;
        let key = compute_cache_key(tool_name, args);
        let cached = CachedToolResult::new(result, self.default_ttl, dependencies);
        cache.insert(key, cached);
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
        self.register_tools_with_source(tools, ToolSource::Plugin)
            .await;
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
            let dependencies = tool.get_dependencies(&args);
            self.cache_result(name, &args, result.clone(), dependencies)
                .await;
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
            fn name(&self) -> &str {
                "tool_a"
            }
            fn description(&self) -> &str {
                "Tool A"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("a"))
            }
        }

        #[derive(Clone)]
        struct ToolB;
        #[async_trait]
        impl Tool for ToolB {
            fn name(&self) -> &str {
                "tool_b"
            }
            fn description(&self) -> &str {
                "Tool B"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
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
            .execute(
                "test_tool_with_args",
                serde_json::json!({"input": "hello"}),
                None,
            )
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
                assert!(
                    msg.contains("not found"),
                    "Error should indicate tool not found"
                );
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
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Custom tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("custom"))
            }
        }

        #[derive(Clone)]
        struct BuiltinTool;
        #[async_trait]
        impl Tool for BuiltinTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Builtin tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("builtin"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(CustomTool, ToolSource::CustomGlobal)
            .await;
        registry
            .register_with_source(BuiltinTool, ToolSource::Builtin)
            .await;

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
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Global tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("global"))
            }
        }

        #[derive(Clone)]
        struct PluginTool;
        #[async_trait]
        impl Tool for PluginTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Plugin tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("plugin"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(GlobalTool, ToolSource::CustomGlobal)
            .await;
        registry
            .register_with_source(PluginTool, ToolSource::Plugin)
            .await;

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
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Project tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("project"))
            }
        }

        #[derive(Clone)]
        struct PluginTool;
        #[async_trait]
        impl Tool for PluginTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Plugin tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("plugin"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(ProjectTool, ToolSource::CustomProject)
            .await;
        registry
            .register_with_source(PluginTool, ToolSource::Plugin)
            .await;

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
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Global tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("global"))
            }
        }

        #[derive(Clone)]
        struct ProjectTool;
        #[async_trait]
        impl Tool for ProjectTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Project tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("project"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(GlobalTool, ToolSource::CustomGlobal)
            .await;
        registry
            .register_with_source(ProjectTool, ToolSource::CustomProject)
            .await;

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
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "First tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("first"))
            }
        }

        #[derive(Clone)]
        struct SecondTool;
        #[async_trait]
        impl Tool for SecondTool {
            fn name(&self) -> &str {
                "collision_tool"
            }
            fn description(&self) -> &str {
                "Second tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("second"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(FirstTool, ToolSource::CustomProject)
            .await;
        registry
            .register_with_source(SecondTool, ToolSource::CustomProject)
            .await;

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
            fn name(&self) -> &str {
                "test_tool"
            }
            fn description(&self) -> &str {
                "Tool A"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("a"))
            }
        }

        #[derive(Clone)]
        struct ToolB;
        #[async_trait]
        impl Tool for ToolB {
            fn name(&self) -> &str {
                "test_tool"
            }
            fn description(&self) -> &str {
                "Tool B"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("b"))
            }
        }

        let registry1 = ToolRegistry::new();
        registry1
            .register_with_source(ToolA, ToolSource::Builtin)
            .await;
        registry1
            .register_with_source(ToolB, ToolSource::CustomGlobal)
            .await;

        let result1 = registry1
            .execute("test_tool", serde_json::json!({}), None)
            .await
            .unwrap();

        let registry2 = ToolRegistry::new();
        registry2
            .register_with_source(ToolB, ToolSource::CustomGlobal)
            .await;
        registry2
            .register_with_source(ToolA, ToolSource::Builtin)
            .await;

        let result2 = registry2
            .execute("test_tool", serde_json::json!({}), None)
            .await
            .unwrap();

        assert_eq!(result1.content, "a");
        assert_eq!(result2.content, "a");
    }

    #[tokio::test]
    async fn test_builtin_tool_cannot_be_overridden_by_lower_priority() {
        #[derive(Clone)]
        struct BuiltinTool;
        #[async_trait]
        impl Tool for BuiltinTool {
            fn name(&self) -> &str {
                "my_tool"
            }
            fn description(&self) -> &str {
                "Builtin"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("builtin"))
            }
        }

        #[derive(Clone)]
        struct CustomTool;
        #[async_trait]
        impl Tool for CustomTool {
            fn name(&self) -> &str {
                "my_tool"
            }
            fn description(&self) -> &str {
                "Custom"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("custom"))
            }
        }

        let registry = ToolRegistry::new();

        registry
            .register_with_source(BuiltinTool, ToolSource::Builtin)
            .await;
        registry
            .register_with_source(CustomTool, ToolSource::CustomGlobal)
            .await;

        let result = registry
            .execute("my_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert_eq!(result.content, "builtin");

        let registry2 = ToolRegistry::new();

        registry2
            .register_with_source(CustomTool, ToolSource::CustomGlobal)
            .await;
        registry2
            .register_with_source(BuiltinTool, ToolSource::Builtin)
            .await;

        let result2 = registry2
            .execute("my_tool", serde_json::json!({}), None)
            .await
            .unwrap();
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
            let input = args
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(ToolResult::ok(format!(
                "called {} times: {}",
                *count, input
            )))
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
            let input = args
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(ToolResult::ok(format!(
                "called {} times: {}",
                *count, input
            )))
        }
    }

    #[tokio::test]
    async fn test_result_caching_safe_tools_cached() {
        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry
            .execute("safe_test_tool", args.clone(), None)
            .await
            .unwrap();
        let result2 = registry
            .execute("safe_test_tool", args.clone(), None)
            .await
            .unwrap();

        assert_eq!(result1.content, result2.content);
        assert!(result1.content.contains("called 1 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_unsafe_tools_not_cached() {
        let registry = ToolRegistry::new();
        registry.register(UnsafeTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry
            .execute("unsafe_test_tool", args.clone(), None)
            .await
            .unwrap();
        let result2 = registry
            .execute("unsafe_test_tool", args.clone(), None)
            .await
            .unwrap();

        assert!(result1.content.contains("called 1 times:"));
        assert!(result2.content.contains("called 2 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_different_args_not_cached() {
        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;

        let args1 = serde_json::json!({"input": "test1"});
        let args2 = serde_json::json!({"input": "test2"});

        let result1 = registry
            .execute("safe_test_tool", args1.clone(), None)
            .await
            .unwrap();
        let result2 = registry
            .execute("safe_test_tool", args2.clone(), None)
            .await
            .unwrap();

        assert!(result1.content.contains("called 1 times:"));
        assert!(result2.content.contains("called 2 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_cache_invalidation() {
        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;

        let args = serde_json::json!({"input": "test"});

        let result1 = registry
            .execute("safe_test_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result1.content.contains("called 1 times:"));

        registry.invalidate_cache_for_tool("safe_test_tool").await;

        let result2 = registry
            .execute("safe_test_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result2.content.contains("called 2 times:"));
    }

    #[tokio::test]
    async fn test_result_caching_invalidate_all() {
        #[derive(Clone)]
        struct AnotherSafeTool;
        #[async_trait]
        impl Tool for AnotherSafeTool {
            fn name(&self) -> &str {
                "another_safe_tool"
            }
            fn description(&self) -> &str {
                "Another safe tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(AnotherSafeTool)
            }
            fn is_safe(&self) -> bool {
                true
            }
            async fn execute(
                &self,
                args: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                let input = args
                    .get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                Ok(ToolResult::ok(format!("result: {}", input)))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(SafeTestTool::new()).await;
        registry.register(AnotherSafeTool).await;

        registry
            .execute("safe_test_tool", serde_json::json!({"input": "test"}), None)
            .await
            .unwrap();
        registry
            .execute(
                "another_safe_tool",
                serde_json::json!({"input": "test"}),
                None,
            )
            .await
            .unwrap();

        registry.invalidate_all_cache().await;

        let result = registry
            .execute("safe_test_tool", serde_json::json!({"input": "test"}), None)
            .await
            .unwrap();
        assert!(result.content.contains("called"));
    }

    #[tokio::test]
    async fn test_result_caching_failure_not_cached() {
        #[derive(Clone)]
        struct SafeFailingTool;
        #[async_trait]
        impl Tool for SafeFailingTool {
            fn name(&self) -> &str {
                "safe_failing_tool"
            }
            fn description(&self) -> &str {
                "Safe failing tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(SafeFailingTool)
            }
            fn is_safe(&self) -> bool {
                true
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::err("intentional failure"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(SafeFailingTool).await;

        let args = serde_json::json!({});

        let result1 = registry
            .execute("safe_failing_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(!result1.success);

        let result2 = registry
            .execute("safe_failing_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(!result2.success);
        assert!(result2.error.unwrap().contains("intentional failure"));
    }

    #[tokio::test]
    async fn test_result_caching_ttl_invalidation() {
        use std::sync::atomic::{AtomicI32, Ordering};
        use std::time::Duration;

        static CALL_COUNT: AtomicI32 = AtomicI32::new(0);

        #[derive(Clone)]
        struct TtlTestTool;
        #[async_trait]
        impl Tool for TtlTestTool {
            fn name(&self) -> &str {
                "ttl_test_tool"
            }
            fn description(&self) -> &str {
                "TTL test tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            fn is_safe(&self) -> bool {
                true
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                let count = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
                Ok(ToolResult::ok(format!("call: {}", count + 1)))
            }
        }

        let registry = ToolRegistry::with_ttl(Duration::from_millis(50));
        registry.register(TtlTestTool).await;

        let args = serde_json::json!({});

        let result1 = registry
            .execute("ttl_test_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result1.content.contains("call: 1"));

        tokio::time::sleep(Duration::from_millis(60)).await;

        let result2 = registry
            .execute("ttl_test_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(
            result2.content.contains("call: 2"),
            "Should re-execute after TTL expiration, got: {}",
            result2.content
        );
    }

    #[tokio::test]
    async fn test_result_caching_dependency_invalidation() {
        use tempfile::TempDir;

        #[derive(Clone)]
        struct FileDepTestTool;
        #[async_trait]
        impl Tool for FileDepTestTool {
            fn name(&self) -> &str {
                "file_dep_test_tool"
            }
            fn description(&self) -> &str {
                "File dependency test tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            fn is_safe(&self) -> bool {
                true
            }
            fn get_dependencies(&self, args: &serde_json::Value) -> HashSet<PathBuf> {
                if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
                    let mut deps = HashSet::new();
                    deps.insert(PathBuf::from(path));
                    deps
                } else {
                    HashSet::new()
                }
            }
            async fn execute(
                &self,
                args: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let content = std::fs::read_to_string(path).unwrap_or_default();
                Ok(ToolResult::ok(format!("content: {}", content)))
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "original").unwrap();

        let registry = ToolRegistry::new();
        registry.register(FileDepTestTool).await;

        let args = serde_json::json!({"path": file_path.to_str().unwrap()});

        let result1 = registry
            .execute("file_dep_test_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result1.content.contains("content: original"));

        std::fs::write(&file_path, "modified").unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let result2 = registry
            .execute("file_dep_test_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(
            result2.content.contains("content: modified"),
            "Should detect file change and re-execute, got: {}",
            result2.content
        );
    }

    #[tokio::test]
    async fn test_invalidate_cache_for_file() {
        use tempfile::TempDir;

        #[derive(Clone)]
        struct MultiFileDepTool;
        #[async_trait]
        impl Tool for MultiFileDepTool {
            fn name(&self) -> &str {
                "multi_file_dep_tool"
            }
            fn description(&self) -> &str {
                "Multi file dependency tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            fn is_safe(&self) -> bool {
                true
            }
            fn get_dependencies(&self, args: &serde_json::Value) -> HashSet<PathBuf> {
                let mut deps = HashSet::new();
                if let Some(paths) = args.get("paths").and_then(|v| v.as_array()) {
                    for path in paths {
                        if let Some(p) = path.as_str() {
                            deps.insert(PathBuf::from(p));
                        }
                    }
                }
                deps
            }
            async fn execute(
                &self,
                args: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                let paths = args
                    .get("paths")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                    .unwrap_or_default();
                let contents: String = paths
                    .iter()
                    .map(|p| std::fs::read_to_string(p).unwrap_or_default())
                    .collect();
                Ok(ToolResult::ok(format!("contents: {}", contents)))
            }
        }

        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        std::fs::write(&file1_path, "file1").unwrap();
        std::fs::write(&file2_path, "file2").unwrap();

        let registry = ToolRegistry::new();
        registry.register(MultiFileDepTool).await;

        let args = serde_json::json!({
            "paths": [file1_path.to_str().unwrap(), file2_path.to_str().unwrap()]
        });

        let result1 = registry
            .execute("multi_file_dep_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(result1.content.contains("contents: file1file2"));

        std::fs::write(&file1_path, "file1_modified").unwrap();

        registry.invalidate_cache_for_file(&file1_path).await;

        let result2 = registry
            .execute("multi_file_dep_tool", args.clone(), None)
            .await
            .unwrap();
        assert!(
            result2.content.contains("file1_modified"),
            "Should re-execute after invalidating file1 dependency, got: {}",
            result2.content
        );
    }

    #[tokio::test]
    async fn test_execute_parallel_multiple_tools() {
        #[derive(Clone)]
        struct ToolAlpha;
        #[async_trait]
        impl Tool for ToolAlpha {
            fn name(&self) -> &str {
                "alpha"
            }
            fn description(&self) -> &str {
                "Alpha tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("alpha_result"))
            }
        }

        #[derive(Clone)]
        struct ToolBeta;
        #[async_trait]
        impl Tool for ToolBeta {
            fn name(&self) -> &str {
                "beta"
            }
            fn description(&self) -> &str {
                "Beta tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("beta_result"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(ToolAlpha).await;
        registry.register(ToolBeta).await;

        let calls = vec![
            ToolCall {
                name: "alpha".to_string(),
                args: serde_json::json!({}),
                ctx: None,
            },
            ToolCall {
                name: "beta".to_string(),
                args: serde_json::json!({}),
                ctx: None,
            },
        ];

        let results = registry.execute_parallel(calls).await;
        assert_eq!(results.len(), 2);

        let alpha_result = results.iter().find(|r| r.name == "alpha").unwrap();
        assert!(alpha_result.result.as_ref().is_ok());
        assert_eq!(
            alpha_result.result.as_ref().unwrap().content,
            "alpha_result"
        );

        let beta_result = results.iter().find(|r| r.name == "beta").unwrap();
        assert!(beta_result.result.as_ref().is_ok());
        assert_eq!(beta_result.result.as_ref().unwrap().content, "beta_result");
    }

    #[tokio::test]
    async fn test_execute_parallel_with_nonexistent_tool() {
        let registry = ToolRegistry::new();

        let calls = vec![ToolCall {
            name: "nonexistent".to_string(),
            args: serde_json::json!({}),
            ctx: None,
        }];

        let results = registry.execute_parallel(calls).await;
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert!(result.result.is_err());
        assert!(result
            .result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("not found"));
    }

    #[tokio::test]
    async fn test_execute_parallel_with_disabled_tool() {
        #[derive(Clone)]
        struct ParallelTool;
        #[async_trait]
        impl Tool for ParallelTool {
            fn name(&self) -> &str {
                "parallel_tool"
            }
            fn description(&self) -> &str {
                "Parallel tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("parallel_result"))
            }
        }

        let mut registry = ToolRegistry::new();
        registry.register(ParallelTool).await;
        registry.set_disabled(HashSet::from(["parallel_tool".to_string()]));

        let calls = vec![ToolCall {
            name: "parallel_tool".to_string(),
            args: serde_json::json!({}),
            ctx: None,
        }];

        let results = registry.execute_parallel(calls).await;
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert!(result.result.is_err());
        assert!(result
            .result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("disabled"));
    }

    #[tokio::test]
    async fn test_list_filtered_empty_registry() {
        let registry = ToolRegistry::new();
        let tools = registry.list_filtered(None).await;
        assert!(tools.is_empty());
    }

    #[tokio::test]
    async fn test_list_filtered_returns_all_registered_tools() {
        #[derive(Clone)]
        struct ListToolA;
        #[async_trait]
        impl Tool for ListToolA {
            fn name(&self) -> &str {
                "list_tool_a"
            }
            fn description(&self) -> &str {
                "List tool A"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("a"))
            }
        }

        #[derive(Clone)]
        struct ListToolB;
        #[async_trait]
        impl Tool for ListToolB {
            fn name(&self) -> &str {
                "list_tool_b"
            }
            fn description(&self) -> &str {
                "List tool B"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("b"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(ListToolA).await;
        registry.register(ListToolB).await;

        let tools = registry.list_filtered(None).await;
        assert_eq!(tools.len(), 2);

        let names: Vec<&str> = tools.iter().map(|(n, _, _)| n.as_str()).collect();
        assert!(names.contains(&"list_tool_a"));
        assert!(names.contains(&"list_tool_b"));
    }

    #[tokio::test]
    async fn test_register_tools_with_source_batch() {
        #[derive(Clone)]
        struct BatchTool1;
        #[async_trait]
        impl Tool for BatchTool1 {
            fn name(&self) -> &str {
                "batch_tool_1"
            }
            fn description(&self) -> &str {
                "Batch tool 1"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("batch1"))
            }
        }

        #[derive(Clone)]
        struct BatchTool2;
        #[async_trait]
        impl Tool for BatchTool2 {
            fn name(&self) -> &str {
                "batch_tool_2"
            }
            fn description(&self) -> &str {
                "Batch tool 2"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("batch2"))
            }
        }

        let registry = ToolRegistry::new();
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(BatchTool1), Box::new(BatchTool2)];
        registry
            .register_tools_with_source(tools, ToolSource::Plugin)
            .await;

        let tool1 = registry.get("batch_tool_1").await;
        assert!(tool1.is_some());

        let tool2 = registry.get("batch_tool_2").await;
        assert!(tool2.is_some());
    }

    #[tokio::test]
    async fn test_register_plugin_tools() {
        #[derive(Clone)]
        struct PluginBatchTool;
        #[async_trait]
        impl Tool for PluginBatchTool {
            fn name(&self) -> &str {
                "plugin_batch_tool"
            }
            fn description(&self) -> &str {
                "Plugin batch tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("plugin_batch"))
            }
        }

        let registry = ToolRegistry::new();
        let tools: Vec<Box<dyn Tool>> = vec![Box::new(PluginBatchTool)];
        registry.register_plugin_tools(tools).await;

        let tool = registry.get("plugin_batch_tool").await;
        assert!(tool.is_some());
    }

    #[tokio::test]
    async fn test_get_cached_result() {
        #[derive(Clone)]
        struct CacheTestTool;
        #[async_trait]
        impl Tool for CacheTestTool {
            fn name(&self) -> &str {
                "cache_test_tool"
            }
            fn description(&self) -> &str {
                "Cache test tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            fn is_safe(&self) -> bool {
                true
            }
            async fn execute(
                &self,
                args: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                let input = args
                    .get("input")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                Ok(ToolResult::ok(format!("cached: {}", input)))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(CacheTestTool).await;

        let args = serde_json::json!({"input": "test_value"});

        let cached = registry.get_cached_result("cache_test_tool", &args).await;
        assert!(
            cached.is_none(),
            "Should not have cached result before execution"
        );

        registry
            .execute("cache_test_tool", args.clone(), None)
            .await
            .unwrap();

        let cached = registry.get_cached_result("cache_test_tool", &args).await;
        assert!(
            cached.is_some(),
            "Should have cached result after execution"
        );
        assert!(cached.unwrap().content.contains("cached: test_value"));
    }

    #[tokio::test]
    async fn test_empty_registry_returns_none_for_get() {
        let registry = ToolRegistry::new();
        let tool = registry.get("nonexistent").await;
        assert!(tool.is_none());
    }

    #[tokio::test]
    async fn test_empty_registry_returns_none_for_get_with_status() {
        let registry = ToolRegistry::new();
        let result = registry.get_with_status("nonexistent").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_execute_with_context() {
        #[derive(Clone)]
        struct ContextTool;
        #[async_trait]
        impl Tool for ContextTool {
            fn name(&self) -> &str {
                "context_tool"
            }
            fn description(&self) -> &str {
                "Context tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                ctx: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                if let Some(context) = ctx {
                    Ok(ToolResult::ok(format!(
                        "session: {}, agent: {}",
                        context.session_id, context.agent
                    )))
                } else {
                    Ok(ToolResult::ok("no context"))
                }
            }
        }

        let registry = ToolRegistry::new();
        registry.register(ContextTool).await;

        let ctx = ToolContext {
            session_id: "test_session".to_string(),
            message_id: "test_message".to_string(),
            agent: "test_agent".to_string(),
            worktree: None,
            directory: None,
            permission_scope: None,
        };

        let result = registry
            .execute("context_tool", serde_json::json!({}), Some(ctx))
            .await
            .unwrap();
        assert!(result.content.contains("session: test_session"));
        assert!(result.content.contains("agent: test_agent"));
    }

    #[tokio::test]
    async fn test_execute_returns_error_on_tool_failure() {
        #[derive(Clone)]
        struct FailingTool;
        #[async_trait]
        impl Tool for FailingTool {
            fn name(&self) -> &str {
                "failing_tool"
            }
            fn description(&self) -> &str {
                "Failing tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::err("intentional error"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(FailingTool).await;

        let result = registry
            .execute("failing_tool", serde_json::json!({}), None)
            .await
            .unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("intentional error"));
    }

    #[tokio::test]
    async fn test_is_disabled_after_set_disabled() {
        let mut registry = ToolRegistry::new();
        assert!(!registry.is_disabled("any_tool"));

        registry.set_disabled(HashSet::from(["tool_a".to_string(), "tool_b".to_string()]));

        assert!(registry.is_disabled("tool_a"));
        assert!(registry.is_disabled("tool_b"));
        assert!(!registry.is_disabled("tool_c"));
    }

    #[tokio::test]
    async fn test_disabled_tool_still_registered_but_not_executable() {
        #[derive(Clone)]
        struct DisabledTestTool;
        #[async_trait]
        impl Tool for DisabledTestTool {
            fn name(&self) -> &str {
                "disabled_test_tool"
            }
            fn description(&self) -> &str {
                "Disabled test tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("should not reach"))
            }
        }

        let mut registry = ToolRegistry::new();
        registry.register(DisabledTestTool).await;
        registry.set_disabled(HashSet::from(["disabled_test_tool".to_string()]));

        let tool = registry.get("disabled_test_tool").await;
        assert!(tool.is_some(), "Tool should still be registered");

        let result = registry
            .execute("disabled_test_tool", serde_json::json!({}), None)
            .await;
        assert!(result.is_err());

        let list = registry.list_filtered(None).await;
        let entry = list
            .iter()
            .find(|(n, _, _)| n == "disabled_test_tool")
            .unwrap();
        assert!(entry.2, "Tool should be marked as disabled in listing");
    }

    #[tokio::test]
    async fn test_tool_execution_with_complex_args() {
        #[derive(Clone)]
        struct ComplexArgsTool;
        #[async_trait]
        impl Tool for ComplexArgsTool {
            fn name(&self) -> &str {
                "complex_args_tool"
            }
            fn description(&self) -> &str {
                "Complex args tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                args: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                let output = format!(
                    "str: {}, int: {}, float: {}, bool: {}, array: {:?}",
                    args.get("string")
                        .and_then(|v| v.as_str())
                        .unwrap_or("missing"),
                    args.get("integer").and_then(|v| v.as_i64()).unwrap_or(0),
                    args.get("float").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    args.get("boolean")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    args.get("array")
                        .and_then(|v| v.as_array().map(|a| a.len()))
                        .unwrap_or(0),
                );
                Ok(ToolResult::ok(output))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(ComplexArgsTool).await;

        let args = serde_json::json!({
            "string": "hello",
            "integer": 42,
            "float": 3.14,
            "boolean": true,
            "array": [1, 2, 3]
        });

        let result = registry
            .execute("complex_args_tool", args, None)
            .await
            .unwrap();
        assert!(result.success);
        assert!(result.content.contains("str: hello"));
        assert!(result.content.contains("int: 42"));
        assert!(result.content.contains("float: 3.14"));
        assert!(result.content.contains("bool: true"));
        assert!(result.content.contains("array: 3"));
    }

    #[tokio::test]
    async fn test_registry_clone_is_independent() {
        #[derive(Clone)]
        struct CloneTestTool;
        #[async_trait]
        impl Tool for CloneTestTool {
            fn name(&self) -> &str {
                "clone_test_tool"
            }
            fn description(&self) -> &str {
                "Clone test tool"
            }
            fn clone_tool(&self) -> Box<dyn Tool> {
                Box::new(self.clone())
            }
            async fn execute(
                &self,
                _: serde_json::Value,
                _: Option<ToolContext>,
            ) -> Result<ToolResult, OpenCodeError> {
                Ok(ToolResult::ok("clone_test"))
            }
        }

        let registry = ToolRegistry::new();
        registry.register(CloneTestTool).await;

        let tools1 = registry.list_filtered(None).await;
        assert_eq!(tools1.len(), 1);

        let registry2 = ToolRegistry::new();
        let tools2 = registry2.list_filtered(None).await;
        assert_eq!(tools2.len(), 0, "Cloned registry should be independent");
    }
}
