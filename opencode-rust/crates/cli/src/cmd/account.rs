use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args, Debug)]
pub struct AccountArgs {
    #[arg(long)]
    pub json: bool,

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
    if args.json {
        let action_str = match args.action {
            AccountAction::Login => "login",
            AccountAction::Logout => "logout",
            AccountAction::Status => "status",
        };
        let result = json!({
            "action": action_str,
            "logged_in": false,
            "status": "not_implemented"
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    println!("Account action: {:?}", args.action);
}
