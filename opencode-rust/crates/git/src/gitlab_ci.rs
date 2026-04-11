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
            .map(|s| format!("  {}: {}", s.name, format!("${{{}}}", s.name)))
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
