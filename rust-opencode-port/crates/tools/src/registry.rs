use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Tool, ToolContext, ToolResult};
use opencode_core::OpenCodeError;

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
        model_id.starts_with("gpt-") 
            && !model_id.contains("gpt-4") 
            && !model_id.contains("oss")
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
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub async fn get(&self, name: &str) -> Option<Box<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).map(|t| t.clone_tool())
    }

    pub async fn list_filtered(&self, model: Option<&ModelInfo>) -> Vec<(String, String)> {
        let tools = self.tools.read().await;
        
        let Some(model_info) = model else {
            return tools.iter()
                .map(|(name, tool)| (name.clone(), tool.description().to_string()))
                .collect();
        };

        tools.iter()
            .filter(|(name, _)| {
                match name.as_str() {
                    "codesearch" | "websearch" => model_info.provider_id.is_opencode(),
                    "apply_patch" => model_info.use_apply_patch(),
                    "edit" | "write" => !model_info.use_apply_patch(),
                    _ => true,
                }
            })
            .map(|(name, tool)| (name.clone(), tool.description().to_string()))
            .collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let tool = self.get(name).await
            .ok_or_else(|| OpenCodeError::Tool(format!("Tool '{}' not found", name)))?;
        tool.execute(args, ctx).await
    }

    pub async fn execute_parallel(
        &self,
        calls: Vec<ToolCall>,
    ) -> Vec<ToolCallResult> {
        let mut handles = Vec::new();

        for call in calls {
            let registry = Arc::clone(&self.tools);
            let name = call.name.clone();
            let args = call.args;
            let ctx = call.ctx;

            handles.push(tokio::spawn(async move {
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
