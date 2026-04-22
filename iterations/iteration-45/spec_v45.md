# Specification: project Module (Iteration 45)

**Document Version**: 45
**Date**: 2026-04-22
**Status**: Updated from Gap Analysis
**Source PRD**: `packages/opencode/src/project/` (TypeScript)
**Implementation**: `opencode-rust/crates/core/src/project.rs`

---

## Module Overview

- **Module Name**: `project`
- **Source Path**: `packages/opencode/src/project/`
- **Rust Crate**: `crates/core/src/project.rs` (embedded in core)
- **Type**: Integration
- **Purpose**: Project detection and management. Identifies the project root, project type (Node/Rust/Python/Go), package manager, languages, and provides project-specific context for the agent.

---

## Implementation Status Summary

| Component | Status | Gap |
|-----------|--------|-----|
| `ProjectType` enum | ❌ Missing | Must define enum with 11 variants |
| `PackageManager` enum | ❌ Missing | Must define enum with 11 variants |
| `ProjectError` enum | ❌ Missing | Different from `WorkspaceValidationError` |
| `ProjectConfig` struct | ❌ Missing | Not implemented |
| `ProjectInfo` struct | ❌ Incompatible | Wrong fields, incompatible with PRD |
| `ProjectService` struct | ❌ Missing | `ProjectManager` exists but wrong API |
| Async `detect()` | ❌ Missing | `ProjectManager::detect()` is sync |
| Walk-up root detection | ⚠️ Partial | Only finds git root, not worktree root |
| Package manager detection | ❌ Missing | No pnpm/yarn/bun/npm detection |
| Monorepo detection | ❌ Missing | Not implemented |
| Language detection (walkdir) | ❌ Missing | Only checks marker files |
| `is_worktree()` sandbox check | ❌ Missing | Only detects git worktrees |
| Cache invalidation | ❌ Missing | No caching mechanism |
| `root()` shorthand | ❌ Missing | Not implemented |

**Overall Progress**: 8% (gap analysis baseline)

---

## Feature Requirements

### FR-001: ProjectType Enum

**Priority**: P0
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Define the `ProjectType` enum with all 11 variants as specified in the PRD.

#### API
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
```

#### Requirements
- [ ] All 11 variants must be defined
- [ ] Must derive `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`
- [ ] Serde serialization must use `lowercase` rename
- [ ] `Unknown` must be the default variant

---

### FR-002: PackageManager Enum

**Priority**: P0
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Define the `PackageManager` enum with all 11 variants for Node/Rust/Python/Go/Java projects.

#### API
```rust
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
```

#### Requirements
- [ ] All 11 variants must be defined
- [ ] Must derive `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`
- [ ] Serde serialization must use `lowercase` rename
- [ ] `Unknown` must be the default/fallback variant

---

### FR-003: ProjectError Enum

**Priority**: P0
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Define the `ProjectError` enum for project detection and parsing errors.

#### API
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

#### Requirements
- [ ] All 4 variants must be defined with proper error messages
- [ ] Must implement `Debug` and `Error` (via `thiserror`)
- [ ] `NotFound` variant must include the starting path
- [ ] `ReadError` must include source `io::Error`
- [ ] `ParseError` must include source `serde_json::Error`

---

### FR-004: ProjectConfig Struct

**Priority**: P1
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Define the `ProjectConfig` struct to hold parsed project configuration files.

#### API
```rust
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

#### Requirements
- [ ] All 4 fields must be defined with correct types
- [ ] Must derive `Debug, Clone, Default, Serialize, Deserialize`
- [ ] `package_json` must be parsed as `serde_json::Value` not raw string

---

### FR-005: ProjectInfo Struct (Updated)

**Priority**: P0
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Replace the existing incompatible `ProjectInfo` struct with the PRD-compliant version.

