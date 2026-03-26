use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct TruncateTool {
    max_lines: usize,
    max_bytes: usize,
}

#[derive(Deserialize)]
struct TruncateArgs {
    content: String,
    max_lines: Option<usize>,
    max_bytes: Option<usize>,
}

impl TruncateTool {
    pub fn new() -> Self {
        Self {
            max_lines: 2000,
            max_bytes: 51200,
        }
    }
}

impl Default for TruncateTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TruncateTool {
    fn name(&self) -> &str {
        "truncate"
    }

    fn description(&self) -> &str {
        "Truncate large outputs"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(TruncateTool::new())
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: TruncateArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let max_lines = args.max_lines.unwrap_or(self.max_lines);
        let max_bytes = args.max_bytes.unwrap_or(self.max_bytes);

        let lines: Vec<&str> = args.content.lines().take(max_lines).collect();
        let mut result = lines.join("\n");

        if result.len() > max_bytes {
            result.truncate(max_bytes);
            result.push_str("\n\n...(truncated)");
        }

        if args.content.lines().count() > max_lines {
            result.push_str(&format!(
                "\n\n... ({} more lines)",
                args.content.lines().count() - max_lines
            ));
        }

        Ok(ToolResult::ok(result))
    }
}
