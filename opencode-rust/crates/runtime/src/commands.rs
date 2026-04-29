use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitUserInput {
    pub session_id: Option<String>,
    pub input: String,
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
pub enum RuntimeCommand {
    SubmitUserInput(SubmitUserInput),
    TaskControl(TaskControlCommand),
    PermissionResponse(PermissionResponse),
}
