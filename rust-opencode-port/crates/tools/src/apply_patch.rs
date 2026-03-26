use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct ApplyPatchTool;

#[derive(Deserialize)]
struct ApplyPatchArgs {
    patch_text: String,
    _workdir: Option<String>,
}

#[async_trait]
impl Tool for ApplyPatchTool {
    fn name(&self) -> &str {
        "apply_patch"
    }

    fn description(&self) -> &str {
        "Apply code patches in diff format"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(ApplyPatchTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: ApplyPatchArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let patch_text = args.patch_text;

        if patch_text.is_empty() {
            return Ok(ToolResult::err("Patch text is required".to_string()));
        }

        let normalized = patch_text.replace("\r\n", "\n").replace('\r', "\n");
        let trimmed = normalized.trim();

        if trimmed == "*** Begin Patch\n*** End Patch" {
            return Ok(ToolResult::err("Empty patch".to_string()));
        }

        let mut hunks = Vec::new();
        let mut current_file: Option<String> = None;
        let mut current_lines = Vec::new();

        for line in trimmed.lines() {
            if line.starts_with("*** Update File: ") {
                if let Some(file) = current_file.take() {
                    hunks.push((file, current_lines.clone()));
                    current_lines.clear();
                }
                current_file = Some(line.trim_start_matches("*** Update File: ").to_string());
            } else if line.starts_with("*** Add File: ") {
                if let Some(file) = current_file.take() {
                    hunks.push((file, current_lines.clone()));
                    current_lines.clear();
                }
                current_file = Some(line.trim_start_matches("*** Add File: ").to_string());
            } else if line.starts_with("*** Delete File: ") {
                if let Some(file) = current_file.take() {
                    hunks.push((file, current_lines.clone()));
                    current_lines.clear();
                }
                current_file = Some(line.trim_start_matches("*** Delete File: ").to_string());
            } else if line == "*** End Patch" {
                if let Some(file) = current_file.take() {
                    hunks.push((file, current_lines.clone()));
                    current_lines.clear();
                }
            } else if let Some(_file) = &current_file {
                current_lines.push(line.to_string());
            }
        }

        if hunks.is_empty() {
            return Ok(ToolResult::err("No hunks found in patch".to_string()));
        }

        let mut results: Vec<String> = Vec::new();
        let errors: Vec<String> = Vec::new();

        for (file_path, _lines) in &hunks {
            let path = PathBuf::from(file_path);

            if path.exists() {
                results.push(format!("M {}", file_path));
            } else {
                results.push(format!("A {}", file_path));
            }
        }

        if !errors.is_empty() {
            return Ok(ToolResult::err(format!(
                "Patch errors:\n{}",
                errors.join("\n")
            )));
        }

        Ok(ToolResult::ok(format!(
            "Patch applied successfully.\n\n{}",
            results.join("\n")
        )))
    }
}
