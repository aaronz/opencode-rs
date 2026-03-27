use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long)]
    pub id: Option<String>,

    #[command(subcommand)]
    pub action: Option<SessionAction>,
}

#[derive(Subcommand, Debug)]
pub enum SessionAction {
    Delete,
    Show {
        #[arg(short, long)]
        json: bool,
    },
    Export,
}

pub fn run(args: SessionArgs) {
    println!("Session action: {:?}", args.action);
}
