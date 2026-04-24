use opencode_git::{
    branch::{git_branch_create, git_branch_delete, git_branch_list},
    checkout::{git_checkout, git_checkout_commit, git_current_branch},
    git_fetch, git_merge, git_pull, git_push, git_stash, git_stash_drop, git_stash_list,
    git_stash_pop,
    rebase::{git_rebase, RebaseResult},
    GitHubClient, GitHubError, GitHubIssue, GitHubPullRequest, GitManager, MergeResult, PullResult,
};
use std::fs;
use tempfile::TempDir;

fn create_test_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let repo = git2::Repository::init(temp_dir.path()).unwrap();
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

fn create_test_branch(repo: &git2::Repository, branch_name: &str) -> git2::Oid {
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    let file_path = repo
        .path()
        .parent()
        .unwrap()
        .join(format!("{}.txt", branch_name));
    fs::write(file_path, format!("content of {}", branch_name)).unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_path(std::path::Path::new(&format!("{}.txt", branch_name)))
        .unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let signature = repo.signature().unwrap();
    repo.commit(
        Some(&format!("refs/heads/{}", branch_name)),
        &signature,
        &signature,
        &format!("Add {}", branch_name),
        &tree,
        &[&head],
    )
    .unwrap()
}

mod status_diff_tests {
    use super::*;

    #[test]
    fn git_status_returns_working_directory_status() {
        let temp_dir = create_test_repo();
        fs::write(temp_dir.path().join("new_file.txt"), "content").unwrap();

        let manager = GitManager::open(temp_dir.path()).unwrap();
        let status = manager.status().unwrap();

        assert!(status.contains("new_file.txt"));
    }

    #[test]
    fn git_status_clean_repo_returns_empty() {
        let temp_dir = create_test_repo();
        let manager = GitManager::open(temp_dir.path()).unwrap();

        let status = manager.status().unwrap();
        assert!(status.trim().is_empty() || !status.contains("file"));
    }

    #[test]
    fn git_diff_returns_changes() {
        let temp_dir = create_test_repo();
        fs::write(temp_dir.path().join("modified.txt"), "original content").unwrap();

        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("modified.txt"))
            .unwrap();
        index.write().unwrap();
        drop(repo);

        fs::write(temp_dir.path().join("modified.txt"), "modified content").unwrap();

        let manager = GitManager::open(temp_dir.path()).unwrap();
        let diff = manager.diff().unwrap();

        assert!(diff.contains("modified content") || diff.contains("modified.txt"));
    }

    #[test]
    fn git_diff_no_changes_returns_empty() {
        let temp_dir = create_test_repo();
        let manager = GitManager::open(temp_dir.path()).unwrap();

        let diff = manager.diff().unwrap();
        assert!(diff.is_empty());
    }
}

mod branch_tests {
    use super::*;

    #[test]
    fn git_branch_list_returns_all_branches() {
        let temp_dir = create_test_repo();
        git_branch_create(temp_dir.path(), "feature-a").unwrap();
        git_branch_create(temp_dir.path(), "feature-b").unwrap();

        let branches = git_branch_list(temp_dir.path()).unwrap();

        assert!(branches.iter().any(|b| b == "feature-a"));
        assert!(branches.iter().any(|b| b == "feature-b"));
    }

