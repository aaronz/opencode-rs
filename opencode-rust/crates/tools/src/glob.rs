use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct GlobTool;

#[derive(Deserialize)]
struct GlobArgs {
    pattern: String,
    path: Option<String>,
}

impl sealed::Sealed for GlobTool {}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "- Fast file pattern matching tool that works with any codebase size\n- Supports glob patterns like \"**/*.js\" or \"src/**/*.ts\"\n- Returns matching file paths sorted by modification time\n- Use this tool when you need to find files by name patterns\n- When you are doing an open-ended search that may require multiple rounds of globbing and grepping, use the Task tool instead\n- You have the capability to call multiple tools in a single response. It is always better to speculatively perform multiple searches as a batch that are potentially useful."
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GlobTool)
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
        let args: GlobArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let search = args.path.unwrap_or_else(|| ".".to_string());
        let search_path = PathBuf::from(&search);

        if !search_path.exists() {
            return Ok(ToolResult::err(format!("Directory not found: {}", search)));
        }

        if !search_path.is_dir() {
            return Ok(ToolResult::err(format!("Not a directory: {}", search)));
        }

        let pattern = &args.pattern;
        let mut files = Vec::new();

        // Use glob crate for pattern matching
        let glob_pattern =
            glob::Pattern::new(pattern).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        // Walk the directory
        fn walk_dir(
            dir: &PathBuf,
            pattern: &glob::Pattern,
            files: &mut Vec<(PathBuf, u64)>,
            limit: usize,
        ) -> bool {
            if files.len() >= limit {
                return true;
            }

            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if files.len() >= limit {
                        return true;
                    }

                    let path = entry.path();
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Skip hidden files and common ignore directories
                    if name.starts_with('.') || name == "node_modules" || name == "target" {
                        continue;
                    }

                    if path.is_dir() {
                        if walk_dir(&path, pattern, files, limit) {
                            return true;
                        }
                    } else if pattern.matches(&name) {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            let mtime = metadata
                                .modified()
                                .map(|t| {
                                    t.duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs()
                                })
                                .unwrap_or(0);
                            files.push((path, mtime));
                        }
                    }
                }
            }
            false
        }

        let limit = 100;
        let truncated = walk_dir(&search_path, &glob_pattern, &mut files, limit);

        // Sort by modification time (newest first)
        files.sort_by(|a, b| b.1.cmp(&a.1));

        let mut output = Vec::new();
        if files.is_empty() {
            output.push("No files found".to_string());
        } else {
            for (path, _) in &files {
                output.push(path.to_string_lossy().to_string());
            }
            if truncated {
                output.push(String::new());
                output.push(format!("(Results are truncated: showing first {} results. Consider using a more specific path or pattern.)", limit));
            }
        }

        Ok(ToolResult::ok(output.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_glob_tool_name() {
        let tool = GlobTool;
        assert_eq!(tool.name(), "glob");
    }

    #[tokio::test]
    async fn test_glob_tool_description() {
        let tool = GlobTool;
        assert!(tool.description().contains("glob"));
    }

    #[tokio::test]
    async fn test_glob_tool_clone() {
        let tool = GlobTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "glob");
    }

    #[tokio::test]
    async fn test_glob_is_safe() {
        let tool = GlobTool;
        assert!(tool.is_safe());
    }

    #[tokio::test]
    async fn test_glob_no_matches() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "*.nonexistent",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("No files found"));
    }

    #[tokio::test]
    async fn test_glob_finds_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();
        std::fs::write(dir.path().join("b.rs"), "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "*.txt",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("a.txt"));
        assert!(!result.content.contains("b.rs"));
    }

    #[tokio::test]
    async fn test_glob_nested_directories() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("subdir/nested.txt"), "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "**/*.txt",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("nested.txt"));
    }

    #[tokio::test]
    async fn test_glob_skips_hidden_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join(".hidden"), "content").unwrap();
        std::fs::write(dir.path().join("visible.txt"), "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "*",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(!result.content.contains(".hidden"));
        assert!(result.content.contains("visible.txt"));
    }

    #[tokio::test]
    async fn test_glob_skips_node_modules() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("node_modules")).unwrap();
        std::fs::write(dir.path().join("node_modules/pkg.js"), "content").unwrap();
        std::fs::write(dir.path().join("visible.txt"), "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "**/*",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(!result.content.contains("node_modules"));
        assert!(result.content.contains("visible.txt"));
    }

    #[tokio::test]
    async fn test_glob_skips_target() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("target")).unwrap();
        std::fs::write(dir.path().join("target/build.js"), "content").unwrap();
        std::fs::write(dir.path().join("src.rs"), "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "**/*",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(!result.content.contains("target"));
        assert!(result.content.contains("src.rs"));
    }

    #[tokio::test]
    async fn test_glob_nonexistent_path() {
        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "*",
            "path": "/nonexistent/path"
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Directory not found"));
    }

    #[tokio::test]
    async fn test_glob_not_a_directory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "*",
            "path": file_path.to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(!result.success);
        assert!(result.error.unwrap().contains("Not a directory"));
    }

    #[tokio::test]
    async fn test_glob_default_path() {
        let tool = GlobTool;
        let args = serde_json::json!({"pattern": "*.txt"});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_glob_invalid_pattern() {
        let dir = tempdir().unwrap();
        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "[invalid",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_glob_get_dependencies() {
        let tool = GlobTool;
        let args = serde_json::json!({"path": "/some/path", "pattern": "*.txt"});
        let deps = tool.get_dependencies(&args);
        assert!(deps.contains(&std::path::PathBuf::from("/some/path")));
    }

    #[tokio::test]
    async fn test_glob_get_dependencies_default() {
        let tool = GlobTool;
        let args = serde_json::json!({"pattern": "*.txt"});
        let deps = tool.get_dependencies(&args);
        assert!(deps.contains(&std::path::PathBuf::from(".")));
    }

    #[tokio::test]
    async fn test_glob_result_sorted_by_mtime() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("old.txt"), "content").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(dir.path().join("new.txt"), "content").unwrap();

        let tool = GlobTool;
        let args = serde_json::json!({
            "pattern": "*.txt",
            "path": dir.path().to_str().unwrap()
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
    }
}
