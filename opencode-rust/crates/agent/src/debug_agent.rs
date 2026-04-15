use crate::sealed;
use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct DebugAgent {
    system_prompt: String,
    model: Option<String>,
}

impl DebugAgent {
    pub fn new() -> Self {
        Self {
            system_prompt: r#"You are OpenCode DebugAgent, an AI-powered debugging assistant.

Your role is to analyze errors, exceptions, and unexpected behavior to help fix bugs.

When debugging:
1. Parse and understand the error message or stack trace
2. Identify the root cause of the issue
3. Analyze relevant source code
4. Provide specific, actionable fix suggestions
5. Explain the problem clearly

When analyzing failures:
- Test failures: Examine the test code and understand what assertion failed
- Runtime errors: Parse the stack trace to find the failure point
- Compile errors: Understand the type mismatch or syntax issue
- Logic errors: Analyze the code flow and identify the bug

Provide fixes in a clear format:
- Problem explanation
- Root cause
- Suggested fix (with code if applicable)
- Alternative solutions if multiple exist

Be thorough and help the user understand the issue.
"#
            .to_string(),
            model: None,
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

impl Default for DebugAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for DebugAgent {}

#[async_trait]
impl Agent for DebugAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Debug
    }

    fn name(&self) -> &str {
        "debug"
    }

    fn description(&self) -> &str {
        "AI-powered debugging assistant"
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

    fn preferred_model(&self) -> Option<String> {
        self.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_agent_default() {
        let agent = DebugAgent::new();
        assert_eq!(agent.agent_type(), AgentType::Debug);
        assert_eq!(agent.name(), "debug");
    }
}
