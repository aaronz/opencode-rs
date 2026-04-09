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
}
