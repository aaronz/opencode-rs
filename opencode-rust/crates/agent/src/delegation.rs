//! Task delegation mechanism for primary agents to delegate work to subagents.
//!
//! This module provides:
//! - Task struct with ownership, status tracking, and progress reporting
//! - TaskDelegate for delegating tasks to subagents with result handoff
//! - Integration with AgentRuntime's invoke_subagent

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use opencode_core::Message;
use opencode_llm::Provider;
use opencode_tools::ToolRegistry;

use crate::{Agent, AgentResponse, AgentRuntime, AgentType, SubagentResult};

/// Unique identifier for a delegated task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task:{}", self.0)
    }
}

/// Status of a delegated task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task has been created but not yet started.
    Pending,
    /// Task is currently being executed by a subagent.
    InProgress,
    /// Task completed successfully.
    Completed,
    /// Task failed during execution.
    Failed,
    /// Task was cancelled by the delegator.
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Progress update from a task during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// The task this progress belongs to.
    pub task_id: TaskId,
    /// Current status.
    pub status: TaskStatus,
    /// Human-readable progress message.
    pub message: String,
    /// Optional numeric progress (0-100).
    pub progress_percent: Option<u8>,
    /// When this progress was recorded.
    pub timestamp: DateTime<Utc>,
}

impl TaskProgress {
    /// Create a new progress update.
    pub fn new(task_id: TaskId, status: TaskStatus, message: impl Into<String>) -> Self {
        Self {
            task_id,
            status,
            message: message.into(),
            progress_percent: None,
            timestamp: Utc::now(),
        }
    }

    /// Create a progress update with percentage.
    pub fn with_progress(
        task_id: TaskId,
        status: TaskStatus,
        message: impl Into<String>,
        percent: u8,
    ) -> Self {
        Self {
            task_id,
            status,
            message: message.into(),
            progress_percent: Some(percent.min(100)),
            timestamp: Utc::now(),
        }
    }

    /// Create a pending progress update.
    pub fn pending(task_id: TaskId, message: impl Into<String>) -> Self {
        Self::new(task_id, TaskStatus::Pending, message)
    }

    /// Create an in-progress progress update.
    pub fn in_progress(task_id: TaskId, message: impl Into<String>) -> Self {
        Self::new(task_id, TaskStatus::InProgress, message)
    }

    /// Create a completed progress update.
    pub fn completed(task_id: TaskId, message: impl Into<String>) -> Self {
        Self::new(task_id, TaskStatus::Completed, message)
    }

    /// Create a failed progress update.
    pub fn failed(task_id: TaskId, message: impl Into<String>) -> Self {
        Self::new(task_id, TaskStatus::Failed, message)
    }
}

/// Final result from a delegated task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// The task ID.
    pub task_id: TaskId,
    /// The agent type that executed the task.
    pub agent_type: AgentType,
    /// The session ID of the child context.
    pub child_session_id: Uuid,
    /// The response from the subagent.
    pub response: AgentResponse,
    /// Final status (Completed or Failed).
    pub status: TaskStatus,
    /// When the task started.
    pub started_at: DateTime<Utc>,
    /// When the task completed.
    pub completed_at: DateTime<Utc>,
    /// Optional summary of the work done.
    pub summary: Option<String>,
}

impl TaskResult {
    /// Create a successful task result.
    pub fn success(
        task_id: TaskId,
        subagent_result: SubagentResult,
        started_at: DateTime<Utc>,
    ) -> Self {
        Self {
            task_id,
            agent_type: subagent_result.agent_type,
            child_session_id: subagent_result.child_session_id,
            response: subagent_result.response,
            status: TaskStatus::Completed,
            started_at,
            completed_at: Utc::now(),
            summary: None,
        }
    }

    /// Create a failed task result.
    pub fn failure(
        task_id: TaskId,
        agent_type: AgentType,
        child_session_id: Uuid,
        started_at: DateTime<Utc>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            task_id,
            agent_type,
            child_session_id,
            response: AgentResponse {
                content: error_message.into(),
                tool_calls: Vec::new(),
            },
            status: TaskStatus::Failed,
            started_at,
            completed_at: Utc::now(),
            summary: None,
        }
    }

    /// Set the summary for this result.
    #[must_use]
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }
}

