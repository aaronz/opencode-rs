use crate::sealed;
use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct ExploreAgent {
    model: Option<String>,
}

impl ExploreAgent {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    const SYSTEM_PROMPT: &'static str = r#"You are OpenCode Explore, a fast agent specialized for exploring codebases.

You have access to tools to help you find files and search code:
- glob: Find files matching patterns
- grep: Search file contents with regex
- read: Read file contents
- list: List directory contents
- bash: Execute shell commands (for git, find, etc.)
- webfetch: Fetch web content
- websearch: Search the web
- codesearch: Search for code examples

Use these tools to quickly find files, search code patterns, and answer questions about the codebase.
When asked to explore, specify the thoroughness level: "quick" for basic searches, "medium" for moderate exploration, or "very thorough" for comprehensive analysis.

Always be accurate and provide concrete evidence from the code."#;
}

impl Default for ExploreAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for ExploreAgent {}

#[async_trait]
impl Agent for ExploreAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Explore
    }

    fn name(&self) -> &str {
        "explore"
    }

    fn description(&self) -> &str {
        "Fast agent specialized for exploring codebases"
    }

    fn can_execute_tools(&self) -> bool {
        true
    }

    fn can_write_files(&self) -> bool {
        false
    }

    fn can_run_commands(&self) -> bool {
        true
    }

    async fn run(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        _tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError> {
        let mut all_messages: Vec<ChatMessage> = vec![ChatMessage {
            role: "system".to_string(),
            content: Self::SYSTEM_PROMPT.to_string(),
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
    fn test_explore_agent_new() {
        let agent = ExploreAgent::new();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_explore_agent_with_model() {
        let agent = ExploreAgent::new().with_model("gpt-4");
        assert_eq!(agent.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_explore_agent_default() {
        let agent = ExploreAgent::default();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_explore_agent_properties() {
        let agent = ExploreAgent::new();
        assert_eq!(agent.agent_type(), AgentType::Explore);
        assert_eq!(agent.name(), "explore");
        assert_eq!(
            agent.description(),
            "Fast agent specialized for exploring codebases"
        );
        assert!(agent.can_execute_tools());
        assert!(!agent.can_write_files());
        assert!(agent.can_run_commands());
        assert!(agent.is_visible());
    }

    #[test]
    fn test_explore_agent_preferred_model() {
        let agent = ExploreAgent::new().with_model("claude-3");
        assert_eq!(agent.preferred_model(), Some("claude-3".to_string()));
    }
}
