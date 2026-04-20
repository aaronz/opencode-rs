# agent.md — Agent Module

## Module Overview

- **Crate**: `opencode-agent`
- **Source**: `crates/agent/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Defines the `Agent` trait, concrete agent implementations (Build, Plan, Explore, Refactor, Review, Debug, etc.), `AgentRuntime` for agent execution, event emission, and task delegation.

---

## Crate Layout

```
crates/agent/src/
├── lib.rs              ← Re-exports
├── agent.rs            ← Agent trait, AgentType, ToolCall, AgentResponse
├── build_agent.rs      ← BuildAgent implementation
├── debug_agent.rs      ← DebugAgent implementation
├── explore_agent.rs    ← ExploreAgent implementation
├── general_agent.rs    ← GeneralAgent implementation
├── plan_agent.rs        ← PlanAgent implementation
├── refactor_agent.rs   ← RefactorAgent implementation
├── review_agent.rs     ← ReviewAgent implementation
├── runtime.rs          ← AgentRuntime, RuntimeConfig, RuntimeError, PrimaryAgentTracker
├── events.rs           ← AgentEvent, AgentEventEmitter, BroadcastEventEmitter
├── delegation.rs       ← Task, TaskDelegate, DelegationError
├── skills_actions.rs   ← SkillsActions
└── system_agents.rs    ← CompactionAgent, SummaryAgent, TitleAgent
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
tokio = { version = "1.45", features = ["sync", "rt-multi-thread"] }

opencode-core = { path = "../core" }
opencode-llm = { path = "../llm" }
opencode-tools = { path = "../tools" }
opencode-permission = { path = "../permission" }
```

**Public exports from lib.rs**:
```rust
pub use agent::{messages_to_llm_format, Agent, AgentResponse, AgentType, ToolCall};
pub use build_agent::BuildAgent;
pub use debug_agent::DebugAgent;
pub use delegation::{
    DelegationError, DelegationStatusSummary, Task, TaskDelegate, TaskId, TaskProgress, TaskResult,
    TaskStatus,
};
pub use events::{AgentEvent, AgentEventEmitter, BroadcastEventEmitter};
pub use explore_agent::ExploreAgent;
pub use general_agent::GeneralAgent;
pub use plan_agent::PlanAgent;
pub use refactor_agent::RefactorAgent;
pub use review_agent::ReviewAgent;
pub use runtime::{
    AgentRuntime, PrimaryAgentState, PrimaryAgentTracker, RuntimeConfig, RuntimeError,
    SubagentError, SubagentResult,
};
pub use skills_actions::SkillsActions;
pub use system_agents::{CompactionAgent, SummaryAgent, TitleAgent};
```

---

## Core Types

### AgentType

```rust
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

impl std::fmt::Display for AgentType { ... }  // outputs lowercase string
```

### ToolCall

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}
```

### AgentResponse

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}
```

### Agent Trait

```rust
pub mod sealed {
    pub trait Sealed {}
}

#[async_trait]
pub trait Agent: Send + Sync + sealed::Sealed {
    fn agent_type(&self) -> AgentType;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn can_execute_tools(&self) -> bool;
    fn can_write_files(&self) -> bool;
    fn can_run_commands(&self) -> bool;
    fn is_visible(&self) -> bool { true }  // false = hidden from listings

    async fn run(
        &self,
        session: &mut Session,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, OpenCodeError>;

    fn preferred_model(&self) -> Option<String> { None }
    fn preferred_variant(&self) -> Option<String> { None }
    fn preferred_reasoning_budget(&self) -> Option<ReasoningBudget> { None }
}

pub fn messages_to_llm_format(messages: &[Message]) -> Vec<opencode_llm::ChatMessage>
```

---

## Runtime Types

### RuntimeConfig

```rust
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub max_iterations: usize,
    pub max_tool_results_per_iteration: usize,
    pub permission_scope: AgentPermissionScope,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            max_tool_results_per_iteration: 10,
            permission_scope: AgentPermissionScope::Full,
        }
    }
}
```

### RuntimeError

```rust
#[derive(Debug, Clone)]
pub enum RuntimeError {
    SessionNotActive,
    MaxIterationsExceeded { limit: usize },
    NoSuchAgent { agent_type: AgentType },
    ToolExecutionFailed { tool: String, reason: String },
    PermissionDenied { tool: String },
    SessionLocked,
    MultiplePrimaryAgents { current: AgentType, attempted: AgentType },
    AgentTransitionInProgress { current: AgentType },
    NoActivePrimaryAgent,
}

impl std::fmt::Display for RuntimeError { ... }
impl std::error::Error for RuntimeError { ... }
```

### PrimaryAgentState

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimaryAgentState {
    Inactive,
    Running,
    Transitioning,
}
```

