use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct SessionArgs {
    #[arg(short, long)]
    pub id: Option<String>,

    #[command(subcommand)]
    pub action: Option<SessionAction>,
}

#[derive(Subcommand, Debug)]
pub enum SessionAction {
    Delete,
    Show {
        #[arg(short, long)]
        json: bool,
    },
    Export,
    Undo {
        #[arg(short, long, default_value = "1")]
        steps: usize,
    },
    Redo {
        #[arg(short, long, default_value = "1")]
        steps: usize,
    },
    Review {
        #[arg(short, long)]
        file: Option<String>,
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    Diff {
        #[arg(short, long)]
        file: String,
        #[arg(short, long, default_value = "3")]
        context: usize,
    },
}

pub fn run(args: SessionArgs) {
    match args.action {
        Some(SessionAction::Undo { steps }) => {
            if let Some(id) = args.id {
                println!("Undoing {} steps in session {}", steps, id);
            } else {
                eprintln!("Error: Session ID required for undo");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Redo { steps }) => {
            if let Some(id) = args.id {
                println!("Redoing {} steps in session {}", steps, id);
            } else {
                eprintln!("Error: Session ID required for redo");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Review { file, format }) => {
            if let Some(id) = args.id {
                println!(
                    "Reviewing session {} (file: {:?}, format: {})",
                    id, file, format
                );
            } else {
                eprintln!("Error: Session ID required for review");
                std::process::exit(1);
            }
        }
        Some(SessionAction::Diff { file, context }) => {
            if let Some(id) = args.id {
                println!(
                    "Showing diff for {} in session {} (context: {})",
                    file, id, context
                );
            } else {
                eprintln!("Error: Session ID required for diff");
                std::process::exit(1);
            }
        }
        _ => {
            println!("Session action: {:?}", args.action);
        }
    }
}
