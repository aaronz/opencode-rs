use clap::{Args, Subcommand};
use opencode_auth::oauth::OAuthFlow;
use opencode_git::{setup_github_workflow, GitHubAppClient, GitHubClient, WorkflowTemplate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

const GITHUB_API_BASE: &str = "https://api.github.com";
const GITHUB_CLIENT_ID: &str = "Iv1.8a1f8c05dfd1c06e";
const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

fn get_api_base() -> String {
    std::env::var("OPENCODE_GITHUB_API_BASE").unwrap_or_else(|_| GITHUB_API_BASE.to_string())
}

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

#[allow(clippy::items_after_test_module)]
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

    #[test]
    fn test_github_action_repo_list() {
        let action = GitHubAction::RepoList;
        assert!(matches!(action, GitHubAction::RepoList));
    }

    #[test]
    fn test_github_action_workflow() {
        let action = GitHubAction::Workflow {
            token: Some("test-token".to_string()),
            owner: "testowner".to_string(),
            repo: "testrepo".to_string(),
            branch: "develop".to_string(),
        };
        assert!(matches!(action, GitHubAction::Workflow { .. }));
        if let GitHubAction::Workflow {
            token,
            owner,
            repo,
            branch,
        } = action
        {
            assert_eq!(token, Some("test-token".to_string()));
            assert_eq!(owner, "testowner");
            assert_eq!(repo, "testrepo");
            assert_eq!(branch, "develop");
        }
    }

    #[test]
    fn test_parse_repo_owner_format() {
        let repo_str = "owner/repo";
        let parts: Vec<&str> = repo_str.split('/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "owner");
        assert_eq!(parts[1], "repo");
    }

    #[test]
    fn test_parse_repo_with_org_format() {
        let repo_str = "my-org/my-repo-name";
        let parts: Vec<&str> = repo_str.split('/').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "my-org");
        assert_eq!(parts[1], "my-repo-name");
    }

    #[test]
    fn test_invalid_repo_format_no_slash() {
        let repo_str = "invalid-repo";
        let parts: Vec<&str> = repo_str.split('/').collect();
        assert_ne!(parts.len(), 2);
    }

    #[test]
    fn test_invalid_repo_format_too_many_slashes() {
        let repo_str = "owner/repo/path";
        let parts: Vec<&str> = repo_str.split('/').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_github_workflow_record_serialization() {
        let record = GithubWorkflowRecord {
            owner: "testowner".to_string(),
            repo: "testrepo".to_string(),
            branch: "main".to_string(),
            workflow_path: ".github/workflows/opencode.yml".to_string(),
            commit_sha: "abc123".to_string(),
            workflow_yaml: "name: Test\non: push".to_string(),
            installed_at: "1234567890".to_string(),
        };
        let json = serde_json::to_string_pretty(&record).unwrap();
        assert!(json.contains("testowner"));
        assert!(json.contains("testrepo"));
        assert!(json.contains("abc123"));
        assert!(json.contains("1234567890"));
    }

    #[test]
    fn test_github_workflow_record_deserialization() {
        let json = r#"{
            "owner": "myorg",
            "repo": "myrepo",
            "branch": "develop",
            "workflow_path": ".github/workflows/opencode.yml",
            "commit_sha": "def456",
            "workflow_yaml": "name: OpenCode\non: pull_request",
            "installed_at": "9876543210"
        }"#;
        let record: GithubWorkflowRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.owner, "myorg");
        assert_eq!(record.repo, "myrepo");
        assert_eq!(record.branch, "develop");
        assert_eq!(record.commit_sha, "def456");
        assert_eq!(record.workflow_yaml, "name: OpenCode\non: pull_request");
    }

    #[test]
    fn test_get_workspace_opencode_dir_creates_dir_if_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let opencode_dir = get_workspace_opencode_dir();
        assert!(opencode_dir.is_some());

        let dir = opencode_dir.unwrap();
        assert!(dir.exists());
        assert!(dir.is_dir());

        std::env::set_current_dir("/").ok();
    }

    #[test]
    fn test_get_workspace_opencode_dir_finds_existing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let opencode_dir = temp_dir.path().join(".opencode");
        std::fs::create_dir_all(&opencode_dir).unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = get_workspace_opencode_dir();
        assert!(result.is_some());
        let result_path = result.unwrap();
        assert!(result_path.exists());
        assert!(result_path.file_name().unwrap() == ".opencode");

        std::env::set_current_dir("/").ok();
    }

    #[test]
    fn test_save_workflow_to_local_creates_workflows_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let opencode_dir = temp_dir.path().join(".opencode");
        std::fs::create_dir_all(&opencode_dir).unwrap();

        let workflows_dir = opencode_dir.join("workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = opencode_git::SetupResult {
            workflow_path: ".github/workflows/opencode.yml".to_string(),
            secrets_required: vec![],
            commit_sha: "abc123".to_string(),
            workflow_yaml: "name: Test".to_string(),
        };

        let save_result = save_workflow_to_local(result, "myowner", "myrepo", "main");
        assert!(save_result.is_ok(), "save_workflow_to_local failed: {:?}", save_result.err());

        let record_path = workflows_dir.join("myowner-myrepo.json");
        assert!(record_path.exists(), "record file should exist");

        let content = std::fs::read_to_string(&record_path).unwrap();
        assert!(content.contains("myowner"));
        assert!(content.contains("myrepo"));

        std::env::set_current_dir("/").ok();
    }

    #[test]
    fn test_check_existing_workflow_returns_none_for_new_install() {
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = check_existing_workflow("nonexistent", "repo");
        assert!(result.is_none());

        std::env::set_current_dir("/").ok();
    }

    #[test]
    fn test_check_existing_workflow_returns_record() {
        let temp_dir = tempfile::tempdir().unwrap();
        let opencode_dir = temp_dir.path().join(".opencode");
        let workflows_dir = opencode_dir.join("workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        let record = GithubWorkflowRecord {
            owner: "testowner".to_string(),
            repo: "testrepo".to_string(),
            branch: "main".to_string(),
            workflow_path: ".github/workflows/opencode.yml".to_string(),
            commit_sha: "xyz789".to_string(),
            workflow_yaml: "name: Test".to_string(),
            installed_at: "12345".to_string(),
        };
        let json = serde_json::to_string_pretty(&record).unwrap();
        std::fs::write(workflows_dir.join("testowner-testrepo.json"), json).unwrap();

        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = check_existing_workflow("testowner", "testrepo");
        assert!(result.is_some());
        assert_eq!(result.unwrap().commit_sha, "xyz789");

        let _ = std::env::set_current_dir(&orig_cwd);
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

fn check_existing_workflow(owner: &str, repo: &str) -> Option<GithubWorkflowRecord> {
    let opencode_dir = get_workspace_opencode_dir()?;
    let record_path = opencode_dir
        .join("workflows")
        .join(format!("{}-{}.json", owner, repo));
    let content = std::fs::read_to_string(&record_path).ok()?;
    serde_json::from_str(&content).ok()
}

fn run_install(token: Option<String>, owner: &str, repo: &str, branch: &str) {
    let token = get_token(token);
    let client = GitHubAppClient::new(&token);

    if let Some(existing) = check_existing_workflow(owner, repo) {
        println!("Found existing OpenCode workflow installation for {}/{}", owner, repo);
        println!("  Workflow path: {}", existing.workflow_path);
        println!("  Installed at: {}", existing.installed_at);
        println!("  Commit SHA: {}", existing.commit_sha);
        println!("\nTo reinstall, remove the existing record from .opencode/workflows/");
        return;
    }

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

            if let Err(e) = save_workflow_to_local(result, owner, repo, branch) {
                eprintln!("  Warning: Failed to save local workflow record: {}", e);
            } else {
                println!("  Local workflow record saved to .opencode/");
            }
        }
        Err(e) => {
            eprintln!("Error setting up workflow: {}", e);
            std::process::exit(1);
        }
    }
}

