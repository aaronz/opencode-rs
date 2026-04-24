use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub(crate) struct UiArgs {
    #[command(subcommand)]
    pub action: UiAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum UiAction {
    #[command(about = "Manage sidebar")]
    Sidebar(SidebarArgs),
}

#[derive(Args, Debug)]
pub(crate) struct SidebarArgs {
    #[command(subcommand)]
    pub action: SidebarAction,
}

#[derive(Subcommand, Debug)]
pub(crate) enum SidebarAction {
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

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_action_sidebar() {
        let action = UiAction::Sidebar(SidebarArgs {
            action: SidebarAction::Toggle,
        });
        assert!(matches!(action, UiAction::Sidebar(..)));
    }

    #[test]
    fn test_sidebar_action_toggle() {
        let action = SidebarAction::Toggle;
        assert!(matches!(action, SidebarAction::Toggle));
    }

    #[test]
    fn test_sidebar_action_sessions_fields() {
        let action = SidebarAction::Sessions { json: true };
        assert!(matches!(action, SidebarAction::Sessions { .. }));
    }

    #[test]
    fn test_sidebar_action_sessions_no_json() {
        let action = SidebarAction::Sessions { json: false };
        match action {
            SidebarAction::Sessions { json } => assert!(!json),
            _ => panic!("Expected Sessions"),
        }
    }

    #[test]
    fn test_sidebar_action_recent() {
        let action = SidebarAction::Recent {
            limit: Some(10),
            json: true,
        };
        match action {
            SidebarAction::Recent { limit, json } => {
                assert_eq!(limit, Some(10));
                assert!(json);
            }
            _ => panic!("Expected Recent"),
        }
    }

    #[test]
    fn test_sidebar_action_recent_no_limit() {
        let action = SidebarAction::Recent {
            limit: None,
            json: false,
        };
        match action {
            SidebarAction::Recent { limit, json } => {
                assert!(limit.is_none());
                assert!(!json);
            }
            _ => panic!("Expected Recent"),
        }
    }

    #[test]
    fn test_sidebar_args_creation() {
        let args = SidebarArgs {
            action: SidebarAction::Toggle,
        };
        match args.action {
            SidebarAction::Toggle => {}
            _ => panic!("Expected Toggle"),
        }
    }

    #[test]
    fn test_ui_args_creation() {
        let args = UiArgs {
            action: UiAction::Sidebar(SidebarArgs {
                action: SidebarAction::Sessions { json: false },
            }),
        };
        match args.action {
            UiAction::Sidebar(SidebarArgs { action }) => {
                assert!(matches!(action, SidebarAction::Sessions { .. }));
            }
        }
    }
}

pub(crate) fn run(args: UiArgs) {
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
                    println!(
                        "{}",
                        serde_json::to_string(&sessions).expect("failed to serialize JSON output")
                    );
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
                    println!(
                        "{}",
                        serde_json::to_string(&recent).expect("failed to serialize JSON output")
                    );
                } else {
                    println!("Recent sessions (limit: {:?}):", limit);
                    println!("  session-1 - Recent Session");
                }
            }
        },
    }
}
