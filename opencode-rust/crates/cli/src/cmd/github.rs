use clap::{Args, Subcommand};
use opencode_auth::oauth::OAuthFlow;
use opencode_git::{setup_github_workflow, GitHubAppClient, GitHubClient, WorkflowTemplate};

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_CLIENT_ID: &str = "Iv1.8a1f8c05dfd1c06e";
const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

#[derive(Args, Debug)]
pub(crate) struct GitHubArgs {
    #[command(subcommand)]
    pub action: GitHubAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum GitHubAction {
    Login,
    RepoList,
    IssueList {
        repo: String,
    },
    Install {
        #[arg(long)]
        token: Option<String>,

        #[arg(long)]
        owner: String,

        #[arg(long)]
        repo: String,

        #[arg(long, default_value = "main")]
        branch: String,
    },
    Workflow {
        #[arg(long)]
        token: Option<String>,

        #[arg(long)]
        owner: String,

        #[arg(long)]
        repo: String,

        #[arg(long, default_value = "main")]
        branch: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_args_default() {
        let args = GitHubArgs {
            action: GitHubAction::Login,
        };
        assert!(matches!(args.action, GitHubAction::Login));
    }

    #[test]
    fn test_github_action_issue_list() {
        let action = GitHubAction::IssueList {
            repo: "owner/repo".to_string(),
        };
        assert!(matches!(action, GitHubAction::IssueList { .. }));
    }

    #[test]
    fn test_github_action_install_fields() {
        let action = GitHubAction::Install {
            token: None,
            owner: "myowner".to_string(),
            repo: "myrepo".to_string(),
            branch: "main".to_string(),
        };
        assert!(matches!(action, GitHubAction::Install { .. }));
    }
}

pub(crate) fn run(args: GitHubArgs) {
    match args.action {
        GitHubAction::Login => {
            run_login();
        }
        GitHubAction::RepoList => {
            run_repo_list();
        }
        GitHubAction::IssueList { repo } => {
            run_issue_list(&repo);
        }
        GitHubAction::Install {
            token,
            owner,
            repo,
            branch,
        } => {
            run_install(token, &owner, &repo, &branch);
        }
        GitHubAction::Workflow {
            token,
            owner,
            repo,
            branch,
        } => {
            run_workflow(token, &owner, &repo, &branch);
        }
    }
}

fn get_token(token: Option<String>) -> String {
    if let Some(t) = token {
        return t;
    }
    std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| {
        eprintln!("Error: GitHub token required. Provide --token or set GITHUB_TOKEN env var");
        std::process::exit(1);
    })
}

fn run_install(token: Option<String>, owner: &str, repo: &str, branch: &str) {
    let token = get_token(token);
    let client = GitHubAppClient::new(&token);

    println!("Setting up OpenCode workflow for {}/{}...", owner, repo);

    match setup_github_workflow(&client, owner, repo, branch) {
        Ok(result) => {
            println!("\n✓ Workflow file created at: {}", result.workflow_path);
            println!("  Commit SHA: {}", result.commit_sha);
            println!("\nRequired secrets:");
            for secret in &result.secrets_required {
                println!("  - {}: {}", secret.name, secret.description);
            }
            println!("\nTo add secrets, go to:");
            println!("  https://github.com/{}/settings/secrets", owner);
        }
        Err(e) => {
            eprintln!("Error setting up workflow: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_workflow(token: Option<String>, owner: &str, repo: &str, branch: &str) {
    let _token = get_token(token);
    let template = WorkflowTemplate::new(owner, repo);
    let yaml = template.generate_yaml();

    println!(
        "OpenCode Workflow for {}/{} (branch: {}):\n",
        owner, repo, branch
    );
    println!("{}", yaml);

    println!("\nRequired secrets:");
    for secret in &template.secrets {
        println!("  - {}: {}", secret.name, secret.description);
    }
}

fn run_login() {
    println!("Starting GitHub OAuth login flow...");

    let oauth_flow = OAuthFlow::new();

    match oauth_flow.start_device_code_flow(
        "github",
        GITHUB_CLIENT_ID,
        GITHUB_DEVICE_CODE_URL,
        Some("repo read:user"),
    ) {
        Ok(session) => {
            println!("\nTo complete login:");
            println!("  1. Open: {}", session.verification_uri);
            if let Some(ref complete) = session.verification_uri_complete {
                println!("  2. Enter code: {}", session.user_code);
                println!("  Or visit: {}", complete);
            } else {
                println!("  2. Enter code: {}", session.user_code);
            }
            println!("\nWaiting for authentication...\n");

            match oauth_flow.poll_device_code_authorization(
                &session,
                GITHUB_CLIENT_ID,
                "",
                GITHUB_TOKEN_URL,
                None,
            ) {
                Ok(token) => match oauth_flow.store_token("github", &token) {
                    Ok(()) => {
                        println!("✓ GitHub authentication successful!");
                        println!("  Token stored securely.");
                    }
                    Err(e) => {
                        eprintln!("Error storing token: {}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("Error during authentication: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error starting OAuth flow: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_repo_list() {
    let token = std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| {
        eprintln!("Error: GitHub token required. Set GITHUB_TOKEN environment variable or login with 'opencode github login'");
        std::process::exit(1);
    });

    let client = GitHubClient::new(&token, GITHUB_API_BASE);

    match client.list_repos("") {
        Ok(repos) => {
            if repos.is_empty() {
                println!("No repositories found.");
                return;
            }
            println!("Your repositories:\n");
            for repo in &repos {
                let visibility = if repo.private {
                    "[private]"
                } else {
                    "[public]"
                };
                println!(
                    "  {} {} ({})",
                    visibility,
                    repo.full_name,
                    repo.default_branch.as_deref().unwrap_or("main")
                );
            }
            println!("\nTotal: {} repositories", repos.len());
        }
        Err(e) => {
            eprintln!("Error fetching repositories: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_issue_list(repo: &str) {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        eprintln!("Error: Repository must be in format 'owner/repo'");
        std::process::exit(1);
    }
    let (owner, repo_name) = (parts[0], parts[1]);

    let token = std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| {
        eprintln!("Error: GitHub token required. Set GITHUB_TOKEN environment variable or login with 'opencode github login'");
        std::process::exit(1);
    });

    let client = GitHubClient::new(&token, GITHUB_API_BASE);

    match client.list_issues(owner, repo_name, "open") {
        Ok(issues) => {
            let pure_issues: Vec<_> = issues.iter().filter(|i| i.pull_request.is_none()).collect();
            if pure_issues.is_empty() {
                println!("No open issues found for {}/{}", owner, repo_name);
                return;
            }
            println!("Open issues for {}/{}:\n", owner, repo_name);
            for issue in &pure_issues {
                println!(
                    "  #{} - {} (by {})",
                    issue.number,
                    issue.title,
                    issue
                        .user
                        .as_ref()
                        .map(|u| u.login.as_str())
                        .unwrap_or("unknown")
                );
            }
            println!("\nTotal: {} open issues", pure_issues.len());
        }
        Err(e) => {
            eprintln!("Error fetching issues: {}", e);
            std::process::exit(1);
        }
    }
}
