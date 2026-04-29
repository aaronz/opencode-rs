use std::sync::Arc;

use opencode_core::OpenCodeError;
use opencode_tools::{Tool, ToolContext, ToolRegistry};
use tokio::sync::RwLock;

pub struct RuntimeToolRouter {
    registry: Arc<RwLock<ToolRegistry>>,
}

impl RuntimeToolRouter {
    pub fn new(registry: ToolRegistry) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }

    pub async fn execute(
        &self,
        name: &str,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<opencode_tools::ToolResult, OpenCodeError> {
        self.registry.read().await.execute(name, args, ctx).await
    }

    pub async fn get(&self, name: &str) -> Option<Box<dyn Tool>> {
        self.registry.read().await.get(name).await
    }

    pub async fn list(&self) -> Vec<(String, String, bool)> {
        self.registry.read().await.list_filtered(None).await
    }

    pub async fn register<T: Tool + 'static>(&self, tool: T) {
        self.registry.write().await.register(tool).await;
    }

    pub async fn unregister(&self, name: &str) -> bool {
        self.registry.write().await.unregister(name).await
    }

    pub async fn set_disabled(&self, tools: std::collections::HashSet<String>) {
        self.registry.write().await.set_disabled(tools);
    }

    pub fn registry(&self) -> Arc<RwLock<ToolRegistry>> {
        Arc::clone(&self.registry)
    }
}

impl Clone for RuntimeToolRouter {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
        }
    }
}

impl Default for RuntimeToolRouter {
    fn default() -> Self {
        Self::new(ToolRegistry::new())
    }
}
