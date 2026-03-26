use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct MultieditTool;

#[derive(Deserialize)]
struct MultieditArgs {
    path: String,
    edits: Vec<EditOperation>,
}

#[derive(Deserialize)]
struct EditOperation {
    old_string: String,
    new_string: String,
}

#[async_trait]
impl Tool for MultieditTool {
    fn name(&self) -> &str {
        "multiedit"
    }

    fn description(&self) -> &str {
        "Multiple edits to same file"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(MultieditTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: MultieditArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if !path.exists() {
            return Ok(ToolResult::err(format!("File not found: {}", args.path)));
        }

        let mut content = std::fs::read_to_string(&path)
            .map_err(|e| OpenCodeError::Io(e))?;

        let mut edit_count = 0;
        let mut errors = Vec::new();

        for edit in &args.edits {
            let count = content.matches(&edit.old_string).count();
            match count {
                0 => {
                    errors.push(format!("String '{}' not found", edit.old_string));
                }
                1 => {
                    content = content.replace(&edit.old_string, &edit.new_string);
                    edit_count += 1;
                }
                n => {
                    errors.push(format!(
                        "String '{}' matches {} locations",
                        edit.old_string, n
                    ));
                }
            }
        }

        if !errors.is_empty() {
            return Ok(ToolResult::err(errors.join("\n")));
        }

        std::fs::write(&path, &content)
            .map_err(|e| OpenCodeError::Io(e))?;

        Ok(ToolResult::ok(format!(
            "Applied {} edits to {}",
            edit_count, args.path
        )))
    }
}
