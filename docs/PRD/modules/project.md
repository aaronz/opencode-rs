# PRD: project Module

## Module Overview

- **Module Name**: `project`
- **Source Path**: `packages/opencode/src/project/`
- **Type**: Integration
- **Rust Crate**: `crates/project/` or `crates/core/src/project.rs`
- **Purpose**: Project detection and management. Identifies the project root, project type (Node/Rust/Python/Go), package manager, languages, and provides project-specific context for the agent.

---

## Functionality

### Core Features

1. **Project Root Detection** — Walk up from current directory to find the git/worktree root or project marker file
2. **Project Type Identification** — Detect language ecosystem from marker files
3. **Package Manager Detection** — Identify npm/yarn/pnpm/cargo/pip/poetry
4. **Language Detection** — Scan for source files to determine primary languages
5. **Project Config Reading** — Read and parse project config files
6. **Monorepo Support** — Detect workspace root and member packages
7. **Sandbox Detection** — Identify if current directory is an opencode worktree

---

## Project Types

| Type | Detection File(s) | Package Manager |
|------|-------------------|-----------------|
| `Node.js` | `package.json` | npm, yarn, pnpm, bun |
| `Rust` | `Cargo.toml` | cargo |
| `Python` | `pyproject.toml`, `requirements.txt`, `setup.py` | pip, poetry, pdm |
| `Go` | `go.mod` | go |
| `Java` | `pom.xml`, `build.gradle` | maven, gradle |
| `C++` | `CMakeLists.txt`, `Makefile`, `compile_commands.json` | cmake, make |
| `Ruby` | `Gemfile` | bundle |
| `PHP` | `composer.json` | composer |
| `.NET` | `*.csproj`, `*.sln` | dotnet |
| `Swift` | `Package.swift` | swift |

---

## API Surface

### Types

```rust
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// For Node: contents of package.json
    pub package_json: Option<serde_json::Value>,
    /// For Rust: contents of Cargo.toml
    pub cargo_toml: Option<String>,
    /// Custom start command from project config
    pub start_command: Option<String>,
    /// Custom install command
    pub install_command: Option<String>,
}
```

### Service Interface

```rust
pub struct ProjectService {
    cache: Arc<Mutex<Option<ProjectInfo>>>,
    config: Arc<ConfigService>,
}

impl ProjectService {
    /// Detect project from the given working directory (or current dir)
    pub async fn detect(&self, cwd: Option<&Path>) -> Result<ProjectInfo, ProjectError>

    /// Get cached project info (runs detect if not cached)
    pub async fn get(&self) -> Result<ProjectInfo, ProjectError>

    /// Invalidate cache (useful after mkdir / file changes)
    pub fn invalidate(&self)

    /// Check if a path is inside an opencode worktree sandbox
    pub async fn is_worktree(&self, path: &Path) -> bool

    /// Get the project root (shortcut)
    pub async fn root(&self) -> Result<PathBuf, ProjectError>
}
```

### `ProjectError`

```rust
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
```

---

## Detection Algorithm

### Root Finding

```
1. Start from cwd
2. Check for .git, .git/worktrees, .opencode (worktree marker)
3. If found, return parent (git root) or current (worktree)
4. Walk up one directory, repeat
5. If no root found, return cwd as root (type = Unknown)
```

### Type Detection Priority

```
1. Cargo.toml → Rust
2. go.mod → Go
3. pyproject.toml / requirements.txt / setup.py → Python
4. package.json → Node
5. pom.xml / build.gradle → Java
6. CMakeLists.txt / Makefile / compile_commands.json → Cpp
7. Gemfile → Ruby
8. composer.json → Php
9. *.csproj / *.sln → Dotnet
10. Package.swift → Swift
11. Default → Unknown
```

### Package Manager Detection (for Node projects)

```
1. pnpm-lock.yaml → Pnpm
2. yarn.lock (且无 package-lock.json) → Yarn
3. bun.lockb → Bun
4. package-lock.json → Npm
5. Default → Npm (fallback)
```

---

## Crate Layout

```
crates/project/
├── Cargo.toml
├── src/
│   ├── lib.rs           # ProjectService, ProjectInfo, ProjectError
│   ├── detect.rs        # Detection algorithms
│   ├── config.rs        # Config file parsing
│   └── markers.rs       # Marker file detection
└── tests/
    └── project_tests.rs
```

### `Cargo.toml`

```toml
[package]
name = "opencode-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.45", features = ["fs", "sync"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
tracing = "0.1"
walkdir = "2"

[dev-dependencies]
tempfile = "3"
```

---

## Monorepo Support

