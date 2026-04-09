use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct LsTool;

#[derive(Deserialize)]
struct LsArgs {
    path: Option<String>,
    pattern: Option<String>,
}

#[async_trait]
impl Tool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "List directory contents"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(LsTool)
    }

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: LsArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(args.path.unwrap_or_else(|| ".".to_string()));

        if !path.exists() {
            return Ok(ToolResult::err(format!("Directory not found: {}", path.display())));
        }

        if !path.is_dir() {
            return Ok(ToolResult::err(format!("Not a directory: {}", path.display())));
        }

        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&path).map_err(|e| OpenCodeError::Io(e))? {
            let entry = entry.map_err(|e| OpenCodeError::Io(e))?;
            let name = entry.file_name().to_string_lossy().to_string();

            if let Some(ref pattern) = args.pattern {
                if !glob::Pattern::new(pattern)
                    .map_err(|e| OpenCodeError::Tool(e.to_string()))?
                    .matches(&name)
                {
                    continue;
                }
            }

            let file_type = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                "/"
            } else {
                ""
            };

            entries.push(format!("{}{}", name, file_type));
        }

        entries.sort();

        Ok(ToolResult::ok(entries.join("\n")))
    }
}
