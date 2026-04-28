use clap::{Args, Subcommand};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub(crate) static PROJECT_STATE: Lazy<Mutex<ProjectState>> =
    Lazy::new(|| Mutex::new(ProjectState::default()));

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectState {
    pub projects: HashMap<String, ProjectInfo>,
    pub current_project: Option<String>,
    pub _sessions: HashMap<String, Vec<SessionInfo>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub model: Option<String>,
}

fn project_state_path() -> PathBuf {
    if let Ok(data_dir) = std::env::var("OPENCODE_RS_DATA_DIR") {
        let path = PathBuf::from(data_dir);
        let _ = std::fs::create_dir_all(&path);
        return path.join("projects.json");
    }

    directories::ProjectDirs::from("ai", "opencode", "opencode-rs")
        .map(|dirs| dirs.data_dir().join("projects.json"))
        .unwrap_or_else(|| PathBuf::from("./projects.json"))
}

pub(crate) fn load_project_state() -> ProjectState {
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
    let serialized =
        serde_json::to_string_pretty(state).expect("failed to serialize project state");
    std::fs::write(&path, serialized).expect("failed to write project state file");
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct SessionInfo {
    pub id: String,
    pub name: String,
}

#[derive(Args, Debug)]
pub(crate) struct ProjectArgs {
    #[command(subcommand)]
    pub action: Option<ProjectAction>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ProjectAction {
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

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_args_default() {
        let args = ProjectArgs { action: None };
        assert!(args.action.is_none());
    }

    #[test]
    fn test_project_args_with_action() {
        let args = ProjectArgs {
            action: Some(ProjectAction::Current),
        };
        match args.action {
            Some(ProjectAction::Current) => {}
            _ => panic!("Expected Current"),
        }
    }

    #[test]
    fn test_project_action_create() {
        let action = ProjectAction::Create {
            name: "my-project".to_string(),
            path: Some("/path/to/project".to_string()),
        };
        match action {
            ProjectAction::Create { name, path } => {
                assert_eq!(name, "my-project");
                assert_eq!(path.as_deref(), Some("/path/to/project"));
            }
            _ => panic!("Expected Create"),
        }
    }

    #[test]
    fn test_project_action_create_without_path() {
        let action = ProjectAction::Create {
            name: "my-project".to_string(),
            path: None,
        };
        match action {
            ProjectAction::Create { name, path } => {
                assert_eq!(name, "my-project");
                assert!(path.is_none());
            }
            _ => panic!("Expected Create"),
        }
    }

    #[test]
    fn test_project_action_rename() {
        let action = ProjectAction::Rename {
            from: "old-name".to_string(),
            to: "new-name".to_string(),
        };
        match action {
            ProjectAction::Rename { from, to } => {
                assert_eq!(from, "old-name");
                assert_eq!(to, "new-name");
            }
            _ => panic!("Expected Rename"),
        }
    }

    #[test]
    fn test_project_action_edit() {
        let action = ProjectAction::Edit {
            name: "my-project".to_string(),
            description: Some("A test project".to_string()),
            model: Some("gpt-4".to_string()),
        };
        match action {
            ProjectAction::Edit {
                name,
                description,
                model,
            } => {
                assert_eq!(name, "my-project");
                assert_eq!(description.as_deref(), Some("A test project"));
                assert_eq!(model.as_deref(), Some("gpt-4"));
            }
            _ => panic!("Expected Edit"),
        }
    }

    #[test]
    fn test_project_action_edit_partial() {
        let action = ProjectAction::Edit {
            name: "my-project".to_string(),
            description: None,
            model: Some("gpt-4".to_string()),
        };
        match action {
            ProjectAction::Edit {
                name,
                description,
                model,
            } => {
                assert_eq!(name, "my-project");
                assert!(description.is_none());
                assert_eq!(model.as_deref(), Some("gpt-4"));
            }
            _ => panic!("Expected Edit"),
        }
    }

    #[test]
    fn test_project_action_show() {
        let action = ProjectAction::Show {
            name: "my-project".to_string(),
            json: true,
        };
        match action {
            ProjectAction::Show { name, json } => {
                assert_eq!(name, "my-project");
                assert!(json);
            }
            _ => panic!("Expected Show"),
        }
    }

    #[test]
    fn test_project_action_show_no_json() {
        let action = ProjectAction::Show {
            name: "my-project".to_string(),
            json: false,
        };
        match action {
            ProjectAction::Show { name, json } => {
                assert_eq!(name, "my-project");
                assert!(!json);
            }
            _ => panic!("Expected Show"),
        }
    }

    #[test]
    fn test_project_action_list() {
        let action = ProjectAction::List { json: true };
        match action {
            ProjectAction::List { json } => assert!(json),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_project_action_delete() {
        let action = ProjectAction::Delete {
            name: "my-project".to_string(),
            confirm: true,
        };
        match action {
            ProjectAction::Delete { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(confirm);
            }
            _ => panic!("Expected Delete"),
        }
    }

    #[test]
    fn test_project_action_delete_no_confirm() {
        let action = ProjectAction::Delete {
            name: "my-project".to_string(),
            confirm: false,
        };
        match action {
            ProjectAction::Delete { name, confirm } => {
                assert_eq!(name, "my-project");
                assert!(!confirm);
            }
            _ => panic!("Expected Delete"),
        }
    }

    #[test]
    fn test_project_action_switch() {
        let action = ProjectAction::Switch {
            name: "my-project".to_string(),
        };
        match action {
            ProjectAction::Switch { name } => assert_eq!(name, "my-project"),
            _ => panic!("Expected Switch"),
        }
    }

    #[test]
    fn test_project_action_current() {
        let action = ProjectAction::Current;
        match action {
            ProjectAction::Current => {}
            _ => panic!("Expected Current"),
        }
    }

    #[test]
    fn test_project_state_default() {
        let state = ProjectState::default();
        assert!(state.projects.is_empty());
        assert!(state.current_project.is_none());
    }

    #[test]
    fn test_project_info_creation() {
        let info = ProjectInfo {
            name: "test-project".to_string(),
            path: "/path/to/project".to_string(),
            description: Some("A test project".to_string()),
            model: Some("gpt-4".to_string()),
        };
        assert_eq!(info.name, "test-project");
        assert_eq!(info.path, "/path/to/project");
        assert_eq!(info.description.as_deref(), Some("A test project"));
        assert_eq!(info.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_project_info_minimal() {
        let info = ProjectInfo {
            name: "minimal".to_string(),
            path: "/minimal".to_string(),
            description: None,
            model: None,
        };
        assert_eq!(info.name, "minimal");
        assert!(info.description.is_none());
        assert!(info.model.is_none());
    }

    #[test]
    fn test_session_info_creation() {
        let info = SessionInfo {
            id: "session-123".to_string(),
            name: "Test Session".to_string(),
        };
        assert_eq!(info.id, "session-123");
        assert_eq!(info.name, "Test Session");
    }

    #[test]
    fn test_project_state_with_projects() {
        let mut state = ProjectState::default();
        state.projects.insert(
            "test".to_string(),
            ProjectInfo {
                name: "test".to_string(),
                path: "/test".to_string(),
                description: None,
                model: None,
            },
        );
        state.current_project = Some("test".to_string());
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.current_project.as_deref(), Some("test"));
    }
}

pub(crate) fn run(args: ProjectArgs) {
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
            *PROJECT_STATE.lock().unwrap_or_else(|p| p.into_inner()) = state.clone();
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
            *PROJECT_STATE.lock().unwrap_or_else(|p| p.into_inner()) = state.clone();
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
            *PROJECT_STATE.lock().unwrap_or_else(|p| p.into_inner()) = state.clone();
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
                println!(
                    "{}",
                    serde_json::to_string(project).expect("failed to serialize JSON output")
                );
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
                println!(
                    "{}",
                    serde_json::to_string(&projects).expect("failed to serialize JSON output")
                );
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
                *PROJECT_STATE.lock().unwrap_or_else(|p| p.into_inner()) = state.clone();
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
                *PROJECT_STATE.lock().unwrap_or_else(|p| p.into_inner()) = state.clone();
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