```rust
/// Detect if project is a monorepo root
async fn detect_monorepo(root: &Path) -> bool {
    // Node: has "workspaces" in package.json
    // Rust: has members in workspace Cargo.toml
    // Go: has multiple modules
}
```

---

## Dependencies

| Dependency | Purpose |
|---|---|
| `tokio::fs` | Async file reading |
| `walkdir` | Directory walking for language detection |
| `serde_json` | package.json parsing |
| `toml` | Cargo.toml parsing |
| `tracing` | Logging |

---

## Acceptance Criteria

- [ ] `detect()` finds the correct git/worktree root from any subdirectory
- [ ] `detect()` identifies the correct `ProjectType` from marker files
- [ ] `detect()` identifies the correct `PackageManager` for Node projects
- [ ] `detect()` finds the correct package name from `package.json` / `Cargo.toml`
- [ ] `is_worktree()` returns `true` for paths inside `$DATA/worktree/`
- [ ] `detect_monorepo()` returns `true` for workspace roots
- [ ] Cache is invalidated on `invalidate()` call
- [ ] `root()` is a shorthand that returns just the path

---

## Rust Implementation Notes

### Async Detection

```rust
impl ProjectService {
    pub async fn detect(&self, cwd: Option<&Path>) -> Result<ProjectInfo, ProjectError> {
        let cwd = cwd.map(PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
        let root = self.find_root(&cwd).await?;

        // Read config files concurrently
        let (pkg_json, cargo_toml) = tokio::join!(
            self.read_package_json(&root),
            self.read_cargo_toml(&root),
        );

        let project_type = self.detect_type(&root).await;
        let package_manager = self.detect_package_manager(&root, &project_type).await;
        let languages = self.detect_languages(&root).await;
        let is_monorepo = self.detect_monorepo(&root).await;
        let is_worktree = self.is_worktree(&root).await;

        Ok(ProjectInfo { root, name, project_type, package_manager, languages, is_monorepo, is_worktree, config })
    }
}
```

### Config Parsing

```rust
async fn read_package_json(&self, root: &Path) -> Option<serde_json::Value> {
    let path = root.join("package.json");
    let content = tokio::fs::read_to_string(&path).await.ok()?;
    serde_json::from_str(&content).ok()
}

async fn read_cargo_toml(&self, root: &Path) -> Option<String> {
    let path = root.join("Cargo.toml");
    tokio::fs::read_to_string(&path).await.ok()
}
```

---

## Test Design

### Unit Tests

```rust
#[tokio::test]
async fn test_detect_rust_project() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    let svc = ProjectService::new();
    let info = svc.detect(Some(tmp.path())).await.unwrap();
    assert_eq!(info.project_type, ProjectType::Rust);
    assert_eq!(info.package_manager, PackageManager::Cargo);
}

#[tokio::test]
async fn test_detect_node_project_with_pnpm() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("package.json"), "{}").unwrap();
    std::fs::write(tmp.path().join("pnpm-lock.yaml"), "").unwrap();
    let svc = ProjectService::new();
    let info = svc.detect(Some(tmp.path())).await.unwrap();
    assert_eq!(info.project_type, ProjectType::Node);
    assert_eq!(info.package_manager, PackageManager::Pnpm);
}

#[test]
fn test_detect_type_priority() {
    // When both Cargo.toml and package.json exist, Rust wins
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();
    std::fs::write(tmp.path().join("package.json"), "{}").unwrap();
    let info = blocking_detect(tmp.path());
    assert_eq!(info.project_type, ProjectType::Rust);
}

#[tokio::test]
async fn test_find_root_walks_up() {
    let tmp = TempDir::new().unwrap();
    let sub = tmp.path().join("src").join("deep");
    tokio::fs::create_dir_all(&sub).await.unwrap();
    std::fs::write(tmp.path().join(".git"), "").unwrap(); // mark git root

    let svc = ProjectService::new();
    let info = svc.detect(Some(&sub)).await.unwrap();
    assert_eq!(info.root, tmp.path());
}

#[tokio::test]
async fn test_is_worktree_detects_sandbox() {
    let svc = ProjectService::new();
    let is_wt = svc.is_worktree(Path::new("/data/worktree/proj-123/src")).await;
    assert!(is_wt);
}
```

### Integration Tests

| TS Test Pattern | Rust Test |
|-----------------|-----------|
| Type detection from mock directory | `test_detect_rust_project`, `test_detect_node_project_with_pnpm` |
| Priority when multiple markers exist | `test_detect_type_priority` |
| Walk up to find root | `test_find_root_walks_up` |
| Worktree detection | `test_is_worktree_detects_sandbox` |

---

## Source Reference

*Source: `packages/opencode/src/project/index.ts`*
*No existing Rust equivalent — implement in `crates/project/`*