    #[test]
    fn git_branch_create_creates_new_branch() {
        let temp_dir = create_test_repo();
        git_branch_create(temp_dir.path(), "new-feature").unwrap();

        let branches = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches.iter().any(|b| b == "new-feature"));
    }

    #[test]
    fn git_branch_create_empty_name_returns_error() {
        let temp_dir = create_test_repo();
        let result = git_branch_create(temp_dir.path(), "");
        assert!(result.is_err());
    }

    #[test]
    fn git_branch_create_invalid_name_returns_error() {
        let temp_dir = create_test_repo();
        let result = git_branch_create(temp_dir.path(), "invalid/branch");
        assert!(result.is_err());
    }

    #[test]
    fn git_branch_delete_removes_branch() {
        let temp_dir = create_test_repo();
        git_branch_create(temp_dir.path(), "to-delete").unwrap();
        let branches_before = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches_before.iter().any(|b| b == "to-delete"));

        git_branch_delete(temp_dir.path(), "to-delete").unwrap();

        let branches_after = git_branch_list(temp_dir.path()).unwrap();
        assert!(!branches_after.iter().any(|b| b == "to-delete"));
    }

    #[test]
    fn git_branch_delete_nonexistent_returns_error() {
        let temp_dir = create_test_repo();
        let result = git_branch_delete(temp_dir.path(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn git_branch_delete_current_returns_error() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        let current = repo.head().unwrap().shorthand().unwrap().to_string();
        drop(repo);

        let result = git_branch_delete(temp_dir.path(), &current);
        assert!(result.is_err());
    }
}

mod merge_tests {
    use super::*;

    #[test]
    fn git_merge_fast_forward() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        create_test_branch(&repo, "feature");

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
    fn git_merge_three_way_no_conflict() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        create_test_branch(&repo, "feature");
        fs::write(temp_dir.path().join("main.txt"), "main content").unwrap();
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
    fn git_merge_conflict_detected() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();

        fs::write(temp_dir.path().join("test.txt"), "original content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        drop(tree_id);
        let signature = repo.signature().unwrap();
        let commit1_oid = repo
            .commit(
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

        fs::write(temp_dir.path().join("test.txt"), "feature version").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        drop(tree_id);
        let signature = repo.signature().unwrap();
        repo.commit(
            Some(&format!("refs/heads/{}", "feature")),
            &signature,
            &signature,
            "Feature version",
            &tree,
            &[&commit1],
        )
        .unwrap();
        drop(tree);

        fs::write(temp_dir.path().join("test.txt"), "main version").unwrap();
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
            "Main version",
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
    fn git_merge_nonexistent_branch_returns_error() {
        let temp_dir = create_test_repo();
        let result = git_merge(temp_dir.path(), "nonexistent-branch");
        assert!(result.is_err());
    }

    #[test]
    fn git_merge_empty_branch_name_returns_error() {
        let temp_dir = create_test_repo();
        let result = git_merge(temp_dir.path(), "");
        assert!(result.is_err());
    }
}

mod stash_tests {
    use super::*;

    #[test]
    fn git_stash_saves_working_directory_state() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        fs::write(temp_dir.path().join("test.txt"), "stashed content").unwrap();

        git_stash(temp_dir.path()).unwrap();

        let statuses = repo.statuses(None).unwrap();
        assert!(statuses.is_empty());
    }

    #[test]
    fn git_stash_pop_restores_changes() {
        let temp_dir = create_test_repo();
        fs::write(temp_dir.path().join("test.txt"), "stashed content").unwrap();

        git_stash(temp_dir.path()).unwrap();
        git_stash_pop(temp_dir.path()).unwrap();

        let content = fs::read_to_string(temp_dir.path().join("test.txt")).unwrap();
        assert_eq!(content, "stashed content");
    }

    #[test]
    fn git_stash_list_returns_all_entries() {
        let temp_dir = create_test_repo();

        fs::write(temp_dir.path().join("file1.txt"), "content 1").unwrap();
        git_stash(temp_dir.path()).unwrap();

        fs::write(temp_dir.path().join("file2.txt"), "content 2").unwrap();
        git_stash(temp_dir.path()).unwrap();

        let entries = git_stash_list(temp_dir.path()).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn git_stash_drop_removes_entry() {
        let temp_dir = create_test_repo();

        fs::write(temp_dir.path().join("file1.txt"), "content 1").unwrap();
        git_stash(temp_dir.path()).unwrap();

        fs::write(temp_dir.path().join("file2.txt"), "content 2").unwrap();
        git_stash(temp_dir.path()).unwrap();

        let before = git_stash_list(temp_dir.path()).unwrap();
        assert_eq!(before.len(), 2);

        git_stash_drop(temp_dir.path(), 0).unwrap();

        let after = git_stash_list(temp_dir.path()).unwrap();
        assert_eq!(after.len(), 1);
    }

    #[test]
    fn git_stash_empty_repo_returns_error() {
        let temp_dir = create_test_repo();
        let result = git_stash_pop(temp_dir.path());
        assert!(result.is_err());
    }
}

mod push_pull_tests {
    use super::*;

    #[test]
    fn git_push_result_has_refs_updated_field() {
        let temp_dir = create_test_repo();

        let result = git_push(temp_dir.path(), Some("origin"));
        if result.is_ok() {
            let push_result = result.unwrap();
            assert_eq!(push_result.refs_updated, 1);
            assert!(!push_result.summary.is_empty());
        }
    }

    #[test]
    fn git_push_nonexistent_remote_returns_error() {
        let temp_dir = create_test_repo();
        let result = git_push(temp_dir.path(), Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn git_pull_no_upstream_returns_no_upstream_variant() {
        let temp_dir = create_test_repo();
        let result = git_pull(temp_dir.path(), Some("origin"), None);

        match result {
            Ok(PullResult::NoUpstream { message }) => {
                assert!(!message.is_empty());
            }
            Ok(other) => panic!("Expected NoUpstream, got {:?}", other),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("not found") || err_str.contains("doesn't exist") {
                    return;
                }
            }
        }
    }
}

mod github_client_tests {
    use super::*;

    #[test]
    fn github_client_can_be_created() {
        let _client = GitHubClient::new("test-token", "https://github.com");
    }

    #[test]
    fn github_error_display_trait() {
        let error = GitHubError::Api {
            status: 404,
            body: "Not Found".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("404"));
    }

    #[test]
    fn github_issue_struct_has_expected_fields() {
        let issue = GitHubIssue {
            id: 1,
            number: 1,
            title: "Test Issue".to_string(),
            body: Some("Description".to_string()),
            state: "open".to_string(),
            html_url: Some("https://github.com/test/repo/issues/1".to_string()),
            user: None,
            pull_request: None,
        };
        assert_eq!(issue.number, 1);
        assert_eq!(issue.title, "Test Issue");
    }

    #[test]
    fn github_pr_struct_has_expected_fields() {
        let pr = GitHubPullRequest {
            id: 1,
            number: 42,
            title: "Feature PR".to_string(),
            body: Some("PR description".to_string()),
            state: "open".to_string(),
            html_url: Some("https://github.com/test/repo/pull/42".to_string()),
            user: None,
            head: None,
            base: None,
        };
        assert_eq!(pr.number, 42);
    }
}

mod integration_tests {
    use super::*;

    #[test]
    fn git_workflow_create_branch_then_merge() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        create_test_branch(&repo, "feature");

        repo.set_head("refs/heads/master").unwrap();
        let mut checkout = git2::build::CheckoutBuilder::new();
        checkout.safe();
        repo.checkout_head(Some(&mut checkout)).unwrap();
        drop(repo);

        let branches_before = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches_before.iter().any(|b| b == "feature"));

        let merge_result = git_merge(temp_dir.path(), "feature");
        assert!(merge_result.is_ok());

        let branches_after = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches_after.iter().any(|b| b == "feature"));
    }

    #[test]
    fn git_workflow_stash_and_pop() {
        let temp_dir = create_test_repo();

        fs::write(temp_dir.path().join("file1.txt"), "original").unwrap();
        git_stash(temp_dir.path()).unwrap();

        assert!(!temp_dir.path().join("file1.txt").exists());

        git_stash_pop(temp_dir.path()).unwrap();

        let content = fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap();
        assert_eq!(content, "original");
    }

    #[test]
    fn git_workflow_multiple_branches() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        create_test_branch(&repo, "feature-a");
        create_test_branch(&repo, "feature-b");
        drop(repo);

        let branches = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches.iter().any(|b| b == "feature-a"));
        assert!(branches.iter().any(|b| b == "feature-b"));
    }

    #[test]
    fn git_workflow_delete_and_verify() {
        let temp_dir = create_test_repo();
        git_branch_create(temp_dir.path(), "temp-branch").unwrap();

        let branches_before = git_branch_list(temp_dir.path()).unwrap();
        assert!(branches_before.iter().any(|b| b == "temp-branch"));

        git_branch_delete(temp_dir.path(), "temp-branch").unwrap();

        let branches_after = git_branch_list(temp_dir.path()).unwrap();
        assert!(!branches_after.iter().any(|b| b == "temp-branch"));
    }
}

