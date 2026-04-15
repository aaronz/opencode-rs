#![allow(clippy::redundant_closure)]

use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct FileReadTool;

#[derive(Deserialize)]
struct ReadArgs {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

impl sealed::Sealed for FileReadTool {}

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

    fn is_safe(&self) -> bool {
        true
    }

    fn get_dependencies(&self, args: &serde_json::Value) -> HashSet<PathBuf> {
        let mut deps = HashSet::new();
        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
            deps.insert(PathBuf::from(path));
        }
        deps
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

impl sealed::Sealed for FileWriteTool {}

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

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: WriteArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OpenCodeError::Io(e))?;
        }

        std::fs::write(&path, &args.content).map_err(|e| OpenCodeError::Io(e))?;

        Ok(ToolResult::ok(format!("Written to {}", args.path)))
    }
}

pub struct GlobTool;

#[derive(Deserialize)]
struct GlobArgs {
    pattern: String,
    root: Option<String>,
}

impl sealed::Sealed for GlobTool {}

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

    fn is_safe(&self) -> bool {
        true
    }

    fn get_dependencies(&self, args: &serde_json::Value) -> HashSet<PathBuf> {
        let mut deps = HashSet::new();
        if let Some(root) = args.get("root").and_then(|v| v.as_str()) {
            deps.insert(PathBuf::from(root));
        } else {
            deps.insert(PathBuf::from("."));
        }
        deps
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: GlobArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let root = args.root.unwrap_or_else(|| ".".to_string());
        let pattern =
            glob::Pattern::new(&args.pattern).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let mut matches = Vec::new();
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(p) = path.to_str() {
                if pattern.matches(p)
                    || pattern.matches(&path.file_name().unwrap_or_default().to_string_lossy())
                {
                    matches.push(p.to_string());
                }
            }
        }

        Ok(ToolResult::ok(matches.join("\n")))
    }
}

pub struct StatTool;

#[derive(Deserialize)]
struct StatArgs {
    path: String,
}

impl sealed::Sealed for StatTool {}

#[async_trait]
impl Tool for StatTool {
    fn name(&self) -> &str {
        "file_stat"
    }

    fn description(&self) -> &str {
        "Get file metadata (size, timestamps, permissions)"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(StatTool)
    }

    fn is_safe(&self) -> bool {
        true
    }

    fn get_dependencies(&self, args: &serde_json::Value) -> HashSet<PathBuf> {
        let mut deps = HashSet::new();
        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
            deps.insert(PathBuf::from(path));
        }
        deps
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: StatArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if !path.exists() {
            return Ok(ToolResult::err(format!("Path not found: {}", args.path)));
        }

        let metadata = std::fs::metadata(&path).map_err(|e| OpenCodeError::Io(e))?;

        let file_type = if metadata.is_dir() {
            "directory"
        } else if metadata.is_symlink() {
            "symlink"
        } else {
            "regular file"
        };

        let modified = metadata
            .modified()
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|_| "unknown".to_string());

        let created = metadata
            .created()
            .map(|t| format!("{:?}", t))
            .unwrap_or_else(|_| "unknown".to_string());

        let permissions = metadata.permissions();
        let readonly = permissions.readonly();

        let output = format!(
            "File: {}\nType: {}\nSize: {} bytes\nModified: {}\nCreated: {}\nRead-only: {}",
            args.path,
            file_type,
            metadata.len(),
            modified,
            created,
            readonly
        );

        Ok(ToolResult::ok(output))
    }
}

pub struct FileMoveTool;

#[derive(Deserialize)]
struct MoveArgs {
    source: String,
    destination: String,
}

impl sealed::Sealed for FileMoveTool {}

#[async_trait]
impl Tool for FileMoveTool {
    fn name(&self) -> &str {
        "file_move"
    }

    fn description(&self) -> &str {
        "Move/rename a file or directory"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(FileMoveTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: MoveArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let source = PathBuf::from(&args.source);
        let destination = PathBuf::from(&args.destination);

        if !source.exists() {
            return Ok(ToolResult::err(format!(
                "Source not found: {}",
                args.source
            )));
        }

        if destination.exists() {
            return Ok(ToolResult::err(format!(
                "Destination already exists: {}",
                args.destination
            )));
        }

        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OpenCodeError::Io(e))?;
        }

