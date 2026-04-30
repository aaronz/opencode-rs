use crate::sealed;
use crate::{messages_to_llm_format, Agent, AgentResponse, AgentType};
use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session, TokenBudget};
use opencode_llm::provider::{EventCallback, ToolSchema};
use opencode_llm::{ChatMessage, Provider};
use opencode_tools::ToolRegistry;

pub struct GeneralAgent {
    system_prompt: String,
    skill_prompt: Option<String>,
    model: Option<String>,
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

impl Default for GeneralAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::Sealed for GeneralAgent {}

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
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError> {
        let mut all_messages: Vec<ChatMessage> = vec![ChatMessage {
            role: "system".to_string(),
            content: self.composed_system_prompt(),
        }];

        let prompt_messages =
            session.prepare_messages_for_prompt(TokenBudget::default().main_context_tokens());
        all_messages.extend(messages_to_llm_format(&prompt_messages));

        let tool_schemas: Vec<ToolSchema> = tools
            .all_tool_schemas()
            .iter()
            .filter_map(|json| serde_json::from_value(json.clone()).ok())
            .collect();

        let response = if tool_schemas.is_empty() {
            provider.chat(&all_messages).await?
        } else {
            provider
                .chat_with_tools(&all_messages, &tool_schemas)
                .await?
        };

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

        let final_content = provider.complete_with_events(&prompt, None, events).await?;

        let content = final_content.unwrap_or_default();

        Ok(AgentResponse {
            content,
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
    fn test_general_agent_new() {
        let agent = GeneralAgent::new();
        assert!(agent.model.is_none());
        assert!(agent.skill_prompt.is_none());
    }

    #[test]
    fn test_general_agent_with_model() {
        let agent = GeneralAgent::new().with_model("gpt-4");
        assert_eq!(agent.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_general_agent_with_skill_prompt() {
        let agent = GeneralAgent::new().with_skill_prompt("Test skill prompt");
        assert_eq!(agent.skill_prompt.as_deref(), Some("Test skill prompt"));
    }

    #[test]
    fn test_general_agent_default() {
        let agent = GeneralAgent::default();
        assert!(agent.model.is_none());
    }

    #[test]
    fn test_general_agent_properties() {
        let agent = GeneralAgent::new();
        assert_eq!(agent.agent_type(), AgentType::General);
        assert_eq!(agent.name(), "general");
        assert_eq!(
            agent.description(),
            "Subagent for complex multi-step searches and research"
        );
        assert!(agent.can_execute_tools());
        assert!(!agent.can_write_files());
        assert!(!agent.can_run_commands());
        assert!(agent.is_visible());
    }

    #[test]
    fn test_general_agent_composed_system_prompt_without_skill() {
        let agent = GeneralAgent::new();
        let prompt = agent.composed_system_prompt();
        assert!(prompt.contains("GENERAL agent"));
        assert!(!prompt.contains("[Enabled Skills]"));
    }

    #[test]
    fn test_general_agent_composed_system_prompt_with_skill() {
        let agent = GeneralAgent::new().with_skill_prompt("Test skill");
        let prompt = agent.composed_system_prompt();
        assert!(prompt.contains("GENERAL agent"));
        assert!(prompt.contains("[Enabled Skills]"));
        assert!(prompt.contains("Test skill"));
    }

    #[test]
    fn test_general_agent_composed_system_prompt_with_empty_skill() {
        let agent = GeneralAgent::new().with_skill_prompt("   ");
        let prompt = agent.composed_system_prompt();
        assert!(!prompt.contains("[Enabled Skills]"));
    }

    #[test]
    fn test_general_agent_preferred_model() {
        let agent = GeneralAgent::new().with_model("claude-3");
        assert_eq!(agent.preferred_model(), Some("claude-3".to_string()));
    }
}
