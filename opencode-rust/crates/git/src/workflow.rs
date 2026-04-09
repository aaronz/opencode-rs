use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GitHubAppClient {
    client: Client,
    token: String,
    api_base: String,
}

impl GitHubAppClient {
    pub fn new(token: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.to_string(),
            api_base: "https://api.github.com".to_string(),
        }
    }

    pub fn get_app(&self) -> Result<GitHubApp, GitHubError> {
        let url = format!("{}/app", self.api_base);
        self.send(self.authed(self.client.get(url)))
    }

    pub fn list_installations(&self) -> Result<Vec<Installation>, GitHubError> {
        let url = format!("{}/app/installations", self.api_base);
        #[derive(Deserialize)]
        struct InstallationsResponse {
            installations: Vec<Installation>,
        }
        let response: InstallationsResponse = self.send(self.authed(self.client.get(url)))?;
        Ok(response.installations)
    }

    pub fn get_installation(&self, installation_id: u64) -> Result<Installation, GitHubError> {
        let url = format!("{}/app/installations/{}", self.api_base, installation_id);
        self.send(self.authed(self.client.get(url)))
    }

    pub fn create_installation_token(
        &self,
        installation_id: u64,
    ) -> Result<InstallationToken, GitHubError> {
        let url = format!(
            "{}/app/installations/{}/access_tokens",
            self.api_base, installation_id
        );
        self.send(self.authed(self.client.post(url)))
    }

    pub fn get_repository(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepo, GitHubError> {
        let url = format!("{}/repos/{}/{}", self.api_base, owner, repo);
        self.send(self.authed(self.client.get(url)).bearer_auth(token))
    }

    pub fn create_or_update_file(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        path: &str,
        message: &str,
        content: &str,
        sha: Option<&str>,
    ) -> Result<FileCommit, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            self.api_base, owner, repo, path
        );
        let request = self.client.put(url).bearer_auth(token);

        let payload = serde_json::json!({
            "message": message,
            "content": base64_encode(content.as_bytes()),
        });
        let request = if let Some(sha) = sha {
            request.json(&serde_json::json!({
                "message": message,
                "content": base64_encode(content.as_bytes()),
                "sha": sha,
            }))
        } else {
            request.json(&payload)
        };

        self.send(self.authed(request))
    }

    pub fn get_file(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<GitHubFileContent, GitHubError> {
        let url = format!(
            "{}/repos/{}/{}/contents/{}",
            self.api_base, owner, repo, path
        );
        self.send(self.authed(self.client.get(url)).bearer_auth(token))
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

fn base64_encode(data: &[u8]) -> String {
    use std::fmt::Write;
    let mut encoded = String::new();
    let encoder = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
    write!(&mut encoded, "{}", encoder).unwrap();
    encoded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubApp {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub id: u64,
    pub account: Option<GitHubUser>,
    pub repository_selection: String,
    pub repositories_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationToken {
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub default_branch: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFileContent {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCommit {
    pub commit: CommitInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub owner: String,
    pub repo: String,
    pub workflow_path: String,
    pub secrets: Vec<SecretRequirement>,
}

impl WorkflowTemplate {
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            workflow_path: ".github/workflows/opencode.yml".to_string(),
            secrets: vec![
                SecretRequirement::new("OPENCODE_API_KEY", "OpenCode API key for the agent"),
                SecretRequirement::new("OPENCODE_MODEL", "Model to use (optional)"),
            ],
        }
    }

    pub fn generate_yaml(&self) -> String {
        let secrets_yaml: Vec<String> = self
            .secrets
            .iter()
            .map(|s| format!("          - name: {}\n            required: true", s.name))
            .collect();

        format!(
            r#"name: OpenCode Agent

on:
  issue_comment:
    types: [created]
  pull_request_review:
    types: [submitted]
  workflow_dispatch:

permissions:
  contents: write
  pull-requests: write
  issues: write

jobs:
  opencode:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup OpenCode
        run: |
          curl -sL https://raw.githubusercontent.com/opencode-ai/opencode/main/install.sh | sh

      - name: Run OpenCode
        env:
{}
        run: |
          opencode --github --session ${{ github.run_id }}
"#,
            secrets_yaml.join("\n")
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRequirement {
    pub name: String,
    pub description: String,
}

impl SecretRequirement {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

pub fn get_workflow_template(owner: &str, repo: &str) -> WorkflowTemplate {
    WorkflowTemplate::new(owner, repo)
}

pub fn setup_github_workflow(
    client: &GitHubAppClient,
    owner: &str,
    repo: &str,
    _branch: &str,
) -> Result<SetupResult, GitHubError> {
    let template = get_workflow_template(owner, repo);
    let workflow_yaml = template.generate_yaml();

    let existing_file = client.get_file(&template.owner, owner, repo, &template.workflow_path);
    let sha = existing_file.ok().map(|f| f.sha);

    let commit = client.create_or_update_file(
        &template.owner,
        owner,
        repo,
        &template.workflow_path,
        "Add OpenCode workflow",
        &workflow_yaml,
        sha.as_deref(),
    )?;

    Ok(SetupResult {
        workflow_path: template.workflow_path,
        secrets_required: template.secrets,
        commit_sha: commit.commit.sha,
    })
}

#[derive(Debug)]
pub struct SetupResult {
    pub workflow_path: String,
    pub secrets_required: Vec<SecretRequirement>,
    pub commit_sha: String,
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

    #[test]
    fn test_workflow_template_generation() {
        let template = WorkflowTemplate::new("myorg", "myrepo");
        let yaml = template.generate_yaml();

        assert!(yaml.contains("name: OpenCode Agent"));
        assert!(yaml.contains("OPENCODE_API_KEY"));
        assert!(yaml.contains(
            "on:
  issue_comment:"
        ));
    }

    #[test]
    fn test_secret_requirement() {
        let secret = SecretRequirement::new("TEST_SECRET", "A test secret");
        assert_eq!(secret.name, "TEST_SECRET");
        assert_eq!(secret.description, "A test secret");
    }

    #[test]
    fn test_workflow_template_secrets() {
        let template = WorkflowTemplate::new("owner", "repo");
        assert_eq!(template.secrets.len(), 2);
        assert_eq!(template.secrets[0].name, "OPENCODE_API_KEY");
        assert_eq!(template.secrets[1].name, "OPENCODE_MODEL");
    }
}
