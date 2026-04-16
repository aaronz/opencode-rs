#![allow(clippy::redundant_closure)]

use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct LsTool;

#[derive(Deserialize)]
struct LsArgs {
    path: Option<String>,
    pattern: Option<String>,
}

impl sealed::Sealed for LsTool {}

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

    fn is_safe(&self) -> bool {
        true
    }

    fn get_dependencies(&self, args: &serde_json::Value) -> HashSet<PathBuf> {
        let mut deps = HashSet::new();
        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
            deps.insert(PathBuf::from(path));
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
        let args: LsArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(args.path.unwrap_or_else(|| ".".to_string()));

        if !path.exists() {
            return Ok(ToolResult::err(format!(
                "Directory not found: {}",
                path.display()
            )));
        }

        if !path.is_dir() {
            return Ok(ToolResult::err(format!(
                "Not a directory: {}",
                path.display()
            )));
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ls_tool_name() {
        let tool = LsTool;
        assert_eq!(tool.name(), "ls");
    }

    #[tokio::test]
    async fn test_ls_tool_description() {
        let tool = LsTool;
        assert_eq!(tool.description(), "List directory contents");
    }

    #[tokio::test]
    async fn test_ls_tool_clone() {
        let tool = LsTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "ls");
    }

    #[tokio::test]
    async fn test_ls_is_safe() {
        let tool = LsTool;
        assert!(tool.is_safe());
    }

    #[tokio::test]
    async fn test_ls_empty_directory() {
        let dir = tempdir().unwrap();
        let tool = LsTool;
        let args = serde_json::json!({"path": dir.path().to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert_eq!(result.content.trim(), "");
    }

    #[tokio::test]
    async fn test_ls_with_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();
        std::fs::write(dir.path().join("b.txt"), "content").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();

        let tool = LsTool;
        let args = serde_json::json!({"path": dir.path().to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("a.txt"));
        assert!(result.content.contains("b.txt"));
        assert!(result.content.contains("subdir/"));
    }

    #[tokio::test]
    async fn test_ls_nonexistent_path() {
        let tool = LsTool;
        let args = serde_json::json!({"path": "/nonexistent/path"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Directory not found"));
    }

    #[tokio::test]
    async fn test_ls_not_a_directory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, "content").unwrap();

        let tool = LsTool;
        let args = serde_json::json!({"path": file_path.to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Not a directory"));
    }

    #[tokio::test]
    async fn test_ls_with_pattern() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();
        std::fs::write(dir.path().join("b.md"), "content").unwrap();

        let tool = LsTool;
        let args = serde_json::json!({
            "path": dir.path().to_str().unwrap(),
            "pattern": "*.txt"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("a.txt"));
        assert!(!result.content.contains("b.md"));
    }

    #[tokio::test]
    async fn test_ls_with_invalid_pattern() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();

        let tool = LsTool;
        let args = serde_json::json!({
            "path": dir.path().to_str().unwrap(),
            "pattern": "[invalid"
        });
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ls_default_path() {
        let tool = LsTool;
        let args = serde_json::json!({});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_ls_get_dependencies() {
        let tool = LsTool;
        let args = serde_json::json!({"path": "/some/path"});
        let deps = tool.get_dependencies(&args);
        assert!(deps.contains(&std::path::PathBuf::from("/some/path")));
    }

    #[tokio::test]
    async fn test_ls_get_dependencies_default() {
        let tool = LsTool;
        let args = serde_json::json!({});
        let deps = tool.get_dependencies(&args);
        assert!(deps.contains(&std::path::PathBuf::from(".")));
    }
}