        std::fs::rename(&source, &destination).map_err(|e| OpenCodeError::Io(e))?;

        Ok(ToolResult::ok(format!(
            "Moved {} to {}",
            args.source, args.destination
        )))
    }
}

pub struct FileDeleteTool;

#[derive(Deserialize)]
struct DeleteArgs {
    path: String,
    #[serde(default)]
    recursive: bool,
}

impl sealed::Sealed for FileDeleteTool {}

#[async_trait]
impl Tool for FileDeleteTool {
    fn name(&self) -> &str {
        "file_delete"
    }

    fn description(&self) -> &str {
        "Delete a file or directory"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(FileDeleteTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: DeleteArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        if !path.exists() {
            return Ok(ToolResult::err(format!("Path not found: {}", args.path)));
        }

        if path.is_dir() {
            if args.recursive {
                std::fs::remove_dir_all(&path).map_err(|e| OpenCodeError::Io(e))?;
            } else {
                std::fs::remove_dir(&path).map_err(|e| OpenCodeError::Io(e))?;
            }
        } else {
            std::fs::remove_file(&path).map_err(|e| OpenCodeError::Io(e))?;
        }

        Ok(ToolResult::ok(format!("Deleted {}", args.path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_stat_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello").unwrap();

        let tool = StatTool;
        let args = serde_json::json!({"path": file_path.to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("regular file"));
        assert!(result.content.contains("5 bytes"));
    }

    #[tokio::test]
    async fn test_stat_directory() {
        let dir = tempdir().unwrap();
        let tool = StatTool;
        let args = serde_json::json!({"path": dir.path().to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("directory"));
    }

    #[tokio::test]
    async fn test_stat_not_found() {
        let tool = StatTool;
        let args = serde_json::json!({"path": "/nonexistent/path"});
        let result = tool.execute(args, None).await.unwrap();

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Path not found"));
    }

    #[tokio::test]
    async fn test_move_file() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "content").unwrap();

        let tool = FileMoveTool;
        let args = serde_json::json!({
            "source": source.to_str().unwrap(),
            "destination": dest.to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();

        assert!(result.success);
        assert!(!source.exists());
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "content");
    }

    #[tokio::test]
    async fn test_move_source_not_found() {
        let dir = tempdir().unwrap();
        let tool = FileMoveTool;
        let args = serde_json::json!({
            "source": "/nonexistent/source.txt",
            "destination": dir.path().join("dest.txt").to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Source not found"));
    }

    #[tokio::test]
    async fn test_move_destination_exists() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let dest = dir.path().join("dest.txt");
        std::fs::write(&source, "source").unwrap();
        std::fs::write(&dest, "dest").unwrap();

        let tool = FileMoveTool;
        let args = serde_json::json!({
            "source": source.to_str().unwrap(),
            "destination": dest.to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Destination already exists"));
    }

    #[tokio::test]
    async fn test_delete_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        let tool = FileDeleteTool;
        let args = serde_json::json!({"path": file_path.to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();

        assert!(result.success);
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_delete_empty_directory() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        let tool = FileDeleteTool;
        let args = serde_json::json!({"path": subdir.to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();

        assert!(result.success);
        assert!(!subdir.exists());
    }

    #[tokio::test]
    async fn test_delete_directory_recursive() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("file.txt"), "content").unwrap();

        let tool = FileDeleteTool;
        let args = serde_json::json!({
            "path": subdir.to_str().unwrap(),
            "recursive": true
        });
        let result = tool.execute(args, None).await.unwrap();

        assert!(result.success);
        assert!(!subdir.exists());
    }

    #[tokio::test]
    async fn test_delete_directory_not_empty_fails() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("file.txt"), "content").unwrap();

        let tool = FileDeleteTool;
        let args = serde_json::json!({"path": subdir.to_str().unwrap()});
        let result = tool.execute(args, None).await;

        assert!(result.is_err() || !result.unwrap().success);
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let tool = FileDeleteTool;
        let args = serde_json::json!({"path": "/nonexistent/path"});
        let result = tool.execute(args, None).await.unwrap();

        assert!(!result.success);
        assert!(result.error.unwrap().contains("Path not found"));
    }
}