/// Represents a delegatable task with ownership and tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier.
    pub id: TaskId,
    /// Human-readable task description.
    pub description: String,
    /// The agent type this task should be delegated to.
    pub agent_type: AgentType,
    /// The delegator's agent type (who delegated this task).
    pub delegator_type: AgentType,
    /// Current status.
    pub status: TaskStatus,
    /// Progress history.
    #[serde(default)]
    pub progress_history: Vec<TaskProgress>,
    /// Context messages to provide to the subagent.
    #[serde(default)]
    pub context: Vec<Message>,
    /// When the task was created.
    pub created_at: DateTime<Utc>,
    /// When the task was started (first delegated).
    pub started_at: Option<DateTime<Utc>>,
    /// When the task completed (final result).
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    /// Create a new task for delegation.
    pub fn new(
        description: impl Into<String>,
        agent_type: AgentType,
        delegator_type: AgentType,
        context: Vec<Message>,
    ) -> Self {
        Self {
            id: TaskId::new(),
            description: description.into(),
            agent_type,
            delegator_type,
            status: TaskStatus::Pending,
            progress_history: Vec::new(),
            context,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Record a progress update.
    pub fn record_progress(&mut self, progress: TaskProgress) {
        self.status = progress.status;
        self.progress_history.push(progress);
    }

    /// Mark the task as started.
    pub fn mark_started(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(Utc::now());
        self.progress_history.push(TaskProgress::in_progress(
            self.id,
            format!("Task started by {} agent", self.agent_type),
        ));
    }

    /// Mark the task as completed with result.
    pub fn mark_completed(&mut self, result: &TaskResult) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        let content_preview = result.response.content.chars().take(50).collect::<String>();
        self.progress_history.push(TaskProgress::completed(
            self.id,
            format!("Task completed with result: {}", content_preview),
        ));
    }

    /// Mark the task as failed.
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.progress_history
            .push(TaskProgress::failed(self.id, error));
    }

    /// Get the latest progress update.
    pub fn latest_progress(&self) -> Option<&TaskProgress> {
        self.progress_history.last()
    }

    /// Check if the task is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }

    /// Get task duration if completed.
    pub fn duration(&self) -> Option<std::time::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(
                end.signed_duration_since(start)
                    .to_std()
                    .unwrap_or_default(),
            ),
            _ => None,
        }
    }
}

/// Errors that can occur during task delegation.
#[derive(Debug, Clone)]
pub enum DelegationError {
    /// The task is not in a state that allows delegation (e.g., already completed).
    TaskNotDelegatable { task_id: TaskId, status: TaskStatus },
    /// The subagent execution failed.
    SubagentExecutionFailed { task_id: TaskId, reason: String },
    /// The parent session was modified during subagent execution.
    ParentContextModified { task_id: TaskId },
    /// No active primary agent to perform delegation.
    NoActivePrimaryAgent,
    /// Task ownership violation - tried to manipulate task by non-owner.
    OwnershipViolation {
        task_id: TaskId,
        expected_delegator: AgentType,
    },
}

impl std::fmt::Display for DelegationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DelegationError::TaskNotDelegatable { task_id, status } => {
                write!(
                    f,
                    "task {} is not delegatable (status: {})",
                    task_id, status
                )
            }
            DelegationError::SubagentExecutionFailed { task_id, reason } => {
                write!(f, "task {} subagent execution failed: {}", task_id, reason)
            }
            DelegationError::ParentContextModified { task_id } => {
                write!(
                    f,
                    "parent context was modified during task {} execution",
                    task_id
                )
            }
            DelegationError::NoActivePrimaryAgent => {
                write!(f, "no active primary agent for delegation")
            }
            DelegationError::OwnershipViolation {
                task_id,
                expected_delegator,
            } => {
                write!(
                    f,
                    "task {} ownership violation - expected delegator: {}",
                    task_id, expected_delegator
                )
            }
        }
    }
}

impl std::error::Error for DelegationError {}

