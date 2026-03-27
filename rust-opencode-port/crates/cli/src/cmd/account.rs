use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub action: AccountAction,
}

#[derive(Subcommand, Debug)]
pub enum AccountAction {
    Login,
    Logout,
    Status,
}

pub fn run(args: AccountArgs) {
    println!("Account action: {:?}", args.action);
}
