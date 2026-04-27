# Large File Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split monolithic files (session.rs, project.rs, skill.rs) into focused subdirectory modules to improve maintainability.

**Architecture:** Convert each large `.rs` file into a `mod_name/` directory with focused submodules. The `lib.rs` re-exports remain unchanged. Each submodule handles one specific concern (fork, history, tool_invocation, etc.).

**Tech Stack:** Rust 2021, cargo, thiserror

---

## Phase 1: Session Module Refactoring

### Task 1: Create session/ Directory Structure

**Files:**
- Create: `crates/core/src/session/mod.rs`
- Create: `crates/core/src/session/fork.rs`
- Create: `crates/core/src/session/history.rs`
- Create: `crates/core/src/session/tool_invocation.rs`
- Create: `crates/core/src/session/share.rs`
- Create: `crates/core/src/session/session_info.rs`

- [ ] **Step 1: Create session/ directory**

Run: `mkdir -p crates/core/src/session`

- [ ] **Step 2: Create mod.rs with re-exports (empty shell first)**

Create `crates/core/src/session/mod.rs`:
```rust
//! Session management module.
//!
//! This module provides [`Session`] - the core data structure representing
//! a conversation session with messages, tool invocations, and fork/share history.

pub mod fork;
pub mod history;
pub mod session_info;
pub mod share;
pub mod tool_invocation;

// Re-exports for backward compatibility
pub use session_info::{SessionInfo, SessionSummaryMetadata};
pub use share::ShareError;
pub use ForkError;
```

- [ ] **Step 3: Create fork.rs**

Move ForkError and ForkEntry from session.rs to `crates/core/src/session/fork.rs`:
```rust
//! Fork-related types and errors.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForkError {
    MessageIndexOutOfBounds { requested: usize, len: usize },
}

impl std::fmt::Display for ForkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForkError::MessageIndexOutOfBounds { requested, len } => {
                write!(
                    f,
                    "fork message index out of bounds: requested {}, session has {} messages",
                    requested, len
                )
            }
        }
    }
}

impl std::error::Error for ForkError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkEntry {
    pub forked_at: DateTime<Utc>,
    pub child_session_id: Uuid,
}
```

- [ ] **Step 4: Create history.rs**

Move HistoryEntry and Action from session.rs to `crates/core/src/session/history.rs`:
```rust
//! Undo/redo history management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub action: Action,
    pub messages: Vec<Message>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    AddMessage,
    RemoveMessage,
    ClearSession,
}
```

- [ ] **Step 5: Create tool_invocation.rs**

Move ToolInvocationRecord from session.rs to `crates/core/src/session/tool_invocation.rs`:
```rust
//! Tool invocation records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationRecord {
    pub id: Uuid,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub args_hash: String,
    pub result: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub latency_ms: Option<u64>,
}
```

- [ ] **Step 6: Create share.rs**

Move ShareError from session.rs to `crates/core/src/session/share.rs`:
```rust
//! Session sharing functionality.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShareError {
    SharingDisabled,
    InvalidShareMode,
    AccessDenied,
}

impl std::fmt::Display for ShareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShareError::SharingDisabled => write!(f, "sharing is disabled for this session"),
            ShareError::InvalidShareMode => write!(f, "invalid share mode for this operation"),
            ShareError::AccessDenied => write!(f, "access denied for this operation"),
        }
    }
}

impl std::error::Error for ShareError {}
```

- [ ] **Step 7: Create session_info.rs**

Move SessionInfo and SessionSummaryMetadata from session.rs to `crates/core/src/session/session_info.rs`:
```rust
//! Session information types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSummaryMetadata {
    pub summary: String,
    pub created_at: DateTime<Utc>,
}
```

- [ ] **Step 8: Move Session struct to mod.rs and update imports**

Read current session.rs to identify all types that remain in the main file. The Session struct and all its impl blocks should stay in mod.rs along with helper methods.

- [ ] **Step 9: Update lib.rs if needed**