#### Current State (Incompatible)
```rust
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,                    // Not Option<String>
    pub language: String,               // Should be project_type: ProjectType
    pub has_git: bool,                  // Removed (redundant)
    pub has_tests: bool,                // Removed (not in PRD)
    pub has_docs: bool,                 // Removed (not in PRD)
    pub vcs_root: Option<PathBuf>,      // To be replaced by is_worktree
    pub worktree_root: Option<PathBuf>,  // To be replaced by is_worktree
}
```

#### Required State (PRD-Compatible)
```rust
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
```

#### Requirements
- [ ] Replace `name: String` with `name: Option<String>`
- [ ] Replace `language: String` with `project_type: ProjectType`
- [ ] Add `package_manager: PackageManager`
- [ ] Add `languages: Vec<String>`
- [ ] Add `is_monorepo: bool`
- [ ] Add `is_worktree: bool` (replace `worktree_root: Option<PathBuf>`)
- [ ] Add `config: ProjectConfig`
- [ ] Remove `has_git`, `has_tests`, `has_docs`
- [ ] Keep `root: PathBuf` and `vcs_root: Option<PathBuf>` for internal use

---

### FR-006: ProjectService Struct

**Priority**: P0
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Create the `ProjectService` struct with async interface, caching, and all required methods.

#### API
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

#### Requirements
- [ ] `cache` field: `Arc<Mutex<Option<ProjectInfo>>>`
- [ ] `config` field: `Arc<ConfigService>`
- [ ] `detect()` must be async and walk up directories
- [ ] `get()` must return cached value or call detect
- [ ] `invalidate()` must clear the cache
- [ ] `is_worktree()` must check for `$DATA/worktree/` path prefix
- [ ] `root()` must return just the path from cached ProjectInfo

---

### FR-007: Async Detection with Walk-Up Root Finding

**Priority**: P0
**Module**: `detect.rs` (new or inline)
**Status**: ❌ Not Implemented

#### Description
Implement the root finding algorithm that walks up from cwd to find git/worktree root.

#### Algorithm
```
1. Start from cwd
2. Check for .git, .git/worktrees, .opencode (worktree marker)
3. If found, return parent (git root) or current (worktree)
4. Walk up one directory, repeat
5. If no root found, return cwd as root (type = Unknown)
```

#### Requirements
- [ ] Async implementation using `tokio::fs`
- [ ] Walk up from given `cwd` to find project root
- [ ] Detect `.git` directory as git root marker
- [ ] Detect `.git` file (worktree reference) for git worktrees
- [ ] Detect `.opencode` directory as opencode sandbox marker
- [ ] Return root path on success
- [ ] Return `cwd` as fallback root with `ProjectType::Unknown`

---

### FR-008: Type Detection with Priority

**Priority**: P0
**Module**: `detect.rs`
**Status**: ❌ Not Implemented

#### Description
Implement project type detection with proper priority ordering.

#### Detection Priority
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

#### Requirements
- [ ] Check files in priority order (first match wins)
- [ ] `Cargo.toml` → `ProjectType::Rust`
- [ ] `go.mod` → `ProjectType::Go`
- [ ] `pyproject.toml` OR `requirements.txt` OR `setup.py` → `ProjectType::Python`
- [ ] `package.json` → `ProjectType::Node`
- [ ] `pom.xml` OR `build.gradle` → `ProjectType::Java`
- [ ] `CMakeLists.txt` OR `Makefile` OR `compile_commands.json` → `ProjectType::Cpp`
- [ ] `Gemfile` → `ProjectType::Ruby`
- [ ] `composer.json` → `ProjectType::Php`
- [ ] `*.csproj` OR `*.sln` → `ProjectType::Dotnet`
- [ ] `Package.swift` → `ProjectType::Swift`
- [ ] No match → `ProjectType::Unknown`

---

### FR-009: Package Manager Detection for Node Projects

**Priority**: P1
**Module**: `detect.rs`
**Status**: ❌ Not Implemented

#### Description
Detect the package manager used in Node.js projects from lock files.

