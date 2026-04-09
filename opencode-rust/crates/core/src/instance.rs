use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::RwLock;

static INSTANCE: RwLock<Option<InstanceInfo>> = RwLock::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub directory: PathBuf,
    pub worktree: PathBuf,
    pub project_name: String,
}

pub struct Instance;

impl Instance {
    pub fn set(directory: PathBuf, worktree: PathBuf, project_name: String) {
        let info = InstanceInfo {
            directory,
            worktree,
            project_name,
        };
        if let Ok(mut guard) = INSTANCE.write() {
            *guard = Some(info);
        }
    }

    pub fn get() -> Option<InstanceInfo> {
        INSTANCE.read().ok().and_then(|guard| guard.clone())
    }

    pub fn directory() -> Option<PathBuf> {
        Self::get().map(|i| i.directory)
    }

    pub fn worktree() -> Option<PathBuf> {
        Self::get().map(|i| i.worktree)
    }

    pub fn project_name() -> Option<String> {
        Self::get().map(|i| i.project_name)
    }

    pub fn reset() {
        if let Ok(mut guard) = INSTANCE.write() {
            *guard = None;
        }
    }
}
