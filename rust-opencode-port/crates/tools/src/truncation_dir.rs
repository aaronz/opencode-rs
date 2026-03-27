use async_trait::async_trait;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct TruncationDirArgs {
    pub path: String,
    pub max_files: Option<usize>,
}

pub struct TruncationDirTool;

#[async_trait]
impl Tool for TruncationDirTool {
    fn name(&self) -> &str {
        "truncation_dir"
    }

    fn description(&self) -> &str {
        "List files in a directory with truncation for large directory sets"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(TruncationDirTool)
    }

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: TruncationDirArgs = serde_json::from_value(args).map_err(|e| OpenCodeError::Parse(e.to_string()))?;
        let max_files = args.max_files.unwrap_or(50);
        let path = Path::new(&args.path);

        if !path.exists() {
            return Err(OpenCodeError::Tool(format!("Directory not found: {}", args.path)));
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(path).map_err(|e| OpenCodeError::Io(e))? {
            let entry = entry.map_err(|e| OpenCodeError::Io(e))?;
            entries.push(entry.file_name().to_string_lossy().to_string());
        }

        let total = entries.len();
        let mut result = format!("Total files: {}\n", total);
        
        if total > max_files {
            entries.truncate(max_files);
            for name in entries {
                result.push_str(&format!("- {}\n", name));
            }
            result.push_str("... (truncated)");
        } else {
            for name in entries {
                result.push_str(&format!("- {}\n", name));
            }
        }

        Ok(ToolResult::ok(result))
    }
}