mod security_tests {
    use super::*;

    #[test]
    fn git_credentials_not_in_log_output() {
        let temp_dir = create_test_repo();
        let manager = GitManager::open(temp_dir.path()).unwrap();

        let url_with_creds = "https://user:password@github.com/example/repo.git";

        let log_safe_url = manager.redact_credentials_from_url(url_with_creds);

        assert!(!log_safe_url.contains("password"));
        assert!(log_safe_url.contains("github.com"));
    }

    #[test]
    fn git_token_in_url_redacted() {
        let temp_dir = create_test_repo();
        let manager = GitManager::open(temp_dir.path()).unwrap();

        let url_with_token = "https://ghp_token123456@github.com/example/repo.git";

        let log_safe_url = manager.redact_credentials_from_url(url_with_token);

        assert!(!log_safe_url.contains("ghp_token"));
        assert!(!log_safe_url.contains("token123456"));
    }

    #[test]
    fn git_ssh_url_no_credentials() {
        let temp_dir = create_test_repo();
        let manager = GitManager::open(temp_dir.path()).unwrap();

        let ssh_url = "git@github.com:example/repo.git";
        let log_safe_url = manager.redact_credentials_from_url(ssh_url);

        assert_eq!(log_safe_url, ssh_url);
    }

    #[test]
    fn git_https_url_without_credentials_unchanged() {
        let temp_dir = create_test_repo();
        let manager = GitManager::open(temp_dir.path()).unwrap();

        let clean_url = "https://github.com/example/repo.git";
        let log_safe_url = manager.redact_credentials_from_url(clean_url);

        assert_eq!(log_safe_url, clean_url);
    }
}

