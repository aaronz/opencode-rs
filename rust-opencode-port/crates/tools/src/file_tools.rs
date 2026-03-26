use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct FileReadTool;

#[derive(Deserialize)]
struct ReadArgs {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read file contents"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(FileReadTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: ReadArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if !path.exists() {
            return Ok(ToolResult::err(format!("File not found: {}", args.path)));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| OpenCodeError::Io(e))?;

        let lines: Vec<&str> = content.lines().collect();
        let offset = args.offset.unwrap_or(0);
        let limit = args.limit.unwrap_or(lines.len());

        let selected: String = lines
            .iter()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(i, l)| format!("{}: {}", offset + i + 1, l))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult::ok(selected))
    }
}

pub struct FileWriteTool;

#[derive(Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to file"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(FileWriteTool)
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

pub struct GlobTool;

#[derive(Deserialize)]
struct GlobArgs {
    pattern: String,
    root: Option<String>,
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching pattern"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GlobTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: GlobArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let root = args.root.unwrap_or_else(|| ".".to_string());
        let pattern = glob::Pattern::new(&args.pattern)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let mut matches = Vec::new();
        for entry in walkdir::WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(p) = path.to_str() {
                if pattern.matches(p) || pattern.matches(&path.file_name().unwrap_or_default().to_string_lossy()) {
                    matches.push(p.to_string());
                }
            }
        }

        Ok(ToolResult::ok(matches.join("\n")))
    }
}
