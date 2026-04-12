//! GitLab CI Integration (Experimental)
//!
//! ⚠️ **Experimental Feature**: GitLab Duo support is experimental and subject to change.
//!
//! This module provides integration with GitLab CI/CD for the OpenCode agent.
//! GitLab Duo is an experimental feature whose availability depends on:
//! - GitLab product tier (Premium/Ultimate required for Duo features)
//! - Deployment setup and configuration
//! - upstream integration path adopted for the Rust port
//!
//! ## Usage Warning
//!
//! Users are advised that:
//! - API surface may change in future releases
//! - Not all GitLab Duo features may be available
//! - Feature availability is environment-dependent
//!
//! ## Example
//!
//! ```ignore
//! use opencode_git::{GitLabClient, setup_gitlab_ci};
//!
//! let client = GitLabClient::new(token, "https://gitlab.com/api/v4");
//! let result = setup_gitlab_ci(&client, "group/project", "main", false);
//! ```

use crate::gitlab::GitLabClient;
use crate::{GitLabError, SecretRequirement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabCiTemplate {
    pub project_id: String,
    pub branch: String,
    pub ci_file_path: String,
    pub component_path: Option<String>,
    pub secrets: Vec<SecretRequirement>,
}

impl GitLabCiTemplate {
    pub fn new(project_id: &str, branch: &str) -> Self {
        Self {
            project_id: project_id.to_string(),
            branch: branch.to_string(),
            ci_file_path: ".gitlab-ci.yml".to_string(),
            component_path: None,
            secrets: vec![
                SecretRequirement::new("OPENCODE_API_KEY", "OpenCode API key for the agent"),
                SecretRequirement::new("OPENCODE_MODEL", "Model to use (optional)"),
            ],
        }
    }

    pub fn with_component_path(mut self, path: &str) -> Self {
        self.component_path = Some(path.to_string());
        self
    }

    pub fn generate_yaml(&self) -> String {
        if let Some(component_path) = &self.component_path {
            self.generate_component_yaml(component_path)
        } else {
            self.generate_standalone_yaml()
        }
    }

    fn generate_standalone_yaml(&self) -> String {
        let variables_yaml: Vec<String> = self
            .secrets
            .iter()
            .map(|s| format!("  {}: ${}", s.name, s.name))
            .collect();

        format!(
            r#"workflow:
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
    - if: '$CI_PIPELINE_SOURCE == "push" && $CI_COMMIT_BRANCH == "{}"'
    - if: '$CI_PIPELINE_SOURCE == "web"'

stages:
  - opencode

opencode_agent:
  stage: opencode
  image: alpine:latest
  variables:
{}
  before_script:
    - apk add --no-cache curl bash git
    - curl -sL https://raw.githubusercontent.com/opencode-ai/opencode/main/install.sh | sh
  script:
    - opencode --gitlab --session $CI_PIPELINE_ID
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
    - if: '$CI_PIPELINE_SOURCE == "push" && $CI_COMMIT_BRANCH == "{}"'
    - if: '$CI_PIPELINE_SOURCE == "web"'
  tags:
    - docker
"#,
            self.branch,
            variables_yaml.join("\n"),
            self.branch
        )
    }

    fn generate_component_yaml(&self, _component_path: &str) -> String {
        let variables_yaml: Vec<String> = self
            .secrets
            .iter()
            .map(|s| format!("      {}: ${{{}}}", s.name, s.name))
            .collect();

        format!(
            r#"spec:
  inputs:
    opencode_api_key:
      description: "OpenCode API key"
      required: true
      default: ""
    opencode_model:
      description: "Model to use"
      required: false
      default: "claude-3-5-sonnet"
---
workflow:
  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
    - if: '$CI_PIPELINE_SOURCE == "push" && $CI_COMMIT_BRANCH == "{}"'
    - if: '$CI_PIPELINE_SOURCE == "web"'

stages:
  - opencode

opencode_agent:
  stage: opencode
  image: alpine:latest
  variables:
{}
  before_script:
    - apk add --no-cache curl bash git
    - curl -sL https://raw.githubusercontent.com/opencode-ai/opencode/main/install.sh | sh
  script:
    - opencode --gitlab --session $CI_PIPELINE_ID
  tags:
    - docker
"#,
            self.branch,
            variables_yaml.join("\n")
        )
    }
}

pub fn get_gitlab_ci_template(project_id: &str, branch: &str) -> GitLabCiTemplate {
    GitLabCiTemplate::new(project_id, branch)
}

pub fn setup_gitlab_ci(
    client: &GitLabClient,
    project_id: &str,
    branch: &str,
    use_component: bool,
) -> Result<GitLabCiSetupResult, GitLabError> {
    let template = if use_component {
        GitLabCiTemplate::new(project_id, branch).with_component_path("templates/opencode.yml")
    } else {
        GitLabCiTemplate::new(project_id, branch)
    };

    let ci_yaml = template.generate_yaml();
    let file_path = template.ci_file_path.clone();

    let existing_file = client.get_file(project_id, &file_path, branch);
    let sha = existing_file.ok().map(|f| f.sha);

    let commit_message = if use_component {
        "Add OpenCode GitLab CI component"
    } else {
        "Add OpenCode GitLab CI configuration"
    };

    let commit = if let Some(sha) = sha {
        client.update_file(
            project_id,
            &file_path,
            commit_message,
            &ci_yaml,
            branch,
            &sha,
        )
    } else {
        client.create_file(project_id, &file_path, commit_message, &ci_yaml, branch)
    };

    match commit {
        Ok(commit) => Ok(GitLabCiSetupResult {
            ci_file_path: file_path,
            secrets_required: template.secrets,
            commit_sha: commit.commit_sha,
            use_component,
        }),
        Err(e) => Err(e),
    }
}

#[derive(Debug)]
pub struct GitLabCiSetupResult {
    pub ci_file_path: String,
    pub secrets_required: Vec<SecretRequirement>,
    pub commit_sha: String,
    pub use_component: bool,
}

#[derive(Debug, Clone)]
pub struct GitLabCiTrigger {
    client: GitLabClient,
    project_id: String,
}

impl GitLabCiTrigger {
    pub fn new(client: GitLabClient, project_id: &str) -> Self {
        Self {
            client,
            project_id: project_id.to_string(),
        }
    }

    pub fn trigger_pipeline(
        &self,
        branch: &str,
    ) -> Result<GitLabPipelineTriggerResult, GitLabError> {
        let pipeline = self.client.create_pipeline(&self.project_id, branch)?;
        Ok(GitLabPipelineTriggerResult {
            pipeline_id: pipeline.id,
            pipeline_url: pipeline.web_url,
            branch: branch.to_string(),
            status: pipeline.status,
        })
    }

    pub fn get_pipeline_status(
        &self,
        pipeline_id: u64,
    ) -> Result<GitLabPipelineStatus, GitLabError> {
        let pipeline = self.client.get_pipeline(&self.project_id, pipeline_id)?;
        let jobs = self
            .client
            .get_pipeline_jobs(&self.project_id, pipeline_id)?;

        let job_results: Vec<GitLabJobResult> = jobs
            .into_iter()
            .map(|job| GitLabJobResult {
                name: job.name,
                stage: job.stage,
                status: job.status,
                duration: job.duration,
                web_url: job.web_url,
            })
            .collect();

        Ok(GitLabPipelineStatus {
            pipeline_id: pipeline.id,
            status: pipeline.status,
            web_url: pipeline.web_url,
            jobs: job_results,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabPipelineTriggerResult {
    pub pipeline_id: u64,
    pub pipeline_url: String,
    pub branch: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabPipelineStatus {
    pub pipeline_id: u64,
    pub status: String,
    pub web_url: String,
    pub jobs: Vec<GitLabJobResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabJobResult {
    pub name: String,
    pub stage: String,
    pub status: String,
    pub duration: Option<f64>,
    pub web_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_ci_template_generation() {
        let template = GitLabCiTemplate::new("group/my-project", "main");
        let yaml = template.generate_yaml();

        assert!(yaml.contains("stages:"));
        assert!(yaml.contains("opencode_agent:"));
        assert!(yaml.contains("OPENCODE_API_KEY"));
        assert!(yaml.contains("$CI_PIPELINE_ID"));
    }

    #[test]
    fn test_gitlab_ci_template_with_component() {
        let template = GitLabCiTemplate::new("group/my-project", "main")
            .with_component_path("templates/opencode.yml");
        let yaml = template.generate_yaml();

        assert!(yaml.contains("spec:"));
        assert!(yaml.contains("inputs:"));
        assert!(yaml.contains("opencode_api_key:"));
    }

    #[test]
    fn test_gitlab_ci_template_secrets() {
        let template = GitLabCiTemplate::new("owner", "repo");
        assert_eq!(template.secrets.len(), 2);
        assert_eq!(template.secrets[0].name, "OPENCODE_API_KEY");
        assert_eq!(template.secrets[1].name, "OPENCODE_MODEL");
    }

    #[test]
    fn test_gitlab_ci_template_standalone_yaml_structure() {
        let template = GitLabCiTemplate::new("mygroup/myproject", "develop");
        let yaml = template.generate_yaml();

        assert!(yaml.contains("stages:"));
        assert!(yaml.contains("  - opencode"));
        assert!(yaml.contains("workflow:"));
        assert!(yaml.contains("rules:"));
        assert!(yaml.contains("merge_request_event"));
        assert!(yaml.contains("web"));
    }

    #[test]
    fn test_gitlab_pipeline_trigger_result_serialization() {
        let result = GitLabPipelineTriggerResult {
            pipeline_id: 12345,
            pipeline_url: "https://gitlab.com/group/project/-/pipelines/12345".to_string(),
            branch: "main".to_string(),
            status: "pending".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"pipeline_id\":12345"));
        assert!(json.contains("\"branch\":\"main\""));
        assert!(json.contains("\"status\":\"pending\""));
    }

    #[test]
    fn test_gitlab_pipeline_status_serialization() {
        let status = GitLabPipelineStatus {
            pipeline_id: 12345,
            status: "running".to_string(),
            web_url: "https://gitlab.com/group/project/-/pipelines/12345".to_string(),
            jobs: vec![GitLabJobResult {
                name: "opencode_agent".to_string(),
                stage: "opencode".to_string(),
                status: "running".to_string(),
                duration: Some(120.5),
                web_url: "https://gitlab.com/group/project/-/jobs/1".to_string(),
            }],
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"pipeline_id\":12345"));
        assert!(json.contains("\"status\":\"running\""));
        assert!(json.contains("\"jobs\""));
        assert!(json.contains("\"name\":\"opencode_agent\""));
        assert!(json.contains("\"duration\":120.5"));
    }

    #[test]
    fn test_gitlab_job_result_serialization() {
        let job = GitLabJobResult {
            name: "test_job".to_string(),
            stage: "test".to_string(),
            status: "success".to_string(),
            duration: Some(45.0),
            web_url: "https://gitlab.com/job/1".to_string(),
        };

        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("\"name\":\"test_job\""));
        assert!(json.contains("\"stage\":\"test\""));
        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"duration\":45"));
    }

    #[test]
    fn test_gitlab_ci_trigger_creation() {
        let client = GitLabClient::new("test-token", "https://gitlab.com/api/v4");
        let _trigger = GitLabCiTrigger::new(client, "group/project");
    }
}

#[cfg(test)]
mod gitlab_integration_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn spawn_gitlab_mock_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            tx.send(()).unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 8192];
            let n = stream.read(&mut buffer).unwrap();

            let request_str = String::from_utf8_lossy(&buffer[..n]).to_string();
            let request_line = request_str.lines().next().unwrap_or("");
            let parts: Vec<&str> = request_line.split_whitespace().collect();

            let (status, response_body) = if parts.len() >= 2 {
                let method = parts[0];
                let path = parts[1];

                if path.contains("/projects/") && path.contains("/pipeline") && method == "POST" {
                    (
                        200,
                        serde_json::json!({
                            "id": 12345,
                            "status": "pending",
                            "ref": "main",
                            "sha": "abc123",
                            "web_url": "http://localhost/pipelines/12345",
                            "created_at": "2026-04-11T10:00:00Z",
                            "updated_at": "2026-04-11T10:00:00Z"
                        })
                        .to_string(),
                    )
                } else if path.contains("/projects/")
                    && path.contains("/pipelines/")
                    && path.contains("/jobs")
                    && method == "GET"
                {
                    (
                        200,
                        serde_json::json!([{
                            "id": 1,
                            "name": "opencode_agent",
                            "stage": "opencode",
                            "status": "running",
                            "started_at": "2026-04-11T10:00:00Z",
                            "finished_at": null,
                            "duration": null,
                            "web_url": "http://localhost/jobs/1"
                        }])
                        .to_string(),
                    )
                } else if path.contains("/projects/")
                    && path.contains("/pipelines/")
                    && method == "GET"
                {
                    let pipeline_id = path
                        .split("/pipelines/")
                        .nth(1)
                        .map(|s| s.split('/').next().unwrap_or("12345"))
                        .unwrap_or("12345");
                    let status = if pipeline_id == "99999" {
                        "failed"
                    } else {
                        "pending"
                    };
                    (
                        200,
                        serde_json::json!({
                            "id": pipeline_id.parse::<u64>().unwrap_or(12345),
                            "status": status,
                            "ref": "main",
                            "sha": "abc123",
                            "web_url": format!("http://localhost/pipelines/{}", pipeline_id),
                            "created_at": "2026-04-11T10:00:00Z",
                            "updated_at": "2026-04-11T10:00:05Z"
                        })
                        .to_string(),
                    )
                } else if path.contains("/projects/")
                    && path.contains("/repository/files/")
                    && method == "GET"
                {
                    (404, r#"{"message":"file not found"}"#.to_string())
                } else if path.contains("/projects/")
                    && path.contains("/repository/files/")
                    && method == "POST"
                {
                    (
                        201,
                        serde_json::json!({
                            "file_path": ".gitlab-ci.yml",
                            "sha": "newfile123",
                            "blob_sha": "blob456",
                            "content_sha256": "content789",
                            "commit_sha": "commit789",
                            "branch": "main"
                        })
                        .to_string(),
                    )
                } else {
                    (404, r#"{"message":"not found"}"#.to_string())
                }
            } else {
                (400, r#"{"message":"bad request"}"#.to_string())
            };

            let response = format!(
                "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status,
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        rx.recv().unwrap();
        addr
    }

    #[test]
    fn test_gitlab_pipeline_trigger() {
        let api_base = spawn_gitlab_mock_server();
        let client = GitLabClient::new("test-token", &api_base);
        let trigger = GitLabCiTrigger::new(client, "group/project");

        let result = trigger.trigger_pipeline("main");

        assert!(
            result.is_ok(),
            "Pipeline trigger should succeed: {:?}",
            result.err()
        );
        let trigger_result = result.unwrap();
        assert_eq!(trigger_result.pipeline_id, 12345);
        assert_eq!(trigger_result.branch, "main");
        assert_eq!(trigger_result.status, "pending");
        assert!(trigger_result.pipeline_url.contains("/pipelines/12345"));
    }

    #[test]
    fn test_gitlab_pipeline_status_monitoring() {
        let api_base = spawn_gitlab_mock_server();
        let client = GitLabClient::new("test-token", &api_base);
        let trigger = GitLabCiTrigger::new(client, "group/project");

        let status = trigger.get_pipeline_status(12345);

        assert!(
            status.is_ok(),
            "Pipeline status should be retrieved: {:?}",
            status.err()
        );
        let pipeline_status = status.unwrap();
        assert_eq!(pipeline_status.pipeline_id, 12345);
        assert_eq!(pipeline_status.status, "pending");
        assert!(!pipeline_status.web_url.is_empty());
        assert_eq!(pipeline_status.jobs.len(), 1);
        assert_eq!(pipeline_status.jobs[0].name, "opencode_agent");
        assert_eq!(pipeline_status.jobs[0].stage, "opencode");
        assert_eq!(pipeline_status.jobs[0].status, "running");
    }

    #[test]
    fn test_gitlab_pipeline_trigger_and_monitor_end_to_end() {
        let api_base = spawn_gitlab_mock_server();
        let client = GitLabClient::new("test-token", &api_base);
        let trigger = GitLabCiTrigger::new(client, "group/project");

        let trigger_result = trigger.trigger_pipeline("feature-branch");
        assert!(trigger_result.is_ok(), "Pipeline trigger should succeed");
        let pipeline_id = trigger_result.unwrap().pipeline_id;

        let status = trigger.get_pipeline_status(pipeline_id);
        assert!(status.is_ok(), "Pipeline status should be retrieved");
        let pipeline_status = status.unwrap();
        assert_eq!(pipeline_status.pipeline_id, pipeline_id);
        assert_eq!(pipeline_status.status, "pending");
        assert!(!pipeline_status.jobs.is_empty());
    }

    #[test]
    fn test_gitlab_ci_setup_and_trigger() {
        let api_base = spawn_gitlab_mock_server();
        let client = GitLabClient::new("test-token", &api_base);

        let setup_result = setup_gitlab_ci(&client, "group/project", "main", false);
        assert!(
            setup_result.is_ok(),
            "CI setup should succeed: {:?}",
            setup_result.err()
        );
        let setup = setup_result.unwrap();

        assert_eq!(setup.ci_file_path, ".gitlab-ci.yml");
        assert_eq!(setup.commit_sha, "commit789");
        assert!(!setup.use_component);
        assert_eq!(setup.secrets_required.len(), 2);

        let trigger = GitLabCiTrigger::new(client, "group/project");
        let pipeline_result = trigger.trigger_pipeline("main");
        assert!(pipeline_result.is_ok(), "Pipeline should be triggered");
    }

    #[test]
    fn test_gitlab_pipeline_status_with_failed_pipeline() {
        let api_base = spawn_gitlab_mock_server();
        let client = GitLabClient::new("test-token", &api_base);
        let trigger = GitLabCiTrigger::new(client, "group/project");

        let status = trigger.get_pipeline_status(99999);

        assert!(
            status.is_ok(),
            "Pipeline status should be retrieved even for failed pipeline"
        );
        let pipeline_status = status.unwrap();
        assert_eq!(pipeline_status.pipeline_id, 99999);
        assert_eq!(pipeline_status.status, "failed");
    }

    #[test]
    fn test_gitlab_ci_template_end_to_end_with_component() {
        let api_base = spawn_gitlab_mock_server();
        let client = GitLabClient::new("test-token", &api_base);

        let setup_result = setup_gitlab_ci(&client, "group/project", "develop", true);
        assert!(
            setup_result.is_ok(),
            "CI setup with component should succeed"
        );
        let setup = setup_result.unwrap();

        assert!(setup.use_component);
        assert_eq!(setup.ci_file_path, ".gitlab-ci.yml");

        let trigger = GitLabCiTrigger::new(client, "group/project");
        let pipeline_result = trigger.trigger_pipeline("develop");
        assert!(
            pipeline_result.is_ok(),
            "Pipeline should be triggered on develop branch"
        );
        assert_eq!(pipeline_result.unwrap().branch, "develop");
    }

    #[test]
    fn test_gitlab_pipeline_trigger_multiple_branches() {
        let api_base = spawn_gitlab_mock_server();
        let client = GitLabClient::new("test-token", &api_base);
        let trigger = GitLabCiTrigger::new(client, "group/project");

        let branches = vec!["main", "develop", "feature/test", "release/v1.0"];

        for branch in branches {
            let result = trigger.trigger_pipeline(branch);
            assert!(
                result.is_ok(),
                "Pipeline trigger should succeed for branch {}",
                branch
            );
            let trigger_result = result.unwrap();
            assert_eq!(trigger_result.branch, branch);
            assert_eq!(trigger_result.pipeline_id, 12345);
        }
    }

    #[test]
    fn test_gitlab_ci_get_template() {
        let template = get_gitlab_ci_template("group/project", "main");
        let yaml = template.generate_yaml();

        assert!(yaml.contains("stages:"));
        assert!(yaml.contains("opencode_agent:"));
        assert!(yaml.contains("workflow:"));
        assert!(yaml.contains("merge_request_event"));

        let setup_result = setup_gitlab_ci(
            &GitLabClient::new("test-token", "http://localhost:99999"),
            "group/project",
            "main",
            false,
        );
        assert!(setup_result.is_err());
    }
}
