use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
    pub language: String,
    pub has_git: bool,
    pub has_tests: bool,
    pub has_docs: bool,
}

pub struct ProjectManager {
    current: Option<ProjectInfo>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn detect(root: PathBuf) -> Option<ProjectInfo> {
        if !root.exists() {
            return None;
        }

        let has_git = root.join(".git").exists();
        let has_tests = root.join("tests").exists() || root.join("test").exists();
        let has_docs = root.join("docs").exists() || root.join("README.md").exists();

        let language = if root.join("Cargo.toml").exists() {
            "rust".to_string()
        } else if root.join("package.json").exists() {
            "javascript".to_string()
        } else if root.join("pyproject.toml").exists() || root.join("setup.py").exists() {
            "python".to_string()
        } else if root.join("go.mod").exists() {
            "go".to_string()
        } else {
            "unknown".to_string()
        };

        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Some(ProjectInfo {
            root,
            name,
            language,
            has_git,
            has_tests,
            has_docs,
        })
    }

    pub fn set_current(&mut self, info: ProjectInfo) {
        self.current = Some(info);
    }

    pub fn current(&self) -> Option<&ProjectInfo> {
        self.current.as_ref()
    }

    pub fn is_rust(&self) -> bool {
        self.current
            .as_ref()
            .map(|p| p.language == "rust")
            .unwrap_or(false)
    }

    pub fn is_typescript(&self) -> bool {
        self.current
            .as_ref()
            .map(|p| p.language == "javascript")
            .unwrap_or(false)
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}
