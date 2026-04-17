use crate::sealed;
use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct CompactionAgent {
    model: Option<String>,
}

impl CompactionAgent {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    const SYSTEM_PROMPT: &'static str = r#"You are OpenCode Compaction Agent.

Your task is to analyze the conversation history and create a condensed summary that preserves the key information while reducing token count.

Focus on:
- Important decisions and their rationale
- Files that were modified
- Tools that were used and their results
- Any errors or issues encountered
- Remaining tasks or unresolved questions

Provide a concise summary that captures the essence of the conversation."#;
}

impl Default for CompactionAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for CompactionAgent {}

#[async_trait]
impl Agent for CompactionAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Compaction
    }

    fn name(&self) -> &str {
        "compaction"
    }

    fn description(&self) -> &str {
        "Agent for compressing conversation history"
    }

    fn can_execute_tools(&self) -> bool {
        false
    }

    fn can_write_files(&self) -> bool {
        false
    }

    fn can_run_commands(&self) -> bool {
        false
    }

    fn is_visible(&self) -> bool {
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
mod compaction_tests {
    use super::*;

    #[test]
    fn test_compaction_agent_new() {
        let agent = CompactionAgent::new();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_compaction_agent_with_model() {
        let agent = CompactionAgent::new().with_model("gpt-4");
        assert_eq!(agent.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_compaction_agent_default() {
        let agent = CompactionAgent::default();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_compaction_agent_properties() {
        let agent = CompactionAgent::new();
        assert_eq!(agent.agent_type(), AgentType::Compaction);
        assert_eq!(agent.name(), "compaction");
        assert_eq!(
            agent.description(),
            "Agent for compressing conversation history"
        );
        assert!(!agent.can_execute_tools());
        assert!(!agent.can_write_files());
        assert!(!agent.can_run_commands());
        assert!(!agent.is_visible());
    }

    #[test]
    fn test_compaction_agent_preferred_model() {
        let agent = CompactionAgent::new().with_model("claude-3");
        assert_eq!(agent.preferred_model(), Some("claude-3".to_string()));
    }
}

pub struct TitleAgent {
    model: Option<String>,
}

impl TitleAgent {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    const SYSTEM_PROMPT: &'static str = r#"You are OpenCode Title Agent.

Your task is to generate a short, descriptive title for the conversation based on the user's initial request.

The title should:
- Be 3-7 words long
- Capture the essence of what the user is trying to do
- Be specific enough to identify the conversation
- Not include generic phrases like "help" or "question"

Examples:
- "Fix authentication bug"
- "Add user dashboard"
- "Refactor API endpoints""#;
}

impl Default for TitleAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for TitleAgent {}

#[async_trait]
impl Agent for TitleAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Title
    }

    fn name(&self) -> &str {
        "title"
    }

    fn description(&self) -> &str {
        "Agent for generating conversation titles"
    }

    fn can_execute_tools(&self) -> bool {
        false
    }

    fn can_write_files(&self) -> bool {
        false
    }

    fn can_run_commands(&self) -> bool {
        false
    }

    fn is_visible(&self) -> bool {
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
mod title_tests {
    use super::*;

    #[test]
    fn test_title_agent_new() {
        let agent = TitleAgent::new();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_title_agent_with_model() {
        let agent = TitleAgent::new().with_model("gpt-4");
        assert_eq!(agent.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_title_agent_default() {
        let agent = TitleAgent::default();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_title_agent_properties() {
        let agent = TitleAgent::new();
        assert_eq!(agent.agent_type(), AgentType::Title);
        assert_eq!(agent.name(), "title");
        assert_eq!(
            agent.description(),
            "Agent for generating conversation titles"
        );
        assert!(!agent.can_execute_tools());
        assert!(!agent.can_write_files());
        assert!(!agent.can_run_commands());
        assert!(!agent.is_visible());
    }

    #[test]
    fn test_title_agent_preferred_model() {
        let agent = TitleAgent::new().with_model("claude-3");
        assert_eq!(agent.preferred_model(), Some("claude-3".to_string()));
    }
}

pub struct SummaryAgent {
    model: Option<String>,
}

impl SummaryAgent {
    pub fn new() -> Self {
        Self { model: None }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    const SYSTEM_PROMPT: &'static str = r#"You are OpenCode Summary Agent.

Your task is to generate a concise summary of the current conversation state.

Focus on:
- What has been accomplished so far
- What is currently in progress
- What remains to be done
- Any blockers or issues

Provide a 2-4 sentence summary."#;
}

impl Default for SummaryAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for SummaryAgent {}

#[async_trait]
impl Agent for SummaryAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Summary
    }

    fn name(&self) -> &str {
        "summary"
    }

    fn description(&self) -> &str {
        "Agent for generating conversation summaries"
    }

    fn can_execute_tools(&self) -> bool {
        false
    }

    fn can_write_files(&self) -> bool {
        false
    }

    fn can_run_commands(&self) -> bool {
        false
    }

    fn is_visible(&self) -> bool {
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
mod summary_tests {
    use super::*;

    #[test]
    fn test_summary_agent_new() {
        let agent = SummaryAgent::new();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_summary_agent_with_model() {
        let agent = SummaryAgent::new().with_model("gpt-4");
        assert_eq!(agent.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_summary_agent_default() {
        let agent = SummaryAgent::default();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_summary_agent_properties() {
        let agent = SummaryAgent::new();
        assert_eq!(agent.agent_type(), AgentType::Summary);
        assert_eq!(agent.name(), "summary");
        assert_eq!(
            agent.description(),
            "Agent for generating conversation summaries"
        );
        assert!(!agent.can_execute_tools());
        assert!(!agent.can_write_files());
        assert!(!agent.can_run_commands());
        assert!(!agent.is_visible());
    }

    #[test]
    fn test_summary_agent_preferred_model() {
        let agent = SummaryAgent::new().with_model("claude-3");
        assert_eq!(agent.preferred_model(), Some("claude-3".to_string()));
    }
}
