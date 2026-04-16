#![allow(clippy::redundant_closure)]

use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
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

fn count_recursive(path: &Path) -> usize {
    let mut count = 0;
    if let Ok(rd) = fs::read_dir(path) {
        for entry in rd.flatten() {
            let ep = entry.path();
            if ep.is_dir() {
                count += count_recursive(&ep);
            } else {
                count += 1;
            }
        }
    }
    count
}

impl sealed::Sealed for TruncationDirTool {}

#[async_trait]
impl Tool for TruncationDirTool {
    fn name(&self) -> &str {
        "truncation_dir"
    }

    fn description(&self) -> &str {
        "List files in a directory with truncation for large directory sets. Returns total file count and a sorted subset of entries."
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(TruncationDirTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: TruncationDirArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Parse(e.to_string()))?;
        let max_files = args.max_files.unwrap_or(50);
        let path = Path::new(&args.path);

        if !path.exists() {
            return Err(OpenCodeError::Tool(format!(
                "Path not found: {}",
                args.path
            )));
        }

        if !path.is_dir() {
            return Err(OpenCodeError::Tool(format!(
                "Not a directory: {}",
                args.path
            )));
        }

        let mut entries: Vec<String> = fs::read_dir(path)
            .map_err(|e| OpenCodeError::Io(e))?
            .flatten()
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .collect();

        entries.sort();
        let total_direct = entries.len();
        let total_recursive = count_recursive(path);

        let mut result = if total_recursive != total_direct {
            format!(
                "Directory: {}\nEntries: {} (direct), {} (total recursive)\n\n",
                args.path, total_direct, total_recursive
            )
        } else {
            format!("Directory: {}\nEntries: {}\n\n", args.path, total_direct)
        };

        if total_direct > max_files {
            let shown = &entries[..max_files];
            for name in shown {
                result.push_str(&format!("  {}\n", name));
            }
            result.push_str(&format!(
                "\n... and {} more entries (showing first {})",
                total_direct - max_files,
                max_files
            ));
        } else {
            for name in &entries {
                result.push_str(&format!("  {}\n", name));
            }
        }

        Ok(ToolResult::ok(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_truncation_dir_tool_name() {
        let tool = TruncationDirTool;
        assert_eq!(tool.name(), "truncation_dir");
    }

    #[tokio::test]
    async fn test_truncation_dir_tool_description() {
        let tool = TruncationDirTool;
        assert!(tool.description().contains("truncation"));
    }

    #[tokio::test]
    async fn test_truncation_dir_tool_clone() {
        let tool = TruncationDirTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "truncation_dir");
    }

    #[tokio::test]
    async fn test_truncation_dir_empty_directory() {
        let dir = tempdir().unwrap();
        let tool = TruncationDirTool;
        let args = serde_json::json!({"path": dir.path().to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("Entries: 0"));
    }

    #[tokio::test]
    async fn test_truncation_dir_with_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();
        std::fs::write(dir.path().join("b.txt"), "content").unwrap();

        let tool = TruncationDirTool;
        let args = serde_json::json!({"path": dir.path().to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("Entries: 2"));
    }

    #[tokio::test]
    async fn test_truncation_dir_with_subdirectory() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("subdir/b.txt"), "content").unwrap();
        std::fs::write(dir.path().join("subdir/c.txt"), "content").unwrap();

        let tool = TruncationDirTool;
        let args = serde_json::json!({"path": dir.path().to_str().unwrap()});
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("Entries:"));
        assert!(result.content.contains("total recursive"));
    }

    #[tokio::test]
    async fn test_truncation_dir_with_max_files() {
        let dir = tempdir().unwrap();
        for i in 0..10 {
            std::fs::write(dir.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let tool = TruncationDirTool;
        let args = serde_json::json!({
            "path": dir.path().to_str().unwrap(),
            "max_files": 5
        });
        let result = tool.execute(args, None).await.unwrap();
        assert!(result.success);
        assert!(result.content.contains("and 5 more entries"));
    }

    #[tokio::test]
    async fn test_truncation_dir_nonexistent_path() {
        let tool = TruncationDirTool;
        let args = serde_json::json!({"path": "/nonexistent/path"});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_truncation_dir_not_a_directory() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, "content").unwrap();

        let tool = TruncationDirTool;
        let args = serde_json::json!({"path": file_path.to_str().unwrap()});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_count_recursive_empty_dir() {
        let dir = tempdir().unwrap();
        let count = count_recursive(dir.path());
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_count_recursive_with_files() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();
        std::fs::write(dir.path().join("b.txt"), "content").unwrap();
        let count = count_recursive(dir.path());
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_count_recursive_nested() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("a.txt"), "content").unwrap();
        std::fs::write(dir.path().join("subdir/b.txt"), "content").unwrap();
        let count = count_recursive(dir.path());
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_truncation_dir_invalid_args() {
        let tool = TruncationDirTool;
        let args = serde_json::json!({"path": 123});
        let result = tool.execute(args, None).await;
        assert!(result.is_err());
    }
}
