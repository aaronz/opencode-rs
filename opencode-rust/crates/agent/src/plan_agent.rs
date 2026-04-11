use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct PlanAgent {
    system_prompt: String,
    skill_prompt: Option<String>,
    model: Option<String>,
}

impl PlanAgent {
    pub fn new() -> Self {
        Self {
            system_prompt: r#"You are OpenCode in PLAN mode. You are a read-only agent for analysis and code exploration.

You can:
- Read files (file_read)
- Search code (grep, glob)
- View git status and diffs

You CANNOT:
- Write to files
- Run shell commands
- Execute tools without user confirmation

When the user asks you to make changes, explain what you would do instead of doing it.
Be thorough in your analysis and provide clear explanations.
"# .to_string(),
            skill_prompt: None,
            model: None,
        }
    }

    pub fn with_skill_prompt(mut self, skill_prompt: impl Into<String>) -> Self {
        self.skill_prompt = Some(skill_prompt.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    fn composed_system_prompt(&self) -> String {
        if let Some(skill_prompt) = self.skill_prompt.as_deref() {
            if !skill_prompt.trim().is_empty() {
                return format!(
                    "{}\n\n[Enabled Skills]\n{}",
                    self.system_prompt, skill_prompt
                );
            }
        }
        self.system_prompt.clone()
    }
}

impl Default for PlanAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for PlanAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Plan
    }

    fn name(&self) -> &str {
        "plan"
    }

    fn description(&self) -> &str {
        "Read-only agent for code exploration and analysis"
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
            content: self.composed_system_prompt(),
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
