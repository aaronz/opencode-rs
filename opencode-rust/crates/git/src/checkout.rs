use git2::{BranchType, Repository};
use opencode_core::OpenCodeError;
use std::path::Path;

pub fn git_checkout(repo_path: &Path, branch: &str) -> Result<(), OpenCodeError> {
    if branch.is_empty() {
        return Err(OpenCodeError::ValidationError {
            field: "branch".to_string(),
            message: "Branch name cannot be empty".to_string(),
        });
    }

    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let reference = repo
        .find_branch(branch, BranchType::Local)
        .map_err(|e| OpenCodeError::Tool(format!("Branch '{}' not found: {}", branch, e)))?;

    let _commit = reference
        .get()
        .peel_to_commit()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?;

    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder.safe();

    repo.checkout_head(Some(&mut checkout_builder))
        .map_err(|e| OpenCodeError::Tool(format!("Failed to checkout current HEAD: {}", e)))?;

    let references_head = format!("refs/heads/{}", branch);
    repo.set_head(&references_head)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to set HEAD to '{}': {}", branch, e)))?;

    repo.checkout_head(Some(&mut git2::build::CheckoutBuilder::new()))
        .map_err(|e| {
            OpenCodeError::Tool(format!("Failed to checkout branch '{}': {}", branch, e))
        })?;

    Ok(())
}

pub fn git_checkout_create(repo_path: &Path, name: &str) -> Result<(), OpenCodeError> {
    if name.is_empty() {
        return Err(OpenCodeError::ValidationError {
            field: "name".to_string(),
            message: "Branch name cannot be empty".to_string(),
        });
    }

    if name.contains('\0') {
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

    let reference = repo
        .branch(name, &head_commit, false)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to create branch '{}': {}", name, e)))?;

    let _reference_name = reference
        .name()
        .ok()
        .flatten()
        .ok_or_else(|| OpenCodeError::Tool("Invalid branch reference".to_string()))?;

    let mut checkout_builder = git2::build::CheckoutBuilder::new();
    checkout_builder.safe();

    repo.checkout_head(Some(&mut checkout_builder))
        .map_err(|e| OpenCodeError::Tool(format!("Failed to checkout current HEAD: {}", e)))?;

    let references_head = format!("refs/heads/{}", name);
    repo.set_head(&references_head)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to set HEAD to '{}': {}", name, e)))?;

    repo.checkout_head(Some(&mut git2::build::CheckoutBuilder::new()))
        .map_err(|e| {
            OpenCodeError::Tool(format!("Failed to checkout new branch '{}': {}", name, e))
        })?;

    Ok(())
}

pub fn git_checkout_file(
    repo_path: &Path,
    file_path: &str,
    revision: Option<&str>,
) -> Result<(), OpenCodeError> {
    if file_path.is_empty() {
        return Err(OpenCodeError::ValidationError {
            field: "file_path".to_string(),
            message: "File path cannot be empty".to_string(),
        });
    }

    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let commit = if let Some(rev) = revision {
        let reference = repo.resolve_reference_from_short_name(rev).map_err(|e| {
            OpenCodeError::Tool(format!("Failed to resolve revision '{}': {}", rev, e))
        })?;
        reference
            .peel_to_commit()
            .map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?
    } else {
        repo.head()
            .map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?
            .peel_to_commit()
            .map_err(|e| OpenCodeError::Tool(format!("Failed to peel to commit: {}", e)))?
    };

    let tree = commit
        .tree()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get tree: {}", e)))?;

    let entry = tree
        .get_path(std::path::Path::new(file_path))
        .map_err(|e| {
            OpenCodeError::Tool(format!("File '{}' not found in revision: {}", file_path, e))
        })?;

    let blob = repo
        .find_blob(entry.id())
        .map_err(|e| OpenCodeError::Tool(format!("Failed to find blob: {}", e)))?;

    std::fs::write(repo_path.join(file_path), blob.content())
        .map_err(|e| OpenCodeError::Tool(format!("Failed to write file: {}", e)))?;

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
    fn test_git_checkout_empty_branch_error() {
        let temp_dir = create_test_repo();
        let result = git_checkout(temp_dir.path(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_nonexistent_branch_error() {
        let temp_dir = create_test_repo();
        let result = git_checkout(temp_dir.path(), "nonexistent-branch");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_switches_branch() {
        let temp_dir = create_test_repo();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("feature-a", &head, false).unwrap();
        repo.branch("feature-b", &head, false).unwrap();

        git_checkout(temp_dir.path(), "feature-a").unwrap();
        git_checkout(temp_dir.path(), "feature-b").unwrap();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let head_ref = repo.head().unwrap();
        let current_branch = head_ref.shorthand().unwrap();
        assert_eq!(current_branch, "feature-b");
    }

    #[test]
    fn test_git_checkout_create_empty_name_error() {
        let temp_dir = create_test_repo();
        let result = git_checkout_create(temp_dir.path(), "");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_create_with_null_char_error() {
        let temp_dir = create_test_repo();
        let result = git_checkout_create(temp_dir.path(), "branch\0name");
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_create_new_branch() {
        let temp_dir = create_test_repo();

        git_checkout_create(temp_dir.path(), "new-feature").unwrap();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let head_ref = repo.head().unwrap();
        let current_branch = head_ref.shorthand().unwrap();
        assert_eq!(current_branch, "new-feature");
    }

    #[test]
    fn test_git_checkout_create_switches_to_new_branch() {
        let temp_dir = create_test_repo();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.branch("existing-branch", &head, false).unwrap();

        git_checkout(temp_dir.path(), "existing-branch").unwrap();
        git_checkout_create(temp_dir.path(), "another-feature").unwrap();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let head_ref = repo.head().unwrap();
        let current_branch = head_ref.shorthand().unwrap();
        assert_eq!(current_branch, "another-feature");
    }

    #[test]
    fn test_git_checkout_file_empty_path_error() {
        let temp_dir = create_test_repo();
        let result = git_checkout_file(temp_dir.path(), "", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_file_nonexistent_error() {
        let temp_dir = create_test_repo();
        let result = git_checkout_file(temp_dir.path(), "nonexistent.txt", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_git_checkout_file_success() {
        let temp_dir = create_test_repo();

        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, World!").unwrap();

        let repo = Repository::open(temp_dir.path()).unwrap();
        let signature = repo.signature().unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let head_ref = repo.head().unwrap();
        let parent = head_ref.peel_to_commit().unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add test.txt",
            &tree,
            &[&parent],
        )
        .unwrap();

        std::fs::remove_file(&file_path).unwrap();

        git_checkout_file(temp_dir.path(), "test.txt", None).unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }
}
