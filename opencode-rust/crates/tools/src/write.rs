#![allow(clippy::redundant_closure)]

use crate::sealed;
use crate::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub struct WriteTool;

fn is_path_within_worktree(path: &Path, worktree: &Path) -> bool {
    let Ok(target_canonical) = path.canonicalize() else {
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

        let worktree = ctx
            .as_ref()
            .and_then(|c| c.worktree.as_ref())
            .map(PathBuf::from)
            .or_else(|| {
                ctx.as_ref()
                    .and_then(|c| c.directory.as_ref())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        if !is_path_within_worktree(&path, &worktree) {
            return Ok(ToolResult::err(format!(
                "Access to path outside worktree denied: {}",
                args.path
            )));
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OpenCodeError::Io(e))?;
        }

        std::fs::write(&path, &args.content).map_err(|e| OpenCodeError::Io(e))?;

        Ok(ToolResult::ok(format!("Written to {}", args.path)))
    }
}
