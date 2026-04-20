use git2::{BranchType, Repository};
use opencode_core::OpenCodeError;
use std::path::Path;

pub fn git_branch_list(repo_path: &Path) -> Result<Vec<String>, OpenCodeError> {
    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let mut branches = Vec::new();

    let branch_iter = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| OpenCodeError::Tool(format!("Failed to iterate branches: {}", e)))?;

    for branch_result in branch_iter {
        let (branch, _branch_type) = branch_result
            .map_err(|e| OpenCodeError::Tool(format!("Failed to get branch: {}", e)))?;
        let name = branch.name().ok().flatten();
        if let Some(name) = name {
            branches.push(name.to_string());
        }
    }

    Ok(branches)
}

pub fn git_branch_create(repo_path: &Path, name: &str) -> Result<(), OpenCodeError> {
    if name.is_empty() {
        return Err(OpenCodeError::ValidationError {
            field: "name".to_string(),
            message: "Branch name cannot be empty".to_string(),
        });
    }

    if name.contains(['/', ' ', '\0']) {
        return Err(OpenCodeError::ValidationError {
            field: "name".to_string(),
            message: "Branch name contains invalid characters".to_string(),
        });
    }

    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let head = repo
        .head()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?;

    let head_commit = head
        .peel_to_commit()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?;

    repo.branch(name, &head_commit, false)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to create branch '{}': {}", name, e)))?;

    Ok(())
}

pub fn git_branch_delete(repo_path: &Path, name: &str) -> Result<(), OpenCodeError> {
    if name.is_empty() {
        return Err(OpenCodeError::ValidationError {
            field: "name".to_string(),
            message: "Branch name cannot be empty".to_string(),
        });
    }

    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let mut reference = repo
        .find_branch(name, BranchType::Local)
        .map_err(|e| OpenCodeError::Tool(format!("Branch '{}' not found: {}", name, e)))?;

    let head = repo.head().ok();
    let current_branch_name = head.and_then(|h| h.shorthand().map(|s| s.to_string()));
    let is_current_branch = current_branch_name.as_deref() == Some(name);

    if is_current_branch {
        return Err(OpenCodeError::Tool(format!(
            "Cannot delete branch '{}': it is currently the HEAD",
            name
        )));
    }

    reference
        .delete()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to delete branch '{}': {}", name, e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let signature = repo.signature().unwrap();
        let tree_id = repo
            .index()
            .unwrap()
            .write_tree()
            .unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &signature, &signature, "Initial commit", &tree, &[])
            .unwrap();

        temp_dir
    }

    #[test]
    fn test_git_branch_list_returns_list_of_branch_names() {
        let temp_dir = create_test_repo();

        git_branch_create(temp_dir.path(), "feature-a").unwrap();
        git_branch_create(temp_dir.path(), "feature-b").unwrap();

        let branches = git_branch_list(temp_dir.path()).unwrap();

        assert!(branches.contains(&"feature-a".to_string()));
        assert!(branches.contains(&"feature-b".to_string()));
        assert!(branches.contains(&"main".to_string()) || branches.contains(&"master".to_string()));
    }

    #[test]
    fn test_git_branch_create_creates_a_new_branch() {
        let temp_dir = create_test_repo();

        git_branch_create(temp_dir.path(), "new-feature").unwrap();

        let branches = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches.contains(&"new-feature".to_string()));
    }

    #[test]
    fn test_git_branch_create_empty_name_error() {
        let temp_dir = create_test_repo();

        let result = git_branch_create(temp_dir.path(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_branch_create_invalid_name_error() {
        let temp_dir = create_test_repo();

        let result = git_branch_create(temp_dir.path(), "invalid/branch");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_branch_delete_removes_a_branch() {
        let temp_dir = create_test_repo();

        git_branch_create(temp_dir.path(), "to-delete").unwrap();
        let branches_before = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches_before.contains(&"to-delete".to_string()));

        git_branch_delete(temp_dir.path(), "to-delete").unwrap();

        let branches_after = git_branch_list(temp_dir.path()).unwrap();
        assert!(!branches_after.contains(&"to-delete".to_string()));
    }

    #[test]
    fn test_git_branch_delete_nonexistent_error() {
        let temp_dir = create_test_repo();

        let result = git_branch_delete(temp_dir.path(), "nonexistent-branch");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_branch_delete_current_branch_error() {
        let temp_dir = create_test_repo();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let current_branch = repo.head().unwrap().shorthand().unwrap().to_string();

        let result = git_branch_delete(temp_dir.path(), &current_branch);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot delete"));
    }
}