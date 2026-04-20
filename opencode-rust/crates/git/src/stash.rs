use opencode_core::OpenCodeError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub branch: String,
}

pub fn git_stash(repo_path: &Path) -> Result<(), OpenCodeError> {
    let output = Command::new("git")
        .args(["stash", "push", "--include-untracked", "-m", "WIP"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to execute git stash: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("No local changes to save") {
            return Err(OpenCodeError::Tool(format!(
                "Failed to stash changes: {}",
                stderr
            )));
        }
    }

    Ok(())
}

pub fn git_stash_pop(repo_path: &Path) -> Result<(), OpenCodeError> {
    let output = Command::new("git")
        .args(["stash", "pop"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to execute git stash pop: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OpenCodeError::Tool(format!(
            "Failed to stash pop: {}",
            stderr
        )));
    }

    Ok(())
}

pub fn git_stash_list(repo_path: &Path) -> Result<Vec<StashEntry>, OpenCodeError> {
    let output = Command::new("git")
        .args(["stash", "list", "--format=%gd|%s"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to execute git stash list: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OpenCodeError::Tool(format!(
            "Failed to list stash: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for (idx, line) in stdout.lines().enumerate() {
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.is_empty() {
            continue;
        }

        let branch = "(unknown)".to_string();
        let message = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            format!("stash@{{{}}}", idx)
        };

        entries.push(StashEntry {
            index: idx,
            message,
            branch,
        });
    }

    Ok(entries)
}

pub fn git_stash_drop(repo_path: &Path, index: usize) -> Result<(), OpenCodeError> {
    let output = Command::new("git")
        .args(["stash", "drop", &format!("stash@{{{0}}}", index)])
        .current_dir(repo_path)
        .output()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to execute git stash drop: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OpenCodeError::Tool(format!(
            "Failed to drop stash: {}",
            stderr
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Repository;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let signature = repo.signature().unwrap();
        let tree_id = repo.index().unwrap().write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn test_git_stash_saves_working_directory_state() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        std::fs::write(temp_dir.path().join("test.txt"), "stashed content").unwrap();

        git_stash(temp_dir.path()).unwrap();

        let statuses = repo.statuses(None).unwrap();
        assert!(statuses.is_empty());
    }

    #[test]
    fn test_git_stash_pop_restores_stashed_changes() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        std::fs::write(temp_dir.path().join("test.txt"), "stashed content").unwrap();

        git_stash(temp_dir.path()).unwrap();

        drop(repo);

        git_stash_pop(temp_dir.path()).unwrap();

        let content = std::fs::read_to_string(temp_dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "stashed content");
    }

    #[test]
    fn test_git_stash_list_returns_all_stash_entries() {
        let temp_dir = create_test_repo();

        std::fs::write(temp_dir.path().join("file1.txt"), "content 1").unwrap();
        git_stash(temp_dir.path()).unwrap();

        std::fs::write(temp_dir.path().join("file2.txt"), "content 2").unwrap();
        git_stash(temp_dir.path()).unwrap();

        let entries = git_stash_list(temp_dir.path()).unwrap();

        assert_eq!(entries.len(), 2);
        assert!(entries[0].message.contains("WIP") || entries[0].message.contains("stash"));
    }

    #[test]
    fn test_git_stash_drop_removes_specific_stash_entry() {
        let temp_dir = create_test_repo();

        std::fs::write(temp_dir.path().join("file1.txt"), "content 1").unwrap();
        git_stash(temp_dir.path()).unwrap();

        std::fs::write(temp_dir.path().join("file2.txt"), "content 2").unwrap();
        git_stash(temp_dir.path()).unwrap();

        let entries_before = git_stash_list(temp_dir.path()).unwrap();
        assert_eq!(entries_before.len(), 2);

        git_stash_drop(temp_dir.path(), 0).unwrap();

        let entries_after = git_stash_list(temp_dir.path()).unwrap();
        assert_eq!(entries_after.len(), 1);
    }

    #[test]
    fn test_git_stash_empty_repo_error() {
        let temp_dir = create_test_repo();

        let result = git_stash_pop(temp_dir.path());
        assert!(result.is_err());
    }
}