#### Detection Logic
```
1. pnpm-lock.yaml → Pnpm
2. yarn.lock (且无 package-lock.json) → Yarn
3. bun.lockb → Bun
4. package-lock.json → Npm
5. Default → Npm (fallback)
```

#### Requirements
- [ ] Check for `pnpm-lock.yaml` → `PackageManager::Pnpm`
- [ ] Check for `yarn.lock` without `package-lock.json` → `PackageManager::Yarn`
- [ ] Check for `bun.lockb` → `PackageManager::Bun`
- [ ] Check for `package-lock.json` → `PackageManager::Npm`
- [ ] Default fallback → `PackageManager::Npm`
- [ ] Only applicable when `project_type == ProjectType::Node`

---

### FR-010: Sandbox Worktree Detection

**Priority**: P1
**Module**: `detect.rs`
**Status**: ❌ Not Implemented

#### Description
Implement `is_worktree()` to detect if a path is inside an opencode sandbox.

#### Algorithm
```
1. Get DATA directory from environment/config
2. Check if path starts with $DATA/worktree/
3. Return true if inside sandbox, false otherwise
```

#### Requirements
- [ ] Check for paths inside `$DATA/worktree/` directory
- [ ] `is_worktree: bool` field in `ProjectInfo`
- [ ] Async implementation
- [ ] Must handle case where DATA env var is not set

---

### FR-011: Monorepo Detection

**Priority**: P1
**Module**: `detect.rs`
**Status**: ❌ Not Implemented

#### Description
Detect if project is a monorepo root via workspace configuration.

#### Detection Logic
```
Node: has "workspaces" in package.json
Rust: has members in workspace Cargo.toml
Go: has multiple modules
```

#### Requirements
- [ ] Check `package.json` for `"workspaces"` field (Node monorepo)
- [ ] Parse `Cargo.toml` for `[workspace]` section with `members` (Rust monorepo)
- [ ] Check for multiple `go.mod` files in subdirectories (Go monorepo)
- [ ] `is_monorepo: bool` field in `ProjectInfo`

---

### FR-012: Language Detection via Walkdir

**Priority**: P1
**Module**: `detect.rs`
**Status**: ❌ Not Implemented

#### Description
Scan source files using walkdir to determine primary languages in the project.

#### Requirements
- [ ] Use `walkdir::WalkDir` to scan project files
- [ ] Count source files by extension
- [ ] Return sorted list of detected languages
- [ ] Limit depth to avoid scanning node_modules, target, etc.
- [ ] Languages: `.rs` → Rust, `.js/.ts/.jsx/.tsx` → JavaScript, `.py` → Python, `.go` → Go, etc.

---

### FR-013: Project Name Extraction

**Priority**: P1
**Module**: `config.rs`
**Status**: ⚠️ Partial

#### Description
Extract project name from configuration files.

#### Requirements
- [ ] Extract `name` from `package.json` for Node projects → `Option<String>`
- [ ] Extract `name` from `Cargo.toml` for Rust projects → `Option<String>`
- [ ] Return `None` if name not found or not applicable
- [ ] Async file reading with `tokio::fs`

---

### FR-014: TOML Parsing for Cargo.toml

**Priority**: P1
**Module**: `config.rs`
**Status**: ⚠️ Partial (toml dependency exists)

#### Description
Parse `Cargo.toml` to extract project name and workspace information.

#### Requirements
- [ ] Use existing `toml = "0.8"` dependency
- [ ] Parse `[package]` section for `name` field
- [ ] Parse `[workspace]` section for `members` array
- [ ] Store raw TOML string in `ProjectConfig.cargo_toml`

---

### FR-015: Cache Invalidation

**Priority**: P1
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Implement `invalidate()` method to clear cached `ProjectInfo`.

#### Requirements
- [ ] `invalidate()` method on `ProjectService`
- [ ] Clear the `Arc<Mutex<Option<ProjectInfo>>>` cache
- [ ] Thread-safe implementation
- [ ] Called after mkdir/file changes that affect project structure

