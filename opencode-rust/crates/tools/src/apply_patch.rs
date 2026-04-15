use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::path::PathBuf;

pub struct ApplyPatchTool;

#[derive(Deserialize)]
struct ApplyPatchArgs {
    #[serde(rename = "patchText")]
    patch_text: String,
}

#[derive(Debug, Clone)]
enum HunkType {
    Add {
        path: String,
        contents: String,
    },
    Delete {
        path: String,
    },
    Update {
        path: String,
        chunks: Vec<UpdateChunk>,
    },
}

#[derive(Debug, Clone)]
struct UpdateChunk {
    old_lines: Vec<String>,
    new_lines: Vec<String>,
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

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: ApplyPatchArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let patch_text = args.patch_text;

        if patch_text.is_empty() {
            return Ok(ToolResult::err("Patch text is required".to_string()));
        }

        let normalized = patch_text.replace("\r\n", "\n").replace('\r', "\n");
        let trimmed = normalized.trim();

        if trimmed == "*** Begin Patch\n*** End Patch" {
            return Ok(ToolResult::err("Empty patch".to_string()));
        }

        let hunks = parse_hunks(trimmed).map_err(OpenCodeError::Tool)?;

        if hunks.is_empty() {
            return Ok(ToolResult::err("No hunks found in patch".to_string()));
        }

        let mut results: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();

