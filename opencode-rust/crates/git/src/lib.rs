//! Git and CI Integration Library
//!
//! This crate provides Git repository management and CI/CD integration capabilities.
//!
//! ## GitLab Duo (Experimental)
//!
//! ⚠️ **Warning**: GitLab Duo support is **experimental** and subject to change.
//!
//! GitLab Duo features depend on:
//! - GitLab product tier (Premium/Ultimate required)
//! - Deployment configuration
//! - Environment setup
//!
//! See [`crate::gitlab_ci`] module for experimental GitLab CI features.

use git2::{DiffFormat, DiffOptions, Repository, StatusOptions};
use opencode_core::OpenCodeError;
use std::path::Path;

pub mod branch;
pub mod checkout;
pub mod github;
pub mod gitlab;
pub mod gitlab_ci;
pub mod merge;
pub mod push_pull;
pub mod rebase;
pub mod stash;
#[deprecated(
    since = "0.1.0",
    note = "GitLab Duo is experimental and environment-dependent. API may change in future releases."
)]
pub use gitlab_ci::{
    get_gitlab_ci_template, setup_gitlab_ci, GitLabCiSetupResult, GitLabCiTemplate,
    GitLabCiTrigger, GitLabJobResult, GitLabPipelineStatus, GitLabPipelineTriggerResult,
};
pub mod trigger;
pub mod workflow;
pub use github::{
    GitHubClient, GitHubError, GitHubIssue, GitHubPullRequest, GitHubRef, GitHubRepo, GitHubUser,
    IssueUpdates,
};
pub use gitlab::{
    GitLabCiVariable, GitLabClient, GitLabError, GitLabFileCommit, GitLabFileContent, GitLabJob,
    GitLabPipeline, GitLabProject,
};
pub use trigger::{
    CiSecrets, GitHubTrigger, GitHubTrigger as Trigger, TriggerContext, TriggerParseError,
};
pub use workflow::{
    get_workflow_template, setup_github_workflow, GitHubAppClient, GitHubError as AppError,
    SecretRequirement, SetupResult, WorkflowTemplate,
};

pub use merge::{git_merge, MergeResult};
pub use push_pull::{git_fetch, git_pull, git_push, PullResult, PushResult};
pub use rebase::{git_rebase, git_rebase_abort, git_rebase_status, RebaseResult, RebaseStatus};
pub use stash::{git_stash, git_stash_drop, git_stash_list, git_stash_pop, StashEntry};

pub struct GitManager {
    repo: Repository,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitOperation {
    Status,
    Diff,
    Log,
    Commit,
    Branch,
    Checkout,
}

impl GitManager {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, OpenCodeError> {
        let repo = Repository::discover(path)
            .map_err(|e| OpenCodeError::Tool(format!("Failed to open git repo: {}", e)))?;
        Ok(Self { repo })
    }

    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, OpenCodeError> {
        let repo = Repository::init(path)
            .map_err(|e| OpenCodeError::Tool(format!("Failed to init git repo: {}", e)))?;
        Ok(Self { repo })
    }

    pub fn status(&self) -> Result<String, OpenCodeError> {
        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        let statuses = self
            .repo
            .statuses(Some(&mut status_options))
            .map_err(|e| OpenCodeError::Tool(format!("Failed to get status: {}", e)))?;

        let mut result = String::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("unknown");
            let status = entry.status();
            result.push_str(&format!("{:?} {}\n", status, path));
        }
        Ok(result)
    }

    pub fn diff(&self) -> Result<String, OpenCodeError> {
        let mut diff_options = DiffOptions::new();
        let diff = self
            .repo
            .diff_index_to_workdir(None, Some(&mut diff_options))
            .map_err(|e| OpenCodeError::Tool(format!("Failed to get diff: {}", e)))?;

        let mut result = String::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            if let Ok(content) = std::str::from_utf8(line.content()) {
                result.push_str(content);
            }
            true
        })
        .map_err(|e| OpenCodeError::Tool(format!("Failed to print diff: {}", e)))?;

        Ok(result)
    }

    pub fn redact_credentials_from_url(&self, url: &str) -> String {
        if let Some(at_pos) = url.find('@') {
            if let Some(protocol_pos) = url.find("://") {
                if at_pos > protocol_pos + 3 {
                    let protocol = &url[..protocol_pos];
                    let host_and_path = &url[at_pos + 1..];
                    return format!("{}://{}", protocol, host_and_path);
                }
            } else if url.starts_with("git@") {
                if let Some(colon_pos) = url.find(':') {
                    if colon_pos < at_pos {
                        return format!("git@{}:{}", &url[4..colon_pos], &url[at_pos + 1..]);
                    }
                }
            }
        }

        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_operation_has_branch_variant() {
        let op = GitOperation::Branch;
        assert!(matches!(op, GitOperation::Branch));
    }

    #[test]
    fn git_operation_has_checkout_variant() {
        let op = GitOperation::Checkout;
        assert!(matches!(op, GitOperation::Checkout));
    }

    #[test]
    fn git_operation_all_variants() {
        assert!(matches!(GitOperation::Status, GitOperation::Status));
        assert!(matches!(GitOperation::Diff, GitOperation::Diff));
        assert!(matches!(GitOperation::Log, GitOperation::Log));
        assert!(matches!(GitOperation::Commit, GitOperation::Commit));
        assert!(matches!(GitOperation::Branch, GitOperation::Branch));
        assert!(matches!(GitOperation::Checkout, GitOperation::Checkout));
    }

    #[test]
    fn git_security_001_credentials_redacted_in_url() {
        let gm = GitManager::open(".").unwrap();

        let url_with_token = "https://user:token123@github.com/owner/repo.git";
        let redacted = gm.redact_credentials_from_url(url_with_token);
        assert_eq!(redacted, "https://github.com/owner/repo.git");
        assert!(!redacted.contains("token123"));
        assert!(!redacted.contains("user"));
    }

    #[test]
    fn git_security_001_ssh_url_credentials_redacted() {
        let gm = GitManager::open(".").unwrap();

        let ssh_url = "git@github.com:owner/repo.git";
        let redacted = gm.redact_credentials_from_url(ssh_url);
        assert_eq!(redacted, ssh_url);
    }

    #[test]
    fn git_security_001_url_without_credentials_unchanged() {
        let gm = GitManager::open(".").unwrap();

        let url = "https://github.com/owner/repo.git";
        let redacted = gm.redact_credentials_from_url(url);
        assert_eq!(redacted, url);
    }

    #[test]
    fn git_security_001_url_with_password_redacted() {
        let gm = GitManager::open(".").unwrap();

        let url_with_pass = "https://user:password@github.com/owner/repo.git";
        let redacted = gm.redact_credentials_from_url(url_with_pass);
        assert_eq!(redacted, "https://github.com/owner/repo.git");
        assert!(!redacted.contains("password"));
    }

    #[test]
    fn git_security_001_token_in_query_redacted() {
        let gm = GitManager::open(".").unwrap();

        let url_with_query = "https://github.com/owner/repo?token=abc123";
        let redacted = gm.redact_credentials_from_url(url_with_query);
        assert_eq!(redacted, url_with_query);
    }
}