---

### FR-016: Root Shorthand Method

**Priority**: P2
**Module**: `project.rs`
**Status**: ❌ Not Implemented

#### Description
Implement `root()` as a shortcut to get just the project root path.

#### Requirements
- [ ] `root()` async method returning `Result<PathBuf, ProjectError>`
- [ ] Return cached `ProjectInfo.root` if available
- [ ] Call `get()` internally if cache empty
- [ ] Return `ProjectError::NotFound` if no project detected

---

## Data Structures (Complete List)

### ProjectType Enum (FR-001)

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
```

---

### PackageManager Enum (FR-002)

```rust
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
```

---

### ProjectError Enum (FR-003)

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

### ProjectConfig Struct (FR-004)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub package_json: Option<serde_json::Value>,
    pub cargo_toml: Option<String>,
    pub start_command: Option<String>,
    pub install_command: Option<String>,
}
```

---

### ProjectInfo Struct (FR-005)

```rust
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
```

---

### ProjectService Struct (FR-006)

```rust
pub struct ProjectService {
    cache: Arc<Mutex<Option<ProjectInfo>>>,
    config: Arc<ConfigService>,
}

impl ProjectService {
    pub async fn detect(&self, cwd: Option<&Path>) -> Result<ProjectInfo, ProjectError>
    pub async fn get(&self) -> Result<ProjectInfo, ProjectError>
    pub fn invalidate(&self)
    pub async fn is_worktree(&self, path: &Path) -> bool
    pub async fn root(&self) -> Result<PathBuf, ProjectError>
}
```

---

## API Surface (Complete)

```rust
// Enums
pub enum ProjectType { ... }
pub enum PackageManager { ... }
pub enum ProjectError { ... }

// Structs
pub struct ProjectConfig { ... }
pub struct ProjectInfo { ... }
pub struct ProjectService { ... }

// ProjectService Methods
impl ProjectService {
    pub fn new(config: Arc<ConfigService>) -> Self
    pub async fn detect(&self, cwd: Option<&Path>) -> Result<ProjectInfo, ProjectError>
    pub async fn get(&self) -> Result<ProjectInfo, ProjectError>
    pub fn invalidate(&self)
    pub async fn is_worktree(&self, path: &Path) -> bool
    pub async fn root(&self) -> Result<PathBuf, ProjectError>
}
```

---

## Detection Algorithms

### Root Finding Algorithm (FR-007)

```
function find_root(cwd):
    current = cwd
    while current != parent(current):
        if current/.git exists or current/.git is file or current/.opencode exists:
            return current
        current = parent(current)
    return cwd  # fallback: return cwd as unknown project
```

### Type Detection Priority (FR-008)

| Priority | File | ProjectType |
|----------|------|-------------|
| 1 | Cargo.toml | Rust |
| 2 | go.mod | Go |
| 3 | pyproject.toml OR requirements.txt OR setup.py | Python |
| 4 | package.json | Node |
| 5 | pom.xml OR build.gradle | Java |
| 6 | CMakeLists.txt OR Makefile OR compile_commands.json | Cpp |
| 7 | Gemfile | Ruby |
| 8 | composer.json | Php |
| 9 | *.csproj OR *.sln | Dotnet |
| 10 | Package.swift | Swift |
| 11 | (none) | Unknown |

### Package Manager Detection (FR-009)

| Priority | File | PackageManager |
|----------|------|----------------|
| 1 | pnpm-lock.yaml | Pnpm |
| 2 | yarn.lock (no package-lock.json) | Yarn |
| 3 | bun.lockb | Bun |
| 4 | package-lock.json | Npm |
| 5 | (none) | Npm (fallback) |

### Monorepo Detection (FR-011)

```
Node: package.json contains "workspaces" field
Rust: Cargo.toml has [workspace] with members
Go: Multiple go.mod files in subdirectories
```

---

