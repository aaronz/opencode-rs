# Task Checklist: Project Module (Iteration 45)

**Document Version**: 45
**Date**: 2026-04-22
**Status**: Active Implementation

---

## P0 Tasks (Blocking)

### FR-001: ✅ Done
- [ ] Define enum with 11 variants: Node, Rust, Python, Go, Java, Cpp, Ruby, Php, Dotnet, Swift, Unknown
- [ ] Add derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
- [ ] Add serde rename_all = "lowercase"

### FR-002: ✅ Done
- [ ] Define enum with 11 variants: Npm, Yarn, Pnpm, Bun, Cargo, Pip, Poetry, Go, Maven, Gradle, Unknown
- [ ] Add derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
- [ ] Add serde rename_all = "lowercase"

### FR-003: ✅ Done
- [x] Define enum with 4 variants: NotFound, ReadError, ParseError, Ambiguous
- [x] Add thiserror #[derive(Error)]
- [x] Implement proper error messages with #[error()] attributes

### FR-004: ✅ Done
- [ ] Define struct with 4 fields: package_json, cargo_toml, start_command, install_command
- [ ] package_json: Option<serde_json::Value>
- [ ] cargo_toml: Option<String>
- [ ] Add derives: Debug, Clone, Default, Serialize, Deserialize

### FR-005: ✅ Done
- [x] Replace `name: String` with `name: Option<String>`
- [x] Replace `language: String` with `project_type: ProjectType`
- [x] Add `package_manager: PackageManager`
- [x] Add `languages: Vec<String>`
- [x] Add `is_monorepo: bool`
- [x] Add `is_worktree: bool`
- [x] Add `config: ProjectConfig`
- [x] Remove `has_git`, `has_tests`, `has_docs`
- [x] Keep `root: PathBuf` and `vcs_root: Option<PathBuf>`

### FR-006: ✅ Done
- [ ] Create struct with cache: Arc<Mutex<Option<ProjectInfo>>> and config: Arc<ConfigService>
- [ ] Implement new(config: Arc<ConfigService>) -> Self
- [ ] Implement async detect(cwd: Option<&Path>) -> Result<ProjectInfo, ProjectError>
- [ ] Implement async get() -> Result<ProjectInfo, ProjectError>
- [ ] Implement invalidate() -> ()
- [ ] Implement async is_worktree(path: &Path) -> bool
- [ ] Implement async root() -> Result<PathBuf, ProjectError>

### FR-007: ✅ Done
- [ ] Implement walk-up algorithm starting from cwd
- [ ] Check for .git directory (git root)
- [ ] Check for .git file (git worktree reference)
- [ ] Check for .opencode directory (opencode sandbox)
- [ ] Return root path or cwd as fallback with ProjectType::Unknown

### FR-008: Type Detection with Priority ✅ Done
- [x] Check Cargo.toml → Rust
- [x] Check go.mod → Go
- [x] Check pyproject.toml OR requirements.txt OR setup.py → Python
- [x] Check package.json → Node
- [x] Check pom.xml OR build.gradle → Java
- [x] Check CMakeLists.txt OR Makefile OR compile_commands.json → Cpp
- [x] Check Gemfile → Ruby
- [x] Check composer.json → Php
- [x] Check *.csproj OR *.sln → Dotnet
- [x] Check Package.swift → Swift
- [x] No match → Unknown

---

## P1 Tasks (High Priority)

### FR-009: Package Manager Detection for Node Projects
- [ ] Check for pnpm-lock.yaml → Pnpm
- [ ] Check for yarn.lock (without package-lock.json) → Yarn
- [ ] Check for bun.lockb → Bun
- [ ] Check for package-lock.json → Npm
- [ ] Default fallback → Npm

### FR-010: Sandbox Worktree Detection
- [ ] Get DATA directory from environment/config
- [ ] Check if path starts with $DATA/worktree/
- [ ] Return true if inside sandbox

### FR-011: Monorepo Detection
- [ ] Check package.json for "workspaces" field (Node monorepo)
- [ ] Parse Cargo.toml for [workspace] section with members (Rust monorepo)
- [ ] Check for multiple go.mod files in subdirectories (Go monorepo)