/// Manages task delegation and tracks delegated tasks.
///
/// This struct maintains:
/// - Active tasks being executed by subagents
/// - Completed tasks for history
///
/// Usage:
/// ```ignore
/// let mut delegate = TaskDelegate::new();
/// let task = Task::new("description", AgentType::Explore, AgentType::Build, vec![]);
/// let result = delegate.delegate_task(task, &explore_agent, &provider, &tools, &runtime).await?;
/// ```
#[derive(Debug, Default)]
pub struct TaskDelegate {
    /// Currently delegated tasks (active).
    active_tasks: Vec<Task>,
    /// Completed tasks (for history).
    completed_tasks: Vec<Task>,
}

impl TaskDelegate {
    /// Create a new TaskDelegate.
    pub fn new() -> Self {
        Self {
            active_tasks: Vec::new(),
            completed_tasks: Vec::new(),
        }
    }

    /// Delegate a task to a subagent.
    ///
    /// This method:
    /// 1. Validates the task is delegatable
    /// 2. Records task start
    /// 3. Invokes the subagent via runtime
    /// 4. Records completion/failure
    /// 5. Returns the result
    pub async fn delegate_task<A: Agent>(
        &mut self,
        task: Task,
        agent: &A,
        provider: &dyn Provider,
        tools: &ToolRegistry,
        runtime: &AgentRuntime,
    ) -> Result<TaskResult, DelegationError> {
        // Validate task is delegatable
        if task.is_terminal() {
            return Err(DelegationError::TaskNotDelegatable {
                task_id: task.id,
                status: task.status,
            });
        }

        // Clone task for mutation
        let mut task = task;
        task.mark_started();

        // Store started task
        let task_id = task.id;
        self.active_tasks.push(task.clone());

        let started_at = task.started_at.unwrap_or_else(Utc::now);

        // Invoke subagent
        let result = runtime
            .invoke_subagent(agent, task.context.clone(), provider, tools)
            .await;

        // Process result
        let task_result = match result {
            Ok(subagent_result) => {
                let result = TaskResult::success(task.id, subagent_result, started_at);

                // Update task status
                if let Some(active_task) = self.active_tasks.iter_mut().find(|t| t.id == task_id) {
                    active_task.mark_completed(&result);
                }

                result
            }
            Err(e) => {
                let error_message = e.to_string();

                // Update task status
                if let Some(active_task) = self.active_tasks.iter_mut().find(|t| t.id == task_id) {
                    active_task.mark_failed(&error_message);
                }

                TaskResult::failure(
                    task.id,
                    task.agent_type,
                    Uuid::nil(),
                    started_at,
                    error_message,
                )
            }
        };

        // Move task to completed
        if let Some(pos) = self.active_tasks.iter().position(|t| t.id == task_id) {
            let completed_task = self.active_tasks.remove(pos);
            self.completed_tasks.push(completed_task);
        }

        Ok(task_result)
    }

    /// Report progress for an active task.
    pub fn report_progress(&mut self, progress: TaskProgress) -> Result<(), DelegationError> {
        let task_id = progress.task_id;

        let task = self
            .active_tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or(DelegationError::TaskNotDelegatable {
                task_id,
                status: TaskStatus::Pending,
            })?;

        task.record_progress(progress);
        Ok(())
    }

    /// Get an active task by ID.
    pub fn get_active_task(&self, task_id: TaskId) -> Option<&Task> {
        self.active_tasks.iter().find(|t| t.id == task_id)
    }

    /// Get a mutable active task by ID.
    pub fn get_active_task_mut(&mut self, task_id: TaskId) -> Option<&mut Task> {
        self.active_tasks.iter_mut().find(|t| t.id == task_id)
    }

    /// Get all active tasks.
    pub fn active_tasks(&self) -> &[Task] {
        &self.active_tasks
    }

    /// Get all completed tasks.
    pub fn completed_tasks(&self) -> &[Task] {
        &self.completed_tasks
    }

    /// Get a completed task by ID.
    pub fn get_completed_task(&self, task_id: TaskId) -> Option<&Task> {
        self.completed_tasks.iter().find(|t| t.id == task_id)
    }