fn get_workspace_opencode_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        let opencode_dir = ancestor.join(".opencode");
        if opencode_dir.exists() && opencode_dir.is_dir() {
            return Some(opencode_dir);
        }
    }
    let new_opencode = cwd.join(".opencode");
    if std::fs::create_dir_all(&new_opencode).is_ok() {
        Some(new_opencode)
    } else {
        None
    }
}

fn save_workflow_to_local(
    result: opencode_git::SetupResult,
    owner: &str,
    repo: &str,
    branch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let opencode_dir = get_workspace_opencode_dir().ok_or("Could not find or create .opencode directory")?;

    let workflows_dir = opencode_dir.join("workflows");
    std::fs::create_dir_all(&workflows_dir)?;

    let record = GithubWorkflowRecord {
        owner: owner.to_string(),
        repo: repo.to_string(),
        branch: branch.to_string(),
        workflow_path: result.workflow_path.clone(),
        commit_sha: result.commit_sha,
        workflow_yaml: result.workflow_yaml,
        installed_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    };

    let record_path = workflows_dir.join(format!("{}-{}.json", owner, repo));
    let json = serde_json::to_string_pretty(&record)?;
    std::fs::write(&record_path, json)?;

    println!("  Workflow record saved to: {}", record_path.display());
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubWorkflowRecord {
    owner: String,
    repo: String,
    branch: String,
    workflow_path: String,
    commit_sha: String,
    workflow_yaml: String,
    installed_at: String,
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

    let client = GitHubClient::new(&token, &get_api_base());

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

    let client = GitHubClient::new(&token, &get_api_base());

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