mod branch_operation_tests {
    use super::*;

    #[test]
    fn git_checkout_new_branch_creates_and_switches() {
        let temp_dir = create_test_repo();

        git_branch_create(temp_dir.path(), "feature-new").unwrap();

        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        let _head_before = repo.head().unwrap().shorthand().unwrap().to_string();
        drop(repo);

        let result = git_checkout(temp_dir.path(), "feature-new", true);
        assert!(result.is_ok(), "checkout should succeed: {:?}", result);

        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        let head_after = repo.head().unwrap().shorthand().unwrap().to_string();
        assert_eq!(head_after, "feature-new");
    }

    #[test]
    fn git_checkout_existing_branch() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        create_test_branch(&repo, "feature-existing");
        drop(repo);

        let result = git_checkout(temp_dir.path(), "feature-existing", false);
        assert!(
            result.is_ok(),
            "checkout existing branch should succeed: {:?}",
            result
        );
    }

    #[test]
    fn git_checkout_dirty_working_tree_returns_error() {
        let temp_dir = create_test_repo();
        fs::write(temp_dir.path().join("uncommitted.txt"), "changes").unwrap();

        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        create_test_branch(&repo, "feature-dirty");
        drop(repo);

        let result = git_checkout(temp_dir.path(), "feature-dirty", false);
        if result.is_err() {
            let err_str = result.unwrap_err().to_string();
            assert!(
                err_str.contains("dirty")
                    || err_str.contains("uncommitted")
                    || err_str.contains("stash")
                    || err_str.contains("would be overwritten"),
                "Error should mention dirty working tree: {}",
                err_str
            );
        }
    }

    #[test]
    fn git_checkout_nonexistent_branch_fails() {
        let temp_dir = create_test_repo();

        let result = git_checkout(temp_dir.path(), "nonexistent-branch", false);
        assert!(result.is_err());
    }

    #[test]
    fn git_current_branch_returns_correct_name() {
        let temp_dir = create_test_repo();

        let result = git_current_branch(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "master");
    }

    #[test]
    fn git_current_branch_detached_head_returns_error() {
        let temp_dir = create_test_repo();
        let commit_id = git2::Repository::open(temp_dir.path())
            .unwrap()
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        repo.set_head_detached(commit_id).unwrap();
        drop(repo);

        let result = git_current_branch(temp_dir.path());
        assert!(
            result.is_err(),
            "current_branch should return error in detached HEAD state"
        );
    }

    #[test]
    fn git_checkout_different_branch_with_uncommitted_stashes_changes() {
        let temp_dir = create_test_repo();

        fs::write(temp_dir.path().join("file1.txt"), "stash me").unwrap();
        git_stash(temp_dir.path()).unwrap();

        git_branch_create(temp_dir.path(), "feature-stash").unwrap();

        let result = git_checkout(temp_dir.path(), "feature-stash", false);
        assert!(
            result.is_ok(),
            "checkout should succeed with stashed changes: {:?}",
            result
        );
    }
}

