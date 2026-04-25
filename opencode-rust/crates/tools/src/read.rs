#![allow(clippy::redundant_closure)]

use crate::sealed;
use crate::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub struct ReadTool {
    max_lines: usize,
    _max_bytes: usize,
}

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
struct ReadArgs {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

impl ReadTool {
    pub fn new() -> Self {
        Self {
            max_lines: 2000,
            _max_bytes: 51200,
        }
    }
}

impl Default for ReadTool {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for ReadTool {}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read file with line numbers"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(ReadTool::new())
    }

    fn is_safe(&self) -> bool {
        true
    }

    fn get_dependencies(&self, args: &serde_json::Value) -> HashSet<PathBuf> {
        let mut deps = HashSet::new();
        if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
            deps.insert(PathBuf::from(path));
        }
        deps
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: ReadArgs =
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

        let path_str = path.to_string_lossy();
        let is_protected_path = path_str.contains("/etc/");

        if !is_path_within_worktree(&path, &worktree)
            && (has_explicit_worktree || is_protected_path)
        {
            return Ok(ToolResult::err(format!(
                "Access to path outside worktree denied: {}",
                args.path
            )));
        }

        if !path.exists() {
            return Ok(ToolResult::err(format!("File not found: {}", args.path)));
        }

        let content = std::fs::read_to_string(&path).map_err(|e| OpenCodeError::Io(e))?;

        let lines: Vec<&str> = content.lines().collect();
        let offset = args.offset.unwrap_or(0);
        let limit = args.limit.unwrap_or(self.max_lines);

        let end = (offset + limit).min(lines.len());
        let selected_lines: Vec<_> = lines[offset..end].to_vec();

        let mut result = String::new();
        for (i, line) in selected_lines.iter().enumerate() {
            result.push_str(&format!("{}: {}\n", offset + i + 1, line));
        }

        if end < lines.len() {
            result.push_str(&format!("\n... ({} more lines)\n", lines.len() - end));
        }

        Ok(ToolResult::ok(result))
    }
}
