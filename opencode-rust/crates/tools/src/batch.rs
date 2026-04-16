use crate::sealed;
use crate::{Tool, ToolRegistry, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BatchTool {
    registry: Arc<RwLock<ToolRegistry>>,
}

impl BatchTool {
    pub fn new(registry: Arc<RwLock<ToolRegistry>>) -> Self {
        Self { registry }
    }
}

#[derive(Deserialize)]
struct BatchArgs {
    invocations: Vec<ToolInvocation>,
}

#[derive(Deserialize)]
struct ToolInvocation {
    tool_name: String,
    input: serde_json::Value,
}

impl sealed::Sealed for BatchTool {}

#[async_trait]
impl Tool for BatchTool {
    fn name(&self) -> &str {
        "batch"
    }

    fn description(&self) -> &str {
        "Execute multiple tools in parallel"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(Self {
            registry: Arc::clone(&self.registry),
        })
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: BatchArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let mut handles = Vec::new();

        for invocation in args.invocations {
            let registry = Arc::clone(&self.registry);
            let ctx = ctx.clone();
            handles.push(tokio::spawn(async move {
                let tool = {
                    let reg = registry.read().await;
                    reg.get(&invocation.tool_name).await
                };
                match tool {
                    Some(t) => t.execute(invocation.input, ctx).await,
                    None => Err(OpenCodeError::Tool(format!(
                        "Tool '{}' not found",
                        invocation.tool_name
                    ))),
                }
            }));
        }

        let mut results = Vec::new();
        let mut errors = Vec::new();

        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => errors.push(format!("{}: {}", "unknown", e)),
                Err(e) => errors.push(format!("unknown: {}", e)),
            }
        }

        if !errors.is_empty() {
            return Ok(ToolResult::err(format!(
                "Some tools failed: {}",
                errors.join(", ")
            )));
        }

        let combined = results
            .iter()
            .map(|r| r.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(ToolResult::ok(combined))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Tool, ToolRegistry};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_batch_tool_name() {
        let registry = Arc::new(RwLock::new(ToolRegistry::default()));
        let tool = BatchTool::new(registry);
        assert_eq!(tool.name(), "batch");
    }

    #[tokio::test]
    async fn test_batch_tool_description() {
        let registry = Arc::new(RwLock::new(ToolRegistry::default()));
        let tool = BatchTool::new(registry);
        assert_eq!(tool.description(), "Execute multiple tools in parallel");
    }

    #[tokio::test]
    async fn test_batch_tool_clone() {
        let registry = Arc::new(RwLock::new(ToolRegistry::default()));
        let tool = BatchTool::new(registry.clone());
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "batch");
    }

    #[tokio::test]
    async fn test_batch_empty_invocations() {
        let registry = Arc::new(RwLock::new(ToolRegistry::default()));
        let tool = BatchTool::new(registry);
        let args = serde_json::json!({"invocations": []});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "");
    }

    #[tokio::test]
    async fn test_batch_nonexistent_tool() {
        let registry = Arc::new(RwLock::new(ToolRegistry::default()));
        let tool = BatchTool::new(registry);
        let args = serde_json::json!({
            "invocations": [{
                "tool_name": "nonexistent_tool",
                "input": {}
            }]
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(!result.success, "Expected failure, got success=true");
        let error_msg = result.error.unwrap_or_default();
        assert!(
            error_msg.contains("nonexistent_tool") || error_msg.contains("not found"),
            "Expected error to contain 'nonexistent_tool' or 'not found', got: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_batch_invalid_args() {
        let registry = Arc::new(RwLock::new(ToolRegistry::default()));
        let tool = BatchTool::new(registry);
        let args = serde_json::json!({"not_invocations": []});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }
}
