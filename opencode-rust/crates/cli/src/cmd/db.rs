use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct DbArgs {
    #[command(subcommand)]
    pub action: DbAction,
}

#[derive(Subcommand, Debug)]
pub enum DbAction {
    Init,
    Migrate,
    Backup,
}

pub fn run(args: DbArgs) {
    println!("DB action: {:?}", args.action);
}
