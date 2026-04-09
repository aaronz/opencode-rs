use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::{OpenCodeError, SkillManager};
use std::sync::Arc;

pub struct SkillTool {
    skill_manager: Arc<SkillManager>,
}

impl SkillTool {
    pub fn new(skill_manager: Arc<SkillManager>) -> Self {
        Self { skill_manager }
    }
}

#[derive(Deserialize)]
struct SkillArgs {
    skill_name: String,
    parameters: Option<serde_json::Value>,
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str {
        "skill"
    }

    fn description(&self) -> &str {
        "Execute a skill by name with optional parameters"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(Self {
            skill_manager: Arc::clone(&self.skill_manager),
        })
    }

    async fn execute(&self, args: serde_json::Value, _ctx: Option<crate::ToolContext>) -> Result<ToolResult, OpenCodeError> {
        let args: SkillArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        let skill = self.skill_manager
            .get_skill(&args.skill_name)
            .ok_or_else(|| OpenCodeError::Tool(format!("Skill '{}' not found", args.skill_name)))?;

        let content = self.skill_manager.inject_into_prompt(&skill);

        let mut result = format!("Skill '{}' content:\n\n{}", args.skill_name, content);

        if let Some(params) = &args.parameters {
            result.push_str(&format!("\n\nParameters:\n{}", serde_json::to_string_pretty(params).unwrap_or_default()));
        }

        Ok(ToolResult::ok(result))
    }
}
