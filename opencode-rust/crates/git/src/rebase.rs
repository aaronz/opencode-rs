use git2::{BranchType, Repository};
use opencode_core::OpenCodeError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

thread_local! {
    static REBASE_STATE: RefCell<HashMap<std::path::PathBuf, RebaseState>> = RefCell::new(HashMap::new());
}

#[derive(Clone)]
struct RebaseState {
    original_head: String,
    original_branch: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RebaseResult {
    Completed { commit: String },
    Conflict { conflicted_files: Vec<String> },
    UpToDate { commit: String },
}

pub fn git_rebase(repo_path: &Path, branch: &str) -> Result<RebaseResult, OpenCodeError> {
    if branch.is_empty() {
        return Err(OpenCodeError::ValidationError {
            field: "branch".to_string(),
            message: "Branch name cannot be empty".to_string(),
        });
    }

    let repo_path_buf = repo_path.to_path_buf();
    let git_dir = repo_path_buf.join(".git");

    for rebase_file in &[
        "rebase-orig-head",
        "rebase-head",
        "MERGE_HEAD",
        "index.lock",
    ] {
        let file_path = git_dir.join(rebase_file);
        if file_path.exists() {
            std::fs::remove_file(&file_path).ok();
        }
    }

    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let branch_reference = repo
        .find_branch(branch, BranchType::Local)
        .map_err(|e| OpenCodeError::Tool(format!("Branch '{}' not found: {}", branch, e)))?;

    let _branch_oid = branch_reference
        .get()
        .target()
        .ok_or_else(|| OpenCodeError::Tool(format!("Branch '{}' has no target", branch)))?;

    drop(branch_reference);

    let head = repo
        .head()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?;
    let head_commit = head
        .peel_to_commit()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?;
    let head_oid = head_commit.id();
    let original_head_str = head_oid.to_string();

    drop(head);

    let current_branch_name = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "HEAD".to_string());

    {
        REBASE_STATE.with(|state| {
            let mut rebase_state = state.borrow_mut();
            rebase_state.insert(
                repo_path_buf.clone(),
                RebaseState {
                    original_head: original_head_str.clone(),
                    original_branch: current_branch_name.clone(),
                },
            );
        });
    }

