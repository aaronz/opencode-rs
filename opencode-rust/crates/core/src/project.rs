use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Node,
    Rust,
    Python,
    Go,
    Java,
    Cpp,
    Ruby,
    Php,
    Dotnet,
    Swift,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
    Cargo,
    Pip,
    Poetry,
    Go,
    Maven,
    Gradle,
    Unknown,
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("No project found from: {0}")]
    NotFound(PathBuf),

    #[error("Failed to read project file: {0}")]
    ReadError(PathBuf, #[source] std::io::Error),

    #[error("Failed to parse config: {0}")]
    ParseError(String, #[source] serde_json::Error),

    #[error("Ambiguous project: multiple roots found")]
    Ambiguous,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub package_json: Option<serde_json::Value>,
    pub cargo_toml: Option<String>,
    pub start_command: Option<String>,
    pub install_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,
    pub language: String,
    pub has_git: bool,
    pub has_tests: bool,
    pub has_docs: bool,
    pub vcs_root: Option<PathBuf>,
    pub worktree_root: Option<PathBuf>,
}

#[allow(dead_code)]
pub struct ProjectManager {
    current: Option<ProjectInfo>,
}

#[allow(dead_code)]
impl ProjectManager {
    pub(crate) fn new() -> Self {
        Self { current: None }
    }

    pub(crate) fn detect(root: PathBuf) -> Option<ProjectInfo> {
        if root.is_relative() {
            return None;
        }

        let validated = match validate_workspace(&root) {
            Ok(v) => v,
            Err(_) => return None,
        };

        let vcs_root = Self::find_git_repository(&validated);
        let has_git = vcs_root.is_some();
        let has_tests = validated.join("tests").exists() || validated.join("test").exists();
        let has_docs = validated.join("docs").exists() || validated.join("README.md").exists();

        let is_worktree = Self::is_worktree_path(&validated);
        let detected_worktree_root = Self::detect_worktree_root_from_subdirectory(&validated);
        let worktree_root = if is_worktree {
            detected_worktree_root.or(Some(validated.clone()))
        } else {
            None
        };

        let language = if validated.join("Cargo.toml").exists() {
            "rust".to_string()
        } else if validated.join("package.json").exists() {
            "javascript".to_string()
        } else if validated.join("pyproject.toml").exists() || validated.join("setup.py").exists() {
            "python".to_string()
        } else if validated.join("go.mod").exists() {
            "go".to_string()
        } else {
            "unknown".to_string()
        };

        let name = validated
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        Some(ProjectInfo {
            root: validated,
            name,
            language,
            has_git,
            has_tests,
            has_docs,
            vcs_root,
            worktree_root,
        })
    }

    #[allow(clippy::ptr_arg)]
    fn find_git_repository(start: &PathBuf) -> Option<PathBuf> {
        let mut current = start.clone();
        loop {
            let git_path = current.join(".git");
            if git_path.exists() {
                return Some(current.clone());
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    #[allow(clippy::ptr_arg)]
    fn detect_worktree_root_from_subdirectory(start: &PathBuf) -> Option<PathBuf> {
        let mut current = start.clone();
        loop {
            let git_path = current.join(".git");
            if git_path.exists() {
                if git_path.is_file() {
                    return Self::parse_worktree_git_file(&git_path);
                }
                return None;
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    #[allow(clippy::ptr_arg)]
    fn is_worktree_path(start: &PathBuf) -> bool {
        let mut current = start.clone();
        loop {
            let git_path = current.join(".git");
            if git_path.exists() {
                return git_path.is_file();
            }
            if !current.pop() {
                break;
            }
        }
        false
    }

    fn parse_worktree_git_file(git_path: &Path) -> Option<PathBuf> {
        if let Ok(content) = std::fs::read_to_string(git_path) {
            for line in content.lines() {
                if line.starts_with("gitdir:") {
                    let path = line.trim_start_matches("gitdir:").trim();
                    let worktree_path = PathBuf::from(path);
                    if let Some(parent) = worktree_path.parent() {
                        if parent.file_name().map(|n| n == "worktrees" || n == "git") == Some(true)
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
        None
    }

    pub(crate) fn set_current(&mut self, info: ProjectInfo) {
        self.current = Some(info);
    }

    pub(crate) fn current(&self) -> Option<&ProjectInfo> {
        self.current.as_ref()
    }

    pub(crate) fn is_rust(&self) -> bool {
        self.current
            .as_ref()
            .map(|p| p.language == "rust")
            .unwrap_or(false)
    }

    pub(crate) fn is_typescript(&self) -> bool {
        self.current
            .as_ref()
            .map(|p| p.language == "javascript")
            .unwrap_or(false)
    }
}

pub(crate) fn normalize_path(path: &PathBuf) -> std::io::Result<PathBuf> {
    normalize_path_with_context(path, None)
}

pub(crate) fn normalize_path_with_context(
    path: &PathBuf,
    context: Option<&PathBuf>,
) -> std::io::Result<PathBuf> {
    if path.is_relative() {
        if context.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Relative paths are not allowed in this context. Provide an absolute path.",
            ));
        }
        let absolute_path = std::env::current_dir()?.join(path);
        let canonical_path = absolute_path.canonicalize()?;
        Ok(canonical_path)
    } else {
        let canonical_path = path.canonicalize()?;
        if let Some(context_path) = context {
            validate_path_within_workspace(&canonical_path, context_path)?;
        }
        Ok(canonical_path)
    }
}

fn validate_path_within_workspace(path: &Path, workspace: &Path) -> io::Result<()> {
    let path_str = path.to_string_lossy();
    let workspace_str = workspace.to_string_lossy();

    if !path_str.starts_with(&*workspace_str) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Path '{}' resolves to '{}' which is outside the workspace '{}'",
                path.display(),
                path_str,
                workspace_str
            ),
        ));
    }
    Ok(())
}

fn is_circular_symlink_error(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::InvalidData
        || err
            .to_string()
            .contains("Too many levels of symbolic links")
        || err.to_string().contains("ELOOP")
}

fn is_permission_denied_error(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::PermissionDenied || err.to_string().contains("Permission denied")
}

fn check_path_traversal(path: &Path) -> Option<String> {
    let components: Vec<_> = path.components().collect();
    let mut traversal_count = 0;

    for component in &components {
        match component {
            std::path::Component::ParentDir => traversal_count += 1,
            std::path::Component::Normal(_) => {
                if traversal_count > 0 && component.as_os_str().to_string_lossy().starts_with('.') {
                    return Some(format!(
                        "Suspicious path component '{}' following '..' detected",
                        component.as_os_str().to_string_lossy()
                    ));
                }
                traversal_count = 0;
            }
            _ => {}
        }
    }
    None
}

#[allow(dead_code)]
pub(crate) fn check_path_traversal_safe(path: &str) -> Option<String> {
    check_path_traversal(Path::new(path))
}

#[allow(dead_code)]
pub(crate) fn is_path_traversal_attempt(path: &str) -> bool {
    let p = Path::new(path);

    let path_str = path.replace('\\', "/");
    if path_str.contains("..%2f") || path_str.contains("%2e%2e") {
        return true;
    }

    if path.contains('\0') {
        return true;
    }

    if path_str.starts_with('/') && !path_str.starts_with("./") {
        return true;
    }

    let bytes = path_str.as_bytes();
    for i in 0..bytes.len() - 1 {
        if bytes[i] == b'.' && bytes[i + 1] == b'.' {
            if i == 0 {
                return true;
            }
            let prev = bytes[i.saturating_sub(1)];
            if prev == b'/' {
                if i >= 2 && bytes[i.saturating_sub(2)].is_ascii_alphabetic() {
                    continue;
                }
                return true;
            }
        }
    }

    check_path_traversal(p).is_some()
}

#[derive(Debug)]
pub enum WorkspaceValidationError {
    PathNotFound(String),
    PathNotAccessible(String),
    PathNotDirectory(String),
    PathNotReadable(String),
    PathPermissionDenied(String),
    PathCircularSymlink(String),
    PathTraversalDetected(String),
    PathNotAbsolute(String),
}

#[allow(dead_code)]
impl WorkspaceValidationError {
    pub(crate) fn code(&self) -> u16 {
        match self {
            Self::PathNotFound(_) => 7011,
            Self::PathNotAccessible(_) => 7012,
            Self::PathNotDirectory(_) => 7013,
            Self::PathNotReadable(_) => 7014,
            Self::PathPermissionDenied(_) => 7015,
            Self::PathCircularSymlink(_) => 7016,
            Self::PathTraversalDetected(_) => 7017,
            Self::PathNotAbsolute(_) => 7018,
        }
    }
}

impl std::fmt::Display for WorkspaceValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceValidationError::PathNotFound(p) => {
                write!(f, "Workspace path does not exist: {}", p)
            }
            WorkspaceValidationError::PathNotAccessible(p) => {
                write!(f, "Workspace path is not accessible: {}", p)
            }
            WorkspaceValidationError::PathNotDirectory(p) => {
                write!(f, "Workspace path is not a directory: {}", p)
            }
            WorkspaceValidationError::PathNotReadable(p) => {
                write!(f, "Workspace path is not readable: {}", p)
            }
            WorkspaceValidationError::PathPermissionDenied(p) => {
                write!(f, "Permission denied accessing workspace path: {}", p)
            }
            WorkspaceValidationError::PathCircularSymlink(p) => {
                write!(f, "Circular symbolic link detected: {}", p)
            }
            WorkspaceValidationError::PathTraversalDetected(p) => {
                write!(f, "Path traversal detected: {}", p)
            }
            WorkspaceValidationError::PathNotAbsolute(p) => {
                write!(f, "Absolute path required, got relative path: {}", p)
            }
        }
    }
}

impl std::error::Error for WorkspaceValidationError {}

pub type WorkspaceValidationResult = Result<PathBuf, WorkspaceValidationError>;

pub(crate) fn validate_workspace(path: &PathBuf) -> WorkspaceValidationResult {
    validate_workspace_impl(path, None)
}

#[allow(dead_code)]
pub(crate) fn validate_workspace_path(path: &str) -> WorkspaceValidationResult {
    let path_buf = PathBuf::from(path);
    validate_workspace(&path_buf)
}

#[allow(dead_code)]
pub(crate) fn validate_workspace_with_allowed_roots(
    path: &PathBuf,
    allowed_roots: &[PathBuf],
) -> WorkspaceValidationResult {
    if allowed_roots.is_empty() {
        return validate_workspace_impl(path, None);
    }

    let normalized = normalize_path_with_context(path, None).map_err(|e| {
        if is_permission_denied_error(&e) {
            WorkspaceValidationError::PathPermissionDenied(format!("{}: {}", path.display(), e))
        } else if is_circular_symlink_error(&e) {
            WorkspaceValidationError::PathCircularSymlink(format!("{}: {}", path.display(), e))
        } else {
            WorkspaceValidationError::PathNotAccessible(format!("{}: {}", path.display(), e))
        }
    })?;

    let path_str = normalized.to_string_lossy();
    for root in allowed_roots {
        let canonical_root = root.canonicalize().map_err(|e| {
            WorkspaceValidationError::PathNotAccessible(format!(
                "Failed to canonicalize allowed root '{}': {}",
                root.display(),
                e
            ))
        })?;
        let root_str = canonical_root.to_string_lossy();
        if path_str.starts_with(&*root_str) {
            return Ok(normalized);
        }
    }

    Err(WorkspaceValidationError::PathNotAccessible(format!(
        "Path '{}' is not within any allowed workspace root",
        path.display()
    )))
}

fn validate_workspace_impl(
    path: &PathBuf,
    _context: Option<&PathBuf>,
) -> WorkspaceValidationResult {
    if path.is_relative() {
        return Err(WorkspaceValidationError::PathNotAbsolute(format!(
            "{} (relative paths must be converted to absolute first)",
            path.display()
        )));
    }

    if !path.exists() {
        return Err(WorkspaceValidationError::PathNotFound(
            path.display().to_string(),
        ));
    }

    let normalized = normalize_path(path).map_err(|e| {
        if is_permission_denied_error(&e) {
            WorkspaceValidationError::PathPermissionDenied(format!("{}: {}", path.display(), e))
        } else if is_circular_symlink_error(&e) {
            WorkspaceValidationError::PathCircularSymlink(format!("{}: {}", path.display(), e))
        } else if let Some(traversal_msg) = check_path_traversal(path) {
            WorkspaceValidationError::PathTraversalDetected(traversal_msg)
        } else {
            WorkspaceValidationError::PathNotAccessible(format!("{}: {}", path.display(), e))
        }
    })?;

    if !normalized.is_dir() {
        return Err(WorkspaceValidationError::PathNotDirectory(
            path.display().to_string(),
        ));
    }

    let read_test = normalized.join(".opencode_read_test");
    match std::fs::write(&read_test, "") {
        Ok(_) => {
            let _ = std::fs::remove_file(read_test);
        }
        Err(e) => {
            if is_permission_denied_error(&e) {
                return Err(WorkspaceValidationError::PathPermissionDenied(format!(
                    "{}: {}",
                    path.display(),
                    e
                )));
            }
            return Err(WorkspaceValidationError::PathNotReadable(format!(
                "{}: {}",
                path.display(),
                e
            )));
        }
    }

    Ok(normalized)
}

pub fn is_absolute_path(path: &Path) -> bool {
    path.is_absolute()
}

pub fn resolve_relative_path(path: &PathBuf) -> std::io::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.clone());
    }
    std::env::current_dir().map(|cwd| cwd.join(path))
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_manager_new() {
        let pm = ProjectManager::new();
        assert!(pm.current().is_none());
    }

    #[test]
    fn test_project_detect_rust() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert_eq!(info.language, "rust");
    }

    #[test]
    fn test_project_detect_javascript() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("package.json"), "{}").unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert_eq!(info.language, "javascript");
    }

    #[test]
    fn test_project_detect_nonexistent() {
        let info = ProjectManager::detect(PathBuf::from("/nonexistent/path"));
        assert!(info.is_none());
    }

    #[test]
    fn test_project_manager_set_current() {
        let mut pm = ProjectManager::new();
        let info = ProjectInfo {
            root: PathBuf::from("/test"),
            name: "test".to_string(),
            language: "rust".to_string(),
            has_git: false,
            has_tests: false,
            has_docs: false,
            vcs_root: None,
            worktree_root: None,
        };
        pm.set_current(info);
        assert!(pm.current().is_some());
    }

    #[test]
    fn test_project_is_rust() {
        let mut pm = ProjectManager::new();
        pm.set_current(ProjectInfo {
            root: PathBuf::from("/test"),
            name: "test".to_string(),
            language: "rust".to_string(),
            has_git: false,
            has_tests: false,
            has_docs: false,
            vcs_root: None,
            worktree_root: None,
        });
        assert!(pm.is_rust());
        assert!(!pm.is_typescript());
    }

    #[test]
    fn test_project_detect_worktree() {
        let tmp = TempDir::new().unwrap();
        let git_file = tmp.path().join(".git");
        let main_repo_git = tmp.path().join("main-repo").join(".git");
        let worktree_ref_path = main_repo_git.join("worktrees").join("feature-branch");
        std::fs::create_dir_all(&worktree_ref_path).unwrap();
        std::fs::write(
            &git_file,
            format!("gitdir: {}", worktree_ref_path.to_string_lossy()),
        )
        .unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(info.has_git);
        assert!(info.worktree_root.is_some());
        assert_eq!(info.worktree_root.unwrap(), tmp.path().join("main-repo"));
    }

    #[test]
    fn test_project_detect_git() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(info.has_git);
        assert!(info.worktree_root.is_none());
    }

    #[test]
    fn test_project_detect_regular_git() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(info.has_git);
        assert!(info.worktree_root.is_none());
    }

    #[test]
    fn test_project_detect_no_git() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(!info.has_git);
        assert!(info.worktree_root.is_none());
    }

    #[test]
    fn test_vcs_detect_worktree_from_subdirectory() {
        let tmp = TempDir::new().unwrap();
        let git_file = tmp.path().join(".git");
        let main_repo_git = tmp.path().join("main-repo").join(".git");
        let worktree_ref_path = main_repo_git.join("worktrees").join("feature-branch");
        std::fs::create_dir_all(&worktree_ref_path).unwrap();
        std::fs::write(
            &git_file,
            format!("gitdir: {}", worktree_ref_path.to_string_lossy()),
        )
        .unwrap();

        let subdir = tmp.path().join("src").join("components");
        std::fs::create_dir_all(&subdir).unwrap();

        let info = ProjectManager::detect(subdir.clone()).unwrap();
        assert!(info.has_git);
        assert!(info.worktree_root.is_some());
        assert_eq!(info.worktree_root.unwrap(), tmp.path().join("main-repo"));
        assert_eq!(info.root, subdir.canonicalize().unwrap());
    }

    #[test]
    fn test_vcs_detect_regular_git_from_subdirectory() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let subdir = tmp.path().join("src").join("components");
        std::fs::create_dir_all(&subdir).unwrap();

        let info = ProjectManager::detect(subdir.clone()).unwrap();
        assert!(info.has_git);
        assert!(info.worktree_root.is_none());
        assert_eq!(info.root, subdir.canonicalize().unwrap());
    }

    #[test]
    fn test_vcs_project_root_and_worktree_root_both_accessible() {
        let tmp = TempDir::new().unwrap();
        let git_file = tmp.path().join(".git");
        let main_repo_git = tmp.path().join("main-repo").join(".git");
        let worktree_ref_path = main_repo_git.join("worktrees").join("feature-branch");
        std::fs::create_dir_all(&worktree_ref_path).unwrap();
        std::fs::write(
            &git_file,
            format!("gitdir: {}", worktree_ref_path.to_string_lossy()),
        )
        .unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(info.has_git);
        assert!(info.worktree_root.is_some());
        assert_eq!(info.root, tmp.path().canonicalize().unwrap());
        assert_eq!(info.worktree_root.unwrap(), tmp.path().join("main-repo"));

        std::fs::create_dir(tmp.path().join("src")).unwrap();
        let subdir_info = ProjectManager::detect(tmp.path().join("src")).unwrap();
        assert_eq!(
            subdir_info.root,
            tmp.path().join("src").canonicalize().unwrap()
        );
        assert_eq!(
            subdir_info.worktree_root.unwrap(),
            tmp.path().join("main-repo")
        );
    }

    #[test]
    fn test_normalize_path_absolute() {
        let tmp = TempDir::new().unwrap();
        let result = normalize_path(&tmp.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_normalize_path_relative() {
        let tmp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let result = normalize_path(&PathBuf::from("."));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().canonicalize().unwrap());
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_normalize_path_symlink() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target");
        std::fs::create_dir(&target).unwrap();
        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = normalize_path(&link);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), target.canonicalize().unwrap());
    }

    #[test]
    fn test_validate_workspace_valid() {
        let tmp = TempDir::new().unwrap();
        let result = validate_workspace(&tmp.path().to_path_buf());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn test_validate_workspace_nonexistent() {
        let result = validate_workspace(&PathBuf::from("/nonexistent/path"));
        assert!(result.is_err());
        match result.unwrap_err() {
            WorkspaceValidationError::PathNotFound(_) => {}
            e => panic!("Expected PathNotFound, got: {}", e),
        }
    }

    #[test]
    fn test_validate_workspace_not_directory() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("file.txt");
        std::fs::write(&file, "").unwrap();

        let result = validate_workspace(&file);
        assert!(result.is_err());
        match result.unwrap_err() {
            WorkspaceValidationError::PathNotDirectory(_) => {}
            e => panic!("Expected PathNotDirectory, got: {}", e),
        }
    }

    #[test]
    fn test_validate_workspace_relative_path() {
        let tmp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        let result = validate_workspace(&PathBuf::from("."));
        assert!(result.is_err());
        match result.unwrap_err() {
            WorkspaceValidationError::PathNotAbsolute(_) => {}
            e => panic!("Expected PathNotAbsolute, got: {}", e),
        }
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_validate_workspace_symlink_to_dir() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target");
        std::fs::create_dir(&target).unwrap();
        let link = tmp.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let result = validate_workspace(&link);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), target.canonicalize().unwrap());
    }

    #[test]
    fn test_is_absolute_path() {
        assert!(is_absolute_path(&PathBuf::from("/usr/local")));
        assert!(!is_absolute_path(&PathBuf::from("relative/path")));
        assert!(!is_absolute_path(&PathBuf::from("./current")));
        assert!(!is_absolute_path(&PathBuf::from("../parent")));
    }

    #[test]
    fn test_resolve_relative_path() {
        let tmp = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let resolved = resolve_relative_path(&PathBuf::from("subdir")).unwrap();
        assert!(resolved.is_absolute());
        assert!(resolved.to_string_lossy().ends_with("subdir"));

        let absolute = PathBuf::from("/usr/local");
        let resolved = resolve_relative_path(&absolute).unwrap();
        assert_eq!(resolved, absolute);

        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_validate_workspace_absolute_required() {
        let result = validate_workspace(&PathBuf::from("relative/path"));
        assert!(result.is_err());
        match result.unwrap_err() {
            WorkspaceValidationError::PathNotAbsolute(_) => {}
            e => panic!("Expected PathNotAbsolute, got: {}", e),
        }
    }

    #[test]
    fn test_validate_workspace_with_allowed_roots_valid() {
        let tmp = TempDir::new().unwrap();
        let allowed = vec![tmp.path().to_path_buf()];
        let result = validate_workspace_with_allowed_roots(&tmp.path().to_path_buf(), &allowed);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_workspace_with_allowed_roots_invalid() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();
        let allowed = vec![tmp1.path().to_path_buf()];
        let result = validate_workspace_with_allowed_roots(&tmp2.path().to_path_buf(), &allowed);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_path_symlink_to_outside() {
        let workspace = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let link = workspace.path().join("link_to_outside");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside.path(), &link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&outside.path(), &link).unwrap();

        let result = normalize_path_with_context(&link, Some(&workspace.path().to_path_buf()));
        assert!(result.is_err());
    }

    #[test]
    fn test_workspace_error_codes() {
        let tmp = TempDir::new().unwrap();
        let non_existent = PathBuf::from("/nonexistent/path");

        assert_eq!(
            WorkspaceValidationError::PathNotFound("x".into()).code(),
            7011
        );
        assert_eq!(
            WorkspaceValidationError::PathNotAccessible("x".into()).code(),
            7012
        );
        assert_eq!(
            WorkspaceValidationError::PathNotDirectory("x".into()).code(),
            7013
        );
        assert_eq!(
            WorkspaceValidationError::PathNotReadable("x".into()).code(),
            7014
        );
        assert_eq!(
            WorkspaceValidationError::PathPermissionDenied("x".into()).code(),
            7015
        );
        assert_eq!(
            WorkspaceValidationError::PathCircularSymlink("x".into()).code(),
            7016
        );
        assert_eq!(
            WorkspaceValidationError::PathTraversalDetected("x".into()).code(),
            7017
        );
        assert_eq!(
            WorkspaceValidationError::PathNotAbsolute("x".into()).code(),
            7018
        );

        assert_eq!(validate_workspace(&non_existent).unwrap_err().code(), 7011);
    }

    #[test]
    fn test_project_type_enum_variants() {
        assert_eq!(ProjectType::Node, ProjectType::Node);
        assert_eq!(ProjectType::Rust, ProjectType::Rust);
        assert_eq!(ProjectType::Python, ProjectType::Python);
        assert_eq!(ProjectType::Go, ProjectType::Go);
        assert_eq!(ProjectType::Java, ProjectType::Java);
        assert_eq!(ProjectType::Cpp, ProjectType::Cpp);
        assert_eq!(ProjectType::Ruby, ProjectType::Ruby);
        assert_eq!(ProjectType::Php, ProjectType::Php);
        assert_eq!(ProjectType::Dotnet, ProjectType::Dotnet);
        assert_eq!(ProjectType::Swift, ProjectType::Swift);
        assert_eq!(ProjectType::Unknown, ProjectType::Unknown);
    }

    #[test]
    fn test_project_type_serialize_to_lowercase() {
        let rust = ProjectType::Rust;
        let json = serde_json::to_string(&rust).unwrap();
        assert_eq!(json, "\"rust\"");
    }

    #[test]
    fn test_project_type_deserialize_from_unknown() {
        let unknown: ProjectType = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(unknown, ProjectType::Unknown);
    }

    #[test]
    fn test_project_type_derives() {
        fn assert_debug<T: std::fmt::Debug>() {}
        fn assert_clone<T: Clone>() {}
        fn assert_copy<T: Copy>() {}
        fn assert_partial_eq<T: PartialEq>() {}
        fn assert_eq<T: Eq>() {}
        fn assert_serialize<T: serde::Serialize>() {}
        fn assert_deserialize<T: for<'de> serde::Deserialize<'de>>() {}

        assert_debug::<ProjectType>();
        assert_clone::<ProjectType>();
        assert_copy::<ProjectType>();
        assert_partial_eq::<ProjectType>();
        assert_eq::<ProjectType>();
        assert_serialize::<ProjectType>();
        assert_deserialize::<ProjectType>();
    }

    #[test]
    fn test_package_manager_enum_variants() {
        assert_eq!(PackageManager::Npm, PackageManager::Npm);
        assert_eq!(PackageManager::Yarn, PackageManager::Yarn);
        assert_eq!(PackageManager::Pnpm, PackageManager::Pnpm);
        assert_eq!(PackageManager::Bun, PackageManager::Bun);
        assert_eq!(PackageManager::Cargo, PackageManager::Cargo);
        assert_eq!(PackageManager::Pip, PackageManager::Pip);
        assert_eq!(PackageManager::Poetry, PackageManager::Poetry);
        assert_eq!(PackageManager::Go, PackageManager::Go);
        assert_eq!(PackageManager::Maven, PackageManager::Maven);
        assert_eq!(PackageManager::Gradle, PackageManager::Gradle);
        assert_eq!(PackageManager::Unknown, PackageManager::Unknown);
    }

    #[test]
    fn test_package_manager_pnpm_serialization() {
        let pnpm = PackageManager::Pnpm;
        let json = serde_json::to_string(&pnpm).unwrap();
        assert_eq!(json, "\"pnpm\"");
    }

    #[test]
    fn test_package_manager_cargo_deserialization() {
        let cargo: PackageManager = serde_json::from_str("\"cargo\"").unwrap();
        assert_eq!(cargo, PackageManager::Cargo);
    }

    #[test]
    fn test_package_manager_derives() {
        fn assert_debug<T: std::fmt::Debug>() {}
        fn assert_clone<T: Clone>() {}
        fn assert_copy<T: Copy>() {}
        fn assert_partial_eq<T: PartialEq>() {}
        fn assert_eq<T: Eq>() {}
        fn assert_serialize<T: serde::Serialize>() {}
        fn assert_deserialize<T: for<'de> serde::Deserialize<'de>>() {}

        assert_debug::<PackageManager>();
        assert_clone::<PackageManager>();
        assert_copy::<PackageManager>();
        assert_partial_eq::<PackageManager>();
        assert_eq::<PackageManager>();
        assert_serialize::<PackageManager>();
        assert_deserialize::<PackageManager>();
    }

    #[test]
    fn test_project_error_display_not_found() {
        let err = ProjectError::NotFound(PathBuf::from("/some/path"));
        assert_eq!(err.to_string(), "No project found from: /some/path");
    }

    #[test]
    fn test_project_error_display_read_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ProjectError::ReadError(PathBuf::from("/some/file"), io_err);
        assert_eq!(err.to_string(), "Failed to read project file: /some/file");
    }

    #[test]
    fn test_project_error_display_parse_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let err = ProjectError::ParseError("package.json".to_string(), json_err);
        let msg = err.to_string();
        assert!(msg.contains("Failed to parse config:"));
        assert!(msg.contains("package.json"));
    }

    #[test]
    fn test_project_error_display_ambiguous() {
        let err = ProjectError::Ambiguous;
        assert_eq!(err.to_string(), "Ambiguous project: multiple roots found");
    }

    #[test]
    fn test_project_error_implements_error_trait() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ProjectError>();
    }

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default();
        assert!(config.package_json.is_none());
        assert!(config.cargo_toml.is_none());
        assert!(config.start_command.is_none());
        assert!(config.install_command.is_none());
    }

    #[test]
    fn test_project_config_serialization_roundtrip() {
        let config = ProjectConfig {
            package_json: Some(serde_json::json!({"name": "test-project", "version": "1.0.0"})),
            cargo_toml: Some("[package]\nname = \"test\"".to_string()),
            start_command: Some("npm start".to_string()),
            install_command: Some("npm install".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProjectConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.package_json.as_ref().unwrap()["name"], "test-project");
        assert_eq!(deserialized.cargo_toml.as_ref().unwrap(), "[package]\nname = \"test\"");
        assert_eq!(deserialized.start_command.as_ref().unwrap(), "npm start");
        assert_eq!(deserialized.install_command.as_ref().unwrap(), "npm install");
    }

    #[test]
    fn test_project_config_clone() {
        let config = ProjectConfig {
            package_json: Some(serde_json::json!({"name": "clone-test"})),
            cargo_toml: Some("cargo_toml".to_string()),
            start_command: Some("start".to_string()),
            install_command: Some("install".to_string()),
        };
        let cloned = config.clone();
        assert_eq!(cloned.package_json, config.package_json);
        assert_eq!(cloned.cargo_toml, config.cargo_toml);
        assert_eq!(cloned.start_command, config.start_command);
        assert_eq!(cloned.install_command, config.install_command);
    }

    #[test]
    fn test_project_config_debug() {
        let config = ProjectConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ProjectConfig"));
    }

    #[test]
    fn test_project_config_derives() {
        fn assert_debug<T: std::fmt::Debug>() {}
        fn assert_clone<T: Clone>() {}
        fn assert_default<T: Default>() {}
        fn assert_serialize<T: serde::Serialize>() {}
        fn assert_deserialize<T: for<'de> serde::Deserialize<'de>>() {}

        assert_debug::<ProjectConfig>();
        assert_clone::<ProjectConfig>();
        assert_default::<ProjectConfig>();
        assert_serialize::<ProjectConfig>();
        assert_deserialize::<ProjectConfig>();
    }
}
