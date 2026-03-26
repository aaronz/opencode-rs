use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

pub struct SkillTool;

#[derive(Deserialize)]
struct SkillArgs {
    skill_name: String,
    _parameters: Option<serde_json::Value>,
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill"
    }

    fn description(&self) -> &str {
        "Execute skills"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(SkillTool)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: SkillArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "Skill '{}' executed (placeholder).\n\n\
            Skills provide specialized functionality.",
            args.skill_name
        )))
    }
}
