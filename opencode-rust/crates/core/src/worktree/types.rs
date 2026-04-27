use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worktree {
    pub path: PathBuf,
    pub branch: String,
    pub head: String,
    pub is_bare: bool,
}

pub struct WorktreeManager {
    pub(crate) worktrees: Vec<Worktree>,
}

impl WorktreeManager {
    pub fn new() -> Self {
        Self {
            worktrees: Vec::new(),
        }
    }

    pub fn add(&mut self, worktree: Worktree) {
        self.worktrees.push(worktree);
    }

    pub fn list(&self) -> &[Worktree] {
        &self.worktrees
    }

    pub fn find_by_branch(&self, branch: &str) -> Option<&Worktree> {
        self.worktrees.iter().find(|w| w.branch == branch)
    }

    pub fn find_by_path(&self, path: &PathBuf) -> Option<&Worktree> {
        self.worktrees.iter().find(|w| w.path == *path)
    }
}

impl Default for WorktreeManager {
    fn default() -> Self {
        Self::new()
    }
}