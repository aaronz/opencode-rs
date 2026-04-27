use std::path::{Path, PathBuf};

mod types;

pub use types::{AgentsMdInfo, AgentsMdScanConfig};

pub struct AgentsMdScanner {
    config: AgentsMdScanConfig,
}

impl AgentsMdScanner {
    pub fn new() -> Self {
        Self {
            config: AgentsMdScanConfig::new(),
        }
    }

    pub fn with_config(config: AgentsMdScanConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &AgentsMdScanConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: AgentsMdScanConfig) {
        self.config = config;
    }

    fn detect_worktree_root(start: &Path) -> Option<PathBuf> {
        let git_path = start.join(".git");
        if !git_path.exists() {
            return None;
        }

        if git_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&git_path) {
                for line in content.lines() {
                    if line.starts_with("gitdir:") {
                        let path = line.trim_start_matches("gitdir:").trim();
                        let worktree_path = PathBuf::from(path);
                        if let Some(parent) = worktree_path.parent() {
                            if parent.file_name().map(|n| n == "worktrees" || n == "git")
                                == Some(true)
                            {
                                if let Some(git_dir) = parent.parent() {
                                    if let Some(project_root) = git_dir.parent() {
                                        return Some(project_root.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn scan_from_cwd(&self) -> Vec<AgentsMdInfo> {
        let cwd = std::env::current_dir().ok();
        let cwd = match cwd {
            Some(c) => c,
            None => return Vec::new(),
        };
        self.scan_from_path(&cwd)
    }

    pub fn scan_from_path(&self, start: &Path) -> Vec<AgentsMdInfo> {
        if !self.config.enabled {
            return Vec::new();
        }

        let worktree_root = Self::detect_worktree_root(start);
        let stop_at = if self.config.stop_at_worktree_root {
            worktree_root.as_ref()
        } else {
            None
        };

        let mut results = Vec::new();
        let mut current = start.to_path_buf();

        loop {
            let agents_md = current.join("AGENTS.md");
            if agents_md.exists() && agents_md.is_file() {
                if let Ok(content) = std::fs::read_to_string(&agents_md) {
                    if self.config.include_hidden || !self.is_hidden(&agents_md) {
                        results.push(AgentsMdInfo {
                            path: agents_md,
                            content,
                        });
                    }
                }
            }

            if stop_at.map(|s| *s == current).unwrap_or(false) {
                break;
            }

            if !current.pop() {
                break;
            }
        }

        results
    }

    fn is_hidden(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false)
    }

    pub fn scan_upward(&self, start: &Path, stop: Option<&Path>) -> Vec<AgentsMdInfo> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut results = Vec::new();
        let mut current = start.to_path_buf();
        let stop = stop.map(PathBuf::from);

        loop {
            let agents_md = current.join("AGENTS.md");
            if agents_md.exists() && agents_md.is_file() {
                if let Ok(content) = std::fs::read_to_string(&agents_md) {
                    results.push(AgentsMdInfo {
                        path: agents_md,
                        content,
                    });
                }
            }

            if stop.as_ref() == Some(&current) {
                break;
            }

            if !current.pop() {
                break;
            }
        }

        results
    }

    pub fn get_first_agents_md(&self, start: &Path) -> Option<AgentsMdInfo> {
        let results = self.scan_upward(start, None);
        results.into_iter().next()
    }
}

impl Default for AgentsMdScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_agents_md_scanner_disabled() {
        let scanner = AgentsMdScanner::new();
        assert!(scanner.config.enabled);

        let config = AgentsMdScanConfig::default();
        assert!(config.enabled);
        assert!(config.stop_at_worktree_root);
        assert!(!config.include_hidden);
    }

    #[test]
    fn test_agents_md_scanner_enabled() {
        let config = AgentsMdScanConfig::new().with_enabled(true);
        let scanner = AgentsMdScanner::with_config(config);
        assert!(scanner.config.enabled);
    }

    #[test]
    fn test_scan_from_path_no_agents_md() {
        let temp = TempDir::new().unwrap();
        let scanner = AgentsMdScanner::new();

        let results = scanner.scan_from_path(temp.path());
        assert!(results.is_empty());
    }

    #[test]
    fn test_scan_from_path_finds_agents_md() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("AGENTS.md"), "# Agent Instructions").unwrap();

        let scanner = AgentsMdScanner::new();
        let results = scanner.scan_from_path(temp.path());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "# Agent Instructions");
    }

    #[test]
    fn test_scan_upward_finds_multiple_agents_md() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("subdir").join("deeper");
        std::fs::create_dir_all(&subdir).unwrap();

        std::fs::write(temp.path().join("AGENTS.md"), "# Root Agents").unwrap();
        std::fs::write(
            temp.path().join("subdir").join("AGENTS.md"),
            "# Subdir Agents",
        )
        .unwrap();

        let scanner = AgentsMdScanner::new();
        let results = scanner.scan_upward(&subdir, None);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_scan_upward_stops_at_stop_path() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("subdir").join("deeper");
        std::fs::create_dir_all(&subdir).unwrap();

        std::fs::write(temp.path().join("AGENTS.md"), "# Root Agents").unwrap();
        std::fs::write(
            temp.path().join("subdir").join("AGENTS.md"),
            "# Subdir Agents",
        )
        .unwrap();

        let scanner = AgentsMdScanner::new();
        let results = scanner.scan_upward(&subdir, Some(&temp.path().join("subdir")));

        assert_eq!(results.len(), 1);
        assert!(results[0].path.to_string_lossy().contains("subdir"));
    }

    #[test]
    fn test_get_first_agents_md() {
        let temp = TempDir::new().unwrap();
        let subdir = temp.path().join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();

        std::fs::write(temp.path().join("AGENTS.md"), "# Root Agents").unwrap();
        std::fs::write(subdir.join("AGENTS.md"), "# Subdir Agents").unwrap();

        let scanner = AgentsMdScanner::new();
        let first = scanner.get_first_agents_md(&subdir);

        assert!(first.is_some());
        assert_eq!(first.unwrap().content, "# Subdir Agents");
    }

    #[test]
    fn test_scan_disabled_returns_empty() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("AGENTS.md"), "# Agents").unwrap();

        let config = AgentsMdScanConfig::new().with_enabled(false);
        let scanner = AgentsMdScanner::with_config(config);

        let results = scanner.scan_from_path(temp.path());
        assert!(results.is_empty());
    }

    #[test]
    fn test_detect_worktree_root_regular_git() {
        let temp = TempDir::new().unwrap();
        std::fs::create_dir(temp.path().join(".git")).unwrap();

        let root = AgentsMdScanner::detect_worktree_root(temp.path());
        assert!(root.is_none());
    }

    #[test]
    fn test_detect_worktree_root_with_worktree() {
        let temp = TempDir::new().unwrap();
        let git_file = temp.path().join(".git");
        let main_repo_git = temp.path().join("main-repo").join(".git");
        let worktree_ref_path = main_repo_git.join("worktrees").join("feature-branch");
        std::fs::create_dir_all(&worktree_ref_path).unwrap();
        std::fs::write(
            &git_file,
            format!("gitdir: {}", worktree_ref_path.to_string_lossy()),
        )
        .unwrap();

        let root = AgentsMdScanner::detect_worktree_root(temp.path());
        assert!(root.is_some());
        assert_eq!(root.unwrap(), temp.path().join("main-repo"));
    }
}
