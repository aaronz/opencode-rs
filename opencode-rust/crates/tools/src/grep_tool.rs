use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use regex::Regex;
use serde::Deserialize;

pub struct GrepTool;

#[derive(Deserialize)]
struct GrepArgs {
    pattern: String,
    path: Option<String>,
    file_type: Option<String>,
    count: Option<bool>,
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents using regex"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(GrepTool)
    }

    fn is_safe(&self) -> bool {
        true
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