### PrimaryAgentTracker

```rust
#[derive(Debug, Clone)]
pub struct PrimaryAgentTracker {
    pub state: PrimaryAgentState,
    pub agent_type: Option<AgentType>,
}

impl PrimaryAgentTracker {
    pub fn new() -> Self;
    pub fn new_active(agent_type: AgentType) -> Self;
    pub fn activate(&mut self, agent_type: AgentType) -> Result<(), RuntimeError>;
    pub fn begin_transition(&mut self) -> Result<AgentType, RuntimeError>;
    pub fn complete_transition(&mut self, new_type: AgentType);
    pub fn deactivate(&mut self) -> Result<AgentType, RuntimeError>;
    pub fn is_active(&self) -> bool;
    pub fn active_type(&self) -> Option<AgentType>;
}
```

### AgentRuntime

```rust
pub struct AgentRuntime {
    session: Arc<RwLock<Session>>,
    config: RuntimeConfig,
    primary_tracker: PrimaryAgentTracker,
}

impl AgentRuntime {
    pub fn new(session: Session, agent_type: AgentType) -> Self;
    pub fn with_config(session: Session, agent_type: AgentType, config: RuntimeConfig) -> Self;

    pub async fn run_loop<A: Agent>(
        &self,
        agent: &A,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<AgentResponse, RuntimeError>;

    pub async fn switch_primary_agent(&mut self, new_type: AgentType) -> Result<(), RuntimeError>;
    pub fn active_agent(&self) -> Option<AgentType>;
    pub fn is_primary_agent_active(&self) -> bool;
    pub fn primary_agent_state(&self) -> PrimaryAgentState;
    pub async fn deactivate_primary_agent(&mut self) -> Result<AgentType, RuntimeError>;
    pub async fn activate_primary_agent(&mut self, new_type: AgentType) -> Result<(), RuntimeError>;
    pub async fn session(&self) -> Session;
    pub async fn into_session(self) -> Session;

    pub async fn invoke_subagent<A: Agent>(
        &self,
        agent: &A,
        context: Vec<Message>,
        provider: &dyn Provider,
        tools: &ToolRegistry,
    ) -> Result<SubagentResult, RuntimeError>;

    pub fn get_permission_scope(&self) -> AgentPermissionScope;
    pub fn with_permission_scope(self, scope: AgentPermissionScope) -> Self;
}
```

### SubagentResult and SubagentError

```rust
#[derive(Debug, Clone)]
pub struct SubagentResult {
    pub response: AgentResponse,
    pub child_session_id: Uuid,
    pub agent_type: AgentType,
    pub effective_permission_scope: AgentPermissionScope,
}

#[derive(Debug, Clone)]
pub enum SubagentError {
    SessionNotActive,
    SubagentExecutionFailed { reason: String },
    ParentContextModified,
    ForkFailed { reason: String },
}
```

---

## Event Types

```rust
// From events.rs
pub enum AgentEvent { ... }
pub trait AgentEventEmitter { ... }
pub struct BroadcastEventEmitter { ... }
```

---

## Delegation Types

```rust
// From delegation.rs
pub struct Task { ... }
pub struct TaskDelegate { ... }
pub struct TaskId(pub String);
pub struct TaskProgress { ... }
pub struct TaskResult { ... }
pub enum TaskStatus { ... }
pub struct DelegationStatusSummary { ... }
pub enum DelegationError { ... }
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-agent` |
|---|---|
| `opencode-server` | `AgentRuntime`, `AgentEventEmitter` to run agents |
| `opencode-core` | `Agent` trait, `AgentType`, `AgentResponse` |
| `opencode-tui` | `AgentType`, `AgentEvent` for UI updates |
| `opencode-cli` | `Agent` implementations for CLI mode |

