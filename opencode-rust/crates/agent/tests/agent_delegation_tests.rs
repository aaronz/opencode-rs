use opencode_agent::{
    AgentRuntime, AgentType, RuntimeConfig, Task, TaskDelegate, TaskId, TaskProgress, TaskStatus,
};
use opencode_core::{Message, Session};
use opencode_permission::AgentPermissionScope;
use uuid::Uuid;

#[test]
fn test_task_id_new_creates_unique_ids() {
    let id1 = TaskId::new();
    let id2 = TaskId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_task_id_default_creates_new_id() {
    let id1 = TaskId::default();
    let id2 = TaskId::default();
    assert_ne!(id1, id2);
}

#[test]
fn test_task_id_display_format() {
    let id = TaskId::new();
    let display = format!("{}", id);
    assert!(display.starts_with("task:"));
}

#[test]
fn test_task_id_debug_format() {
    let id = TaskId::new();
    let debug = format!("{:?}", id);
    assert!(debug.contains("TaskId"));
}

#[test]
fn test_task_id_clone_is_equal() {
    let id = TaskId::new();
    let cloned = id.clone();
    assert_eq!(id, cloned);
}

#[test]
fn test_task_status_display_pending() {
    assert_eq!(format!("{}", TaskStatus::Pending), "pending");
}

#[test]
fn test_task_status_display_in_progress() {
    assert_eq!(format!("{}", TaskStatus::InProgress), "in_progress");
}

#[test]
fn test_task_status_display_completed() {
    assert_eq!(format!("{}", TaskStatus::Completed), "completed");
}

#[test]
fn test_task_status_display_failed() {
    assert_eq!(format!("{}", TaskStatus::Failed), "failed");
}

#[test]
fn test_task_status_display_cancelled() {
    assert_eq!(format!("{}", TaskStatus::Cancelled), "cancelled");
}

#[test]
fn test_task_status_serialization() {
    let status = TaskStatus::InProgress;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"inprogress\"");
    let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, TaskStatus::InProgress);
}

#[test]
fn test_task_progress_new_without_percentage() {
    let task_id = TaskId::new();
    let progress = TaskProgress::new(task_id, TaskStatus::InProgress, "Working...");

    assert_eq!(progress.task_id, task_id);
    assert_eq!(progress.status, TaskStatus::InProgress);
    assert_eq!(progress.message, "Working...");
    assert!(progress.progress_percent.is_none());
}

#[test]
fn test_task_progress_new_with_percentage() {
    let task_id = TaskId::new();
    let progress = TaskProgress::with_progress(task_id, TaskStatus::InProgress, "Half done", 50);

    assert_eq!(progress.progress_percent, Some(50));
}

#[test]
fn test_task_progress_timestamp_is_set() {
    let task_id = TaskId::new();
    let before = chrono::Utc::now();
    let progress = TaskProgress::new(task_id, TaskStatus::Pending, "Started");
    let after = chrono::Utc::now();

    assert!(progress.timestamp >= before && progress.timestamp <= after);
}

#[test]
fn test_task_progress_serialization() {
    let task_id = TaskId::new();
    let progress = TaskProgress::with_progress(task_id, TaskStatus::Completed, "Done", 100);
    let json = serde_json::to_string(&progress).unwrap();

    assert!(json.contains("\"status\":\"completed\""));
    assert!(json.contains("\"progress_percent\":100"));
}

#[test]
fn test_task_new_with_all_parameters() {
    let task = Task::new(
        "test task",
        AgentType::Explore,
        AgentType::Build,
        vec![Message::user("context")],
    );

    assert_eq!(task.description, "test task");
    assert_eq!(task.agent_type, AgentType::Explore);
    assert_eq!(task.delegator_type, AgentType::Build);
    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.context.len(), 1);
}

#[test]
fn test_task_record_progress() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    let task_id = task.id;

    let progress = TaskProgress::new(task_id, TaskStatus::InProgress, "Making progress");
    task.record_progress(progress);

    assert_eq!(task.status, TaskStatus::InProgress);
    assert_eq!(task.progress_history.len(), 1);
}

#[test]
fn test_task_mark_started() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);

    task.mark_started();

    assert_eq!(task.status, TaskStatus::InProgress);
    assert!(task.started_at.is_some());
    assert_eq!(task.progress_history.len(), 1);
}

#[test]
fn test_task_mark_failed() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);

    task.mark_failed("error message");

    assert_eq!(task.status, TaskStatus::Failed);
    assert!(task.completed_at.is_some());
}

#[test]
fn test_task_is_terminal() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);

    assert!(!task.is_terminal());

    task.mark_started();
    assert!(!task.is_terminal());

    task.mark_failed("failed");
    assert!(task.is_terminal());
}

#[test]
fn test_task_duration() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    assert!(task.duration().is_none());

    task.mark_started();
    std::thread::sleep(std::time::Duration::from_millis(10));
    task.mark_failed("failed");

    assert!(task.duration().is_some());
}

