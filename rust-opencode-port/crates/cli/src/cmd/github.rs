use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct GitHubArgs {
    #[command(subcommand)]
    pub action: GitHubAction,
}

#[derive(Subcommand, Debug)]
pub enum GitHubAction {
    Login,
    RepoList,
    IssueList { repo: String },
}

pub fn run(args: GitHubArgs) {
    println!("GitHub action: {:?}", args.action);
}
