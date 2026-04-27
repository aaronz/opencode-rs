mod types;
pub use types::{Worktree, WorktreeManager};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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