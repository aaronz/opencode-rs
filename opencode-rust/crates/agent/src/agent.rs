use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session};
use opencode_llm::provider_abstraction::ReasoningBudget;
use opencode_llm::provider::{EventCallback, LlmEvent};
use opencode_llm::Provider;
use opencode_tools::ToolRegistry;
use serde::{Deserialize, Serialize};

pub mod sealed {
    pub trait Sealed {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Build,
    Plan,
    General,
    Explore,
    Compaction,
    Title,
    Summary,
    Review,
    Refactor,
    Debug,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Build => write!(f, "build"),
            AgentType::Plan => write!(f, "plan"),
            AgentType::General => write!(f, "general"),
            AgentType::Explore => write!(f, "explore"),
            AgentType::Compaction => write!(f, "compaction"),
            AgentType::Title => write!(f, "title"),
            AgentType::Summary => write!(f, "summary"),
            AgentType::Review => write!(f, "review"),
            AgentType::Refactor => write!(f, "refactor"),
            AgentType::Debug => write!(f, "debug"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

#[async_trait]
pub trait Agent: Send + Sync + sealed::Sealed {
    fn agent_type(&self) -> AgentType;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn can_execute_tools(&self) -> bool;
    fn can_write_files(&self) -> bool;
    fn can_run_commands(&self) -> bool;
    /// Returns true if the agent should appear in standard agent lists (UI, selection menus, etc.).
    /// Hidden agents (return false) still execute normally but don't appear in visible listings.
    fn is_visible(&self) -> bool {
        true
    }

    async fn run(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError>;

    async fn run_streaming(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &ToolRegistry,
        mut events: EventCallback,
    ) -> Result<AgentResponse, OpenCodeError> {
        let response = self.run(session, provider, tools).await?;
        events(LlmEvent::TextChunk(response.content.clone()));
        events(LlmEvent::Done);
        Ok(response)
    }

    fn preferred_model(&self) -> Option<String> {
        None
    }

    fn preferred_variant(&self) -> Option<String> {
        None
    }

    fn preferred_reasoning_budget(&self) -> Option<ReasoningBudget> {
        None
    }
}

pub fn messages_to_llm_format(messages: &[Message]) -> Vec<opencode_llm::ChatMessage> {
    messages
        .iter()
        .map(opencode_llm::ChatMessage::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Build.to_string(), "build");
        assert_eq!(AgentType::Plan.to_string(), "plan");
        assert_eq!(AgentType::General.to_string(), "general");
        assert_eq!(AgentType::Explore.to_string(), "explore");
        assert_eq!(AgentType::Compaction.to_string(), "compaction");
        assert_eq!(AgentType::Title.to_string(), "title");
        assert_eq!(AgentType::Summary.to_string(), "summary");
        assert_eq!(AgentType::Review.to_string(), "review");
        assert_eq!(AgentType::Refactor.to_string(), "refactor");
        assert_eq!(AgentType::Debug.to_string(), "debug");
    }

    #[test]
    fn test_agent_type_serialization() {
        let agent_type = AgentType::Build;
        let json = serde_json::to_string(&agent_type).unwrap();
        assert_eq!(json, "\"build\"");
        let deserialized: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, AgentType::Build);
    }

    #[test]
    fn test_tool_call_serialization() {
        let tool_call = ToolCall {
            id: "test-id".to_string(),
            name: "read".to_string(),
            arguments: serde_json::json!({"path": "/test"}),
        };
        let json = serde_json::to_string(&tool_call).unwrap();
        assert!(json.contains("\"name\":\"read\""));
        let deserialized: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "read");
    }

    #[test]
    fn test_agent_response_serialization() {
        let response = AgentResponse {
            content: "Hello".to_string(),
            tool_calls: vec![ToolCall {
                id: "test-id".to_string(),
                name: "test".to_string(),
                arguments: serde_json::json!({}),
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"content\":\"Hello\""));
        let deserialized: AgentResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.content, "Hello");
        assert_eq!(deserialized.tool_calls.len(), 1);
    }

    #[test]
    fn test_tool_call_clone() {
        let tool_call = ToolCall {
            id: "test-id".to_string(),
            name: "read".to_string(),
            arguments: serde_json::json!({"path": "/test"}),
        };
        let cloned = tool_call.clone();
        assert_eq!(cloned.name, tool_call.name);
        assert_eq!(cloned.arguments, tool_call.arguments);
    }

    #[test]
    fn test_agent_response_clone() {
        let response = AgentResponse {
            content: "Hello".to_string(),
            tool_calls: vec![],
        };
        let cloned = response.clone();
        assert_eq!(cloned.content, response.content);
    }

    #[test]
    fn test_messages_to_llm_format_empty() {
        let messages: Vec<Message> = vec![];
        let result = messages_to_llm_format(&messages);
        assert!(result.is_empty());
    }

    #[test]
    fn test_messages_to_llm_format_with_messages() {
        use opencode_core::Message;
        let messages = vec![
            Message::user("Hello".to_string()),
            Message::assistant("Hi there".to_string()),
        ];
        let result = messages_to_llm_format(&messages);
        assert_eq!(result.len(), 2);
    }
}
