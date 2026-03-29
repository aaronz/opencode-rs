use clap::{Args, Subcommand};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub static PROJECT_STATE: Lazy<Mutex<ProjectState>> =
    Lazy::new(|| Mutex::new(ProjectState::default()));

#[derive(Default)]
pub struct ProjectState {
    pub projects: HashMap<String, String>,
    pub current_project: Option<String>,
    pub sessions: HashMap<String, Vec<SessionInfo>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
}

#[derive(Args, Debug)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub action: Option<ProjectAction>,
}

#[derive(Subcommand, Debug)]
pub enum ProjectAction {
    #[command(about = "Create a new project")]
    Create {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        path: Option<String>,
    },

    #[command(about = "List all projects")]
    List {
        #[arg(long)]
        json: bool,
    },

    #[command(about = "Delete a project")]
    Delete {
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        confirm: bool,
    },

    #[command(about = "Switch to a project")]
    Switch {
        #[arg(short, long)]
        name: String,
    },

    #[command(about = "Show current project")]
    Current,
}

pub fn run(args: ProjectArgs) {
    match args.action {
        Some(ProjectAction::Create { name, path: _ }) => {
            let mut state = PROJECT_STATE.lock().unwrap();
            state.projects.insert(name.clone(), name.clone());
            if state.current_project.is_none() {
                state.current_project = Some(name.clone());
            }
            println!("Created project: {}", name);
        }
        Some(ProjectAction::List { json }) => {
            let state = PROJECT_STATE.lock().unwrap();
            if json {
                let projects: Vec<_> = state
                    .projects
                    .keys()
                    .map(|name| serde_json::json!({"name": name, "path": format!("/tmp/{}", name)}))
                    .collect();
                println!("{}", serde_json::to_string(&projects).unwrap());
            } else {
                println!("Projects:");
                for name in state.projects.keys() {
                    println!("  {}", name);
                }
            }
        }
        Some(ProjectAction::Delete { name, confirm }) => {
            let mut state = PROJECT_STATE.lock().unwrap();
            if confirm {
                state.projects.remove(&name);
                if state.current_project.as_ref() == Some(&name) {
                    state.current_project = None;
                }
                println!("Deleted project: {}", name);
            } else {
                eprintln!("Error: Use --confirm to delete");
                std::process::exit(1);
            }
        }
        Some(ProjectAction::Switch { name }) => {
            let mut state = PROJECT_STATE.lock().unwrap();
            if state.projects.contains_key(&name) {
                state.current_project = Some(name.clone());
                println!("Switched to project: {}", name);
            } else {
                eprintln!("Error: Project '{}' does not exist", name);
                std::process::exit(1);
            }
        }
        Some(ProjectAction::Current) => {
            let state = PROJECT_STATE.lock().unwrap();
            if let Some(project) = &state.current_project {
                println!("{}", project);
            } else {
                println!("No project selected");
            }
        }
        None => {
            println!("Usage: opencode-rs project <action>");
            println!("Actions: create, list, delete, switch, current");
        }
    }
}
