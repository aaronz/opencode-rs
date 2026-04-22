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
    pub name: Option<String>,
    pub project_type: ProjectType,
    pub package_manager: PackageManager,
    pub languages: Vec<String>,
    pub is_monorepo: bool,
    pub is_worktree: bool,
    pub config: ProjectConfig,
    pub vcs_root: Option<PathBuf>,
}

impl Default for ProjectInfo {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            name: None,
            project_type: ProjectType::Unknown,
            package_manager: PackageManager::Unknown,
            languages: Vec::new(),
            is_monorepo: false,
            is_worktree: false,
            config: ProjectConfig::default(),
            vcs_root: None,
        }
    }
}

use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::RwLock;
use opencode_config::Config;

#[derive(Debug, Clone)]
pub struct ConfigService {
    inner: Arc<RwLock<Config>>,
}

impl ConfigService {
    pub fn new(config: Arc<RwLock<Config>>) -> Self {
        Self { inner: config }
    }

    pub async fn get_config(&self) -> Config {
        self.inner.read().await.clone()
    }

    pub fn get_config_sync(&self) -> Arc<RwLock<Config>> {
        self.inner.clone()
    }
}

pub struct ProjectService {
    cache: Arc<Mutex<Option<ProjectInfo>>>,
    #[allow(dead_code)]
    config: Arc<ConfigService>,
}

unsafe impl Send for ProjectService {}
unsafe impl Sync for ProjectService {}