### FR-012: Language Detection via Walkdir
- [ ] Use walkdir::WalkDir to scan project files
- [ ] Count source files by extension
- [ ] Return sorted list of detected languages
- [ ] Limit depth to avoid node_modules, target, etc.

### FR-013: Project Name Extraction
- [ ] Extract name from package.json for Node projects
- [ ] Extract name from Cargo.toml for Rust projects
- [ ] Return None if not found

### FR-014: TOML Parsing for Cargo.toml
- [ ] Use toml crate to parse Cargo.toml
- [ ] Extract [package] section name
- [ ] Extract [workspace] section members
- [ ] Store raw TOML string in ProjectConfig.cargo_toml

### FR-015: Cache Invalidation
- [ ] Implement invalidate() method
- [ ] Clear Arc<Mutex<Option<ProjectInfo>>> cache
- [ ] Ensure thread-safe implementation

### FR-016: Root Shorthand Method
- [ ] Implement root() async method
- [ ] Return cached ProjectInfo.root if available
- [ ] Call get() internally if cache empty
- [ ] Return ProjectError::NotFound if no project

---

## P2 Tasks (Medium Priority)

- [ ] Consider crate separation to crates/project/

---

## Unit Tests Required

| Test | Feature | Status |
|------|---------|--------|
| test_project_type_enum_variants | ProjectType enum | ⬜ |
| test_package_manager_enum_variants | PackageManager enum | ⬜ |
| test_project_error_display | ProjectError enum | ⬜ |
| test_detect_rust_project | Type detection | ⬜ |
| test_detect_node_project | Type detection | ⬜ |
| test_detect_node_project_with_pnpm | Package manager | ⬜ |
| test_detect_node_project_with_yarn | Package manager | ⬜ |
| test_detect_type_priority | Priority order | ⬜ |
| test_find_root_walks_up | Walk-up detection | ⬜ |
| test_is_worktree_detects_sandbox | Sandbox detection | ⬜ |
| test_detect_monorepo_workspace | Monorepo detection | ⬜ |
| test_project_service_cache | Caching | ⬜ |
| test_project_service_invalidate | Cache invalidation | ⬜ |
| test_project_service_root | Root shortcut | ⬜ |

---

## Acceptance Criteria

| ID | Criteria | Priority | Status |
|----|----------|----------|--------|
| AC-001 | detect() finds correct git/worktree root from any subdirectory | P0 | ⬜ |
| AC-002 | detect() identifies correct ProjectType from marker files | P0 | ⬜ |
| AC-003 | detect() identifies correct PackageManager for Node projects | P1 | ⬜ |
| AC-004 | detect() finds correct package name from package.json / Cargo.toml | P1 | ⬜ |
| AC-005 | is_worktree() returns true for paths inside $DATA/worktree/ | P1 | ⬜ |
| AC-006 | detect_monorepo() returns true for workspace roots | P1 | ⬜ |
| AC-007 | Cache is invalidated on invalidate() call | P1 | ⬜ |
| AC-008 | root() returns just the path from cached project | P2 | ⬜ |
| AC-009 | Type detection follows priority order (Rust before Node) | P0 | ⬜ |
| AC-010 | Language detection via walkdir returns Vec<String> | P1 | ⬜ |
| AC-011 | All operations are Send + Sync safe | P0 | ⬜ |

---

## Verification Checklist

- [ ] ProjectType enum with 11 variants defined
- [ ] PackageManager enum with 11 variants defined
- [ ] ProjectError enum with 4 variants defined
- [ ] ProjectConfig struct defined
- [ ] ProjectInfo struct matches PRD exactly
- [ ] ProjectService struct with cache and config fields
- [ ] detect() is async and walks up directories
- [ ] Type detection follows priority order
- [ ] Package manager detection works for Node projects
- [ ] Monorepo detection for Node, Rust, Go
- [ ] Language detection via walkdir
- [ ] is_worktree() detects $DATA/worktree/ paths
- [ ] get() returns cached or detects new
- [ ] invalidate() clears cache
- [ ] root() returns just the path
- [ ] All operations are Send + Sync safe
- [ ] Unit tests pass for all features