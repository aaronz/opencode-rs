use async_trait::async_trait;
use opencode_core::{Message, OpenCodeError, Session};
use opencode_llm::Provider;
use opencode_tools::ToolRegistry;
use serde::{Deserialize, Serialize};

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
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn can_execute_tools(&self) -> bool;
    fn can_write_files(&self) -> bool;
    fn can_run_commands(&self) -> bool;

    async fn run(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError>;
}

pub fn messages_to_llm_format(messages: &[Message]) -> Vec<opencode_llm::ChatMessage> {
    messages
        .iter()
        .map(opencode_llm::ChatMessage::from)
        .collect()
}
