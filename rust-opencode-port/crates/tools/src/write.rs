use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct WriteTool;

#[derive(Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write files"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(WriteTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: WriteArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OpenCodeError::Io(e))?;
        }

        std::fs::write(&path, &args.content)
            .map_err(|e| OpenCodeError::Io(e))?;

        Ok(ToolResult::ok(format!("Written to {}", args.path)))
    }
}
