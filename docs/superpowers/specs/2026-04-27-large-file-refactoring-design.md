# Large File Refactoring Design

## Goal

Split monolithic files (session.rs, project.rs, skill.rs) into focused subdirectory modules to improve maintainability and navigation for developers and AI agents.

## Problem

| File | Lines | Issues |
|------|-------|--------|
| `session.rs` | 2,495 | Mixes Session data, fork, history, share, tool invocation records |
| `project.rs` | 2,231 | Mixes ProjectManager, ProjectInfo, ProjectService, ConfigService |
| `skill.rs` | 1,523 | Mixes SkillManager, skill matching, skill loading |

Files are difficult to navigate, edit, and understand in isolation.

## Solution

Convert each large `.rs` file into a `mod_name/` directory with focused files.

---

## Session Module Structure

```
session/
├── mod.rs              # Re-exports + Session struct (core data only)
├── fork.rs             # ForkError, ForkEntry, fork logic
├── history.rs          # HistoryEntry, Action, undo/redo
├── tool_invocation.rs  # ToolInvocationRecord
├── share.rs            # ShareError, sharing logic
└── session_info.rs    # SessionInfo, SessionSummaryMetadata
```

## Project Module Structure

```
project/
├── mod.rs              # Re-exports + core types (ProjectInfo, ProjectType, etc.)
├── service.rs          # ProjectService, ConfigService
├── detection.rs        # Project detection logic
└── error.rs            # ProjectError (already separate but will move)
```

## Skill Module Structure

```
skill/
├── mod.rs              # Re-exports + SkillManager
├── match.rs            # MatchType, SkillMatch, matching logic
├── loader.rs           # Skill loading/installation
└── skill.rs            # Skill struct definition
```

---

## Migration Pattern

1. Create `mod_name/` directory alongside existing `.rs` file
2. Create `mod.rs` with all existing re-exports
3. Create focused submodules with `pub mod submod_name`
4. Move code from main file into submodules
5. Update `lib.rs` to use new module path (`mod session;` → `mod session;` still works)
6. Delete original `.rs` file
7. Verify `cargo build` passes
8. Run `cargo clippy` to ensure no warnings

---

## Dependency Rules

1. **No circular dependencies** between top-level modules
2. **Submodules** can reference each other via `use super::submodule`
3. **Session** does not depend on Project
4. **Project** does not depend on Session
5. **Skill** does not depend on Session or Project

---

## Verification

- `cargo build -p opencode-core` must pass after each module conversion
- `cargo clippy -p opencode-core --lib` must have 0 warnings
- All re-exports in `lib.rs` must continue to work unchanged

---

## Implementation Order

1. `session/` (largest, highest payoff)
2. `project/` (second largest)
3. `skill/` (smallest, lowest risk)

---

## Files to Modify

- `crates/core/src/session.rs` → `crates/core/src/session/mod.rs` + submodules
- `crates/core/src/project.rs` → `crates/core/src/project/mod.rs` + submodules
- `crates/core/src/skill.rs` → `crates/core/src/skill/mod.rs` + submodules
- `crates/core/src/lib.rs` — Update module declarations if needed
