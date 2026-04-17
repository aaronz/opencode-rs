use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct AgentArgs {
    #[command(subcommand)]
    pub action: Option<AgentAction>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum AgentAction {
    List,
    Run {
        #[arg(short, long)]
        agent: String,
        #[arg(short, long)]
        prompt: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_args_no_action() {
        let args = AgentArgs { action: None };
        assert!(args.action.is_none());
    }

    #[test]
    fn test_agent_action_list() {
        let action = AgentAction::List;
        assert!(matches!(action, AgentAction::List));
    }

    #[test]
    fn test_agent_action_run_fields() {
        let action = AgentAction::Run {
            agent: "expert".to_string(),
            prompt: "Fix the bug".to_string(),
        };
        assert!(matches!(action, AgentAction::Run { .. }));
    }
}

pub(crate) fn run(args: AgentArgs) {
    println!("Agent action: {:?}", args.action);
}
