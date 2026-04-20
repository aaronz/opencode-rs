use git2::Repository;
use opencode_core::OpenCodeError;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushResult {
    pub refs_updated: usize,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullResult {
    Clean {
        commit: String,
        summary: String,
    },
    UpToDate {
        commit: String,
    },
    Conflict {
        conflicted_files: Vec<String>,
        summary: String,
    },
    NoUpstream {
        message: String,
    },
}

pub fn git_push(repo_path: &Path, remote: Option<&str>) -> Result<PushResult, OpenCodeError> {
    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let remote_name = remote.unwrap_or("origin");
    let mut remote_obj = repo
        .find_remote(remote_name)
        .map_err(|e| OpenCodeError::Tool(format!("Remote '{}' not found: {}", remote_name, e)))?;

    let head = repo
        .head()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?;

    let branch_name = head
        .shorthand()
        .ok_or_else(|| OpenCodeError::Tool("HEAD is not a branch".to_string()))?
        .to_string();

    let remote_branch_name = format!("refs/heads/{}", branch_name);

    let mut callbacks = git2::PushOptions::new();
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(|_url, username_from_url, _cred_type| {
        if let Some(username) = username_from_url {
            git2::Cred::ssh_key_from_agent(username)
        } else {
            git2::Cred::ssh_key_from_agent("git")
        }
    });
    callbacks.remote_callbacks(remote_callbacks);

    let refspec = format!("{}:{}", remote_branch_name, remote_branch_name);

    remote_obj
        .push(&[&refspec], Some(&mut callbacks))
        .map_err(|e| {
            let msg = e.message();
            if msg.contains("authentication") || msg.contains("auth") || msg.contains("credential")
            {
                OpenCodeError::ProviderAuthFailed(format!(
                    "Authentication failed for remote '{}': {}",
                    remote_name, msg
                ))
            } else if msg.contains("rejected") || msg.contains("denied") {
                OpenCodeError::InsufficientPermissions {
                    detail: Some(format!(
                        "Push to remote '{}' was rejected: {}",
                        remote_name, msg
                    )),
                    required_role: None,
                }
            } else {
                OpenCodeError::Tool(format!("Push failed: {}", msg))
            }
        })?;

    let summary = format!("Successfully pushed {} to {}", branch_name, remote_name);

    Ok(PushResult {
        refs_updated: 1,
        summary,
    })
}

pub fn git_pull(
    repo_path: &Path,
    remote: Option<&str>,
    branch: Option<&str>,
) -> Result<PullResult, OpenCodeError> {
    let repo = Repository::discover(repo_path)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to discover repository: {}", e)))?;

    let remote_name = remote.unwrap_or("origin");

    let mut remote_obj = repo
        .find_remote(remote_name)
        .map_err(|e| OpenCodeError::Tool(format!("Remote '{}' not found: {}", remote_name, e)))?;

    let branch_name = branch.map(String::from).unwrap_or_else(|| {
        repo.head()
            .ok()
            .and_then(|h| h.shorthand().map(String::from))
            .unwrap_or_else(|| "HEAD".to_string())
    });

    if branch_name == "HEAD" || branch_name.is_empty() {
        return Ok(PullResult::NoUpstream {
            message: "No upstream branch configured for the current branch".to_string(),
        });
    }

    let fetch_refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);

    let mut fetch_options = git2::FetchOptions::new();
    let mut remote_callbacks = git2::RemoteCallbacks::new();
    remote_callbacks.credentials(|_url, username_from_url, _cred_type| {
        if let Some(username) = username_from_url {
            git2::Cred::ssh_key_from_agent(username)
        } else {
            git2::Cred::ssh_key_from_agent("git")
        }
    });
    fetch_options.remote_callbacks(remote_callbacks);

    remote_obj
        .fetch(&[&fetch_refspec], Some(&mut fetch_options), None)
        .map_err(|e| {
            let msg = e.message();
            if msg.contains("authentication") || msg.contains("auth") || msg.contains("credential")
            {
                OpenCodeError::ProviderAuthFailed(format!(
                    "Authentication failed for remote '{}': {}",
                    remote_name, msg
                ))
            } else {
                OpenCodeError::Tool(format!("Fetch failed: {}", msg))
            }
        })?;

    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| OpenCodeError::Tool(format!("Failed to find FETCH_HEAD: {}", e)))?;

    let fetch_commit = fetch_head
        .peel_to_commit()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to peel FETCH_HEAD to commit: {}", e)))?;

    let head = repo
        .head()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get HEAD: {}", e)))?;
    let head_commit = head
        .peel_to_commit()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to peel HEAD to commit: {}", e)))?;

    if fetch_commit.id() == head_commit.id() {
        return Ok(PullResult::UpToDate {
            commit: head_commit.id().to_string(),
        });
    }

    let fetch_annotated = repo
        .find_annotated_commit(fetch_commit.id())
        .map_err(|e| OpenCodeError::Tool(format!("Failed to find annotated commit: {}", e)))?;

    let analysis = repo
        .merge_analysis(&[&fetch_annotated])
        .map_err(|e| OpenCodeError::Tool(format!("Failed to analyze merge: {}", e)))?;

    if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch_name);
        repo.reference(&refname, fetch_commit.id(), true, "fast-forward")
            .map_err(|e| OpenCodeError::Tool(format!("Failed to fast-forward: {}", e)))?;

        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.safe();
        repo.checkout_head(Some(&mut checkout_builder))
            .map_err(|e| OpenCodeError::Tool(format!("Failed to checkout: {}", e)))?;

        return Ok(PullResult::Clean {
            commit: fetch_commit.id().to_string(),
            summary: format!("Fast-forwarded {} to {}", branch_name, fetch_commit.id()),
        });
    }

    if analysis.0.is_up_to_date() {
        return Ok(PullResult::UpToDate {
            commit: head_commit.id().to_string(),
        });
    }

    let signature = repo
        .signature()
        .map_err(|e| OpenCodeError::Tool(format!("Failed to get signature: {}", e)))?;

    let mut merge_index = repo
        .merge_commits(&head_commit, &fetch_commit, None)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to merge commits: {}", e)))?;

    if merge_index.has_conflicts() {
        let conflicted_files: Vec<String> = merge_index
            .conflicts()
            .map_err(|e| OpenCodeError::Tool(format!("Failed to get conflicts: {}", e)))?
            .filter_map(|c| c.ok())
            .filter_map(|c| {
                let path = c.ancestor.or(c.our).or(c.their)?;
                String::from_utf8(path.path.to_vec()).ok()
            })
            .collect();

        repo.cleanup_state().ok();

        return Ok(PullResult::Conflict {
            conflicted_files,
            summary: format!(
                "Pull with merge conflicts detected for branch {}",
                branch_name
            ),
        });
    }

    let tree_oid = merge_index
        .write_tree_to(&repo)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to write merge tree: {}", e)))?;

    let tree = repo
        .find_tree(tree_oid)
        .map_err(|e| OpenCodeError::Tool(format!("Failed to find tree: {}", e)))?;

    let fetch_commit_for_parent = repo
        .find_commit(fetch_commit.id())
        .map_err(|e| OpenCodeError::Tool(format!("Failed to find fetch commit: {}", e)))?;

    let commit_oid = repo
        .commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!(
                "Merge branch '{}' of {} into {}",
                branch_name, remote_name, branch_name
            ),
            &tree,
            &[&head_commit, &fetch_commit_for_parent],
        )
        .map_err(|e| OpenCodeError::Tool(format!("Failed to create merge commit: {}", e)))?;

    Ok(PullResult::Clean {
        commit: commit_oid.to_string(),
        summary: format!(
            "Merged {} from {} into {}",
            fetch_commit.id(),
            remote_name,
            branch_name
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_result_struct_has_refs_updated() {
        let result = PushResult {
            refs_updated: 1,
            summary: "test".to_string(),
        };
        assert_eq!(result.refs_updated, 1);
    }

    #[test]
    fn test_pull_result_up_to_date_variant() {
        let result = PullResult::UpToDate {
            commit: "abc123".to_string(),
        };
        assert!(matches!(result, PullResult::UpToDate { .. }));
    }

    #[test]
    fn test_pull_result_no_upstream_variant() {
        let result = PullResult::NoUpstream {
            message: "no upstream".to_string(),
        };
        assert!(matches!(result, PullResult::NoUpstream { .. }));
    }

    #[test]
    fn test_pull_result_conflict_variant() {
        let result = PullResult::Conflict {
            conflicted_files: vec!["file1.txt".to_string()],
            summary: "conflict".to_string(),
        };
        assert!(matches!(result, PullResult::Conflict { .. }));
    }

    #[test]
    fn test_pull_result_clean_variant() {
        let result = PullResult::Clean {
            commit: "abc123".to_string(),
            summary: "clean merge".to_string(),
        };
        assert!(matches!(result, PullResult::Clean { .. }));
    }
}
