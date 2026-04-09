use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct DebugArgs {
    #[command(subcommand)]
    pub action: DebugAction,
}

#[derive(Subcommand, Debug)]
pub enum DebugAction {
    Config,
    Lsp,
    Agent,
}

pub fn run(args: DebugArgs) {
    println!("Debug action: {:?}", args.action);
}