    /// Cancel an active task.
    pub fn cancel_task(&mut self, task_id: TaskId) -> Result<(), DelegationError> {
        let task = self
            .active_tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or(DelegationError::TaskNotDelegatable {
                task_id,
                status: TaskStatus::Pending,
            })?;

        task.status = TaskStatus::Cancelled;
        task.completed_at = Some(Utc::now());
        task.progress_history.push(TaskProgress::failed(
            task_id,
            "Task was cancelled by delegator",
        ));

        // Move to completed
        if let Some(pos) = self.active_tasks.iter().position(|t| t.id == task_id) {
            let cancelled_task = self.active_tasks.remove(pos);
            self.completed_tasks.push(cancelled_task);
        }

        Ok(())
    }

    /// Get the count of active tasks.
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.len()
    }

    /// Get the count of completed tasks.
    pub fn completed_task_count(&self) -> usize {
        self.completed_tasks.len()
    }

    /// Clear completed tasks older than the given timestamp.
    pub fn clear_completed_before(&mut self, before: DateTime<Utc>) {
        self.completed_tasks.retain(|t| t.created_at > before);
    }

    /// Get task status summary.
    pub fn status_summary(&self) -> DelegationStatusSummary {
        DelegationStatusSummary {
            active_count: self.active_tasks.len(),
            completed_count: self.completed_tasks.len(),
            pending_count: self
                .active_tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Pending)
                .count(),
            in_progress_count: self
                .active_tasks
                .iter()
                .filter(|t| t.status == TaskStatus::InProgress)
                .count(),
        }
    }
}

/// Summary of delegation status for reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationStatusSummary {
    pub active_count: usize,
    pub completed_count: usize,
    pub pending_count: usize,
    pub in_progress_count: usize,
}

