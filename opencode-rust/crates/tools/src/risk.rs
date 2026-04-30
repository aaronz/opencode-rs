use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    ReadOnly,
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn is_allowed(&self, _scope: &PermissionScope) -> bool {
        match self {
            RiskLevel::ReadOnly | RiskLevel::Low => true,
            RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical => false,
        }
    }

    pub fn requires_approval(&self) -> bool {
        matches!(
            self,
            RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical
        )
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RiskLevel::ReadOnly => "read-only",
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::ReadOnly => "Read operations like file reading, git status",
            RiskLevel::Low => "Low-risk operations like search, context inspection",
            RiskLevel::Medium => "Medium-risk operations like writing files, applying patches",
            RiskLevel::High => "High-risk operations like shell commands, git commits",
            RiskLevel::Critical => {
                "Critical operations like git reset, directory deletion, secret access"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PermissionScope {
    #[default]
    ReadOnly,
    Restricted,
    Full,
}

impl PermissionScope {
    pub fn allows(&self, risk_level: RiskLevel) -> bool {
        match self {
            PermissionScope::ReadOnly => matches!(risk_level, RiskLevel::ReadOnly),
            PermissionScope::Restricted => {
                matches!(risk_level, RiskLevel::ReadOnly | RiskLevel::Low)
            }
            PermissionScope::Full => true,
        }
    }

    pub fn requires_approval(&self, risk_level: RiskLevel) -> bool {
        match self {
            PermissionScope::ReadOnly => risk_level != RiskLevel::ReadOnly,
            PermissionScope::Restricted => {
                matches!(
                    risk_level,
                    RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical
                )
            }
            PermissionScope::Full => matches!(risk_level, RiskLevel::High | RiskLevel::Critical),
        }
    }
}

pub struct RiskClassifier;

impl RiskClassifier {
    pub fn classify_tool(tool_name: &str) -> RiskLevel {
        match tool_name {
            "read"
            | "git_status"
            | "git_diff"
            | "git_log"
            | "git_branch"
            | "lsp_goto_definition"
            | "lsp_find_references"
            | "lsp_symbols"
            | "session_info" => RiskLevel::ReadOnly,

            "grep" | "codesearch" | "glob" | "ls" | "look_at" | "webfetch" | "todowrite"
            | "session_load" | "session_save" => RiskLevel::Low,

            "write" | "edit" | "multiedit" | "apply_patch" => RiskLevel::Medium,

            "bash" | "git_commit" | "git_push" | "mcp_call_tool" => RiskLevel::High,

            "git_reset" | "rm" | "delete_directory" | "write_secret" | "read_secret" => {
                RiskLevel::Critical
            }

            _ => RiskLevel::Medium,
        }
    }
}
