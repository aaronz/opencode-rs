use clap::{Args, Subcommand};
use opencode_git::{get_gitlab_ci_template, setup_gitlab_ci, GitLabClient};

const GITLAB_EXPERIMENTAL_WARNING: &str = r#"
⚠️  GitLab Duo support is **experimental** and subject to change.

GitLab Duo features depend on:
- GitLab product tier (Premium/Ultimate required)
- Deployment configuration
- Environment setup

Users are advised that:
- API surface may change in future releases
- Not all GitLab Duo features may be available
- Feature availability is environment-dependent
"#;

#[derive(Args, Debug)]
pub(crate) struct GitLabArgs {
    #[command(subcommand)]
    pub action: GitLabAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum GitLabAction {
    Login,
    ProjectList,
    Install {
        #[arg(long)]
        token: Option<String>,

        #[arg(long)]
        project: String,

        #[arg(long, default_value = "main")]
        branch: String,

        #[arg(long)]
        use_component: bool,
    },
    Workflow {
        #[arg(long)]
        token: Option<String>,

        #[arg(long)]
        project: String,

        #[arg(long, default_value = "main")]
        branch: String,

        #[arg(long)]
        use_component: bool,
    },
    Variables {
        #[arg(long)]
        token: Option<String>,

        #[arg(long)]
        project: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_args_login() {
        let args = GitLabArgs {
            action: GitLabAction::Login,
        };
        assert!(matches!(args.action, GitLabAction::Login));
    }

    #[test]
    fn test_gitlab_action_install_fields() {
        let action = GitLabAction::Install {
            token: None,
            project: "group/project".to_string(),
            branch: "main".to_string(),
            use_component: false,
        };
        assert!(matches!(action, GitLabAction::Install { .. }));
    }

    #[test]
    fn test_gitlab_experimental_warning_not_empty() {
        assert!(!GITLAB_EXPERIMENTAL_WARNING.is_empty());
        assert!(GITLAB_EXPERIMENTAL_WARNING.contains("experimental"));
    }
}

pub(crate) fn run(args: GitLabArgs) {
    eprintln!("{}", GITLAB_EXPERIMENTAL_WARNING);
    match args.action {
        GitLabAction::Login => {
            println!("GitLab login - TODO: Implement OAuth flow");
        }
        GitLabAction::ProjectList => {
            println!("GitLab project list - TODO: List projects");
        }
        GitLabAction::Install {
            token,
            project,
            branch,
            use_component,
        } => {
            run_install(token, &project, &branch, use_component);
        }
        GitLabAction::Workflow {
            token,
            project,
            branch,
            use_component,
        } => {
            run_workflow(token, &project, &branch, use_component);
        }
        GitLabAction::Variables { token, project } => {
            run_variables(token, &project);
        }
    }
}

fn get_token(token: Option<String>) -> String {
    if let Some(t) = token {
        return t;
    }
    std::env::var("GITLAB_TOKEN").unwrap_or_else(|_| {
        eprintln!("Error: GitLab token required. Provide --token or set GITLAB_TOKEN env var");
        std::process::exit(1);
    })
}

fn run_install(token: Option<String>, project: &str, branch: &str, use_component: bool) {
    let token = get_token(token);
    let client = GitLabClient::new(&token, "https://gitlab.com/api/v4");

    println!(
        "Setting up OpenCode GitLab CI for {} (branch: {})...",
        project, branch
    );

    match setup_gitlab_ci(&client, project, branch, use_component) {
        Ok(result) => {
            println!("\n✓ CI file created at: {}", result.ci_file_path);
            println!("  Commit SHA: {}", result.commit_sha);
            if result.use_component {
                println!("  Using GitLab CI Component format");
            }
            println!("\nRequired CI variables:");
            for secret in &result.secrets_required {
                println!("  - {}: {}", secret.name, secret.description);
            }
            println!("\nTo add CI variables, go to:");
            println!("  https://gitlab.com/{}/-/settings/ci_cd", project);
        }
        Err(e) => {
            eprintln!("Error setting up GitLab CI: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_workflow(token: Option<String>, project: &str, branch: &str, use_component: bool) {
    let _token = get_token(token);
    let template = if use_component {
        get_gitlab_ci_template(project, branch).with_component_path("templates/opencode.yml")
    } else {
        get_gitlab_ci_template(project, branch)
    };
    let yaml = template.generate_yaml();

    println!("OpenCode GitLab CI for {} (branch: {}):\n", project, branch);
    println!("{}", yaml);

    println!("\nRequired CI variables:");
    for secret in &template.secrets {
        println!("  - {}: {}", secret.name, secret.description);
    }
}

fn run_variables(token: Option<String>, project: &str) {
    let token = get_token(token);
    let client = GitLabClient::new(&token, "https://gitlab.com/api/v4");

    println!("CI variables for {}:\n", project);

    match client.get_ci_variables(project) {
        Ok(variables) => {
            if variables.is_empty() {
                println!("No CI variables configured.");
                println!("\nTo add CI variables, go to:");
                println!("  https://gitlab.com/{}/-/settings/ci_cd", project);
            } else {
                for var in &variables {
                    let protected = if var.protected { " [protected]" } else { "" };
                    println!("  - {}{}", var.key, protected);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching CI variables: {}", e);
            std::process::exit(1);
        }
    }
}
