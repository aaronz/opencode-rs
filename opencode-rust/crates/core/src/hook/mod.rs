mod engine;
mod types;

pub use engine::{HookEngine, HookEvent, HookResult};
pub use types::{HookAction, HookDefinition, HookFailurePolicy, HookId, HookTrigger};

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookPoint {
    BeforeSessionStart,
    AfterSessionStart,
    BeforeContextBuild,
    AfterContextBuild,
    BeforeLlmRequest,
    AfterLlmResponse,
    BeforeToolExecution,
    AfterToolExecution,
    AfterFilePatch,
    AfterCommandCompleted,
    BeforeValidation,
    AfterValidation,
    OnTaskFailed,
    BeforeSessionPersist,
}

impl fmt::Display for HookPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HookPoint::BeforeSessionStart => write!(f, "before_session_start"),
            HookPoint::AfterSessionStart => write!(f, "after_session_start"),
            HookPoint::BeforeContextBuild => write!(f, "before_context_build"),
            HookPoint::AfterContextBuild => write!(f, "after_context_build"),
            HookPoint::BeforeLlmRequest => write!(f, "before_llm_request"),
            HookPoint::AfterLlmResponse => write!(f, "after_llm_response"),
            HookPoint::BeforeToolExecution => write!(f, "before_tool_execution"),
            HookPoint::AfterToolExecution => write!(f, "after_tool_execution"),
            HookPoint::AfterFilePatch => write!(f, "after_file_patch"),
            HookPoint::AfterCommandCompleted => write!(f, "after_command_completed"),
            HookPoint::BeforeValidation => write!(f, "before_validation"),
            HookPoint::AfterValidation => write!(f, "after_validation"),
            HookPoint::OnTaskFailed => write!(f, "on_task_failed"),
            HookPoint::BeforeSessionPersist => write!(f, "before_session_persist"),
        }
    }
}