    Command::new("git")
        .args(["reset", "--hard", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok();

    let stash_output = Command::new("git")
        .args(["stash", "--include-untracked"])
        .current_dir(repo_path)
        .output()
        .ok();

    let needs_stash_pop = stash_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let output = Command::new("git")
        .args(["rebase", branch])
        .current_dir(repo_path)
        .output()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to execute git rebase: {}", e)))?;

    if output.status.success() {
        if needs_stash_pop {
            Command::new("git")
                .args(["stash", "pop"])
                .current_dir(repo_path)
                .output()
                .ok();
        }

        let repo = Repository::open(repo_path)
            .map_err(|e| OpenCodeError::Tool(format!("Failed to reopen repository: {}", e)))?;
        let head = repo
            .head()
            .map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?;
        let head_commit = head
            .peel_to_commit()
            .map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?;
        let new_head = head_commit.id().to_string();

        if new_head == original_head_str {
            return Ok(RebaseResult::UpToDate {
                commit: original_head_str,
            });
        }

        return Ok(RebaseResult::Completed { commit: new_head });
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let output_combined = format!("{}{}", stdout, stderr);

    if output_combined.contains("CONFLICT") || output_combined.contains("could not apply") {
        let mut conflicted_files = Vec::new();

        if let Ok(repo) = Repository::open(repo_path) {
            if let Ok(index) = repo.index() {
                for entry in index.iter() {
                    let path = entry.path;
                    let path_str = String::from_utf8_lossy(&path);
                    if path_str.contains(".git/MERGE_MSG") || path_str.contains(".git/REBASE_HEAD")
                    {
                        continue;
                    }
                    conflicted_files.push(path_str.to_string());
                }
            }
        }

        return Ok(RebaseResult::Conflict { conflicted_files });
    }

    if needs_stash_pop {
        Command::new("git")
            .args(["stash", "pop"])
            .current_dir(repo_path)
            .output()
            .ok();
    }

    Err(OpenCodeError::Tool(format!("Rebase failed: {}", stderr)))
}

pub fn git_rebase_abort(repo_path: &Path) -> Result<(), OpenCodeError> {
    let repo_path_buf = repo_path.to_path_buf();

    let state = REBASE_STATE.with(|state| {
        let mut rebase_state = state.borrow_mut();
        rebase_state.remove(&repo_path_buf)
    });

    state.ok_or_else(|| OpenCodeError::Tool("No rebase in progress".to_string()))?;

    cleanup_rebase_files(repo_path);

    let output = Command::new("git")
        .args(["rebase", "--abort"])
        .current_dir(repo_path)
        .output()
        .ok();

    if let Some(output) = output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_lower = stderr.to_lowercase();
            if !stderr_lower.contains("no rebase in progress")
                && !stderr_lower.contains("there is no rebase")
                && !stderr_lower.contains("unable to read current working directory")
            {
                return Err(OpenCodeError::Tool(format!(
                    "Failed to abort rebase: {}",
                    stderr
                )));
            }
        }
    }

    Ok(())
}

fn cleanup_rebase_files(repo_path: &Path) {
    let git_dir = repo_path.join(".git");
    for rebase_file in &[
        "rebase-orig-head",
        "rebase-head",
        "MERGE_HEAD",
        "index.lock",
    ] {
        let file_path = git_dir.join(rebase_file);
        if file_path.exists() {
            std::fs::remove_file(&file_path).ok();
        }
    }
}

pub fn git_rebase_status(repo_path: &Path) -> Result<Option<RebaseStatus>, OpenCodeError> {
    let repo_path_buf = repo_path.to_path_buf();
    let result = REBASE_STATE.with(|state| {
        let rebase_state = state.borrow();
        rebase_state.get(&repo_path_buf).cloned()
    });

    Ok(result.map(|state| RebaseStatus {
        original_head: state.original_head,
        original_branch: state.original_branch,
        repo_path: repo_path_buf,
    }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseStatus {
    pub original_head: String,
    pub original_branch: String,
    pub repo_path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn create_test_branch_commit(
        repo: &Repository,
        branch_name: &str,
        parent: &git2::Commit,
        file_name: &str,
        content: &str,
    ) -> git2::Oid {
        let file_path = repo.path().parent().unwrap().join(file_name);
        std::fs::write(file_path, content).unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(file_name)).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let signature = repo.signature().unwrap();
        repo.commit(
            Some(&format!("refs/heads/{}", branch_name)),
            &signature,
            &signature,
            &format!("Add {}", file_name),
            &tree,
            &[parent],
        )
        .unwrap()
    }

    #[test]
    fn test_git_rebase_empty_branch_error() {
        let temp_dir = create_test_repo();
        let result = git_rebase(temp_dir.path(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_rebase_nonexistent_branch_error() {
        let temp_dir = create_test_repo();
        let result = git_rebase(temp_dir.path(), "nonexistent-branch");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_rebase_up_to_date() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();

        drop(head);
        drop(repo);

        let result = git_rebase(temp_dir.path(), "feature");
        assert!(result.is_ok());

        match result.unwrap() {
            RebaseResult::UpToDate { commit: _ } => {}
            other => panic!("Expected UpToDate rebase, got {:?}", other),
        }
    }

    #[test]
    fn test_git_rebase_performs_rebase_correctly() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let master_head = repo.head().unwrap().peel_to_commit().unwrap();
        create_test_branch_commit(
            &repo,
            "feature",
            &master_head,
            "feature.txt",
            "feature content",
        );

        drop(master_head);
        drop(repo);

        {
            let repo = Repository::open(temp_dir.path()).unwrap();
            let mut checkout_builder = git2::build::CheckoutBuilder::new();
            checkout_builder.safe();
            checkout_builder.force();
            repo.checkout_head(Some(&mut checkout_builder)).unwrap();
            repo.cleanup_state().ok();
        }

        std::fs::write(temp_dir.path().join("main.txt"), "main content").unwrap();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("main.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = repo.signature().unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add main.txt",
            &tree,
            &[&repo.head().unwrap().peel_to_commit().unwrap()],
        )
        .unwrap();

        drop(tree);
        drop(repo);

        let result = git_rebase(temp_dir.path(), "feature");
        assert!(result.is_ok(), "rebase failed: {:?}", result);

        match result.unwrap() {
            RebaseResult::Completed { commit: _ } => {}
            other => panic!("Expected Completed rebase, got {:?}", other),
        }
    }

    #[test]
    fn test_git_rebase_abort_cancels_rebase() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "original content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = repo.signature().unwrap();
        let commit1_oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Add original test.txt",
                &tree,
                &[&repo.head().unwrap().peel_to_commit().unwrap()],
            )
            .unwrap();

        drop(tree);
        let _ = tree_id;
        let commit1 = repo.find_commit(commit1_oid).unwrap();

        create_test_branch_commit(&repo, "feature", &commit1, "test.txt", "feature version");

        drop(commit1);
        drop(repo);

        std::fs::write(&file_path, "main version").unwrap();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = repo.signature().unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Modify test.txt on main",
            &tree,
            &[&repo.head().unwrap().peel_to_commit().unwrap()],
        )
        .unwrap();

        drop(tree);
        drop(repo);

        let result = git_rebase(temp_dir.path(), "feature");
        assert!(result.is_ok());

        match result.unwrap() {
            RebaseResult::Conflict {
                conflicted_files: _,
            } => {
                let abort_result = git_rebase_abort(temp_dir.path());
                assert!(abort_result.is_ok(), "abort failed: {:?}", abort_result);
            }
            RebaseResult::Completed { commit: _ } => {
                let status = git_rebase_status(temp_dir.path()).unwrap();
                assert!(
                    status.is_none(),
                    "rebase should be done after completion, status: {:?}",
                    status
                );
            }
            RebaseResult::UpToDate { commit: _ } => {
                let status = git_rebase_status(temp_dir.path()).unwrap();
                assert!(
                    status.is_none(),
                    "rebase should be done after up-to-date, status: {:?}",
                    status
                );
            }
        }
    }

    #[test]
    fn test_git_rebase_abort_without_rebase_error() {
        let temp_dir = create_test_repo();
        let result = git_rebase_abort(temp_dir.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No rebase in progress"));
    }

    #[test]
    fn test_git_rebase_handles_conflicts() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "original content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = repo.signature().unwrap();
        let commit1_oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Add original test.txt",
                &tree,
                &[&repo.head().unwrap().peel_to_commit().unwrap()],
            )
            .unwrap();

        drop(tree);
        let commit1 = repo.find_commit(commit1_oid).unwrap();

        create_test_branch_commit(&repo, "feature", &commit1, "test.txt", "feature version");

        drop(commit1);
        drop(repo);

        {
            let repo = Repository::open(temp_dir.path()).unwrap();
            let mut checkout_builder = git2::build::CheckoutBuilder::new();
            checkout_builder.safe();
            checkout_builder.force();
            repo.checkout_head(Some(&mut checkout_builder)).unwrap();
            repo.cleanup_state().ok();
        }

        std::fs::write(&file_path, "main version").unwrap();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = repo.signature().unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Modify test.txt on main",
            &tree,
            &[&repo.head().unwrap().peel_to_commit().unwrap()],
        )
        .unwrap();

        drop(tree);
        drop(repo);

        let result = git_rebase(temp_dir.path(), "feature");
        assert!(result.is_ok());

        match result.unwrap() {
            RebaseResult::Conflict { conflicted_files } => {
                assert!(conflicted_files.contains(&"test.txt".to_string()));
            }
            other => panic!("Expected Conflict rebase, got {:?}", other),
        }
    }
}
