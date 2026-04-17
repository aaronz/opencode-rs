use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct DebugArgs {
    #[command(subcommand)]
    pub action: DebugAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum DebugAction {
    Config,
    Lsp,
    Agent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_action_config() {
        let action = DebugAction::Config;
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Config"));
    }

    #[test]
    fn test_debug_action_lsp() {
        let action = DebugAction::Lsp;
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Lsp"));
    }

    #[test]
    fn test_debug_action_agent() {
        let action = DebugAction::Agent;
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("Agent"));
    }

    #[test]
    fn test_debug_args_with_config() {
        let args = DebugArgs {
            action: DebugAction::Config,
        };
        match args.action {
            DebugAction::Config => {}
            _ => panic!("Expected Config variant"),
        }
    }

    #[test]
    fn test_debug_args_with_lsp() {
        let args = DebugArgs {
            action: DebugAction::Lsp,
        };
        match args.action {
            DebugAction::Lsp => {}
            _ => panic!("Expected Lsp variant"),
        }
    }

    #[test]
    fn test_debug_args_with_agent() {
        let args = DebugArgs {
            action: DebugAction::Agent,
        };
        match args.action {
            DebugAction::Agent => {}
            _ => panic!("Expected Agent variant"),
        }
    }
}

pub(crate) fn run(args: DebugArgs) {
    println!("Debug action: {:?}", args.action);
}
