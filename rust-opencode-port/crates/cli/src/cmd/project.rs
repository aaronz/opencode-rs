use clap::{Args, Subcommand};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub static PROJECT_STATE: Lazy<Mutex<ProjectState>> =
    Lazy::new(|| Mutex::new(ProjectState::default()));

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProjectState {
    pub projects: HashMap<String, ProjectInfo>,
    pub current_project: Option<String>,
    pub _sessions: HashMap<String, Vec<SessionInfo>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub model: Option<String>,
}

fn project_state_path() -> PathBuf {
    if let Ok(data_dir) = std::env::var("OPENCODE_DATA_DIR") {
        let path = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&path);
        return path.join("projects.json");
    }

    directories::ProjectDirs::from("com", "opencode", "rs")
        .map(|dirs| dirs.data_dir().join("projects.json"))
        .unwrap_or_else(|| PathBuf::from("./projects.json"))
}

pub fn load_project_state() -> ProjectState {
    let path = project_state_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<ProjectState>(&content).ok())
        .unwrap_or_default()
}

fn save_project_state(state: &ProjectState) {
    let path = project_state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let serialized = serde_json::to_string_pretty(state).unwrap();
    std::fs::write(path, serialized).unwrap();
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

    #[command(about = "Rename a project")]
    Rename {
        #[arg(long)]
        from: String,

        #[arg(long)]
        to: String,
    },

    #[command(about = "Edit project metadata")]
    Edit {
        #[arg(short, long)]
        name: String,

        #[arg(long)]
        description: Option<String>,

        #[arg(long)]
        model: Option<String>,
    },

    #[command(about = "Show a project")]
    Show {
        #[arg(short, long)]
        name: String,

        #[arg(long)]
        json: bool,
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
        Some(ProjectAction::Create { name, path }) => {
            let mut state = load_project_state();
            let path = path.unwrap_or_else(|| format!("/tmp/{}", name));
            state.projects.insert(
                name.clone(),
                ProjectInfo {
                    name: name.clone(),
                    path,
                    description: None,
                    model: None,
                },
            );
            if state.current_project.is_none() {
                state.current_project = Some(name.clone());
            }
            *PROJECT_STATE.lock().unwrap() = state.clone();
            save_project_state(&state);
            println!("Created project: {}", name);
        }
        Some(ProjectAction::Rename { from, to }) => {
            let mut state = load_project_state();
            let mut project = match state.projects.remove(&from) {
                Some(project) => project,
                None => {
                    eprintln!("Error: Project '{}' does not exist", from);
                    std::process::exit(1);
                }
            };
            project.name = to.clone();
            if project.path.ends_with(&format!("/{}", from)) {
                project.path = format!("/tmp/{}", to);
            }
            state.projects.insert(to.clone(), project);
            if state.current_project.as_ref() == Some(&from) {
                state.current_project = Some(to.clone());
            }
            *PROJECT_STATE.lock().unwrap() = state.clone();
            save_project_state(&state);
            println!("Renamed project: {} -> {}", from, to);
        }
        Some(ProjectAction::Edit {
            name,
            description,
            model,
        }) => {
            let mut state = load_project_state();
            let project = match state.projects.get_mut(&name) {
                Some(project) => project,
                None => {
                    eprintln!("Error: Project '{}' does not exist", name);
                    std::process::exit(1);
                }
            };
            if let Some(description) = description {
                project.description = Some(description);
            }
            if let Some(model) = model {
                project.model = Some(model);
            }
            *PROJECT_STATE.lock().unwrap() = state.clone();
            save_project_state(&state);
            println!("Updated project: {}", name);
        }
        Some(ProjectAction::Show { name, json }) => {
            let state = load_project_state();
            let project = match state.projects.get(&name) {
                Some(project) => project,
                None => {
                    eprintln!("Error: Project '{}' does not exist", name);
                    std::process::exit(1);
                }
            };
            if json {
                println!("{}", serde_json::to_string(project).unwrap());
            } else {
                println!("Project: {}", project.name);
                println!("Path: {}", project.path);
                if let Some(description) = &project.description {
                    println!("Description: {}", description);
                }
                if let Some(model) = &project.model {
                    println!("Model: {}", model);
                }
            }
        }
        Some(ProjectAction::List { json }) => {
            let state = load_project_state();
            if json {
                let projects: Vec<_> = state
                    .projects
                    .values()
                    .map(|project| {
                        serde_json::json!({
                            "name": project.name,
                            "path": project.path,
                            "description": project.description,
                            "model": project.model,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string(&projects).unwrap());
            } else {
                println!("Projects:");
                for project in state.projects.values() {
                    println!("  {}", project.name);
                }
            }
        }
        Some(ProjectAction::Delete { name, confirm }) => {
            let mut state = load_project_state();
            if confirm {
                state.projects.remove(&name);
                if state.current_project.as_ref() == Some(&name) {
                    state.current_project = None;
                }
                *PROJECT_STATE.lock().unwrap() = state.clone();
                save_project_state(&state);
                println!("Deleted project: {}", name);
            } else {
                eprintln!("Error: Use --confirm to delete");
                std::process::exit(1);
            }
        }
        Some(ProjectAction::Switch { name }) => {
            let mut state = load_project_state();
            if state.projects.contains_key(&name) {
                state.current_project = Some(name.clone());
                *PROJECT_STATE.lock().unwrap() = state.clone();
                save_project_state(&state);
                println!("Switched to project: {}", name);
            } else {
                eprintln!("Error: Project '{}' does not exist", name);
                std::process::exit(1);
            }
        }
        Some(ProjectAction::Current) => {
            let state = load_project_state();
            if let Some(project) = &state.current_project {
                println!("{}", project);
            } else {
                println!("No project selected");
            }
        }
        None => {
            println!("Usage: opencode-rs project <action>");
            println!("Actions: create, rename, edit, show, list, delete, switch, current");
        }
    }
}
