use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use opencode_llm::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeFacadeStatus {
    Idle,
    Busy,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFacadeResponse {
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub accepted: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<opencode_core::Session>,
}

/// Unique identifier for a trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub Uuid);

impl TraceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "trace:{}", self.0)
    }
}

/// Unique identifier for a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeFacadeTaskId(pub Uuid);

impl RuntimeFacadeTaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RuntimeFacadeTaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RuntimeFacadeTaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task:{}", self.0)
    }
}

/// Kind of executable work a task represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// A subagent delegated task.
    Subagent,
    /// A direct tool execution task.
    ToolExecution,
    /// A shell command execution task.
    Command,
    /// A context building task.
    ContextBuild,
    /// A validation task.
    Validation,
    /// A general agent task (default).
    #[default]
    Agent,
}

/// Status of a runtime task, following the design doc state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFacadeTaskStatus {
    /// Task has been created but not yet started.
    #[default]
    Pending,
    /// Task is being prepared (context, resources, etc.).
    Preparing,
    /// Task is actively executing.
    Running,
    /// Task is waiting for user permission (e.g., tool approval).
    WaitingForPermission,
    /// Task is being cancelled.
    Cancelling,
    /// Task completed successfully.
    Completed,
    /// Task failed during execution.
    Failed,
    /// Task was cancelled by the delegator or user.
    Cancelled,
}

/// A generalized executable unit of work managed by the runtime.
///
/// This supersedes the delegation-specific Task in agent/delegation.rs
/// by being agnostic to the execution mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFacadeTask {
    /// Unique task identifier.
    pub id: RuntimeFacadeTaskId,
    /// The session this task belongs to.
    pub session_id: Uuid,
    /// The turn this task belongs to.
    pub turn_id: Uuid,
    /// Optional parent task (for subagent-coordinated tasks).
    pub parent_task_id: Option<RuntimeFacadeTaskId>,
    /// Kind of work this task performs.
    pub kind: TaskKind,
    /// Human-readable description.
    pub description: String,
    /// Current status.
    pub status: RuntimeFacadeTaskStatus,
    /// Token that can be signaled to cancel the task.
    #[serde(skip)]
    pub cancellation_token: CancellationToken,
    /// Trace identifier for this task's execution.
    pub trace_id: TraceId,
    /// When the task was created.
    pub created_at: DateTime<Utc>,
    /// When the task started executing.
    pub started_at: Option<DateTime<Utc>>,
    /// When the task reached a terminal state.
    pub completed_at: Option<DateTime<Utc>>,
}

impl RuntimeFacadeTask {
    /// Create a new runtime task.
    pub fn new(
        session_id: Uuid,
        turn_id: Uuid,
        kind: TaskKind,
        description: String,
        parent_task_id: Option<RuntimeFacadeTaskId>,
    ) -> Self {
        Self {
            id: RuntimeFacadeTaskId::new(),
            session_id,
            turn_id,
            parent_task_id,
            kind,
            description,
            status: RuntimeFacadeTaskStatus::Pending,
            cancellation_token: CancellationToken::new(),
            trace_id: TraceId::new(),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Mark the task as preparing.
    pub fn mark_preparing(&mut self) {
        self.status = RuntimeFacadeTaskStatus::Preparing;
    }

    /// Mark the task as started.
    pub fn mark_started(&mut self) {
        self.status = RuntimeFacadeTaskStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark the task as waiting for permission.
    pub fn mark_waiting_for_permission(&mut self) {
        self.status = RuntimeFacadeTaskStatus::WaitingForPermission;
    }

    /// Mark the task as cancelling.
    pub fn mark_cancelling(&mut self) {
        self.status = RuntimeFacadeTaskStatus::Cancelling;
    }

    /// Mark the task as completed.
    pub fn mark_completed(&mut self) {
        self.status = RuntimeFacadeTaskStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark the task as failed.
    pub fn mark_failed(&mut self) {
        self.status = RuntimeFacadeTaskStatus::Failed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark the task as cancelled.
    pub fn mark_cancelled(&mut self) {
        self.status = RuntimeFacadeTaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Request cancellation of this task.
    pub fn request_cancellation(&self) {
        self.cancellation_token.cancel();
    }

    /// Check if the task is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            RuntimeFacadeTaskStatus::Completed | RuntimeFacadeTaskStatus::Failed | RuntimeFacadeTaskStatus::Cancelled
        )
    }

    /// Check if the task can be cancelled.
    pub fn can_cancel(&self) -> bool {
        matches!(
            self.status,
            RuntimeFacadeTaskStatus::Pending
                | RuntimeFacadeTaskStatus::Preparing
                | RuntimeFacadeTaskStatus::Running
                | RuntimeFacadeTaskStatus::WaitingForPermission
        )
    }
}
