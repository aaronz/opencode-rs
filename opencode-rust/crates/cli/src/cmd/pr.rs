use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct PrArgs {
    #[command(subcommand)]
    pub action: PrAction,
}

#[derive(Subcommand, Debug)]
pub enum PrAction {
    List { repo: String },
    Create { repo: String, title: String },
}

pub fn run(args: PrArgs) {
    println!("PR action: {:?}", args.action);
}
