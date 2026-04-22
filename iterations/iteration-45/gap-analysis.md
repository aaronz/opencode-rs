# Gap Analysis: Project Module (Iteration 45)

## Executive Summary

**PRD Reference**: `packages/opencode/src/project/` - Project detection and management module
**Implementation Location**: `opencode-rust/crates/core/src/project.rs`
**Implementation Status**: ❌ **P0 BLOCKING** - Core API surface missing, requires complete redesign

---

## 1. Gap Summary Table

| Gap Item | Severity | Module | Fix Suggestion |
|----------|----------|--------|---------------|
| Missing `ProjectType` enum | P0 | core/project | Define enum matching PRD (Node, Rust, Python, Go, Java, Cpp, Ruby, Php, Dotnet, Swift, Unknown) |
| Missing `PackageManager` enum | P0 | core/project | Define enum matching PRD (Npm, Yarn, Pnpm, Bun, Cargo, Pip, Poetry, Go, Maven, Gradle, Unknown) |
| Missing `ProjectService` struct | P0 | core/project | Create service with `cache`, `config` fields and `detect()`, `get()`, `invalidate()`, `is_worktree()`, `root()` methods |
| Missing `ProjectError` enum | P0 | core/project | Define error enum with NotFound, ReadError, ParseError, Ambiguous variants |
| `ProjectInfo` struct incompatible | P0 | core/project | Replace `name: String, language: String, has_git: bool, has_tests: bool, has_docs: bool` with PRD fields |
| Missing `ProjectConfig` struct | P1 | core/project | Add struct with `package_json`, `cargo_toml`, `start_command`, `install_command` |
| No async detection | P0 | core/project | Implement `detect(cwd: Option<&Path>) -> Result<ProjectInfo, ProjectError>` |
| No package manager detection for Node | P1 | core/project | Implement pnpm/yarn/bun/npm lock file detection |
| No monorepo detection | P1 | core/project | Implement workspace detection for Node, Rust, Go |
| No language detection via walkdir | P1 | core/project | Scan source files to determine primary languages |
| Missing `is_worktree()` async method | P1 | core/project | Implement check for paths inside `$DATA/worktree/` |
| No cache invalidation | P1 | core/project | Implement `invalidate()` method |
| Missing `root()` shorthand | P2 | core/project | Implement shortcut method returning just the path |
| No crate separation | P1 | workspace | Consider extracting to `crates/project/` per PRD |
| Missing TOML parsing dependency | P1 | core/project | Add `toml` crate for Cargo.toml parsing |
| Insufficient tests | P1 | core/project | Add tests for type detection priority, pnpm detection, walk-up root finding |

---

## 2. P0 Blocking Issues (Must Fix)

### P0-1: Missing `ProjectType` Enum

**PRD Requirement**:
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

**Current State**: Uses ad-hoc strings (`"rust"`, `"javascript"`, `"python"`, `"go"`, `"unknown"`)

**Impact**: API incompatibility, no type safety, no serialization support

---

### P0-2: Missing `PackageManager` Enum

**PRD Requirement**:
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

**Current State**: Not implemented

**Impact**: Cannot detect or report package manager for Node projects

---

### P0-3: Missing `ProjectService` Struct

**PRD Requirement**:
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

**Current State**: `ProjectManager` is a simple struct without async support, caching, or the specified interface

**Impact**: Cannot integrate with async agent framework, no caching strategy

---

### P0-4: Missing `ProjectError` Enum

**PRD Requirement**:
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

**Current State**: Uses `WorkspaceValidationError` which has different variants

**Impact**: Inconsistent error handling, cannot properly signal project-specific failures

---

### P0-5: `ProjectInfo` Struct Incompatible

**PRD Requirement**:
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

**Current State**:
```rust
pub struct ProjectInfo {
    pub root: PathBuf,
    pub name: String,                    // Not Option<String>
    pub language: String,               // Should be project_type: ProjectType
    pub has_git: bool,                  // Removed (redundant with vcs_root)
    pub has_tests: bool,                // Removed (not in PRD)
    pub has_docs: bool,                 // Removed (not in PRD)
    pub vcs_root: Option<PathBuf>,      // Kept (good)
    pub worktree_root: Option<PathBuf>,  // Kept (good, maps to is_worktree)
}
```

**Impact**: API incompatibility with consuming code expecting PRD structure

---

### P0-6: No Async Detection

**PRD Requirement**: `detect()` must be async and walk up directories to find git/worktree root

**Current State**: `ProjectManager::detect()` is sync and doesn't walk up from subdirectories to find root

**Impact**: Cannot detect project from nested subdirectories

---

## 3. P1 High Priority Issues

### P1-1: Missing `ProjectConfig` Struct