**Dependencies of `opencode-agent`**:
| Crate | What `opencode-agent` uses |
|---|---|
| `opencode-core` | `Session`, `Message`, `OpenCodeError` |
| `opencode-llm` | `Provider`, `ReasoningBudget`, `ChatMessage` |
| `opencode-tools` | `ToolRegistry`, `ToolContext` |
| `opencode-permission` | `AgentPermissionScope` |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // AgentType display and serialization
    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::Build.to_string(), "build");
        assert_eq!(AgentType::Plan.to_string(), "plan");
    }

    #[test]
    fn test_agent_type_serialization() {
        let agent_type = AgentType::Build;
        let json = serde_json::to_string(&agent_type).unwrap();
        assert_eq!(json, "\"build\"");
    }

    // PrimaryAgentTracker invariants
    #[test]
    fn test_primary_agent_tracker_activate() {
        let mut tracker = PrimaryAgentTracker::new();
        assert!(tracker.activate(AgentType::Build).is_ok());
        assert!(tracker.is_active());
    }

    #[test]
    fn test_primary_agent_tracker_cannot_activate_second() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        let result = tracker.activate(AgentType::Plan);
        assert!(result.is_err());
        match result {
            Err(RuntimeError::MultiplePrimaryAgents { current, attempted }) => {
                assert_eq!(current, AgentType::Build);
                assert_eq!(attempted, AgentType::Plan);
            }
            _ => panic!("expected MultiplePrimaryAgents"),
        }
    }

    #[test]
    fn test_primary_agent_tracker_transition() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        let current = tracker.begin_transition().unwrap();
        assert_eq!(current, AgentType::Build);
        assert_eq!(tracker.state, PrimaryAgentState::Transitioning);
        tracker.complete_transition(AgentType::Plan);
        assert_eq!(tracker.state, PrimaryAgentState::Running);
    }

    #[test]
    fn test_primary_agent_tracker_deactivate() {
        let mut tracker = PrimaryAgentTracker::new();
        tracker.activate(AgentType::Build).unwrap();
        let deactivated = tracker.deactivate().unwrap();
        assert_eq!(deactivated, AgentType::Build);
        assert!(!tracker.is_active());
    }

    // AgentRuntime tests
    #[tokio::test]
    async fn test_runtime_starts_with_exactly_one_primary_agent() {
        let session = Session::default();
        let runtime = AgentRuntime::new(session, AgentType::Build);
        assert!(runtime.is_primary_agent_active());
        assert_eq!(runtime.active_agent(), Some(AgentType::Build));
    }

    #[tokio::test]
    async fn test_runtime_switch_primary_agent() {
        let session = Session::default();
        let mut runtime = AgentRuntime::new(session, AgentType::Build);
        runtime.switch_primary_agent(AgentType::Plan).await.unwrap();
        assert_eq!(runtime.active_agent(), Some(AgentType::Plan));
    }

    #[tokio::test]
    async fn test_runtime_invoke_subagent_inherits_parent_permissions() {
        // MockSubagent + runtime.invoke_subagent() → effective_permission_scope
    }

    // ToolCall / AgentResponse serialization
    #[test]
    fn test_tool_call_serialization() {
        let tc = ToolCall { name: "read".into(), arguments: serde_json::json!({"path": "/test"}) };
        let json = serde_json::to_string(&tc).unwrap();
        assert!(json.contains("\"name\":\"read\""));
    }

    #[test]
    fn test_agent_response_serialization() {
        let resp = AgentResponse { content: "Hello".into(), tool_calls: vec![] };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_messages_to_llm_format() {
        let messages = vec![
            Message::user("Hello".into()),
            Message::assistant("Hi".into()),
        ];
        let result = messages_to_llm_format(&messages);
        assert_eq!(result.len(), 2);
    }
}
```

---

## Agent Implementations (concrete types)

Each concrete agent struct implements `Agent` via `#[async_trait]` and `sealed::Sealed`:

```rust
// BuildAgent — full access, can write files and run commands
pub struct BuildAgent;

// PlanAgent — read-only planning agent
pub struct PlanAgent;

// GeneralAgent — general purpose, can execute tools
pub struct GeneralAgent;

// ExploreAgent — code exploration/search
pub struct ExploreAgent;

// DebugAgent — debugging assistance
pub struct DebugAgent;

// RefactorAgent — refactoring assistance
pub struct RefactorAgent;

// ReviewAgent — code review
pub struct ReviewAgent;
```

System agents (not user-visible):
```rust
pub struct CompactionAgent;  // Session compaction/summarization
pub struct SummaryAgent;     // Generate session summaries
pub struct TitleAgent;       // Generate session titles
```

---

## Usage Example

```rust
use opencode_agent::{AgentRuntime, BuildAgent, AgentType};
use opencode_llm::Provider;
use opencode_tools::ToolRegistry;
use opencode_core::Session;

async fn run_build_agent() -> Result<(), RuntimeError> {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);
    
    let agent = BuildAgent;
    let provider = /* get provider */;
    let tools = ToolRegistry::new();
    
    let response = runtime.run_loop(&agent, &provider, &tools).await?;
    println!("Agent response: {}", response.content);
    
    Ok(())
}

async fn invoke_subagent() -> Result<SubagentResult, RuntimeError> {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);
    
    let explore_agent = ExploreAgent;
    let context = vec![Message::user("Explain the codebase structure".into())];
    
    runtime.invoke_subagent(&explore_agent, context, &provider, &tools).await
}
```
