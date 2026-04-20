use git2::{BranchType, Repository};
use opencode_core::OpenCodeError;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeResult {
    Clean {
        commit: String,
    },
    UpToDate {
        commit: String,
    },
    Conflict {
        conflicted_files: Vec<String>,
    },
}

fn path_to_string(path: Vec<u8>) -> String {
    String::from_utf8(path).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).to_string())
}

pub fn git_merge(repo_path: &Path, branch: &str) -> Result<MergeResult, OpenCodeError> {
    if branch.is_empty() {
        return Err(OpenCodeError::ValidationError {
            field: "branch".to_string(),
            message: "Branch name cannot be empty".to_string(),
        });
    }

    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let branch_reference = repo
        .find_branch(branch, BranchType::Local)
        .map_err(|e| OpenCodeError::Tool(format!("Branch '{}' not found: {}", branch, e)))?;

    let branch_oid = branch_reference
        .get()
        .target()
        .ok_or_else(|| OpenCodeError::Tool(format!("Branch '{}' has no target", branch)))?;

    drop(branch_reference);

    let branch_annotated = repo
        .find_annotated_commit(branch_oid)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get annotated commit: {}", e)))?;

    let analysis = repo
        .merge_analysis(&[&branch_annotated])
        .map_err(|e| OpenCodeError::Tool(format!("Failed to analyze merge: {}", e)))?;

    drop(branch_annotated);

    let head = repo.head().map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?;
    let head_commit = head.peel_to_commit().map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?;
    let head_oid = head_commit.id();

    drop(head);

    if analysis.0.is_up_to_date() {
        return Ok(MergeResult::UpToDate {
            commit: head_oid.to_string(),
        });
    }

    if analysis.0.is_fast_forward() {
        repo.reference("HEAD", branch_oid, true, "fast-forward")
            .map_err(|e| OpenCodeError::Tool(format!("Failed to update HEAD: {}", e)))?;

        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.safe();
        repo.checkout_head(Some(&mut checkout_builder))
            .map_err(|e| OpenCodeError::Tool(format!("Failed to checkout: {}", e)))?;

        return Ok(MergeResult::Clean {
            commit: branch_oid.to_string(),
        });
    }

    let branch_commit = repo.find_commit(branch_oid).map_err(|e| OpenCodeError::Tool(format!("Failed to find branch commit: {}", e)))?;

    let signature = repo
        .signature()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get signature: {}", e)))?;

    let mut merge_index = repo
        .merge_commits(&head_commit, &branch_commit, None)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to merge commits: {}", e)))?;

    drop(head_commit);
    drop(branch_commit);

    if merge_index.has_conflicts() {
        let mut conflicted_files = Vec::new();
        for conflict_result in merge_index.conflicts().map_err(|e| OpenCodeError::Tool(format!("Failed to get conflicts: {}", e)))? {
            let conflict = conflict_result.map_err(|e| OpenCodeError::Tool(format!("Failed to get conflict entry: {}", e)))?;
            if let Some(path) = conflict.ancestor {
                conflicted_files.push(path_to_string(path.path));
            }
            if let Some(path) = conflict.our {
                conflicted_files.push(path_to_string(path.path));
            }
            if let Some(path) = conflict.their {
                conflicted_files.push(path_to_string(path.path));
            }
        }

        repo.cleanup_state().ok();

        return Ok(MergeResult::Conflict {
            conflicted_files,
        });
    }

    let tree_oid = merge_index
        .write_tree_to(&repo)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to write merge tree: {}", e)))?;

    let tree = repo.find_tree(tree_oid).map_err(|e| OpenCodeError::Tool(format!("Failed to find tree: {}", e)))?;

    let branch_commit_for_parent = repo.find_commit(branch_oid).map_err(|e| OpenCodeError::Tool(format!("Failed to find branch commit: {}", e)))?;
    let head_commit_for_parent = repo.head().map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?.peel_to_commit().map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?;

    let commit_oid = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Merge branch '{}'", branch),
            &tree,
            &[&head_commit_for_parent, &branch_commit_for_parent],
        )
        .map_err(|e| OpenCodeError::Tool(format!("Failed to create merge commit: {}", e)))?;

    Ok(MergeResult::Clean {
        commit: commit_oid.to_string(),
    })
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
        repo.commit(Some("HEAD"), &signature, &signature, "Initial commit", &tree, &[])
            .unwrap();

        temp_dir
    }

    fn create_test_branch(repo: &Repository, branch_name: &str, parent: &git2::Commit, file_name: &str, content: &str) -> git2::Oid {
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
    fn test_git_merge_fast_forward() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        drop(head);
        create_test_branch(&repo, "feature", &repo.head().unwrap().peel_to_commit().unwrap(), "feature.txt", "feature content");

        repo.set_head("refs/heads/master").unwrap();
        let mut checkout = git2::build::CheckoutBuilder::new();
        checkout.safe();
        repo.checkout_head(Some(&mut checkout)).unwrap();

        drop(repo);

        let result = git_merge(temp_dir.path(), "feature");
        assert!(result.is_ok());

        match result.unwrap() {
            MergeResult::Clean { commit: _ } => {}
            other => panic!("Expected Clean fast-forward merge, got {:?}", other),
        }
    }

    #[test]
    fn test_git_merge_three_way() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let master_branch = repo.find_branch("master", BranchType::Local).unwrap();
        let master_oid = master_branch.get().target().unwrap();
        let base_commit = repo.find_commit(master_oid).unwrap();

        create_test_branch(&repo, "feature", &base_commit, "feature.txt", "feature content");

        drop(master_branch);
        drop(base_commit);

        std::fs::write(temp_dir.path().join("main.txt"), "main content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("main.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        drop(tree_id);
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

        let result = git_merge(temp_dir.path(), "feature");
        assert!(result.is_ok(), "merge failed: {:?}", result);

        match result.unwrap() {
            MergeResult::Clean { commit: _ } => {}
            other => panic!("Expected Clean three-way merge, got {:?}", other),
        }
    }

    #[test]
    fn test_git_merge_conflict() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let head = repo.head().unwrap().peel_to_commit().unwrap();

        std::fs::write(temp_dir.path().join("test.txt"), "original content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        drop(tree_id);
        let signature = repo.signature().unwrap();
        let commit1_oid = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add original test.txt",
            &tree,
            &[&head],
        )
        .unwrap();

        drop(tree);
        let commit1 = repo.find_commit(commit1_oid).unwrap();

        create_test_branch(&repo, "feature", &commit1, "test.txt", "feature version");

        std::fs::write(temp_dir.path().join("test.txt"), "main version").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        drop(tree_id);
        let signature = repo.signature().unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Modify test.txt on main",
            &tree,
            &[&commit1],
        )
        .unwrap();

        drop(tree);
        drop(head);
        drop(commit1);
        drop(repo);

        let result = git_merge(temp_dir.path(), "feature");
        assert!(result.is_ok());

        match result.unwrap() {
            MergeResult::Conflict { conflicted_files } => {
                assert!(conflicted_files.contains(&"test.txt".to_string()));
            }
            other => panic!("Expected Conflict merge, got {:?}", other),
        }
    }

    #[test]
    fn test_git_merge_empty_branch_error() {
        let temp_dir = create_test_repo();

        let result = git_merge(temp_dir.path(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_merge_nonexistent_branch_error() {
        let temp_dir = create_test_repo();

        let result = git_merge(temp_dir.path(), "nonexistent-branch");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_merge_up_to_date() {
        let temp_dir = create_test_repo();
        let repo = Repository::open(temp_dir.path()).unwrap();

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature", &head, false).unwrap();

        drop(head);
        drop(repo);

        let result = git_merge(temp_dir.path(), "feature");
        assert!(result.is_ok());

        match result.unwrap() {
            MergeResult::UpToDate { commit: _ } => {}
            other => panic!("Expected UpToDate merge, got {:?}", other),
        }
    }
}