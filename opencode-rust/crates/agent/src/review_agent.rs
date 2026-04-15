use crate::sealed;
use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct ReviewAgent {
    system_prompt: String,
    #[allow(dead_code)]
    review_focus: ReviewFocus,
    model: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub enum ReviewFocus {
    #[default]
    General,
    Security,
    Performance,
}

impl ReviewAgent {
    pub fn new() -> Self {
        Self::with_focus(ReviewFocus::default())
    }

    pub fn with_focus(focus: ReviewFocus) -> Self {
        let focus_prompt = match focus {
            ReviewFocus::General => "",
            ReviewFocus::Security => "\nFocus on security vulnerabilities, potential exploits, and secure coding practices.",
            ReviewFocus::Performance => "\nFocus on performance issues, algorithmic complexity, and optimization opportunities.",
        };

        Self {
            system_prompt: format!(
                r#"You are OpenCode ReviewAgent, an AI-powered code review assistant.

Your role is to analyze code changes, identify issues, and provide constructive feedback.
{focus_prompt}

When reviewing code:
1. Analyze the provided code diff or file content
2. Identify potential issues, bugs, or improvements
3. Provide specific, actionable suggestions
4. Consider code quality, readability, and best practices
5. Be constructive and helpful in your feedback

Provide your review in a structured format with clear sections for:
- Critical Issues (bugs, security vulnerabilities)
- Warnings (potential issues, code smells)
- Suggestions (improvements, best practices)
- Comments (questions, observations)

Respond with your analysis in a clear, organized manner.
"#,
                focus_prompt = focus_prompt
            ),
            review_focus: focus,
            model: None,
        }
    }

    pub fn with_security_focus() -> Self {
        Self::with_focus(ReviewFocus::Security)
    }

    pub fn with_performance_focus() -> Self {
        Self::with_focus(ReviewFocus::Performance)
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

impl Default for ReviewAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for ReviewAgent {}

#[async_trait]
impl Agent for ReviewAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Review
    }

    fn name(&self) -> &str {
        "review"
    }

    fn description(&self) -> &str {
        "AI-powered code review agent for analyzing code changes"
    }

    fn can_execute_tools(&self) -> bool {
        true
    }

    fn can_write_files(&self) -> bool {
        false
    }

    fn can_run_commands(&self) -> bool {
        false
    }

    async fn run(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        _tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError> {
        let mut all_messages: Vec<ChatMessage> = vec![ChatMessage {
            role: "system".to_string(),
            content: self.system_prompt.clone(),
        }];

        let prompt_messages =
            session.prepare_messages_for_prompt(TokenBudget::default().main_context_tokens());
        all_messages.extend(messages_to_llm_format(&prompt_messages));

        let response = provider.chat(&all_messages).await?;

        session.add_message(Message::assistant(response.content.clone()));

        Ok(AgentResponse {
            content: response.content,
            tool_calls: Vec::new(),
        })
    }

    fn preferred_model(&self) -> Option<String> {
        self.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_agent_default() {
        let agent = ReviewAgent::new();
        assert_eq!(agent.agent_type(), AgentType::Review);
        assert_eq!(agent.name(), "review");
    }

    #[test]
    fn test_review_agent_security_focus() {
        let agent = ReviewAgent::with_security_focus();
        assert!(agent.system_prompt.contains("security"));
    }

    #[test]
    fn test_review_agent_performance_focus() {
        let agent = ReviewAgent::with_performance_focus();
        assert!(agent.system_prompt.contains("performance"));
    }
}
