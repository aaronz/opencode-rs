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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_args_login() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Login,
        };
        match args.action {
            AccountAction::Login => {}
            _ => panic!("Expected Login"),
        }
    }

    #[test]
    fn test_account_args_logout() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Logout,
        };
        match args.action {
            AccountAction::Logout => {}
            _ => panic!("Expected Logout"),
        }
    }

    #[test]
    fn test_account_args_status() {
        let args = AccountArgs {
            json: false,
            action: AccountAction::Status,
        };
        match args.action {
            AccountAction::Status => {}
            _ => panic!("Expected Status"),
        }
    }

    #[test]
    fn test_account_args_with_json() {
        let args = AccountArgs {
            json: true,
            action: AccountAction::Login,
        };
        assert!(args.json);
    }
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
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("failed to serialize JSON output")
        );
        return;
    }

    println!("Account action: {:?}", args.action);
}
