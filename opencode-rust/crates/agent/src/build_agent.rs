use crate::sealed;
use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::provider::EventCallback;
use opencode_llm::provider_abstraction::ReasoningBudget;
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct BuildAgent {
    system_prompt: String,
    skill_prompt: Option<String>,
    agents_md_content: Option<String>,
    model: Option<String>,
    variant: Option<String>,
    reasoning_budget: Option<ReasoningBudget>,
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
"#
            .to_string(),
            skill_prompt: None,
            agents_md_content: None,
            model: None,
            variant: None,
            reasoning_budget: None,
        }
    }

    pub fn with_skill_prompt(mut self, skill_prompt: impl Into<String>) -> Self {
        self.skill_prompt = Some(skill_prompt.into());
        self
    }

    pub fn with_agents_md_content(mut self, content: impl Into<String>) -> Self {
        self.agents_md_content = Some(content.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    pub fn with_reasoning_budget(mut self, budget: ReasoningBudget) -> Self {
        self.reasoning_budget = Some(budget);
        self
    }

    fn composed_system_prompt(&self) -> String {
        let mut prompt = self.system_prompt.clone();

        if let Some(agents_md) = self.agents_md_content.as_deref() {
            if !agents_md.trim().is_empty() {
                prompt = format!(
                    "{}\n\n# Project Guidelines (from AGENTS.md)\n{}",
                    prompt, agents_md
                );
            }
        }

        if let Some(skill_prompt) = self.skill_prompt.as_deref() {
            if !skill_prompt.trim().is_empty() {
                prompt = format!("{}\n\n[Enabled Skills]\n{}", prompt, skill_prompt);
            }
        }

        prompt
    }

    #[cfg(test)]
    pub fn system_prompt_for_testing(&self) -> String {
        self.composed_system_prompt()
    }
}

impl Default for BuildAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for BuildAgent {}

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

    async fn run_streaming(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        _tools: &ToolRegistry,
        events: EventCallback,
    ) -> Result<AgentResponse, OpenCodeError> {
        let mut all_messages: Vec<ChatMessage> = vec![ChatMessage {
            role: "system".to_string(),
            content: self.composed_system_prompt(),
        }];

        let prompt_messages =
            session.prepare_messages_for_prompt(TokenBudget::default().main_context_tokens());
        all_messages.extend(messages_to_llm_format(&prompt_messages));

        let prompt = all_messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let final_content = provider
            .complete_with_events(&prompt, None, events)
            .await?;

        let content = final_content.unwrap_or_default();

        Ok(AgentResponse {
            content,
            tool_calls: Vec::new(),
        })
    }

    fn preferred_model(&self) -> Option<String> {
        self.model.clone()
    }

    fn preferred_variant(&self) -> Option<String> {
        self.variant.clone()
    }

    fn preferred_reasoning_budget(&self) -> Option<ReasoningBudget> {
        self.reasoning_budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_agent_with_agents_md_content() {
        let agents_md = r#"## Testing Conventions

All dialog tests must cover:
1. Empty collection + Enter closes
2. Empty collection + navigation doesn't panic
"#;
        let agent = BuildAgent::new().with_agents_md_content(agents_md);

        let prompt = agent.system_prompt_for_testing();
        assert!(prompt.contains("# Project Guidelines (from AGENTS.md)"));
        assert!(prompt.contains("Testing Conventions"));
        assert!(prompt.contains("Empty collection + Enter closes"));
    }

    #[test]
    fn test_build_agent_without_agents_md_content() {
        let agent = BuildAgent::new();

        let prompt = agent.system_prompt_for_testing();
        assert!(!prompt.contains("# Project Guidelines (from AGENTS.md)"));
    }

    #[test]
    fn test_build_agent_with_empty_agents_md() {
        let agent = BuildAgent::new().with_agents_md_content("");

        let prompt = agent.system_prompt_for_testing();
        assert!(!prompt.contains("# Project Guidelines (from AGENTS.md)"));
    }

    #[test]
    fn test_build_agent_with_skill_prompt_and_agents_md() {
        let agents_md = "## Project Rules\nUse clippy before commit.";
        let skill_prompt = "# Enabled Skills\n- skill1: Test skill";

        let agent = BuildAgent::new()
            .with_agents_md_content(agents_md)
            .with_skill_prompt(skill_prompt);

        let prompt = agent.system_prompt_for_testing();
        assert!(prompt.contains("# Project Guidelines (from AGENTS.md)"));
        assert!(prompt.contains("# Enabled Skills"));
        assert!(prompt.contains("Project Rules"));
        assert!(prompt.contains("skill1"));
    }
}
