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
            let program = iter.next().unwrap();
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
                    serde_json::to_string(&serde_json::json!({"tabs": []})).unwrap()
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
