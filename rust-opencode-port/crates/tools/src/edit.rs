use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct EditTool;

#[derive(Deserialize)]
struct EditArgs {
    path: String,
    old_string: String,
    new_string: String,
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Edit files with exact string matching"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(EditTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: EditArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if !path.exists() {
            return Ok(ToolResult::err(format!("File not found: {}", args.path)));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| OpenCodeError::Io(e))?;

        let matches: Vec<_> = content.match_indices(&args.old_string).collect();

        match matches.len() {
            0 => {
                Ok(ToolResult::err(format!(
                    "String '{}' not found in file {}",
                    args.old_string, args.path
                )))
            }
            1 => {
                let new_content = content.replace(&args.old_string, &args.new_string);
                std::fs::write(&path, &new_content)
                    .map_err(|e| OpenCodeError::Io(e))?;

                Ok(ToolResult::ok(format!(
                    "Successfully replaced '{}' with '{}' in {}",
                    args.old_string, args.new_string, args.path
                )))
            }
            n => {
                Ok(ToolResult::err(format!(
                    "String '{}' matches {} locations in {}. Please provide more context to disambiguate.",
                    args.old_string, n, args.path
                )))
            }
        }
    }
}