**PRD Requirement**:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub package_json: Option<serde_json::Value>,
    pub cargo_toml: Option<String>,
    pub start_command: Option<String>,
    pub install_command: Option<String>,
}
```

**Current State**: Not implemented

---

### P1-2: No Package Manager Detection (Node)

**PRD Requirement**:
```
1. pnpm-lock.yaml → Pnpm
2. yarn.lock (且无 package-lock.json) → Yarn
3. bun.lockb → Bun
4. package-lock.json → Npm
5. Default → Npm (fallback)
```

**Current State**: Not implemented

---

### P1-3: No Monorepo Detection

**PRD Requirement**: Detect if project is a monorepo root via:
- Node: has "workspaces" in package.json
- Rust: has members in workspace Cargo.toml
- Go: has multiple modules

**Current State**: Not implemented

---

### P1-4: No Language Detection via Walkdir

**PRD Requirement**: Scan for source files to determine primary languages

**Current State**: Only checks for marker files (Cargo.toml, package.json, etc.)

---

### P1-5: Missing `is_worktree()` for Sandbox Detection

**PRD Requirement**: Check if path is inside `$DATA/worktree/`

**Current State**: Only detects git worktrees via `.git` file parsing, not opencode sandbox worktrees

---

### P1-6: No Cache Invalidation

**PRD Requirement**: `invalidate()` method to clear cached `ProjectInfo`

**Current State**: `ProjectManager` has no caching mechanism

---

### P1-7: No Async `root()` Shortcut

**PRD Requirement**: `root()` returns just the path from cached project

**Current State**: Not implemented

---

### P1-8: Missing Crate Separation

**PRD Requirement**: Crate layout at `crates/project/` with separate Cargo.toml

**Current State**: Implemented as part of `crates/core/src/project.rs`

---

### P1-9: Missing TOML Dependency

**PRD Requirement**: Need `toml` crate for Cargo.toml parsing

**Current State**: Not in Cargo.toml

---

## 4. P2 Medium Priority Issues

### P2-1: Missing `root()` Shorthand

**PRD Requirement**: `root()` returns just `PathBuf`

**Current State**: Not implemented

---

### P2-2: Detection Priority Not Implemented

**PRD Requirement**: Type detection priority order:
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

**Current State**: Only handles Rust, JavaScript, Python, Go, Unknown

---

## 5. Technical Debt

| Item | Description | Estimated Effort |
|------|-------------|-------------------|
| TD-1 | Replace `language: String` with `project_type: ProjectType` throughout codebase | High |
| TD-2 | Add `toml` dependency to Cargo.toml | Low |
| TD-3 | Consider extracting `crates/project/` for cleaner separation | Medium |
| TD-4 | Implement walkdir-based language detection | Medium |
| TD-5 | Add async support to project detection | Medium |

---

## 6. Implementation Progress

| Acceptance Criteria | Status | Notes |
|---------------------|--------|-------|
| `detect()` finds correct git/worktree root from subdirectory | ❌ | No walk-up logic implemented |
| `detect()` identifies correct `ProjectType` | ❌ | No `ProjectType` enum |
| `detect()` identifies correct `PackageManager` for Node | ❌ | Not implemented |
| `detect()` finds correct package name | ⚠️ | Partial - reads from file name |
| `is_worktree()` returns true for paths inside `$DATA/worktree/` | ❌ | Only checks git worktrees |
| `detect_monorepo()` returns true for workspace roots | ❌ | Not implemented |
| Cache invalidation on `invalidate()` call | ❌ | No caching implemented |
| `root()` shorthand | ❌ | Not implemented |

---

## 7. Required Changes Summary

### Phase 1: Core Types (P0)
1. Define `ProjectType` enum with all variants
2. Define `PackageManager` enum with all variants
3. Define `ProjectError` enum
4. Define `ProjectConfig` struct
5. Update `ProjectInfo` to match PRD

### Phase 2: Service Implementation (P0)
6. Create `ProjectService` with async interface
7. Implement `detect()` with directory walk-up
8. Implement `is_worktree()` for sandbox detection
9. Implement `get()`, `invalidate()`, `root()` methods
10. Add caching with `Arc<Mutex<Option<ProjectInfo>>>`

### Phase 3: Detection Logic (P1)
11. Implement package manager detection for Node
12. Implement monorepo detection
13. Implement language detection via walkdir
14. Add TOML parsing for Cargo.toml

### Phase 4: Testing & Polish (P1/P2)
15. Add comprehensive unit tests
16. Consider crate separation
17. Add missing detection priorities

---

## 8. Files to Modify

| File | Changes |
|------|---------|
| `opencode-rust/crates/core/Cargo.toml` | Add `toml` dependency |
| `opencode-rust/crates/core/src/project.rs` | Complete rewrite to match PRD |
| `opencode-rust/crates/core/src/lib.rs` | Update exports if needed |

---

*Report generated: 2026-04-22*
*Iteration: 45*