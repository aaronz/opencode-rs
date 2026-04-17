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
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("List"));
    }

    #[test]
    fn test_agent_action_run() {
        let action = AgentAction::Run {
            agent: "expert".to_string(),
            prompt: "Fix the bug".to_string(),
        };
        match &action {
            AgentAction::Run { agent, prompt } => {
                assert_eq!(agent, "expert");
                assert_eq!(prompt, "Fix the bug");
            }
        }
    }

    #[test]
    fn test_agent_args_with_list() {
        let args = AgentArgs {
            action: Some(AgentAction::List),
        };
        match &args.action {
            Some(AgentAction::List) => {}
            _ => panic!("Expected List variant"),
        }
    }

    #[test]
    fn test_agent_args_with_run() {
        let args = AgentArgs {
            action: Some(AgentAction::Run {
                agent: "review".to_string(),
                prompt: "Review code".to_string(),
            }),
        };
        match &args.action {
            Some(AgentAction::Run { agent, prompt }) => {
                assert_eq!(agent, "review");
                assert_eq!(prompt, "Review code");
            }
            _ => panic!("Expected Run variant"),
        }
    }
}

pub fn run(args: AgentArgs) {
    println!("Agent action: {:?}", args.action);
}
