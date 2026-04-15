use crate::sealed;
use crate::{Tool, ToolResult};
use async_trait::async_trait;
use opencode_core::OpenCodeError;
use serde::Deserialize;

pub struct QuestionTool;

#[derive(Deserialize)]
struct QuestionArgs {
    question: String,
    _options: Option<Vec<String>>,
}

#[async_trait]
impl Tool for QuestionTool {
    fn name(&self) -> &str {
        "question"
    }

    fn description(&self) -> &str {
        "Ask user for input"
    }

    fn clone_tool(&self) -> Box<dyn Tool> {
        Box::new(QuestionTool)
    }

    async fn execute(
        &self,
        args: serde_json::Value,
        _ctx: Option<crate::ToolContext>,
    ) -> Result<ToolResult, OpenCodeError> {
        let args: QuestionArgs =
            serde_json::from_value(args).map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "Question: {}\n\nPlease provide your response.",
            args.question
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_question_tool_name() {
        let tool = QuestionTool;
        assert_eq!(tool.name(), "question");
    }

    #[tokio::test]
    async fn test_question_tool_description() {
        let tool = QuestionTool;
        assert_eq!(tool.description(), "Ask user for input");
    }

    #[tokio::test]
    async fn test_question_tool_execute() {
        let tool = QuestionTool;
        let result = tool
            .execute(serde_json::json!({"question": "test?"}), None)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("test?"));
    }

    #[tokio::test]
    async fn test_question_tool_clone() {
        let tool = QuestionTool;
        let cloned = tool.clone_tool();
        assert_eq!(cloned.name(), "question");
    }
}
