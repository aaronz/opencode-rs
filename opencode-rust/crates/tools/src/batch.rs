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
