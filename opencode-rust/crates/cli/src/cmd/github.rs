use clap::{Args, Subcommand};
use opencode_git::{setup_github_workflow, GitHubAppClient, WorkflowTemplate};

#[derive(Args, Debug)]
pub struct GitHubArgs {
    #[command(subcommand)]
    pub action: GitHubAction,
}

#[derive(Subcommand, Debug)]
pub enum GitHubAction {
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

pub fn run(args: GitHubArgs) {
    match args.action {
        GitHubAction::Login => {
            println!("GitHub login - TODO: Implement OAuth flow");
        }
        GitHubAction::RepoList => {
            println!("GitHub repo list - TODO: List repositories");
        }
        GitHubAction::IssueList { repo } => {
            println!("GitHub issues for {} - TODO", repo);
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
