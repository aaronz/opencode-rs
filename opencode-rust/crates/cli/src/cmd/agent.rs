use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub action: Option<AgentAction>,
}

#[derive(Subcommand, Debug)]
pub enum AgentAction {
    List,
    Run {
        #[arg(short, long)]
        agent: String,
        #[arg(short, long)]
        prompt: String,
    },
}

pub fn run(args: AgentArgs) {
    println!("Agent action: {:?}", args.action);
}