## Acceptance Criteria

| ID | Criteria | Priority | Status |
|----|----------|----------|--------|
| AC-001 | `detect()` finds correct git/worktree root from any subdirectory | P0 | ❌ |
| AC-002 | `detect()` identifies correct `ProjectType` from marker files | P0 | ❌ |
| AC-003 | `detect()` identifies correct `PackageManager` for Node projects | P1 | ❌ |
| AC-004 | `detect()` finds correct package name from package.json / Cargo.toml | P1 | ⚠️ |
| AC-005 | `is_worktree()` returns `true` for paths inside `$DATA/worktree/` | P1 | ❌ |
| AC-006 | `detect_monorepo()` returns `true` for workspace roots | P1 | ❌ |
| AC-007 | Cache is invalidated on `invalidate()` call | P1 | ❌ |
| AC-008 | `root()` returns just the path from cached project | P2 | ❌ |
| AC-009 | Type detection follows priority order (Rust before Node) | P0 | ❌ |
| AC-010 | Language detection via walkdir returns Vec<String> | P1 | ❌ |
| AC-011 | All operations are `Send + Sync` safe | P0 | ❌ |

---

## Test Design

### Unit Tests Required

| Test | Feature | FR |
|------|---------|-----|
| `test_project_type_enum_variants` | ProjectType enum | FR-001 |
| `test_package_manager_enum_variants` | PackageManager enum | FR-002 |
| `test_project_error_display` | ProjectError enum | FR-003 |
| `test_detect_rust_project` | Type detection | FR-008 |
| `test_detect_node_project` | Type detection | FR-008 |
| `test_detect_node_project_with_pnpm` | Package manager | FR-009 |
| `test_detect_node_project_with_yarn` | Package manager | FR-009 |
| `test_detect_type_priority` | Priority order | FR-008 |
| `test_find_root_walks_up` | Walk-up detection | FR-007 |
| `test_is_worktree_detects_sandbox` | Sandbox detection | FR-010 |
| `test_detect_monorepo_workspace` | Monorepo detection | FR-011 |
| `test_project_service_cache` | Caching | FR-015 |
| `test_project_service_invalidate` | Cache invalidation | FR-015 |
| `test_project_service_root` | Root shortcut | FR-016 |

### Test Implementation

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
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "").unwrap();
    std::fs::write(tmp.path().join("package.json"), "{}").unwrap();
    let info = blocking_detect(tmp.path());
    assert_eq!(info.project_type, ProjectType::Rust);  // Rust wins
}