impl ProjectService {
    pub fn new(config: Arc<ConfigService>) -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            config,
        }
    }

    pub async fn detect(&self, cwd: Option<&Path>) -> Result<ProjectInfo, ProjectError> {
        let start_path = match cwd {
            Some(p) => p.to_path_buf(),
            None => std::env::current_dir().map_err(|e| ProjectError::ReadError(PathBuf::from("."), e))?,
        };

        let root = self.find_root(&start_path).await?;
        let project_info = self.detect_project_info(&root).await?;
        Ok(project_info)
    }

    async fn find_root(&self, start: &Path) -> Result<PathBuf, ProjectError> {
        let mut current = start.to_path_buf();
        let mut visited = std::collections::HashSet::new();

        loop {
            if !visited.insert(current.clone()) {
                return Ok(current);
            }

            let git_path = current.join(".git");
            let opencode_path = current.join(".opencode");

            if let Ok(metadata) = tokio::fs::symlink_metadata(&git_path).await {
                if metadata.is_dir() || metadata.is_file() {
                    return Ok(current);
                }
            }

            if tokio::fs::symlink_metadata(&opencode_path).await.is_ok() {
                return Ok(current);
            }

            if !current.pop() {
                break;
            }
        }

        Ok(start.to_path_buf())
    }

    async fn detect_project_info(&self, root: &Path) -> Result<ProjectInfo, ProjectError> {
        let project_type = self.detect_project_type(root);
        let package_manager = self.detect_package_manager(root, project_type);
        let is_worktree = self.check_is_worktree(root).await;
        let vcs_root = self.find_vcs_root(root).await;
        let name = self.extract_project_name(root, project_type).await;
        let languages = Vec::new();
        let is_monorepo = false;
        let config = ProjectConfig::default();

        Ok(ProjectInfo {
            root: root.to_path_buf(),
            name,
            project_type,
            package_manager,
            languages,
            is_monorepo,
            is_worktree,
            config,
            vcs_root,
        })
    }

    fn detect_project_type(&self, root: &Path) -> ProjectType {
        if root.join("Cargo.toml").exists() {
            return ProjectType::Rust;
        }
        if root.join("go.mod").exists() {
            return ProjectType::Go;
        }
        if root.join("pyproject.toml").exists()
            || root.join("requirements.txt").exists()
            || root.join("setup.py").exists()
        {
            return ProjectType::Python;
        }
        if root.join("package.json").exists() {
            return ProjectType::Node;
        }
        if root.join("pom.xml").exists() || root.join("build.gradle").exists() {
            return ProjectType::Java;
        }
        if root.join("CMakeLists.txt").exists()
            || root.join("Makefile").exists()
            || root.join("compile_commands.json").exists()
        {
            return ProjectType::Cpp;
        }
        if root.join("Gemfile").exists() {
            return ProjectType::Ruby;
        }
        if root.join("composer.json").exists() {
            return ProjectType::Php;
        }

        for entry in std::fs::read_dir(root).ok().into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".csproj") || name.ends_with(".sln") {
                return ProjectType::Dotnet;
            }
        }

        if root.join("Package.swift").exists() {
            return ProjectType::Swift;
        }

        ProjectType::Unknown
    }

    fn detect_package_manager(&self, root: &Path, project_type: ProjectType) -> PackageManager {
        match project_type {
            ProjectType::Node => {
                if root.join("pnpm-lock.yaml").exists() {
                    return PackageManager::Pnpm;
                }
                if root.join("yarn.lock").exists() && !root.join("package-lock.json").exists() {
                    return PackageManager::Yarn;
                }
                if root.join("bun.lockb").exists() {
                    return PackageManager::Bun;
                }
                if root.join("package-lock.json").exists() {
                    return PackageManager::Npm;
                }
                PackageManager::Npm
            }
            ProjectType::Rust => PackageManager::Cargo,
            ProjectType::Go => PackageManager::Go,
            ProjectType::Python => {
                if root.join("pyproject.toml").exists() {
                    PackageManager::Poetry
                } else {
                    PackageManager::Pip
                }
            }
            ProjectType::Java => {
                if root.join("pom.xml").exists() {
                    PackageManager::Maven
                } else {
                    PackageManager::Gradle
                }
            }
            _ => PackageManager::Unknown,
        }
    }

    async fn check_is_worktree(&self, path: &Path) -> bool {
        let data_dir = std::env::var("DATA").ok();
        if let Some(data) = data_dir {
            let worktree_path = PathBuf::from(data).join("worktree");
            let worktree_str = worktree_path.to_string_lossy().into_owned();
            let path_str = path.to_string_lossy().into_owned();
            return path_str.starts_with(&worktree_str);
        }
        false
    }

    async fn find_vcs_root(&self, start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();

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

    async fn extract_project_name(&self, root: &Path, project_type: ProjectType) -> Option<String> {
        match project_type {
            ProjectType::Node => {
                let pkg_path = root.join("package.json");
                if let Ok(content) = tokio::fs::read_to_string(&pkg_path).await {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        return json.get("name").and_then(|n| n.as_str()).map(String::from);
                    }
                }
            }
            ProjectType::Rust => {
                let cargo_path = root.join("Cargo.toml");
                if let Ok(content) = tokio::fs::read_to_string(&cargo_path).await {
                    if let Ok(toml) = content.parse::<toml::Value>() {
                        return toml.get("package")
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                            .map(String::from);
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub async fn get(&self) -> Result<ProjectInfo, ProjectError> {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(info) = cache.as_ref() {
                return Ok(info.clone());
            }
        }

        let info = self.detect(None).await?;

        {
            let mut cache = self.cache.lock().unwrap();
            *cache = Some(info.clone());
        }

        Ok(info)
    }

    pub fn invalidate(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = None;
    }

    pub async fn is_worktree(&self, path: &Path) -> bool {
        let data_dir = std::env::var("DATA").ok();
        if let Some(data) = data_dir {
            let worktree_path = PathBuf::from(data).join("worktree");
            let worktree_str = worktree_path.to_string_lossy().into_owned();
            let path_str = path.to_string_lossy().into_owned();
            return path_str.starts_with(&worktree_str);
        }
        false
    }

    pub async fn root(&self) -> Result<PathBuf, ProjectError> {
        let info = self.get().await?;
        Ok(info.root)
    }
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
        let is_worktree = Self::is_worktree_path(&validated);

        let project_type = if validated.join("Cargo.toml").exists() {
            ProjectType::Rust
        } else if validated.join("go.mod").exists() {
            ProjectType::Go
        } else if validated.join("pyproject.toml").exists()
            || validated.join("requirements.txt").exists()
            || validated.join("setup.py").exists()
        {
            ProjectType::Python
        } else if validated.join("package.json").exists() {
            ProjectType::Node
        } else if validated.join("pom.xml").exists() || validated.join("build.gradle").exists() {
            ProjectType::Java
        } else if validated.join("CMakeLists.txt").exists()
            || validated.join("Makefile").exists()
            || validated.join("compile_commands.json").exists()
        {
            ProjectType::Cpp
        } else if validated.join("Gemfile").exists() {
            ProjectType::Ruby
        } else if validated.join("composer.json").exists() {
            ProjectType::Php
        } else if validated.join("Package.swift").exists() {
            ProjectType::Swift
        } else {
            ProjectType::Unknown
        };

        let package_manager = match project_type {
            ProjectType::Node => {
                if validated.join("pnpm-lock.yaml").exists() {
                    PackageManager::Pnpm
                } else if validated.join("yarn.lock").exists()
                    && !validated.join("package-lock.json").exists()
                {
                    PackageManager::Yarn
                } else if validated.join("bun.lockb").exists() {
                    PackageManager::Bun
                } else {
                    PackageManager::Npm
                }
            }
            ProjectType::Rust => PackageManager::Cargo,
            ProjectType::Go => PackageManager::Go,
            ProjectType::Python => PackageManager::Pip,
            ProjectType::Java => PackageManager::Maven,
            _ => PackageManager::Unknown,
        };

        let name = validated
            .file_name()
            .map(|n| n.to_string_lossy().to_string());

        Some(ProjectInfo {
            root: validated,
            name,
            project_type,
            package_manager,
            languages: Vec::new(),
            is_monorepo: false,
            is_worktree,
            config: ProjectConfig::default(),
            vcs_root,
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
            .map(|p| p.project_type == ProjectType::Rust)
            .unwrap_or(false)
    }

    pub(crate) fn is_typescript(&self) -> bool {
        self.current
            .as_ref()
            .map(|p| p.project_type == ProjectType::Node)
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
        assert_eq!(info.project_type, ProjectType::Rust);
    }

    #[test]
    fn test_project_detect_javascript() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("package.json"), "{}").unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert_eq!(info.project_type, ProjectType::Node);
    }

    #[test]
    fn test_project_detect_nonexistent() {
        let info = ProjectManager::detect(PathBuf::from("/nonexistent/path"));
        assert!(info.is_none());
    }

    #[test]
    fn test_project_info_default() {
        let info = ProjectInfo::default();
        assert!(info.root.as_os_str().is_empty());
        assert!(info.name.is_none());
        assert_eq!(info.project_type, ProjectType::Unknown);
        assert_eq!(info.package_manager, PackageManager::Unknown);
        assert!(info.languages.is_empty());
        assert!(!info.is_monorepo);
        assert!(!info.is_worktree);
        assert!(info.config.package_json.is_none());
        assert!(info.config.cargo_toml.is_none());
        assert!(info.vcs_root.is_none());
    }

    #[test]
    fn test_project_info_serialization() {
        let info = ProjectInfo {
            root: PathBuf::from("/test"),
            name: Some("test-project".to_string()),
            project_type: ProjectType::Rust,
            package_manager: PackageManager::Cargo,
            languages: vec!["rust".to_string()],
            is_monorepo: false,
            is_worktree: false,
            config: ProjectConfig::default(),
            vcs_root: Some(PathBuf::from("/test")),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: ProjectInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.root, info.root);
        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.project_type, info.project_type);
        assert_eq!(deserialized.package_manager, info.package_manager);
        assert_eq!(deserialized.languages, info.languages);
        assert_eq!(deserialized.is_monorepo, info.is_monorepo);
        assert_eq!(deserialized.is_worktree, info.is_worktree);
        assert_eq!(deserialized.vcs_root, info.vcs_root);
    }

    #[test]
    fn test_project_info_is_monorepo_default_false() {
        let info = ProjectInfo::default();
        assert!(!info.is_monorepo);
    }

    #[test]
    fn test_project_info_is_worktree_default_false() {
        let info = ProjectInfo::default();
        assert!(!info.is_worktree);
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
        assert!(info.vcs_root.is_some());
        assert!(info.is_worktree);
    }

    #[test]
    fn test_project_detect_git() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(info.vcs_root.is_some());
        assert!(!info.is_worktree);
    }

    #[test]
    fn test_project_detect_regular_git() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(info.vcs_root.is_some());
        assert!(!info.is_worktree);
    }

    #[test]
    fn test_project_detect_no_git() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();

        let info = ProjectManager::detect(tmp.path().to_path_buf()).unwrap();
        assert!(info.vcs_root.is_none());
        assert!(!info.is_worktree);
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
        assert!(info.vcs_root.is_some());
        assert!(info.is_worktree);
        assert_eq!(info.root, subdir.canonicalize().unwrap());
    }

    #[test]
    fn test_vcs_detect_regular_git_from_subdirectory() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join(".git")).unwrap();

        let subdir = tmp.path().join("src").join("components");
        std::fs::create_dir_all(&subdir).unwrap();

        let info = ProjectManager::detect(subdir.clone()).unwrap();
        assert!(info.vcs_root.is_some());
        assert!(!info.is_worktree);
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
        assert!(info.vcs_root.is_some());
        assert!(info.is_worktree);
        assert_eq!(info.root, tmp.path().canonicalize().unwrap());

        std::fs::create_dir(tmp.path().join("src")).unwrap();
        let subdir_info = ProjectManager::detect(tmp.path().join("src")).unwrap();
        assert_eq!(
            subdir_info.root,
            tmp.path().join("src").canonicalize().unwrap()
        );
        assert!(subdir_info.is_worktree);
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

    #[tokio::test]
    async fn test_project_service_new_creates_instance_with_empty_cache() {
        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let cache = service.cache.lock().unwrap();
        assert!(cache.is_none());
    }

    #[tokio::test]
    async fn test_project_service_get_returns_cached_value_on_second_call() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let original_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();

        let info1 = service.get().await.unwrap();
        assert_eq!(info1.project_type, ProjectType::Rust);

        let info2 = service.get().await.unwrap();
        assert_eq!(info2.project_type, ProjectType::Rust);
        assert_eq!(info1.root, info2.root);

        std::env::set_current_dir(original_cwd).unwrap();
    }

    #[tokio::test]
    async fn test_project_service_invalidate_clears_the_cache() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let info1 = service.detect(Some(tmp.path())).await.unwrap();
        assert_eq!(info1.project_type, ProjectType::Rust);

        service.invalidate();

        let cache = service.cache.lock().unwrap();
        assert!(cache.is_none());
    }

    #[tokio::test]
    async fn test_project_service_is_worktree_returns_correct_boolean() {
        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        std::env::set_var("DATA", "/tmp/test_data");
        let worktree_path = PathBuf::from("/tmp/test_data/worktree/project-123");
        let is_wt = service.is_worktree(&worktree_path).await;
        assert!(is_wt);

        let normal_path = PathBuf::from("/home/user/project/src");
        let is_normal = service.is_worktree(&normal_path).await;
        assert!(!is_normal);

        std::env::remove_var("DATA");
    }

    #[tokio::test]
    async fn test_project_service_root_returns_pathbuf() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let root = service.root().await.unwrap();
        assert!(root.is_absolute());
    }

    #[test]
    fn test_project_service_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ProjectService>();
        assert_sync::<ProjectService>();
    }

    #[test]
    fn test_config_service_debug() {
        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let debug_str = format!("{:?}", config_service);
        assert!(debug_str.contains("ConfigService"));
    }

    #[tokio::test]
    async fn test_config_service_get_config() {
        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config.clone());
        let retrieved = config_service.get_config().await;
        assert!(retrieved.schema.is_none());
    }

    #[tokio::test]
    async fn test_config_service_get_config_sync() {
        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config.clone());
        let retrieved = config_service.get_config_sync();
        Arc::ptr_eq(&retrieved, &config);
    }

    #[tokio::test]
    async fn test_find_root_walks_up() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("src").join("deep");
        tokio::fs::create_dir_all(&sub).await.unwrap();
        tokio::fs::write(tmp.path().join(".git"), b"").await.unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let info = service.detect(Some(&sub)).await.unwrap();
        assert_eq!(info.root, tmp.path());
    }

    #[tokio::test]
    async fn test_find_root_walks_up_git_directory() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("src").join("components");
        tokio::fs::create_dir_all(&sub).await.unwrap();
        tokio::fs::create_dir(tmp.path().join(".git")).await.unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let info = service.detect(Some(&sub)).await.unwrap();
        assert_eq!(info.root, tmp.path());
    }

    #[tokio::test]
    async fn test_find_root_walks_up_git_file_worktree_reference() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("src").join("components");
        tokio::fs::create_dir_all(&sub).await.unwrap();

        let main_repo_git = tmp.path().join("main-repo").join(".git");
        let worktree_ref_path = main_repo_git.join("worktrees").join("feature-branch");
        tokio::fs::create_dir_all(&worktree_ref_path).await.unwrap();
        tokio::fs::write(
            tmp.path().join(".git"),
            format!("gitdir: {}", worktree_ref_path.to_string_lossy()),
        )
        .await
        .unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let info = service.detect(Some(&sub)).await.unwrap();
        assert_eq!(info.root, tmp.path());
    }

    #[tokio::test]
    async fn test_find_root_walks_up_opencode_directory() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("src").join("deep");
        tokio::fs::create_dir_all(&sub).await.unwrap();
        tokio::fs::create_dir(tmp.path().join(".opencode")).await.unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let info = service.detect(Some(&sub)).await.unwrap();
        assert_eq!(info.root, tmp.path());
    }

    #[tokio::test]
    async fn test_find_root_returns_cwd_fallback_when_no_markers() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("src").join("nested");
        tokio::fs::create_dir_all(&sub).await.unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let info = service.detect(Some(&sub)).await.unwrap();
        assert_eq!(info.root, sub);
        assert_eq!(info.project_type, ProjectType::Unknown);
    }

    #[tokio::test]
    async fn test_find_root_terminates_without_infinite_loop() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::create_dir(tmp.path().join(".opencode")).await.unwrap();

        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        let service = ProjectService::new(Arc::new(config_service));

        let result = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            service.detect(Some(tmp.path())),
        )
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    fn create_project_service(tmp: &tempfile::TempDir) -> ProjectService {
        let config = Arc::new(RwLock::new(Config::default()));
        let config_service = ConfigService::new(config);
        ProjectService::new(Arc::new(config_service))
    }

    #[tokio::test]
    async fn test_detect_rust_project() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Rust);
        assert_eq!(info.package_manager, PackageManager::Cargo);
    }

    #[tokio::test]
    async fn test_detect_node_project() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Node);
        assert_eq!(info.package_manager, PackageManager::Npm);
    }

    #[tokio::test]
    async fn test_detect_type_priority() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").await.unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Rust);
    }

    #[tokio::test]
    async fn test_detect_unknown_type() {
        let tmp = TempDir::new().unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Unknown);
    }

    #[tokio::test]
    async fn test_detect_go_project() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("go.mod"), "module test").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Go);
        assert_eq!(info.package_manager, PackageManager::Go);
    }

    #[tokio::test]
    async fn test_detect_python_project_pyproject() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("pyproject.toml"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Python);
        assert_eq!(info.package_manager, PackageManager::Poetry);
    }

    #[tokio::test]
    async fn test_detect_python_project_requirements() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("requirements.txt"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Python);
        assert_eq!(info.package_manager, PackageManager::Pip);
    }

    #[tokio::test]
    async fn test_detect_java_project_maven() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("pom.xml"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Java);
        assert_eq!(info.package_manager, PackageManager::Maven);
    }

    #[tokio::test]
    async fn test_detect_java_project_gradle() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("build.gradle"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Java);
        assert_eq!(info.package_manager, PackageManager::Gradle);
    }

    #[tokio::test]
    async fn test_detect_cpp_project_cmake() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("CMakeLists.txt"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Cpp);
    }

    #[tokio::test]
    async fn test_detect_cpp_project_makefile() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Makefile"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Cpp);
    }

    #[tokio::test]
    async fn test_detect_ruby_project() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Gemfile"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Ruby);
    }

    #[tokio::test]
    async fn test_detect_php_project() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("composer.json"), "{}").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Php);
    }

    #[tokio::test]
    async fn test_detect_dotnet_project_csproj() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("TestProject.csproj"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Dotnet);
    }

    #[tokio::test]
    async fn test_detect_dotnet_project_sln() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Solution.sln"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Dotnet);
    }

    #[tokio::test]
    async fn test_detect_swift_project() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Package.swift"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Swift);
    }

    #[tokio::test]
    async fn test_type_priority_regression_rust_before_go() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Cargo.toml"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("go.mod"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Rust);
    }

    #[tokio::test]
    async fn test_type_priority_regression_rust_before_python() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Cargo.toml"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("pyproject.toml"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Rust);
    }

    #[tokio::test]
    async fn test_type_priority_regression_go_before_python() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("go.mod"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("pyproject.toml"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Go);
    }

    #[tokio::test]
    async fn test_type_priority_regression_go_before_node() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("go.mod"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Go);
    }

    #[tokio::test]
    async fn test_type_priority_regression_python_before_node() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("pyproject.toml"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Python);
    }

    #[tokio::test]
    async fn test_type_priority_regression_node_before_java() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("pom.xml"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Node);
    }

    #[tokio::test]
    async fn test_type_priority_regression_java_before_cpp() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("pom.xml"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("CMakeLists.txt"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Java);
    }

    #[tokio::test]
    async fn test_type_priority_regression_cpp_before_ruby() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("CMakeLists.txt"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("Gemfile"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Cpp);
    }

    #[tokio::test]
    async fn test_type_priority_regression_ruby_before_php() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Gemfile"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("composer.json"), "{}").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Ruby);
    }

    #[tokio::test]
    async fn test_type_priority_regression_php_before_dotnet() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("composer.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("Test.csproj"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Php);
    }

    #[tokio::test]
    async fn test_type_priority_regression_dotnet_before_swift() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Test.csproj"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("Package.swift"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Dotnet);
    }

    #[tokio::test]
    async fn test_type_priority_all_markers_rust_wins() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("Cargo.toml"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("go.mod"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("pyproject.toml"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("pom.xml"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("CMakeLists.txt"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("Gemfile"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("composer.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("Test.csproj"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("Package.swift"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Rust);
    }

    #[tokio::test]
    async fn test_detect_node_project_with_pnpm() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("pnpm-lock.yaml"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Node);
        assert_eq!(info.package_manager, PackageManager::Pnpm);
    }

    #[tokio::test]
    async fn test_detect_node_project_with_yarn() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("yarn.lock"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Node);
        assert_eq!(info.package_manager, PackageManager::Yarn);
    }

    #[tokio::test]
    async fn test_detect_node_project_with_bun() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("bun.lockb"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Node);
        assert_eq!(info.package_manager, PackageManager::Bun);
    }

    #[tokio::test]
    async fn test_yarn_not_selected_when_package_lock_exists() {
        let tmp = TempDir::new().unwrap();
        tokio::fs::write(tmp.path().join("package.json"), "{}").await.unwrap();
        tokio::fs::write(tmp.path().join("yarn.lock"), "").await.unwrap();
        tokio::fs::write(tmp.path().join("package-lock.json"), "").await.unwrap();

        let service = create_project_service(&tmp);
        let info = service.detect(Some(tmp.path())).await.unwrap();

        assert_eq!(info.project_type, ProjectType::Node);
        assert_eq!(info.package_manager, PackageManager::Npm);
    }
}
