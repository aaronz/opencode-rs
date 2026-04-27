use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AgentsMdInfo {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct AgentsMdScanConfig {
    pub enabled: bool,
    pub stop_at_worktree_root: bool,
    pub include_hidden: bool,
}

impl AgentsMdScanConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_stop_at_worktree_root(mut self, stop: bool) -> Self {
        self.stop_at_worktree_root = stop;
        self
    }

    pub fn with_include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }
}

impl Default for AgentsMdScanConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stop_at_worktree_root: true,
            include_hidden: false,
        }
    }
}