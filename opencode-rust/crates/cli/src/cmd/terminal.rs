use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct TerminalArgs {
    #[command(subcommand)]
    pub action: Option<TerminalAction>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum TerminalAction {
    Open,
    Exec {
        #[arg(last = true)]
        command: Vec<String>,
    },
    Tabs {
        #[arg(long)]
        json: bool,
    },
    Close {
        #[arg(long)]
        tab: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_args_no_action() {
        let args = TerminalArgs { action: None };
        assert!(args.action.is_none());
    }

    #[test]
    fn test_terminal_args_with_action() {
        let args = TerminalArgs {
            action: Some(TerminalAction::Open),
        };
        match args.action {
            Some(TerminalAction::Open) => {}
            _ => panic!("Expected Open"),
        }
    }

    #[test]
    fn test_terminal_action_open() {
        let action = TerminalAction::Open;
        assert!(matches!(action, TerminalAction::Open));
    }

    #[test]
    fn test_terminal_action_exec_fields() {
        let action = TerminalAction::Exec {
            command: vec!["ls".to_string(), "-la".to_string()],
        };
        assert!(matches!(action, TerminalAction::Exec { .. }));
    }

    #[test]
    fn test_terminal_action_exec_empty_command() {
        let action = TerminalAction::Exec { command: vec![] };
        match action {
            TerminalAction::Exec { command } => assert!(command.is_empty()),
            _ => panic!("Expected Exec"),
        }
    }

    #[test]
    fn test_terminal_action_exec_single_command() {
        let action = TerminalAction::Exec {
            command: vec!["pwd".to_string()],
        };
        match action {
            TerminalAction::Exec { command } => assert_eq!(command.len(), 1),
            _ => panic!("Expected Exec"),
        }
    }

    #[test]
    fn test_terminal_action_exec_multiple_args() {
        let action = TerminalAction::Exec {
            command: vec![
                "git".to_string(),
                "commit".to_string(),
                "-m".to_string(),
                "msg".to_string(),
            ],
        };
        match action {
            TerminalAction::Exec { command } => assert_eq!(command.len(), 4),
            _ => panic!("Expected Exec"),
        }
    }

    #[test]
    fn test_terminal_action_tabs_fields() {
        let action = TerminalAction::Tabs { json: true };
        assert!(matches!(action, TerminalAction::Tabs { .. }));
    }

    #[test]
    fn test_terminal_action_tabs_no_json() {
        let action = TerminalAction::Tabs { json: false };
        match action {
            TerminalAction::Tabs { json } => assert!(!json),
            _ => panic!("Expected Tabs"),
        }
    }

    #[test]
    fn test_terminal_action_close() {
        let action = TerminalAction::Close {
            tab: Some("1".to_string()),
        };
        match action {
            TerminalAction::Close { tab } => assert_eq!(tab.as_deref(), Some("1")),
            _ => panic!("Expected Close"),
        }
    }

    #[test]
    fn test_terminal_action_close_no_tab() {
        let action = TerminalAction::Close { tab: None };
        match action {
            TerminalAction::Close { tab } => assert!(tab.is_none()),
            _ => panic!("Expected Close"),
        }
    }
}

pub(crate) fn run(args: TerminalArgs) {
    match args.action {
        Some(TerminalAction::Open) => {
            println!("Terminal panel opened");
        }
        Some(TerminalAction::Exec { command }) => {
            if command.is_empty() {
                eprintln!("No command provided");
                std::process::exit(1);
            }

            let mut iter = command.iter();
            let program = iter.next().expect("command iterator should not be empty");
            let output = std::process::Command::new(program).args(iter).output();
            match output {
                Ok(output) => {
                    if !output.stdout.is_empty() {
                        print!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    if output.status.success() {
                        return;
                    }
                    std::process::exit(output.status.code().unwrap_or(1));
                }
                Err(error) => {
                    eprintln!("Failed to execute command: {}", error);
                    std::process::exit(1);
                }
            }
        }
        Some(TerminalAction::Tabs { json }) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::json!({"tabs": []}))
                        .expect("failed to serialize JSON output")
                );
            } else {
                println!("Terminal tabs: 0");
            }
        }
        Some(TerminalAction::Close { tab }) => {
            println!(
                "Closed terminal tab {}",
                tab.unwrap_or_else(|| "0".to_string())
            );
        }
        None => {
            eprintln!("terminal action required");
            std::process::exit(1);
        }
    }
}
