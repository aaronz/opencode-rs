use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct GitLabClient {
    client: Client,
    token: String,
    api_base: String,
}

impl GitLabClient {
    pub fn new(token: &str, api_base: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.to_string(),
            api_base: api_base.trim_end_matches('/').to_string(),
        }
    }

    pub fn get_project(&self, project_id: &str) -> Result<GitLabProject, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!("{}/projects/{}", self.api_base, encoded_id);
        self.send(self.authed(self.client.get(url)))
    }

    pub fn create_file(
        &self,
        project_id: &str,
        file_path: &str,
        message: &str,
        content: &str,
        branch: &str,
    ) -> Result<GitLabFileCommit, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!(
            "{}/projects/{}/repository/files/{}",
            self.api_base,
            encoded_id,
            urlencoding::encode(file_path)
        );
        let payload = serde_json::json!({
            "branch": branch,
            "content": base64_encode(content.as_bytes()),
            "commit_message": message,
        });
        let request = self.client.post(url).json(&payload);
        self.send(self.authed(request))
    }

    pub fn update_file(
        &self,
        project_id: &str,
        file_path: &str,
        message: &str,
        content: &str,
        branch: &str,
        sha: &str,
    ) -> Result<GitLabFileCommit, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!(
            "{}/projects/{}/repository/files/{}",
            self.api_base,
            encoded_id,
            urlencoding::encode(file_path)
        );
        let payload = serde_json::json!({
            "branch": branch,
            "content": base64_encode(content.as_bytes()),
            "commit_message": message,
            "sha": sha,
        });
        let request = self.client.put(url).json(&payload);
        self.send(self.authed(request))
    }

    pub fn get_file(
        &self,
        project_id: &str,
        file_path: &str,
        ref_branch: &str,
    ) -> Result<GitLabFileContent, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!(
            "{}/projects/{}/repository/files/{}",
            self.api_base,
            encoded_id,
            urlencoding::encode(file_path)
        );
        let request = self.client.get(url).query(&[("ref", ref_branch)]);
        self.send(self.authed(request))
    }

    pub fn create_pipeline(
        &self,
        project_id: &str,
        branch: &str,
    ) -> Result<GitLabPipeline, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!("{}/projects/{}/pipeline", self.api_base, encoded_id);
        let payload = serde_json::json!({
            "ref": branch,
        });
        let request = self.client.post(url).json(&payload);
        self.send(self.authed(request))
    }

    pub fn list_pipelines(
        &self,
        project_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<GitLabPipeline>, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!("{}/projects/{}/pipelines", self.api_base, encoded_id);
        let request = if let Some(status) = status {
            self.client.get(url).query(&[("status", status)])
        } else {
            self.client.get(url)
        };
        #[derive(Deserialize)]
        struct PipelinesResponse {
            pipelines: Vec<GitLabPipeline>,
        }
        let response: PipelinesResponse = self.send(self.authed(request))?;
        Ok(response.pipelines)
    }

    pub fn get_ci_variables(&self, project_id: &str) -> Result<Vec<GitLabCiVariable>, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!("{}/projects/{}/variables", self.api_base, encoded_id);
        self.send(self.authed(self.client.get(url)))
    }

    pub fn create_ci_variable(
        &self,
        project_id: &str,
        key: &str,
        value: &str,
        protected: bool,
    ) -> Result<GitLabCiVariable, GitLabError> {
        let encoded_id = urlencoding::encode(project_id);
        let url = format!("{}/projects/{}/variables", self.api_base, encoded_id);
        let payload = serde_json::json!({
            "key": key,
            "value": value,
            "protected": protected,
        });
        let request = self.client.post(url).json(&payload);
        self.send(self.authed(request))
    }

    fn authed(&self, req: RequestBuilder) -> RequestBuilder {
        req.bearer_auth(&self.token)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("User-Agent", "opencode-rs")
    }

    fn send<T: for<'de> Deserialize<'de>>(&self, req: RequestBuilder) -> Result<T, GitLabError> {
        let response = req.send()?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(GitLabError::Api { status, body });
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
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub web_url: String,
    pub default_branch: Option<String>,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabFileContent {
    pub file_name: String,
    pub file_path: String,
    pub sha: String,
    pub content: String,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabFileCommit {
    pub file_path: String,
    pub sha: String,
    pub blob_sha: String,
    pub content_sha256: String,
    pub commit_sha: String,
    pub branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabPipeline {
    pub id: u64,
    pub status: String,
    pub ref_: String,
    #[serde(rename = "sha")]
    pub commit_sha: String,
    pub web_url: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabCiVariable {
    pub id: u64,
    pub key: String,
    pub value: String,
    pub protected: bool,
    pub masked: bool,
    pub variable_type: String,
}

#[derive(Debug, Error)]
pub enum GitLabError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("gitlab api error ({status}): {body}")]
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
    fn get_project_decodes_response() {
        let body = serde_json::json!({
            "id": 123,
            "name": "my-project",
            "path_with_namespace": "group/my-project",
            "web_url": "https://gitlab.com/group/my-project",
            "default_branch": "main",
            "visibility": "private"
        })
        .to_string();

        let api_base = spawn_server(200, body);
        let client = GitLabClient::new("token", &api_base);

        let project = client.get_project("group/my-project").unwrap();
        assert_eq!(project.id, 123);
        assert_eq!(project.name, "my-project");
        assert_eq!(project.path_with_namespace, "group/my-project");
    }

    #[test]
    fn returns_api_error_on_non_success_status() {
        let api_base = spawn_server(404, "{\"message\":\"Not Found\"}".to_string());
        let client = GitLabClient::new("token", &api_base);

        let err = client.get_project("missing/project").unwrap_err();
        match err {
            GitLabError::Api { status, body } => {
                assert_eq!(status, 404);
                assert!(body.contains("Not Found"));
            }
            _ => panic!("expected api error"),
        }
    }
}
