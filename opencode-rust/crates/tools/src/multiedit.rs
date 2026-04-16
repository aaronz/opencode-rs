use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edit {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
}

#[derive(Deserialize, Debug)]
pub struct MultiEditArgs {
    pub edits: Vec<Edit>,
}

pub struct MultiEditTool;

impl sealed::Sealed for MultiEditTool {}

#[async_trait]
impl Tool for MultiEditTool {
    fn name(&self) -> &str {
        "multi_edit"
    }

    fn description(&self) -> &str {
        "Apply multiple edits across different files atomically. All edits succeed or none are applied."
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(MultiEditTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        // Support both {edits: [...]} and direct array formats
        let edits: Vec<Edit> = if let Some(_arr) = args.as_array() {
            serde_json::from_value(args).map_err(|e| OpenCodeError::Parse(e.to_string()))?
        } else {
            let multi_args: MultiEditArgs = serde_json::from_value(args).map_err(|e| {
                OpenCodeError::Parse(format!(
                    "Expected {{edits: [...]}} or array of edits: {}",
                    e
                ))
            })?;
            multi_args.edits
        };

        if edits.is_empty() {
            return Ok(ToolResult::err("No edits provided"));
        }

        // Phase 1: Read all files and validate all edits (fail fast, no writes yet)
        let mut file_contents: HashMap<String, String> = HashMap::new();
        let mut validation_errors: Vec<String> = Vec::new();

        for edit in &edits {
            if edit.path.is_empty() {
                validation_errors.push("Edit has empty path".to_string());
                continue;
            }

            if !file_contents.contains_key(&edit.path) {
                match std::fs::read_to_string(&edit.path) {
                    Ok(content) => {
                        file_contents.insert(edit.path.clone(), content);
                    }
                    Err(e) => {
                        validation_errors.push(format!("Cannot read {}: {}", edit.path, e));
                        continue;
                    }
                }
            }

            let Some(content) = file_contents.get(&edit.path) else {
                validation_errors.push(format!(
                    "Internal error: file {} not found after load",
                    edit.path
                ));
                continue;
            };
            if !content.contains(&edit.old_string) {
                validation_errors.push(format!(
                    "old_string not found in {}: {:?}",
                    edit.path,
                    if edit.old_string.len() > 60 {
                        format!("{}...", &edit.old_string[..60])
                    } else {
                        edit.old_string.clone()
                    }
                ));
            }
        }

        // If any validation fails, abort — no files are touched
        if !validation_errors.is_empty() {
            return Ok(ToolResult::err(format!(
                "Validation failed. No files were modified.\n\n{}",
                validation_errors.join("\n")
            )));
        }

        // Phase 2: Apply all edits in memory (order matters for same-file edits)
        let mut updated_contents = file_contents.clone();
        for edit in &edits {
            let Some(content) = updated_contents.get_mut(&edit.path) else {
                validation_errors.push(format!(
                    "Internal error: file {} not found in updated contents",
                    edit.path
                ));
                continue;
            };
            *content = content.replacen(&edit.old_string, &edit.new_string, 1);
        }

        // Phase 3: Write all files — on any failure, restore backups
        let mut written_files: Vec<String> = Vec::new();
        for (path, new_content) in &updated_contents {
            let Some(original_content) = file_contents.get(path) else {
                tracing::warn!(
                    "File {} not found in original contents during write phase, skipping",
                    path
                );
                continue;
            };
            if *new_content != *original_content {
                if let Err(e) = std::fs::write(path, new_content) {
                    // Rollback already-written files
                    for written_path in &written_files {
                        if let Some(original) = file_contents.get(written_path) {
                            let _ = std::fs::write(written_path, original);
                        }
                    }
                    return Err(OpenCodeError::Tool(format!(
                        "Write failed for {} (all changes rolled back): {}",
                        path, e
                    )));
                }
                written_files.push(path.clone());
            }
        }

        let edited_paths: Vec<String> = edits
            .iter()
            .map(|e| e.path.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        Ok(ToolResult::ok(format!(
            "Applied {} edit(s) across {} file(s):\n{}",
            edits.len(),
            edited_paths.len(),
            edited_paths.join("\n")
        )))
    }
}
