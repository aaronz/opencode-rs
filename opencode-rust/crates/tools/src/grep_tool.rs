use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct GrepTool;

#[derive(Deserialize)]
struct GrepArgs {
    pattern: String,
    path: Option<String>,
    file_type: Option<String>,
    count: Option<bool>,
}

impl sealed::Sealed for GrepTool {}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using regex"
    }

    fn input_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Path to search in (optional, defaults to current directory)"
                },
                "file_type": {
                    "type": "string",
                    "description": "Filter by file type/extension (optional, e.g., 'rs', 'py')"
                },
                "count": {
                    "type": "boolean",
                    "description": "Only show count of matches per file (optional)"
                }
            },
            "required": ["pattern"]
        }))
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GrepTool)
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
        let args: GrepArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let regex = Regex::new(&args.pattern).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = args.path.unwrap_or_else(|| ".".to_string());
        let count_only = args.count.unwrap_or(false);

        let mut results = Vec::new();
        let entries = walkdir::WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in entries {
            let file_path = entry.path();

            if let Some(ref ft) = args.file_type {
                if let Some(ext) = file_path.extension() {
                    if ext.to_string_lossy() != *ft {
                        continue;
                    }
                }
            }

            if let Ok(content) = std::fs::read_to_string(file_path) {
                let mut file_matches = Vec::new();
                for (line_num, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        file_matches.push(format!(
                            "{}:{}: {}",
                            file_path.display(),
                            line_num + 1,
                            line
                        ));
                    }
                }

                if !file_matches.is_empty() {
                    if count_only {
                        results.push(format!(
                            "{}:{} matches",
                            file_path.display(),
                            file_matches.len()
                        ));
                    } else {
                        results.extend(file_matches);
                    }
                }
            }
        }

        if results.is_empty() {
            return Ok(ToolResult::ok("No matches found".to_string()));
        }

        Ok(ToolResult::ok(results.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_grep_tool_name() {
        let tool = GrepTool;
        assert_eq!(tool.name(), "grep");
    }

    #[tokio::test]
    async fn test_grep_tool_description() {
        let tool = GrepTool;
        assert_eq!(tool.description(), "Search file contents using regex");
    }

    #[tokio::test]
    async fn test_grep_tool_clone() {
        let tool = GrepTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "grep");
    }

    #[tokio::test]
    async fn test_grep_is_safe() {
        let tool = GrepTool;
        assert!(tool.is_safe());
    }

    #[tokio::test]
    async fn test_grep_finds_match() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "line1\nhello world\nline3").unwrap();

        let tool = GrepTool;
        let args = serde_json::json!({
            "pattern": "hello",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_grep_no_matches() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "line1\nhello world\nline3").unwrap();

        let tool = GrepTool;
        let args = serde_json::json!({
            "pattern": "goodbye",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert_eq!(result.content, "No matches found");
    }

    #[tokio::test]
    async fn test_grep_with_file_type() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
        std::fs::write(dir.path().join("test.md"), "hello").unwrap();

        let tool = GrepTool;
        let args = serde_json::json!({
            "pattern": "hello",
            "path": dir.path().to_str().unwrap(),
            "file_type": "txt"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("test.txt"));
        assert!(!result.content.contains("test.md"));
    }

    #[tokio::test]
    async fn test_grep_count_only() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("test.txt"), "hello\nhello\nhello").unwrap();

        let tool = GrepTool;
        let args = serde_json::json!({
            "pattern": "hello",
            "path": dir.path().to_str().unwrap(),
            "count": true
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("3 matches"));
    }

    #[tokio::test]
    async fn test_grep_default_path() {
        let tool = GrepTool;
        let args = serde_json::json!({"pattern": "test"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() {
        let tool = GrepTool;
        let args = serde_json::json!({
            "pattern": "[invalid",
            "path": "."
        });
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_grep_get_dependencies() {
        let tool = GrepTool;
        let args = serde_json::json!({"path": "/some/path", "pattern": "test"});
        let deps = tool.get_dependencies(&args);
        assert!(deps.contains(&std::path::PathBuf::from("/some/path")));
    }

    #[tokio::test]
    async fn test_grep_get_dependencies_default() {
        let tool = GrepTool;
        let args = serde_json::json!({"pattern": "test"});
        let deps = tool.get_dependencies(&args);
        assert!(deps.contains(&std::path::PathBuf::from(".")));
    }

    #[tokio::test]
    async fn test_grep_nonexistent_path() {
        let tool = GrepTool;
        let args = serde_json::json!({
            "pattern": "test",
            "path": "/nonexistent/path"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }
}
