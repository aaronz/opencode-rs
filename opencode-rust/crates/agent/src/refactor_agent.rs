use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct RefactorAgent {
    system_prompt: String,
    preview_mode: bool,
}

impl RefactorAgent {
    pub fn new() -> Self {
        Self {
            system_prompt:
                r#"You are OpenCode RefactorAgent, an intelligent code refactoring assistant.

Your role is to analyze code, identify refactoring opportunities, and apply improvements.

When refactoring code:
1. Identify code smells and anti-patterns
2. Suggest specific refactoring techniques
3. Provide before/after code examples
4. Apply refactorings safely when approved
5. Validate changes after refactoring

Supported refactorings:
- Extract Method: Extract a code block into a new function
- Rename: Rename variables, functions, or classes for clarity
- Inline: Replace a variable or function with its value/use
- Simplify: Reduce complex expressions
- Extract Variable: Assign complex expressions to named variables

Always ensure refactoring preserves behavior. Run tests to validate.
"#
                .to_string(),
            preview_mode: false,
        }
    }

    pub fn with_preview_mode(mut self) -> Self {
        self.preview_mode = true;
        self.system_prompt = format!(
            "{}\n\n**PREVIEW MODE**: Show diff of proposed changes WITHOUT applying them.",
            self.system_prompt
        );
        self
    }

    pub fn preview() -> Self {
        Self::new().with_preview_mode()
    }
}

impl Default for RefactorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for RefactorAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Refactor
    }

    fn name(&self) -> &str {
        "refactor"
    }

    fn description(&self) -> &str {
        "Intelligent code refactoring assistant"
    }

    fn can_execute_tools(&self) -> bool {
        !self.preview_mode
    }

    fn can_write_files(&self) -> bool {
        !self.preview_mode
    }

    fn can_run_commands(&self) -> bool {
        !self.preview_mode
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refactor_agent_default() {
        let agent = RefactorAgent::new();
        assert_eq!(agent.agent_type(), AgentType::Refactor);
        assert_eq!(agent.name(), "refactor");
    }

    #[test]
    fn test_refactor_agent_preview_mode() {
        let agent = RefactorAgent::preview();
        assert!(agent.preview_mode);
    }
}
