use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitUserInput {
    pub session_id: Option<String>,
    pub workspace: Option<PathBuf>,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAgentCommand {
    pub session: opencode_core::Session,
    pub agent_type: opencode_agent::AgentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteShellCommand {
    pub command: String,
    pub timeout_secs: Option<u64>,
    pub workdir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextCommand {
    Inspect {
        session_id: Option<String>,
    },
    Explain {
        session_id: Option<String>,
    },
    Dump {
        turn_id: String,
        session_id: Option<String>,
    },
    Why {
        file: String,
        session_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskControlCommand {
    Cancel { task_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    pub request_id: String,
    pub granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeFacadeCommand {
    SubmitUserInput(SubmitUserInput),
    #[serde(skip)]
    RunAgent(Box<RunAgentCommand>),
    ExecuteShell(ExecuteShellCommand),
    Context(ContextCommand),
    TaskControl(TaskControlCommand),
    PermissionResponse(PermissionResponse),
}
