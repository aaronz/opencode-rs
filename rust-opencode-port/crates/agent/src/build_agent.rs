use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;
use crate::{Agent, AgentResponse, AgentType, messages_to_llm_format};

pub struct BuildAgent {
    system_prompt: String,
}

impl BuildAgent {
    pub fn new() -> Self {
        Self {
            system_prompt: r#"You are OpenCode, an AI coding assistant built by anomalyco. You are helpful, harmless, and honest.

You have access to tools to help you complete coding tasks:
- file_read: Read file contents
- file_write: Write content to files
- glob: Find files matching patterns
- grep: Search file contents
- git_status: Check git status
- git_diff: View uncommitted changes

When you need to use a tool, respond with a JSON object containing tool_calls.
After receiving tool results, continue with your response.

Always be accurate and honest. If you're unsure about something, say so.
"# .to_string(),
        }
    }
}

impl Default for BuildAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for BuildAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Build
    }

    fn name(&self) -> &str {
        "build"
    }

    fn description(&self) -> &str {
        "Default agent with full file system and command execution access"
    }

    fn can_execute_tools(&self) -> bool {
        true
    }

    fn can_write_files(&self) -> bool {
        true
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
