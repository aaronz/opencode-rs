use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SessionState {
    #[default]
    Idle,
    Thinking,
    AwaitingPermission,
    ExecutingTool,
    Streaming,
    ApplyingChanges,
    Verifying,
    Summarizing,
    Aborted,
    Error,
    Completed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransitionError {
    pub from: SessionState,
    pub to: SessionState,
}

impl std::fmt::Display for StateTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid state transition from {:?} to {:?}",
            self.from, self.to
        )
    }
}

impl std::error::Error for StateTransitionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionEvent {
    PromptReceived,
    ToolExecutionRequested,
    PermissionGranted,
    PermissionDenied,
    ToolExecutionCompleted,
    StreamStarted,
    StreamCompleted,
    ChangesApplied,
    VerificationCompleted,
    ErrorOccurred,
    SummarizeRequested,
    SummarizeCompleted,
    AbortRequested,
    PauseRequested,
    ResumeRequested,
}
