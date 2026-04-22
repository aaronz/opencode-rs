# Implementation Plan: Project Module (Iteration 45)

**Document Version**: 45
**Date**: 2026-04-22
**Status**: Active Implementation
**Based on**: Gap Analysis + Spec v45

---

## Phase 1: Core Types (P0)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 1.1 | Define `ProjectType` enum with 11 variants | ⬜ | Node, Rust, Python, Go, Java, Cpp, Ruby, Php, Dotnet, Swift, Unknown |
| 1.2 | Define `PackageManager` enum with 11 variants | ⬜ | Npm, Yarn, Pnpm, Bun, Cargo, Pip, Poetry, Go, Maven, Gradle, Unknown |
| 1.3 | Define `ProjectError` enum with 4 variants | ⬜ | NotFound, ReadError, ParseError, Ambiguous |
| 1.4 | Define `ProjectConfig` struct | ⬜ | package_json, cargo_toml, start_command, install_command |
| 1.5 | Update `ProjectInfo` struct to match PRD | ⬜ | Replace name: String → name: Option<String>, language → project_type, etc. |

## Phase 2: Service Implementation (P0)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 2.1 | Create `ProjectService` struct with cache and config fields | ⬜ | Arc<Mutex<Option<ProjectInfo>>>, Arc<ConfigService> |
| 2.2 | Implement `detect()` async with walk-up root finding | ⬜ | Walk up from cwd to find .git/.git file/.opencode |
| 2.3 | Implement `get()` method returning cached or detected | ⬜ | |
| 2.4 | Implement `invalidate()` method | ⬜ | Clear cache |
| 2.5 | Implement `is_worktree()` async method | ⬜ | Check $DATA/worktree/ path prefix |
| 2.6 | Implement `root()` shorthand method | ⬜ | Return cached ProjectInfo.root |

## Phase 3: Detection Logic (P1)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 3.1 | Implement type detection with priority order | ⬜ | Cargo.toml > go.mod > pyproject.toml > package.json > ... |
| 3.2 | Implement Node package manager detection | ⬜ | pnpm > yarn > bun > npm (fallback) |
| 3.3 | Implement monorepo detection | ⬜ | Node workspaces, Rust workspace members, Go modules |
| 3.4 | Implement language detection via walkdir | ⬜ | Scan source files by extension |
| 3.5 | Implement project name extraction from config | ⬜ | package.json name, Cargo.toml package.name |
| 3.6 | Add TOML parsing for Cargo.toml | ⬜ | Extract name and workspace members |

## Phase 4: Testing & Polish (P1/P2)

| ID | Task | Status | Notes |
|----|------|--------|-------|
| 4.1 | Add unit tests for all features | ⬜ | 14+ test cases required |
| 4.2 | Add integration tests | ⬜ | |
| 4.3 | Verify Send + Sync safety | ⬜ | |

---

## Implementation Order

```
Phase 1 (Core Types):
1. Add thiserror, tokio, serde derives
2. Define ProjectType enum
3. Define PackageManager enum
4. Define ProjectError enum
5. Define ProjectConfig struct
6. Update ProjectInfo struct

Phase 2 (Service):
7. Create ProjectService struct
8. Implement detect() async
9. Implement get(), invalidate(), root()
10. Implement is_worktree()

Phase 3 (Detection):
11. Implement walk-up root finding
12. Implement type detection priority
13. Implement package manager detection
14. Implement monorepo detection
15. Implement language detection
16. Implement name extraction

Phase 4 (Testing):
17. Write unit tests
18. Run cargo test
```

---

## Key Files

| File | Changes |
|------|---------|
| `opencode-rust/crates/core/src/project.rs` | Complete rewrite |

## Dependencies (already in Cargo.toml)

- tokio (fs, sync)
- serde (derive)
- serde_json
- toml = "0.8"
- walkdir
- tracing
- thiserror
- tempfile (dev)