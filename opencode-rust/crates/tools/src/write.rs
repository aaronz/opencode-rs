#![allow(clippy::redundant_closure)]

use crate::sealed;
use crate::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub struct WriteTool;

fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::RootDir => {
                result = PathBuf::from("/");
            }
            _ => result.push(component),
        }
    }
    result
}

fn is_path_within_worktree(path: &Path, worktree: &Path) -> bool {
    let Ok(target_canonical) = path.canonicalize() else {
        if let Some(parent) = path.parent() {
            if parent.exists() {
                if let Ok(parent_canonical) = parent.canonicalize() {
                    if let Ok(worktree_canonical) = worktree.canonicalize() {
                        return parent_canonical.starts_with(&worktree_canonical);
                    }
                }
            }
        }
        return false;
    };
    let Ok(worktree_canonical) = worktree.canonicalize() else {
        return false;
    };
    target_canonical.starts_with(&worktree_canonical)
}

#[derive(Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

impl sealed::Sealed for WriteTool {}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write files"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(WriteTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: WriteArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let path = PathBuf::from(&args.path);

        let explicit_worktree = ctx
            .as_ref()
            .and_then(|c| c.worktree.as_ref())
            .map(PathBuf::from)
            .or_else(|| {
                ctx.as_ref()
                    .and_then(|c| c.directory.as_ref())
                    .map(PathBuf::from)
            });

        let has_explicit_worktree = explicit_worktree.is_some();

        let worktree =
            explicit_worktree.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let final_path: PathBuf = if path.is_absolute() {
            path.clone()
        } else {
            worktree.join(&path)
        };

        let normalized_final = normalize_path(&final_path);
        let normalized_str = normalized_final.to_string_lossy();
        let is_tmp_path =
            normalized_str.contains("/tmp/") || normalized_str.contains("/private/tmp/");

        if !is_path_within_worktree(&final_path, &worktree) {
            if has_explicit_worktree {
                return Ok(ToolResult::err(format!(
                    "Access to path outside worktree denied: {}",
                    args.path
                )));
            }
            if is_tmp_path {
                return Ok(ToolResult::err(format!(
                    "Access to path outside worktree denied: {}",
                    args.path
                )));
            }
        }

        if let Some(parent) = final_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OpenCodeError::Io(e))?;
        }

        std::fs::write(&final_path, &args.content).map_err(|e| OpenCodeError::Io(e))?;

        Ok(ToolResult::ok(format!("Written to {}", args.path)))
    }
}
