use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;
use crate::{Agent, AgentResponse, AgentType, messages_to_llm_format};

pub struct GeneralAgent {
    system_prompt: String,
}

impl GeneralAgent {
    pub fn new() -> Self {
        Self {
            system_prompt: r#"You are OpenCode's GENERAL agent - a specialized subagent for complex multi-step search and research tasks.

You excel at:
- Searching through large codebases
- Finding specific patterns across many files
- Researching topics thoroughly
- Breaking down complex queries into steps

You have access to search tools:
- glob: Find files by pattern
- grep: Search file contents with regex
- file_read: Read specific files

Be thorough and systematic. Break complex tasks into steps.
Provide comprehensive results with source references.
"# .to_string(),
        }
    }
}

impl Default for GeneralAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for GeneralAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::General
    }

    fn name(&self) -> &str {
        "general"
    }

    fn description(&self) -> &str {
        "Subagent for complex multi-step searches and research"
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
}
