use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;
use crate::{Agent, AgentResponse, AgentType, messages_to_llm_format};

pub struct CompactionAgent;

impl CompactionAgent {
    pub fn new() -> Self {
        Self
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
}

pub struct TitleAgent;

impl TitleAgent {
    pub fn new() -> Self {
        Self
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
}

pub struct SummaryAgent;

impl SummaryAgent {
    pub fn new() -> Self {
        Self
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
}