#[test]
fn test_task_latest_progress() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    assert!(task.latest_progress().is_none());

    let progress = TaskProgress::new(task.id, TaskStatus::InProgress, "test");
    task.record_progress(progress);

    assert!(task.latest_progress().is_some());
    assert_eq!(task.latest_progress().unwrap().message, "test");
}

#[test]
fn test_task_delegate_new() {
    let delegate = TaskDelegate::new();
    assert!(delegate.active_tasks().is_empty());
}

#[test]
fn test_runtime_config_default() {
    let config = RuntimeConfig::default();

    assert_eq!(config.max_iterations, 20);
    assert_eq!(config.max_tool_results_per_iteration, 10);
    assert_eq!(config.permission_scope, AgentPermissionScope::Full);
}

#[test]
fn test_runtime_config_with_custom_values() {
    let config = RuntimeConfig {
        max_iterations: 100,
        max_tool_results_per_iteration: 50,
        permission_scope: AgentPermissionScope::ReadOnly,
    };

    assert_eq!(config.max_iterations, 100);
    assert_eq!(config.max_tool_results_per_iteration, 50);
    assert_eq!(config.permission_scope, AgentPermissionScope::ReadOnly);
}

#[test]
fn test_runtime_config_debug() {
    let config = RuntimeConfig::default();
    let debug = format!("{:?}", config);
    assert!(debug.contains("RuntimeConfig"));
}

#[tokio::test]
async fn test_agent_runtime_new() {
    let session = Session::default();
    let runtime = AgentRuntime::new(session, AgentType::Build);

    assert_eq!(runtime.active_agent(), Some(AgentType::Build));
}

#[test]
fn test_agent_type_all_variants_have_display() {
    let types = [
        AgentType::Build,
        AgentType::Plan,
        AgentType::General,
        AgentType::Explore,
        AgentType::Compaction,
        AgentType::Title,
        AgentType::Summary,
        AgentType::Review,
        AgentType::Refactor,
        AgentType::Debug,
    ];

    for agent_type in types {
        let display = format!("{}", agent_type);
        assert!(!display.is_empty());
    }
}

#[test]
fn test_agent_type_all_variants_serialize_to_lowercase() {
    let types = [
        (AgentType::Build, "build"),
        (AgentType::Plan, "plan"),
        (AgentType::General, "general"),
        (AgentType::Explore, "explore"),
        (AgentType::Compaction, "compaction"),
        (AgentType::Title, "title"),
        (AgentType::Summary, "summary"),
        (AgentType::Review, "review"),
        (AgentType::Refactor, "refactor"),
        (AgentType::Debug, "debug"),
    ];

    for (agent_type, expected) in types {
        let json = serde_json::to_string(&agent_type).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn test_agent_type_deserialize_from_lowercase() {
    let types = [
        (AgentType::Build, "build"),
        (AgentType::Plan, "plan"),
        (AgentType::General, "general"),
        (AgentType::Explore, "explore"),
        (AgentType::Compaction, "compaction"),
        (AgentType::Title, "title"),
        (AgentType::Summary, "summary"),
        (AgentType::Review, "review"),
        (AgentType::Refactor, "refactor"),
        (AgentType::Debug, "debug"),
    ];

    for (expected, json_value) in types {
        let json = format!("\"{}\"", json_value);
        let deserialized: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, expected);
    }
}

#[test]
fn test_task_id_with_uuid_conversion() {
    let uuid = Uuid::new_v4();
    let task_id = TaskId(uuid);
    let back_to_uuid = task_id.0;

    assert_eq!(uuid, back_to_uuid);
}

#[test]
fn test_task_status_all_variants_have_display() {
    let statuses = [
        TaskStatus::Pending,
        TaskStatus::InProgress,
        TaskStatus::Completed,
        TaskStatus::Failed,
        TaskStatus::Cancelled,
    ];

    for status in statuses {
        let display = format!("{}", status);
        assert!(!display.is_empty());
    }
}

#[test]
fn test_task_context_preserved() {
    let context = vec![Message::user("message 1"), Message::assistant("message 2")];
    let task = Task::new(
        "test",
        AgentType::Explore,
        AgentType::Build,
        context.clone(),
    );

    assert_eq!(task.context.len(), 2);
    assert_eq!(task.context[0].content, "message 1");
}

#[test]
fn test_task_progress_history_maintained() {
    let mut task = Task::new("test", AgentType::Explore, AgentType::Build, vec![]);
    let task_id = task.id;

    task.record_progress(TaskProgress::new(task_id, TaskStatus::InProgress, "step 1"));
    task.record_progress(TaskProgress::new(task_id, TaskStatus::InProgress, "step 2"));

    assert_eq!(task.progress_history.len(), 2);
}

#[test]
fn test_task_delegator_type_preserved() {
    let task = Task::new("test", AgentType::Explore, AgentType::Plan, vec![]);

    assert_eq!(task.delegator_type, AgentType::Plan);
    assert_eq!(task.agent_type, AgentType::Explore);
}
