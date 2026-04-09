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
