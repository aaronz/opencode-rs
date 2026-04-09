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
    worktrees: Vec<Worktree>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_manager_new() {
        let wm = WorktreeManager::new();
        assert!(wm.list().is_empty());
    }

    #[test]
    fn test_worktree_add() {
        let mut wm = WorktreeManager::new();
        wm.add(Worktree {
            path: PathBuf::from("/path/to/worktree"),
            branch: "main".to_string(),
            head: "abc123".to_string(),
            is_bare: false,
        });
        assert_eq!(wm.list().len(), 1);
    }

    #[test]
    fn test_worktree_find_by_branch() {
        let mut wm = WorktreeManager::new();
        wm.add(Worktree {
            path: PathBuf::from("/path/to/worktree"),
            branch: "main".to_string(),
            head: "abc123".to_string(),
            is_bare: false,
        });
        let found = wm.find_by_branch("main");
        assert!(found.is_some());
        assert_eq!(found.unwrap().branch, "main");
    }

    #[test]
    fn test_worktree_find_by_path() {
        let mut wm = WorktreeManager::new();
        wm.add(Worktree {
            path: PathBuf::from("/path/to/worktree"),
            branch: "main".to_string(),
            head: "abc123".to_string(),
            is_bare: false,
        });
        let found = wm.find_by_path(&PathBuf::from("/path/to/worktree"));
        assert!(found.is_some());
    }

    #[test]
    fn test_worktree_not_found() {
        let wm = WorktreeManager::new();
        assert!(wm.find_by_branch("nonexistent").is_none());
        assert!(wm.find_by_path(&PathBuf::from("/nonexistent")).is_none());
    }
}
