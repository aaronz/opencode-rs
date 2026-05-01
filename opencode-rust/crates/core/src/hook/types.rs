use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::HookPoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookId {
    Builtin(u32),
    Project(u32),
    User(u32),
    Plugin(u32),
}

impl HookId {
    /// Returns the context cost tier for this hook source.
    /// Tier 0 = zero-context (Builtin), 1 = low (Project), 2 = medium (User), 3 = high (Plugin)
    pub fn cost_tier(&self) -> u8 {
        match self {
            HookId::Builtin(_) => 0,
            HookId::Project(_) => 1,
            HookId::User(_) => 2,
            HookId::Plugin(_) => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookTrigger {
    HookPoint(HookPoint),
    Event(String),
    Command(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub id: HookId,
    pub name: String,
    pub description: Option<String>,
    pub trigger: HookTrigger,
    pub action: HookAction,
    pub timeout_ms: Option<u64>,
    pub failure_policy: HookFailurePolicy,
    pub enabled: bool,
}

impl HookDefinition {
    pub fn new(id: HookId, name: String, trigger: HookTrigger, action: HookAction) -> Self {
        Self {
            id,
            name,
            description: None,
            trigger,
            action,
            timeout_ms: Some(30_000),
            failure_policy: HookFailurePolicy::Log,
            enabled: true,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_failure_policy(mut self, policy: HookFailurePolicy) -> Self {
        self.failure_policy = policy;
        self
    }

    pub fn disabled(self) -> Self {
        Self {
            enabled: false,
            ..self
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookAction {
    RunCommand {
        command: String,
    },
    RunScript {
        script: String,
        interpreter: Option<String>,
    },
    SetEnv {
        key: String,
        value: String,
    },
    Log {
        message: String,
        level: String,
    },
    Notify {
        message: String,
    },
    Block {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HookFailurePolicy {
    #[default]
    Log,
    Warn,
    Error,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecution {
    pub hook_id: HookId,
    pub hook_name: String,
    pub trigger: HookTrigger,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub success: bool,
    pub error: Option<String>,
    pub output: Option<String>,
}

impl HookExecution {
    pub fn new(hook: &HookDefinition) -> Self {
        Self {
            hook_id: hook.id,
            hook_name: hook.name.clone(),
            trigger: hook.trigger.clone(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            success: false,
            error: None,
            output: None,
        }
    }

    pub fn with_output(mut self, output: String) -> Self {
        self.output = Some(output);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self.success = false;
        self
    }

    pub fn complete(&mut self) {
        self.completed_at = Some(chrono::Utc::now());
    }
}
