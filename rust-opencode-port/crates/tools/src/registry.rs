use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::Tool;
use opencode_core::OpenCodeError;

pub struct ToolCall {
    pub name: String,
    pub args: serde_json::Value,
}

pub struct ToolCallResult {
    pub name: String,
    pub result: Result<crate::ToolResult, OpenCodeError>,
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

    pub async fn list(&self) -> Vec<(String, String)> {
        let tools = self.tools.read().await;
        tools
            .iter()
            .map(|(name, tool)| (name.clone(), tool.description().to_string()))
            .collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<crate::ToolResult, OpenCodeError> {
        let tool = self.get(name).await
            .ok_or_else(|| OpenCodeError::Tool(format!("Tool '{}' not found", name)))?;
        tool.execute(args).await
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

            handles.push(tokio::spawn(async move {
                let tool = {
                    let tools = registry.read().await;
                    tools.get(&name).map(|t| t.clone_tool())
                };

                let result = match tool {
                    Some(t) => t.execute(args).await,
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
