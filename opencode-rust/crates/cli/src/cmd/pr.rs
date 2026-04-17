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
}

pub fn run(args: PrArgs) {
    println!("PR action: {:?}", args.action);
}
