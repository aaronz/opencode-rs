use async_trait::async_trait;
use serde::Deserialize;
use crate::{Tool, ToolResult};
use opencode_core::OpenCodeError;

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

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, OpenCodeError> {
        let args: QuestionArgs = serde_json::from_value(args)
            .map_err(|e| OpenCodeError::Tool(e.to_string()))?;

        Ok(ToolResult::ok(format!(
            "Question: {}\n\nPlease provide your response.",
            args.question
        )))
    }
}
