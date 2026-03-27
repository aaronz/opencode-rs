#[cfg(test)]
mod tests {
    use crate::skill::SkillTool;
    use crate::tool::Tool;
    use crate::ToolResult;
    
    #[tokio::test]
    async fn test_skill_execution() {
        let args = serde_json::json!({
            "skill_name": "test-skill"
        });
        
        let tool = SkillTool;
        let result: Result<ToolResult, _> = tool.execute(args, None).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("test-skill"));
    }
    
    #[tokio::test]
    async fn test_skill_name() {
        let tool = SkillTool;
        assert_eq!(tool.name(), "skill");
    }
    
    #[tokio::test]
    async fn test_skill_description() {
        let tool = SkillTool;
        assert!(!tool.description().is_empty());
    }
    
    #[tokio::test]
    async fn test_skill_clone() {
        let tool = SkillTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "skill");
    }
}
