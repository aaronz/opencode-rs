use clap::{Args, Subcommand};
use opencode_git::{GitHubClient, GitHubPullRequest};

#[derive(Args, Debug)]
pub(crate) struct PrArgs {
    #[command(subcommand)]
    pub action: PrAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum PrAction {
    List { repo: String },
    Fetch { repo: String, number: u64 },
    Checkout { repo: String, number: u64 },
    Create { repo: String, title: String },
}

fn get_token() -> String {
    std::env::var("GITHUB_TOKEN").unwrap_or_else(|_| {
        eprintln!("Error: GitHub token required. Set GITHUB_TOKEN env var");
        std::process::exit(1);
    })
}

fn create_client() -> GitHubClient {
    GitHubClient::new(&get_token(), "https://api.github.com")
}

fn print_pr(pr: &GitHubPullRequest) {
    println!("PR #{}", pr.number);
    println!("Title: {}", pr.title);
    if let Some(body) = &pr.body {
        if !body.is_empty() {
            println!("\n{}", body);
        }
    }
    println!("\nState: {}", pr.state);
    if let Some(user) = &pr.user {
        println!("Author: {}", user.login);
    }
    if let Some(head) = &pr.head {
        println!("Head branch: {}", head.ref_name);
        println!("Head SHA: {}", head.sha);
    }
    if let Some(base) = &pr.base {
        println!("Base branch: {}", base.ref_name);
    }
    if let Some(url) = &pr.html_url {
        println!("URL: {}", url);
    }
}

fn run_list(repo: &str) {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        eprintln!("Error: repo must be in format 'owner/repo'");
        std::process::exit(1);
    }
    let (owner, repo_name) = (parts[0], parts[1]);

    let client = create_client();
    match client.list_prs(owner, repo_name, "open") {
        Ok(prs) => {
            if prs.is_empty() {
                println!("No open pull requests found.");
            } else {
                println!("Open Pull Requests in {}/{}:\n", owner, repo_name);
                for pr in &prs {
                    println!(
                        "#{} - {} ({}) [{}]",
                        pr.number,
                        pr.title,
                        pr.state,
                        pr.user
                            .as_ref()
                            .map(|u| u.login.as_str())
                            .unwrap_or("unknown")
                    );
                    if let Some(head) = &pr.head {
                        println!(
                            "   Branch: {} -> {}",
                            head.ref_name,
                            pr.base.as_ref().map(|b| b.ref_name.as_str()).unwrap_or("?")
                        );
                    }
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching PRs: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_fetch(repo: &str, number: u64) {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        eprintln!("Error: repo must be in format 'owner/repo'");
        std::process::exit(1);
    }
    let (owner, repo_name) = (parts[0], parts[1]);

    let client = create_client();
    match client.get_pr(owner, repo_name, number) {
        Ok(pr) => {
            print_pr(&pr);
        }
        Err(e) => {
            eprintln!("Error fetching PR #{}: {}", number, e);
            std::process::exit(1);
        }
    }
}

fn run_checkout(repo: &str, number: u64) {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        eprintln!("Error: repo must be in format 'owner/repo'");
        std::process::exit(1);
    }
    let (owner, repo_name) = (parts[0], parts[1]);

    let client = create_client();
    match client.get_pr(owner, repo_name, number) {
        Ok(pr) => {
            let branch_name = pr
                .head
                .as_ref()
                .map(|h| h.ref_name.clone())
                .unwrap_or_else(|| format!("pr-{}", number));

            println!("Fetching PR #{} from {}/{}", number, owner, repo_name);

            let fetch_url = format!("https://github.com/{}/{}.git", owner, repo_name);

            let status = std::process::Command::new("git")
                .args([
                    "fetch",
                    &fetch_url,
                    &format!("pull/{}/head:{}", number, branch_name),
                ])
                .status();

            match status {
                Ok(exit) if exit.success() => {
                    println!("Successfully fetched branch '{}'", branch_name);

                    let checkout_status = std::process::Command::new("git")
                        .args(["checkout", &branch_name])
                        .status();

                    match checkout_status {
                        Ok(exit) if exit.success() => {
                            println!("Successfully checked out branch '{}'", branch_name);
                        }
                        Ok(exit) => {
                            eprintln!("Git checkout failed with exit code: {}", exit);
                            std::process::exit(1);
                        }
                        Err(e) => {
                            eprintln!("Failed to execute git checkout: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Ok(exit) => {
                    eprintln!("Git fetch failed with exit code: {}", exit);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to execute git fetch: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching PR #{}: {}", number, e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_args_list() {
        let args = PrArgs {
            action: PrAction::List {
                repo: "test/repo".to_string(),
            },
        };
        assert!(matches!(args.action, PrAction::List { .. }));
    }

    #[test]
    fn test_pr_action_create_fields() {
        let action = PrAction::Create {
            repo: "owner/repo".to_string(),
            title: "Add new feature".to_string(),
        };
        assert!(matches!(action, PrAction::Create { .. }));
    }

    #[test]
    fn test_pr_action_fetch_fields() {
        let action = PrAction::Fetch {
            repo: "owner/repo".to_string(),
            number: 42,
        };
        assert!(matches!(action, PrAction::Fetch { .. }));
        if let PrAction::Fetch { repo, number } = action {
            assert_eq!(repo, "owner/repo");
            assert_eq!(number, 42);
        }
    }

    #[test]
    fn test_pr_action_checkout_fields() {
        let action = PrAction::Checkout {
            repo: "owner/repo".to_string(),
            number: 123,
        };
        assert!(matches!(action, PrAction::Checkout { .. }));
        if let PrAction::Checkout { repo, number } = action {
            assert_eq!(repo, "owner/repo");
            assert_eq!(number, 123);
        }
    }
}

pub(crate) fn run(args: PrArgs) {
    match args.action {
        PrAction::List { repo } => {
            run_list(&repo);
        }
        PrAction::Fetch { repo, number } => {
            run_fetch(&repo, number);
        }
        PrAction::Checkout { repo, number } => {
            run_checkout(&repo, number);
        }
        PrAction::Create { repo, title } => {
            println!("PR create - repo: {}, title: {}", repo, title);
            println!(
                "Note: Create PR functionality requires additional parameters (body, head, base)"
            );
        }
    }
}
