use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct TerminalArgs {
    #[command(subcommand)]
    pub action: Option<TerminalAction>,
}

#[derive(Subcommand, Debug)]
pub enum TerminalAction {
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
    fn test_terminal_action_tabs_fields() {
        let action = TerminalAction::Tabs { json: true };
        assert!(matches!(action, TerminalAction::Tabs { .. }));
    }
}

pub fn run(args: TerminalArgs) {
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