impl std::fmt::Display for DelegationStatusSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DelegationStatus(active={}, completed={}, pending={}, in_progress={})",
            self.active_count, self.completed_count, self.pending_count, self.in_progress_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentResponse;

    #[test]
    fn test_task_id_new() {
        let id = TaskId::new();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new();
        let display = format!("{}", id);
        assert!(display.starts_with("task:"));
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(format!("{}", TaskStatus::Pending), "pending");
        assert_eq!(format!("{}", TaskStatus::InProgress), "in_progress");
        assert_eq!(format!("{}", TaskStatus::Completed), "completed");
        assert_eq!(format!("{}", TaskStatus::Failed), "failed");
        assert_eq!(format!("{}", TaskStatus::Cancelled), "cancelled");
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "Test task",
            AgentType::Explore,
            AgentType::Build,
            vec![Message::user("test")],
        );

        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.description, "Test task");
        assert_eq!(task.agent_type, AgentType::Explore);
        assert_eq!(task.delegator_type, AgentType::Build);
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_mark_started() {
        let mut task = Task::new("Test task", AgentType::Explore, AgentType::Build, vec![]);
        task.mark_started();

        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.started_at.is_some());
        assert_eq!(task.progress_history.len(), 1);
    }

    #[test]
    fn test_task_mark_completed() {
        let mut task = Task::new("Test task", AgentType::Explore, AgentType::Build, vec![]);
        task.mark_started();

        let result = TaskResult {
            task_id: task.id,
            agent_type: task.agent_type,
            child_session_id: Uuid::new_v4(),
            response: AgentResponse {
                content: "Done".to_string(),
                tool_calls: vec![],
            },
            status: TaskStatus::Completed,
            started_at: task.started_at.unwrap(),
            completed_at: Utc::now(),
            summary: None,
        };

        task.mark_completed(&result);

        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_is_terminal() {
        let mut task = Task::new("Test", AgentType::Explore, AgentType::Build, vec![]);
        assert!(!task.is_terminal());

        task.mark_started();
        assert!(!task.is_terminal());

        task.status = TaskStatus::Completed;
        assert!(task.is_terminal());

        let mut task2 = Task::new("Test", AgentType::Explore, AgentType::Build, vec![]);
        task2.status = TaskStatus::Failed;
        assert!(task2.is_terminal());

        let mut task3 = Task::new("Test", AgentType::Explore, AgentType::Build, vec![]);
        task3.status = TaskStatus::Cancelled;
        assert!(task3.is_terminal());
    }

    #[test]
    fn test_task_progress_creation() {
        let task_id = TaskId::new();
        let progress = TaskProgress::new(task_id, TaskStatus::InProgress, "Working on it");

        assert_eq!(progress.task_id, task_id);
        assert_eq!(progress.status, TaskStatus::InProgress);
        assert_eq!(progress.message, "Working on it");
        assert!(progress.progress_percent.is_none());
    }

    #[test]
    fn test_task_progress_with_percentage() {
        let task_id = TaskId::new();
        let progress = TaskProgress::with_progress(task_id, TaskStatus::InProgress, "50% done", 50);

        assert_eq!(progress.progress_percent, Some(50));
    }

    #[test]
    fn test_task_result_success() {
        let task_id = TaskId::new();
        let subagent_result = SubagentResult {
            response: AgentResponse {
                content: "Result".to_string(),
                tool_calls: vec![],
            },
            child_session_id: Uuid::new_v4(),
            agent_type: AgentType::Explore,
            effective_permission_scope: opencode_permission::AgentPermissionScope::ReadOnly,
        };
        let started_at = Utc::now();

        let result = TaskResult::success(task_id, subagent_result, started_at);

        assert_eq!(result.task_id, task_id);
        assert_eq!(result.status, TaskStatus::Completed);
        assert_eq!(result.response.content, "Result");
    }

    #[test]
    fn test_task_result_failure() {
        let task_id = TaskId::new();
        let started_at = Utc::now();

        let result = TaskResult::failure(
            task_id,
            AgentType::Explore,
            Uuid::new_v4(),
            started_at,
            "Something went wrong",
        );

        assert_eq!(result.status, TaskStatus::Failed);
        assert!(result.response.content.contains("Something went wrong"));
    }

    #[test]
    fn test_task_result_with_summary() {
        let task_id = TaskId::new();
        let subagent_result = SubagentResult {
            response: AgentResponse {
                content: "Result".to_string(),
                tool_calls: vec![],
            },
            child_session_id: Uuid::new_v4(),
            agent_type: AgentType::Explore,
            effective_permission_scope: opencode_permission::AgentPermissionScope::ReadOnly,
        };
        let started_at = Utc::now();

        let result = TaskResult::success(task_id, subagent_result, started_at)
            .with_summary("Summary of work done");

        assert_eq!(result.summary, Some("Summary of work done".to_string()));
    }

    #[test]
    fn test_delegation_status_summary() {
        let summary = DelegationStatusSummary {
            active_count: 5,
            completed_count: 10,
            pending_count: 2,
            in_progress_count: 3,
        };

        assert_eq!(summary.active_count, 5);
        assert_eq!(summary.completed_count, 10);
    }

    #[test]
    fn test_task_duration() {
        let mut task = Task::new("Test", AgentType::Explore, AgentType::Build, vec![]);
        assert!(task.duration().is_none());

        task.started_at = Some(Utc::now() - chrono::Duration::seconds(10));
        task.completed_at = Some(Utc::now());

        let duration = task.duration().unwrap();
        assert!(duration.as_secs() >= 10);
    }

    #[test]
    fn test_delegation_error_display() {
        let error = DelegationError::NoActivePrimaryAgent;
        assert!(format!("{}", error).contains("no active primary agent"));

        let error = DelegationError::TaskNotDelegatable {
            task_id: TaskId::new(),
            status: TaskStatus::Completed,
        };
        assert!(format!("{}", error).contains("not delegatable"));
    }

    #[test]
    fn test_task_delegate_empty() {
        let delegate = TaskDelegate::new();
        assert_eq!(delegate.active_task_count(), 0);
        assert_eq!(delegate.completed_task_count(), 0);
    }

    #[test]
    fn test_task_delegate_status_summary() {
        let delegate = TaskDelegate::new();
        let summary = delegate.status_summary();

        assert_eq!(summary.active_count, 0);
        assert_eq!(summary.completed_count, 0);
        assert_eq!(summary.pending_count, 0);
        assert_eq!(summary.in_progress_count, 0);
    }

    #[test]
    fn test_task_latest_progress() {
        let mut task = Task::new("Test", AgentType::Explore, AgentType::Build, vec![]);
        assert!(task.latest_progress().is_none());

        task.progress_history
            .push(TaskProgress::pending(task.id, "Starting"));
        task.progress_history
            .push(TaskProgress::in_progress(task.id, "Working"));

        let latest = task.latest_progress().unwrap();
        assert_eq!(latest.message, "Working");
    }
}