Verify `lib.rs` line 88 still works: `pub use session::{Session, SessionInfo, SessionSummaryMetadata, ToolInvocationRecord};`

- [ ] **Step 10: Delete old session.rs file**

Run: `rm crates/core/src/session.rs`

- [ ] **Step 11: Verify build**

Run: `cargo build -p opencode-core 2>&1`
Expected: SUCCESS (or compile errors we can fix)

- [ ] **Step 12: Run clippy**

Run: `cargo clippy -p opencode-core --lib -- -D warnings 2>&1`
Expected: 0 warnings

---

## Phase 2: Project Module Refactoring

### Task 2: Create project/ Directory Structure

**Files:**
- Create: `crates/core/src/project/mod.rs`
- Create: `crates/core/src/project/detection.rs`
- Create: `crates/core/src/project/service.rs`

- [ ] **Step 1: Create project/ directory**

Run: `mkdir -p crates/core/src/project`

- [ ] **Step 2: Create mod.rs with type definitions**

Move ProjectType, PackageManager, ProjectConfig, ProjectInfo to `crates/core/src/project/mod.rs`:
```rust
//! Project and workspace management.
//!
//! This module provides [`ProjectManager`] for detecting and managing
//! development projects across different languages and frameworks.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

pub mod detection;
pub mod error;
pub mod service;

pub use error::{ProjectError, WorkspaceValidationError, WorkspaceValidationResult};
```

- [ ] **Step 3: Create error.rs**

```rust
//! Project error types.

use std::path::PathBuf;
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
```

- [ ] **Step 4: Create service.rs**

Move ConfigService and ProjectService from project.rs to `crates/core/src/project/service.rs`

- [ ] **Step 5: Create detection.rs**

Move project detection logic (find_root, detect_project_info) from project.rs

- [ ] **Step 6: Delete old project.rs**

Run: `rm crates/core/src/project.rs`

- [ ] **Step 7: Verify build**

Run: `cargo build -p opencode-core 2>&1`

- [ ] **Step 8: Run clippy**

Run: `cargo clippy -p opencode-core --lib -- -D warnings 2>&1`

---

## Phase 3: Skill Module Refactoring

### Task 3: Create skill/ Directory Structure

**Files:**
- Create: `crates/core/src/skill/mod.rs`
- Create: `crates/core/src/skill/match.rs`
- Create: `crates/core/src/skill/loader.rs`

- [ ] **Step 1: Create skill/ directory**

Run: `mkdir -p crates/core/src/skill`

- [ ] **Step 2: Create mod.rs**

Move SkillManager and Skill struct to mod.rs

- [ ] **Step 3: Create match.rs**

Move MatchType and SkillMatch to `crates/core/src/skill/match.rs`

- [ ] **Step 4: Create loader.rs**

Move skill loading/installation logic

- [ ] **Step 5: Delete old skill.rs**

Run: `rm crates/core/src/skill.rs`

- [ ] **Step 6: Verify build**

Run: `cargo build -p opencode-core 2>&1`

- [ ] **Step 7: Run clippy**

Run: `cargo clippy -p opencode-core --lib -- -D warnings 2>&1`

---

## Final Verification

- [ ] **Step: Full workspace build**

Run: `cargo build --workspace 2>&1`
Expected: SUCCESS

- [ ] **Step: Full workspace clippy**

Run: `cargo clippy --workspace -- -D warnings 2>&1`
Expected: 0 warnings

- [ ] **Step: Run tests**

Run: `cargo test --workspace 2>&1`
Expected: All tests pass

---

## Rollback Plan (if needed)

If build fails after any phase:
1. Restore original `.rs` file from git
2. Revert any lib.rs changes
3. Investigate and fix submodule organization

Run: `git checkout crates/core/src/session.rs` (or project.rs, skill.rs)

---

## Files to Delete After Success

Once all phases complete and build passes, these empty/unused files may exist:
- Any orphaned `.rs` files that were converted to directories

Verify: `git status` should show new directories and deleted large .rs files