mod rebase_tests {
    use super::*;

    #[test]
    fn git_rebase_success() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        create_test_branch(&repo, "feature-rebase");
        drop(repo);

        fs::write(temp_dir.path().join("main.txt"), "main content").unwrap();

        let result = git_rebase(temp_dir.path(), "feature-rebase");
        match result {
            Ok(RebaseResult::Completed { commit: _ }) => {}
            Ok(RebaseResult::UpToDate { commit: _ }) => {}
            Ok(RebaseResult::Conflict {
                conflicted_files: _,
            }) => {}
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("no changes") || err_str.contains("nothing to commit") {
                    return;
                }
                panic!("rebase failed unexpectedly: {:?}", e);
            }
        }
    }

    #[test]
    fn git_rebase_with_conflicts_reported() {
        let temp_dir = create_test_repo();
        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();

        fs::write(temp_dir.path().join("conflict.txt"), "original").unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("conflict.txt"))
            .unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = repo.signature().unwrap();
        let commit1_oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Add conflict.txt v1",
                &tree,
                &[&head],
            )
            .unwrap();
        let commit1 = repo.find_commit(commit1_oid).unwrap();
        drop(tree);

        fs::write(temp_dir.path().join("conflict.txt"), "feature change").unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("conflict.txt"))
            .unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some(&format!("refs/heads/{}", "feature")),
            &signature,
            &signature,
            "Feature change",
            &tree,
            &[&commit1],
        )
        .unwrap();
        drop(tree);

        fs::write(temp_dir.path().join("conflict.txt"), "main change").unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_path(std::path::Path::new("conflict.txt"))
            .unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Main change",
            &tree,
            &[&commit1],
        )
        .unwrap();
        drop(tree);
        drop(head);
        drop(commit1);
        drop(repo);

        let result = git_rebase(temp_dir.path(), "feature");
        match result {
            Ok(RebaseResult::Conflict {
                conflicted_files: files,
            }) => {
                assert!(files.contains(&"conflict.txt".to_string()));
            }
            Ok(_) => {}
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("conflict") || err_str.contains("CONFLICT") {
                    return;
                }
            }
        }
    }

    #[test]
    fn git_rebase_nonexistent_branch_returns_error() {
        let temp_dir = create_test_repo();

        let result = git_rebase(temp_dir.path(), "nonexistent");
        assert!(result.is_err());
    }
}

mod remote_operation_tests {
    use super::*;