#[tokio::test]
async fn test_find_root_walks_up() {
    let tmp = TempDir::new().unwrap();
    let sub = tmp.path().join("src").join("deep");
    tokio::fs::create_dir_all(&sub).await.unwrap();
    std::fs::write(tmp.path().join(".git"), "").unwrap();

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

---

## Dependencies

| Dependency | Purpose | Status |
|------------|---------|--------|
| `tokio` with fs, sync | Async file I/O | ✅ Exists |
| `serde` with derive | Serialization | ✅ Exists |
| `serde_json` | package.json parsing | ✅ Exists |
| `toml = "0.8"` | Cargo.toml parsing | ✅ Exists |
| `walkdir` | Directory walking | ✅ Exists |
| `tracing` | Logging | ✅ Exists |
| `thiserror` | Error enums | ✅ Exists |
| `tempfile` (dev) | Test fixtures | ✅ Exists |

---

## Technical Debt

| ID | Description | Est. Effort | Priority |
|----|-------------|-------------|----------|
| TD-001 | Replace `language: String` with `project_type: ProjectType` throughout codebase | High | P0 |
| TD-002 | Update `ProjectInfo` struct fields to match PRD | High | P0 |
| TD-003 | Create `ProjectService` with async interface replacing `ProjectManager` | High | P0 |
| TD-004 | Implement `ProjectType` and `PackageManager` enums | Medium | P0 |
| TD-005 | Implement walk-up root detection algorithm | Medium | P0 |
| TD-006 | Implement package manager detection for Node | Medium | P1 |
| TD-007 | Implement monorepo detection | Medium | P1 |
| TD-008 | Implement language detection via walkdir | Medium | P1 |
| TD-009 | Implement sandbox worktree detection | Low | P1 |
| TD-010 | Add cache invalidation and `root()` shorthand | Low | P2 |
| TD-011 | Consider extracting `crates/project/` crate | High | P2 |

---

## Implementation Phases

### Phase 1: Core Types (P0)
1. Define `ProjectType` enum with all 11 variants (FR-001)
2. Define `PackageManager` enum with all 11 variants (FR-002)
3. Define `ProjectError` enum with 4 variants (FR-003)
4. Define `ProjectConfig` struct (FR-004)
5. Update `ProjectInfo` to match PRD (FR-005)

### Phase 2: Service Implementation (P0)
6. Create `ProjectService` struct with `cache` and `config` fields (FR-006)
7. Implement `detect()` with async interface and walk-up (FR-007)
8. Implement type detection with priority order (FR-008)
9. Implement `is_worktree()` for sandbox detection (FR-010)
10. Implement `get()`, `invalidate()`, `root()` methods
11. Add caching with `Arc<Mutex<Option<ProjectInfo>>>`

### Phase 3: Detection Logic (P1)
12. Implement package manager detection for Node (FR-009)
13. Implement monorepo detection (FR-011)
14. Implement language detection via walkdir (FR-012)
15. Implement project name extraction (FR-013)
16. Add TOML parsing for Cargo.toml (FR-014)

### Phase 4: Testing & Polish (P1/P2)
17. Add comprehensive unit tests
18. Add integration tests
19. Consider crate separation to `crates/project/`

---

## Files to Modify

| File | Changes |
|------|---------|
| `opencode-rust/crates/core/src/project.rs` | Complete rewrite to match PRD |
| `opencode-rust/crates/core/src/lib.rs` | Update exports if needed |
| `opencode-rust/crates/core/Cargo.toml` | Already has `toml = "0.8"` |

---

## Current Code Reference

**Existing Implementation**:
- `opencode-rust/crates/core/src/project.rs` (865 lines)
- `ProjectManager` struct (sync, wrong API)
- `ProjectInfo` struct (incompatible fields)
- `WorkspaceValidationError` (different from `ProjectError`)
- `validate_workspace()` function
- `normalize_path()` function
- Path validation and traversal checking

**Known Issues**:
- `ProjectInfo.name` is `String` not `Option<String>`
- `ProjectInfo.language` is `String` not `ProjectType`
- Missing `ProjectType` and `PackageManager` enums
- Missing `ProjectService` with async interface
- `ProjectManager::detect()` is sync, no walk-up
- No package manager detection
- No monorepo detection
- No sandbox worktree detection
- No caching mechanism

---

## Verification Checklist

- [ ] `ProjectType` enum with 11 variants defined
- [ ] `PackageManager` enum with 11 variants defined
- [ ] `ProjectError` enum with 4 variants defined
- [ ] `ProjectConfig` struct defined
- [ ] `ProjectInfo` struct matches PRD exactly
- [ ] `ProjectService` struct with cache and config fields
- [ ] `detect()` is async and walks up directories
- [ ] Type detection follows priority order
- [ ] Package manager detection works for Node projects
- [ ] Monorepo detection for Node, Rust, Go
- [ ] Language detection via walkdir
- [ ] `is_worktree()` detects `$DATA/worktree/` paths
- [ ] `get()` returns cached or detects new
- [ ] `invalidate()` clears cache
- [ ] `root()` returns just the path
- [ ] All operations are `Send + Sync` safe
- [ ] Unit tests pass for all features

---

*Document generated: 2026-04-22*
*Based on: PRD (packages/opencode/src/project/) + Gap Analysis (iteration-45/gap-analysis.md)*