        for hunk in &hunks {
            match hunk {
                HunkType::Add { path, contents } => {
                    let file_path = PathBuf::from(path);
                    if let Some(parent) = file_path.parent() {
                        if !parent.as_os_str().is_empty() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                    }
                    match std::fs::write(&file_path, contents) {
                        Ok(_) => results.push(format!("A {}", path)),
                        Err(e) => errors.push(format!("Failed to write {}: {}", path, e)),
                    }
                }
                HunkType::Delete { path } => {
                    let file_path = PathBuf::from(path);
                    match std::fs::remove_file(&file_path) {
                        Ok(_) => results.push(format!("D {}", path)),
                        Err(e) => errors.push(format!("Failed to delete {}: {}", path, e)),
                    }
                }
                HunkType::Update { path, chunks } => {
                    let file_path = PathBuf::from(path);
                    if !file_path.exists() {
                        errors.push(format!("File not found: {}", path));
                        continue;
                    }

                    let old_content = match std::fs::read_to_string(&file_path) {
                        Ok(c) => c,
                        Err(e) => {
                            errors.push(format!("Failed to read {}: {}", path, e));
                            continue;
                        }
                    };

                    let new_content =
                        apply_chunks(&old_content, chunks).map_err(OpenCodeError::Tool)?;

                    match std::fs::write(&file_path, &new_content) {
                        Ok(_) => results.push(format!("M {}", path)),
                        Err(e) => errors.push(format!("Failed to write {}: {}", path, e)),
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Ok(ToolResult::err(format!(
                "Patch errors:\n{}",
                errors.join("\n")
            )));
        }

        Ok(ToolResult::ok(format!(
            "Success. Updated the following files:\n\n{}",
            results.join("\n")
        )))
    }
}

fn parse_hunks(patch: &str) -> Result<Vec<HunkType>, String> {
    let mut hunks = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_type: Option<&str> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in patch.lines() {
        if line.starts_with("*** Update File: ") {
            if let Some(file) = current_file.take() {
                if !current_lines.is_empty() {
                    hunks.push(process_file_entry(
                        &file,
                        current_type.unwrap_or("update"),
                        &current_lines,
                    ));
                }
                current_lines.clear();
            }
            current_file = Some(line.trim_start_matches("*** Update File: ").to_string());
            current_type = Some("update");
        } else if line.starts_with("*** Add File: ") {
            if let Some(file) = current_file.take() {
                if !current_lines.is_empty() {
                    hunks.push(process_file_entry(
                        &file,
                        current_type.unwrap_or("add"),
                        &current_lines,
                    ));
                }
                current_lines.clear();
            }
            current_file = Some(line.trim_start_matches("*** Add File: ").to_string());
            current_type = Some("add");
        } else if line.starts_with("*** Delete File: ") {
            if let Some(file) = current_file.take() {
                if !current_lines.is_empty() {
                    hunks.push(process_file_entry(
                        &file,
                        current_type.unwrap_or("delete"),
                        &current_lines,
                    ));
                }
                current_lines.clear();
            }
            current_file = Some(line.trim_start_matches("*** Delete File: ").to_string());
            current_type = Some("delete");
        } else if line.starts_with("*** ") {
            // Skip other markers
        } else if current_file.is_some() {
            current_lines.push(line.to_string());
        }
    }

    if let Some(file) = current_file {
        if !current_lines.is_empty() {
            hunks.push(process_file_entry(
                &file,
                current_type.unwrap_or("update"),
                &current_lines,
            ));
        }
    }

    Ok(hunks)
}

fn process_file_entry(path: &str, hunk_type: &str, lines: &[String]) -> HunkType {
    match hunk_type {
        "delete" => HunkType::Delete {
            path: path.to_string(),
        },
        "add" => {
            let contents: String = lines
                .iter()
                .filter(|l| l.starts_with('+'))
                .map(|l| l[1..].to_string())
                .collect::<Vec<_>>()
                .join("\n");
            HunkType::Add {
                path: path.to_string(),
                contents,
            }
        }
        _ => {
            let chunks = parse_update_chunks(lines);
            HunkType::Update {
                path: path.to_string(),
                chunks,
            }
        }
    }
}

fn parse_update_chunks(lines: &[String]) -> Vec<UpdateChunk> {
    let mut chunks = Vec::new();
    let mut current_old = Vec::new();
    let mut current_new = Vec::new();
    let mut in_chunk = false;

    for line in lines {
        if line.starts_with("@@") {
            if !current_old.is_empty() || !current_new.is_empty() {
                chunks.push(UpdateChunk {
                    old_lines: current_old.clone(),
                    new_lines: current_new.clone(),
                });
                current_old.clear();
                current_new.clear();
            }
            in_chunk = true;
        } else if in_chunk {
            if let Some(stripped) = line.strip_prefix('-') {
                current_old.push(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix('+') {
                current_new.push(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix(' ') {
                let ctx = stripped.to_string();
                current_old.push(ctx.clone());
                current_new.push(ctx);
            } else if line.is_empty() {
                current_old.push(String::new());
                current_new.push(String::new());
            }
        }
    }

    if !current_old.is_empty() || !current_new.is_empty() {
        chunks.push(UpdateChunk {
            old_lines: current_old,
            new_lines: current_new,
        });
    }

    chunks
}

fn apply_chunks(content: &str, chunks: &[UpdateChunk]) -> Result<String, String> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let mut search_start = 0;
    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        if chunk.old_lines.is_empty() {
            continue;
        }

        let found_pos = find_chunk_position(&lines, &chunk.old_lines, search_start);

        match found_pos {
            Some(start) => {
                let end = start + chunk.old_lines.len();
                lines.splice(start..end, chunk.new_lines.iter().cloned());
                search_start = start + chunk.new_lines.len();
            }
            None => {
                let context_lines: Vec<&str> =
                    chunk.old_lines.iter().take(3).map(|s| s.as_str()).collect();
                return Err(format!(
                    "Chunk {} context not found in file (searched from line {}).\nExpected lines starting with:\n{}",
                    chunk_idx + 1,
                    search_start + 1,
                    context_lines.join("\n")
                ));
            }
        }
    }

    let mut result = lines.join("\n");
    if content.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    Ok(result)
}

fn find_chunk_position(lines: &[String], needle: &[String], start: usize) -> Option<usize> {
    if needle.is_empty() || lines.len() < needle.len() {
        return None;
    }

    let max_start = lines.len() - needle.len();
    for i in start..=max_start {
        if lines[i..i + needle.len()]
            .iter()
            .zip(needle.iter())
            .all(|(a, b)| a == b)
        {
            return Some(i);
        }
    }
    None
}
