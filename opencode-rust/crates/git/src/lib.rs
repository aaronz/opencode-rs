use git2::{DiffFormat, DiffOptions, Repository, StatusOptions};
use opencode_core::OpenCodeError;
use std::path::Path;

pub mod github;
pub mod gitlab;
pub mod gitlab_ci;
pub mod trigger;
pub mod workflow;
pub use github::{
    GitHubClient, GitHubError, GitHubIssue, GitHubPullRequest, GitHubRef, GitHubRepo, GitHubUser,
    IssueUpdates,
};
pub use gitlab::{
    GitLabCiVariable, GitLabClient, GitLabError, GitLabFileCommit, GitLabFileContent,
    GitLabPipeline, GitLabProject,
};
pub use gitlab_ci::{
    get_gitlab_ci_template, setup_gitlab_ci, GitLabCiSetupResult, GitLabCiTemplate,
};
pub use trigger::{
    CiSecrets, GitHubTrigger, GitHubTrigger as Trigger, TriggerContext, TriggerParseError,
};
pub use workflow::{
    get_workflow_template, setup_github_workflow, GitHubAppClient, GitHubError as AppError,
    SecretRequirement, SetupResult, WorkflowTemplate,
};

pub struct GitManager {
    repo: Repository,
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
}
