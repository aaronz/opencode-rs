use crate::sealed;
use crate::tool::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use opencode_core::{session::Session, OpenCodeError};
use opencode_permission::{check_tool_permission_default, ApprovalResult};
use uuid::Uuid;

pub struct SessionLoadTool;

impl sealed::Sealed for SessionLoadTool {}

#[async_trait]
impl Tool for SessionLoadTool {
    fn name(&self) -> &str {
        "session_load"
    }

    fn description(&self) -> &str {
        "Load a session from storage"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(Self)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let permission_check = check_tool_permission_default("session_load");
        if permission_check != ApprovalResult::AutoApprove {
            return Ok(ToolResult::err(
                "Permission denied: session_load requires approval in current scope",
            ));
        }

        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenCodeError::Parse("session_id required".to_string()))?;

        match Uuid::parse_str(session_id) {
            Ok(uuid) => match Session::load_by_id(&uuid) {
                Ok(session) => {
                    let summary = format!(
                        "Loaded session: {} ({} messages)",
                        session.id,
                        session.messages.len()
                    );
                    Ok(ToolResult::ok(summary))
                }
                Err(e) => Ok(ToolResult::err(format!("Failed to load session: {}", e))),
            },
            Err(_) => Ok(ToolResult::err("Invalid session_id format")),
        }
    }
}

pub struct SessionSaveTool;

impl sealed::Sealed for SessionSaveTool {}

#[async_trait]
impl Tool for SessionSaveTool {
    fn name(&self) -> &str {
        "session_save"
    }

    fn description(&self) -> &str {
        "Save current session to storage"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(Self)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let permission_check = check_tool_permission_default("session_save");
        if permission_check != ApprovalResult::AutoApprove {
            return Ok(ToolResult::err(
                "Permission denied: session_save requires approval in current scope",
            ));
        }

        let session_id = args
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OpenCodeError::Parse("session_id required".to_string()))?;

        match Uuid::parse_str(session_id) {
            Ok(uuid) => match Session::load_by_id(&uuid) {
                Ok(session) => {
                    let path = Session::session_path(&uuid);
                    match session.save(&path) {
                        Ok(_) => Ok(ToolResult::ok(format!("Saved session: {}", session_id))),
                        Err(e) => Ok(ToolResult::err(format!("Failed to save session: {}", e))),
                    }
                }
                Err(e) => Ok(ToolResult::err(format!("Session not found: {}", e))),
            },
            Err(_) => Ok(ToolResult::err("Invalid session_id format")),
        }
    }
}
