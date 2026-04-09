#[cfg(test)]
mod tests {
    use crate::skill::SkillTool;
    use crate::tool::Tool;
    use crate::ToolResult;
    use opencode_core::SkillManager;
    use std::sync::Arc;

    fn make_tool() -> SkillTool {
        SkillTool::new(Arc::new(SkillManager::new()))
    }

    #[tokio::test]
    async fn test_skill_execution() {
        let args = serde_json::json!({
            "skill_name": "test-skill"
        });

        let tool = make_tool();
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_skill_name() {
        let tool = make_tool();
        assert_eq!(tool.name(), "skill");
    }

    #[tokio::test]
    async fn test_skill_description() {
        let tool = make_tool();
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_skill_clone() {
        let tool = make_tool();
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "skill");
    }
}
