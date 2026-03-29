use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct UiArgs {
    #[command(subcommand)]
    pub action: UiAction,
}

#[derive(Subcommand, Debug)]
pub enum UiAction {
    #[command(about = "Manage sidebar")]
    Sidebar(SidebarArgs),
}

#[derive(Args, Debug)]
pub struct SidebarArgs {
    #[command(subcommand)]
    pub action: SidebarAction,
}

#[derive(Subcommand, Debug)]
pub enum SidebarAction {
    #[command(about = "Toggle sidebar")]
    Toggle,

    #[command(about = "List sidebar sessions")]
    Sessions {
        #[arg(long)]
        json: bool,
    },

    #[command(about = "Show recent sessions")]
    Recent {
        #[arg(short, long)]
        limit: Option<usize>,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: UiArgs) {
    match args.action {
        UiAction::Sidebar(sidebar_args) => match sidebar_args.action {
            SidebarAction::Toggle => {
                println!("Sidebar toggled");
            }
            SidebarAction::Sessions { json } => {
                if json {
                    let sessions = vec![
                        serde_json::json!({"id": "sidebar-session-1", "name": "Sidebar Session 1"}),
                        serde_json::json!({"id": "sidebar-session-2", "name": "Sidebar Session 2"}),
                    ];
                    println!("{}", serde_json::to_string(&sessions).unwrap());
                } else {
                    println!("Sidebar sessions:");
                    println!("  sidebar-session-1 - Sidebar Session 1");
                    println!("  sidebar-session-2 - Sidebar Session 2");
                }
            }
            SidebarAction::Recent { limit, json } => {
                if json {
                    let recent =
                        vec![serde_json::json!({"id": "session-1", "name": "Recent Session"})];
                    println!("{}", serde_json::to_string(&recent).unwrap());
                } else {
                    println!("Recent sessions (limit: {:?}):", limit);
                    println!("  session-1 - Recent Session");
                }
            }
        },
    }
}
