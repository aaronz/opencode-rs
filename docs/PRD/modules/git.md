# git.md — Git Module

## Module Overview

- **Crate**: `opencode-git`
- **Source**: `crates/git/src/lib.rs`
- **Status**: Fully implemented — PRD reflects actual Rust API
- **Purpose**: Git operations via `git2` library, GitHub/GitLab API integration for issues, PRs, and repository management.

---

## Crate Layout

```
crates/git/src/
├── lib.rs              ← Public re-exports
└── [various modules]
```

**Key Cargo.toml dependencies**:
```toml
[dependencies]
git2 = "0.19"
reqwest = { version = "0.12" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tokio = { version = "1.45", features = ["full"] }

opencode-core = { path = "../core" }
```

---

## Core Types

### GitManager

```rust
pub struct GitManager {
    repo: Repository,  // git2::Repository
}

impl GitManager {
    pub fn open(path: &Path) -> Result<Self, GitError>;
    pub fn init(path: &Path) -> Result<Self, GitError>;

    pub fn status(&self) -> Result<Vec<StatusEntry>, GitError>;
    pub fn diff(&self, path: Option<&Path>) -> Result<String, GitError>;
    pub fn commit(&self, message: &str) -> Result<CommitId, GitError>;
    pub fn log(&self, max_count: usize) -> Result<Vec<CommitInfo>, GitError>;
    pub fn branches(&self) -> Result<Vec<BranchInfo>, GitError>;
    pub fn current_branch(&self) -> Result<BranchInfo, GitError>;
    pub fn checkout(&self, refspec: &str) -> Result<(), GitError>;
    pub fn pull(&self) -> Result<(), GitError>;
    pub fn push(&self) -> Result<(), GitError>;
}

pub struct StatusEntry {
    pub path: String,
    pub status: FileStatus,
}

pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
}

pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
}

pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Conflicted,
    Ignored,
    Untracked,
}
```

### GitHub Client

```rust
pub struct GitHubClient {
    http: reqwest::Client,
    token: Option<String>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self;
    pub fn with_base_url(base_url: &str, token: Option<String>) -> Self;

    pub async fn get_repos(&self) -> Result<Vec<GitHubRepo>, GitHubError>;
    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo, GitHubError>;
    pub async fn create_issue(&self, owner: &str, repo: &str, issue: GitHubIssue) -> Result<GitHubIssue, GitHubError>;
    pub async fn get_issues(&self, owner: &str, repo: &str) -> Result<Vec<GitHubIssue>, GitHubError>;
    pub async fn create_pr(&self, owner: &str, repo: &str, pr: GitHubPullRequest) -> Result<GitHubPullRequest, GitHubError>;
    pub async fn get_user(&self, username: &str) -> Result<GitHubUser, GitHubError>;
}

pub struct GitHubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub private: bool,
    pub html_url: String,
}

pub struct GitHubIssue {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<String>,
}

pub struct GitHubPullRequest {
    pub title: String,
    pub body: Option<String>,
    pub head: String,
    pub base: String,
    pub draft: bool,
}

pub struct GitHubUser {
    pub login: String,
    pub id: i64,
    pub avatar_url: String,
    pub html_url: String,
}

pub struct GitHubRef {
    pub ref_name: String,
    pub sha: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("API error: {0}")]
    Api(String),
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
```

### GitLab Client

```rust
pub struct GitLabClient {
    http: reqwest::Client,
    token: Option<String>,
    base_url: String,
}

impl GitLabClient {
    pub fn new(token: Option<String>) -> Self;
    pub async fn get_projects(&self) -> Result<Vec<GitLabProject>, GitLabError>;
    pub async fn create_issue(&self, project_id: &str, issue: GitLabIssue) -> Result<GitLabIssue, GitLabError>;
}

pub struct GitLabProject {
    pub id: i64,
    pub name: String,
    pub path_with_namespace: String,
}

pub struct GitLabIssue {
    pub iid: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GitLabError {
    #[error("API error: {0}")]
    Api(String),
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
```

---

## Inter-Crate Dependencies

| Dependant Crate | What it uses from `opencode-git` |
|---|---|
| `opencode-tools` | `GitTools` as a tool (git_commit, git_diff, git_status, etc.) |
| `opencode-server` | GitHub/GitLab API for integrations |

---

## Test Design

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_git_manager_open_invalid_path() {
        let result = GitManager::open(Path::new("/nonexistent/repo"));
        assert!(result.is_err());
    }

    #[test]
    fn test_github_error_display() {
        let err = GitHubError::NotFound;
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_file_status_variants() {
        use std::fs;
        // Create temp repo and test status variants
    }
}
```