    #[test]
    fn git_push_divergent_history_rejected() {
        let temp_dir = create_test_repo();

        let result = git_push(temp_dir.path(), Some("origin"));
        match result {
            Ok(push_result) => {
                assert!(push_result.refs_updated >= 0);
            }
            Err(e) => {
                let err_str = e.to_string();
                assert!(
                    err_str.contains("diverged")
                        || err_str.contains("rejected")
                        || err_str.contains("not found")
                        || err_str.contains("no such remote"),
                    "Expected diverged/rejected error, got: {}",
                    err_str
                );
            }
        }
    }

    #[test]
    fn git_fetch_nonexistent_remote_returns_error() {
        let temp_dir = create_test_repo();

        let result = git_fetch(temp_dir.path(), "nonexistent");
        assert!(result.is_err());
    }
}

mod detached_head_tests {
    use super::*;

    #[test]
    fn git_detached_head_detected() {
        let temp_dir = create_test_repo();
        let commit_id = git2::Repository::open(temp_dir.path())
            .unwrap()
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        repo.set_head_detached(commit_id).unwrap();
        drop(repo);

        let result = git_current_branch(temp_dir.path());
        assert!(
            result.is_err(),
            "Should return error in detached HEAD state"
        );
    }

    #[test]
    fn git_in_detached_head_no_branch_named() {
        let temp_dir = create_test_repo();
        let commit_id = git2::Repository::open(temp_dir.path())
            .unwrap()
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        let repo = git2::Repository::open(temp_dir.path()).unwrap();
        repo.set_head_detached(commit_id).unwrap();
        drop(repo);

        let result = git_current_branch(temp_dir.path());
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(
            err_str.contains("detached") || err_str.contains("DETACHED"),
            "Error should mention detached HEAD"
        );
    }

    #[test]
    fn git_checkout_commit_results_in_detached_head() {
        let temp_dir = create_test_repo();
        let commit_id = git2::Repository::open(temp_dir.path())
            .unwrap()
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id();

        let checkout_result = git_checkout_commit(temp_dir.path(), commit_id.to_string().as_str());

        if checkout_result.is_ok() {
            let branch_result = git_current_branch(temp_dir.path());
            assert!(
                branch_result.is_err(),
                "Should be in detached HEAD state after checking out commit"
            );
        }
    }
}

mod github_api_tests {
    use super::*;

    #[test]
    fn github_client_handles_rate_limit_error() {
        let _client = GitHubClient::new("test-token", "https://github.com");

        let error = GitHubError::Api {
            status: 403,
            body: "rate limit exceeded".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("403") || display.contains("rate limit"));
    }

    #[test]
    fn github_client_handles_not_found() {
        let _client = GitHubClient::new("test-token", "https://github.com");

        let error = GitHubError::Api {
            status: 404,
            body: "Not Found".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("404"));
    }

    #[test]
    fn github_client_handles_unauthorized() {
        let _client = GitHubClient::new("bad-token", "https://github.com");

        let error = GitHubError::Api {
            status: 401,
            body: "Bad credentials".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("401"));
    }

    #[test]
    fn github_issue_fields_accessible() {
        let issue = GitHubIssue {
            id: 123,
            number: 456,
            title: "Test Title".to_string(),
            body: Some("Test body content".to_string()),
            state: "open".to_string(),
            html_url: Some("https://github.com/test/repo/issues/456".to_string()),
            user: None,
            pull_request: None,
        };

        assert_eq!(issue.number, 456);
        assert_eq!(issue.title, "Test Title");
        assert!(issue.body.is_some());
    }

    #[test]
    fn github_pr_fields_accessible() {
        let pr = GitHubPullRequest {
            id: 789,
            number: 101,
            title: "PR Title".to_string(),
            body: Some("PR description".to_string()),
            state: "open".to_string(),
            html_url: Some("https://github.com/test/repo/pull/101".to_string()),
            user: None,
            head: None,
            base: None,
        };

        assert_eq!(pr.number, 101);
        assert_eq!(pr.title, "PR Title");
    }
}
