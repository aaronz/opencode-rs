use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::path::PathBuf;

pub struct ReadTool {
    max_lines: usize,
    _max_bytes: usize,
}

#[derive(Deserialize)]
struct ReadArgs {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

impl ReadTool {
    pub fn new() -> Self {
        Self {
            max_lines: 2000,
            _max_bytes: 51200,
        }
    }
}

impl Default for ReadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read file with line numbers"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(ReadTool::new())
    }

    fn is_safe(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: ReadArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if !path.exists() {
            return Ok(ToolResult::err(format!("File not found: {}", args.path)));
        }

        let content = std::fs::read_to_string(&path).map_err(|e| OpenCodeError::Io(e))?;

        let lines: Vec<&str> = content.lines().collect();
        let offset = args.offset.unwrap_or(0);
        let limit = args.limit.unwrap_or(self.max_lines);

        let end = (offset + limit).min(lines.len());
        let selected_lines: Vec<_> = lines[offset..end].to_vec();

        let mut result = String::new();
        for (i, line) in selected_lines.iter().enumerate() {
            result.push_str(&format!("{}: {}\n", offset + i + 1, line));
        }

        if end < lines.len() {
            result.push_str(&format!("\n... ({} more lines)\n", lines.len() - end));
        }

        Ok(ToolResult::ok(result))
    }
}
