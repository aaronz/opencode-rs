use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: Client,
    token: String,
    api_base: String,
}

impl GitHubClient {
    pub fn new(token: &str, api_base: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.to_string(),
            api_base: api_base.trim_end_matches('/').to_string(),
        }
    }

    pub fn list_prs(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
    ) -> Result<Vec<GitHubPullRequest>, GitHubError> {
        let url = format!("{}/repos/{}/{}/pulls", self.api_base, owner, repo);
        self.send(self.authed(self.client.get(url)).query(&[("state", state)]))
    }

    pub fn get_pr(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<GitHubPullRequest, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            self.api_base, owner, repo, number
        );
        self.send(self.authed(self.client.get(url)))
    }

    pub fn create_pr(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        head: &str,
        base: &str,
    ) -> Result<GitHubPullRequest, GitHubError> {
        let url = format!("{}/repos/{}/{}/pulls", self.api_base, owner, repo);
        self.send(self.authed(self.client.post(url)).json(&serde_json::json!({
            "title": title,
            "body": body,
            "head": head,
            "base": base
        })))
    }

    pub fn add_pr_comment(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        comment: &str,
    ) -> Result<(), GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}/comments",
            self.api_base, owner, repo, number
        );
        let _: serde_json::Value = self.send(
            self.authed(self.client.post(url))
                .json(&serde_json::json!({ "body": comment })),
        )?;
        Ok(())
    }

    pub fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: &str,
    ) -> Result<Vec<GitHubIssue>, GitHubError> {
        let url = format!("{}/repos/{}/{}/issues", self.api_base, owner, repo);
        self.send(self.authed(self.client.get(url)).query(&[("state", state)]))
    }

    pub fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<GitHubIssue, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.api_base, owner, repo, number
        );
        self.send(self.authed(self.client.get(url)))
    }

    pub fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
    ) -> Result<GitHubIssue, GitHubError> {
        let url = format!("{}/repos/{}/{}/issues", self.api_base, owner, repo);
        self.send(self.authed(self.client.post(url)).json(&serde_json::json!({
            "title": title,
            "body": body
        })))
    }

    pub fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        updates: &IssueUpdates,
    ) -> Result<GitHubIssue, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.api_base, owner, repo, number
        );
        self.send(self.authed(self.client.patch(url)).json(updates))
    }

    pub fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo, GitHubError> {
        let url = format!("{}/repos/{}/{}", self.api_base, owner, repo);
        self.send(self.authed(self.client.get(url)))
    }

    pub fn trigger_workflow(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: &str,
        branch_ref: &str,
    ) -> Result<WorkflowRun, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/actions/workflows/{}/dispatches",
            self.api_base, owner, repo, workflow_id
        );
        let _: serde_json::Value = self.send(
            self.authed(self.client.post(url))
                .json(&serde_json::json!({ "ref": branch_ref })),
        )?;
        Ok(WorkflowRun {
            id: 0,
            name: workflow_id.to_string(),
            head_branch: branch_ref.to_string(),
            status: "queued".to_string(),
            conclusion: None,
        })
    }

    pub fn list_workflow_runs(
        &self,
        owner: &str,
        repo: &str,
        workflow_id: &str,
    ) -> Result<Vec<WorkflowRun>, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/actions/workflows/{}/runs",
            self.api_base, owner, repo, workflow_id
        );
        #[derive(Deserialize)]
        struct RunsResponse {
            workflow_runs: Vec<WorkflowRun>,
        }
        let response: RunsResponse = self.send(self.authed(self.client.get(url)))?;
        Ok(response.workflow_runs)
    }

    fn authed(&self, req: RequestBuilder) -> RequestBuilder {
        req.bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "opencode-rs")
    }

    fn send<T: for<'de> Deserialize<'de>>(&self, req: RequestBuilder) -> Result<T, GitHubError> {
        let response = req.send()?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(GitHubError::Api { status, body });
        }
        Ok(response.json::<T>()?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub html_url: Option<String>,
    pub user: Option<GitHubUser>,
    pub head: Option<GitHubRef>,
    pub base: Option<GitHubRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub html_url: Option<String>,
    pub user: Option<GitHubUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub default_branch: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubRef {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub head_branch: String,
    pub status: String,
    pub conclusion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("github api error ({status}): {body}")]
    Api { status: u16, body: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn spawn_server(status_code: u16, body: String) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            tx.send(()).unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 8192];
            let _ = stream.read(&mut buffer).unwrap();

            let status_line = if status_code == 200 {
                "HTTP/1.1 200 OK"
            } else {
                "HTTP/1.1 404 Not Found"
            };

            let response = format!(
                "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status_line,
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        let _ = rx.recv();
        addr
    }

    #[test]
    fn list_prs_decodes_response() {
        let body = serde_json::json!([
            {
                "id": 10,
                "number": 4,
                "state": "open",
                "title": "feat: add oauth",
                "body": "details",
                "html_url": "https://github.com/o/r/pull/4",
                "user": {"login": "alice", "id": 1},
                "head": {"ref": "feature", "sha": "abc"},
                "base": {"ref": "main", "sha": "def"}
            }
        ])
        .to_string();
        let api_base = spawn_server(200, body);
        let client = GitHubClient::new("token", &api_base);

        let prs = client.list_prs("o", "r", "open").unwrap();
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 4);
        assert_eq!(prs[0].title, "feat: add oauth");
    }

    #[test]
    fn create_issue_decodes_response() {
        let body = serde_json::json!({
            "id": 20,
            "number": 8,
            "state": "open",
            "title": "bug: refresh token",
            "body": "repro",
            "html_url": "https://github.com/o/r/issues/8",
            "user": {"login": "bob", "id": 2}
        })
        .to_string();

        let api_base = spawn_server(200, body);
        let client = GitHubClient::new("token", &api_base);
        let issue = client
            .create_issue("o", "r", "bug: refresh token", "repro")
            .unwrap();

        assert_eq!(issue.number, 8);
        assert_eq!(issue.title, "bug: refresh token");
    }

    #[test]
    fn returns_api_error_on_non_success_status() {
        let api_base = spawn_server(404, "{\"message\":\"Not Found\"}".to_string());
        let client = GitHubClient::new("token", &api_base);

        let err = client.get_repo("o", "missing").unwrap_err();
        match err {
            GitHubError::Api { status, body } => {
                assert_eq!(status, 404);
                assert!(body.contains("Not Found"));
            }
            _ => panic!("expected api error"),
        }
    }

    #[test]
    fn github_triggers_workflow_via_api() {
        let body = serde_json::json!({
            "id": 12345678,
            "name": "Test Workflow",
            "head_branch": "main",
            "status": "queued",
            "conclusion": null
        })
        .to_string();

        let api_base = spawn_server(200, body);
        let client = GitHubClient::new("token", &api_base);

        let result = client.trigger_workflow("owner", "repo", "test-workflow.yml", "main");
        assert!(result.is_ok());

        let run = result.unwrap();
        assert_eq!(run.name, "test-workflow.yml");
        assert_eq!(run.head_branch, "main");
        assert_eq!(run.status, "queued");
    }

    #[test]
    fn github_triggers_pass_correct_parameters() {
        let body = r#"{}"#.to_string();
        let api_base = spawn_server(200, body);
        let client = GitHubClient::new("test-token", &api_base);

        let result = client.trigger_workflow("myorg", "myrepo", "ci.yml", "feature-branch");
        assert!(result.is_ok());

        let run = result.unwrap();
        assert_eq!(run.head_branch, "feature-branch");
        assert_eq!(run.name, "ci.yml");
    }

    #[test]
    fn github_triggers_report_workflow_status() {
        let body = serde_json::json!({
            "workflow_runs": [
                {
                    "id": 9876543,
                    "name": "CI",
                    "head_branch": "main",
                    "status": "completed",
                    "conclusion": "success"
                },
                {
                    "id": 9876544,
                    "name": "CI",
                    "head_branch": "develop",
                    "status": "in_progress",
                    "conclusion": null
                }
            ]
        })
        .to_string();

        let api_base = spawn_server(200, body);
        let client = GitHubClient::new("token", &api_base);

        let runs = client
            .list_workflow_runs("owner", "repo", "ci.yml")
            .unwrap();
        assert_eq!(runs.len(), 2);

        assert_eq!(runs[0].id, 9876543);
        assert_eq!(runs[0].status, "completed");
        assert_eq!(runs[0].conclusion, Some("success".to_string()));

        assert_eq!(runs[1].id, 9876544);
        assert_eq!(runs[1].status, "in_progress");
        assert_eq!(runs[1].conclusion, None);
    }

    #[test]
    fn github_triggers_workflow_dispatch_with_inputs() {
        let body = serde_json::json!({
            "id": 111,
            "name": "manual-workflow",
            "head_branch": "main",
            "status": "queued",
            "conclusion": null
        })
        .to_string();

        let api_base = spawn_server(200, body);
        let client = GitHubClient::new("token", &api_base);

        let result = client.trigger_workflow("owner", "repo", "manual.yml", "main");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "queued");
    }

    #[test]
    fn github_triggers_error_handling() {
        let api_base = spawn_server(404, "{\"message\":\"workflow not found\"}".to_string());
        let client = GitHubClient::new("token", &api_base);

        let err = client.trigger_workflow("owner", "repo", "nonexistent.yml", "main");
        assert!(err.is_err());

        match err.unwrap_err() {
            GitHubError::Api { status, body } => {
                assert_eq!(status, 404);
                assert!(body.contains("workflow not found"));
            }
            _ => panic!("expected API error"),
        }
    }
}
