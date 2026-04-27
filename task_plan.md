# Task Plan: Refactor opencode-rs according to Rust best practices

## Goal
Refactor the opencode-rs project to improve long-term maintainability, modularity, and AI-coding-agent-friendliness while preserving all functionality.

## Status: Phase 3 (Implementation) - In Progress

## Phases

### Phase 1: Repository Analysis
- [x] Analyze current crate/module layout
- [x] Identify main entry points and major subsystems
- [x] Identify dependency direction problems
- [x] Identify overly large files
- **Status:** ✅ complete

### Phase 2: Refactoring Plan
- [x] Create target directory structure
- [x] Define crate boundaries
- [x] Establish dependency direction rules
- **Status:** ✅ complete

### Phase 3: Implementation (Documentation Focus)
- [x] Create ARCHITECTURE.md
- [x] Fix 2 collapsible else-if warnings in CLI
- [x] Add module documentation to 29 core modules
- **Status:** 🔄 in progress (47% documentation coverage)

### Phase 4: Validation
- [x] cargo build --workspace - PASSED
- [x] cargo clippy --workspace - 0 warnings
- **Status:** ✅ complete

### Phase 5: Major Refactoring (Future Work)
- [ ] Extract session domain from core (~2500 lines)
- [ ] Extract project domain from core (~2200 lines)
- [ ] Extract skill domain from core (~1500 lines)
- [ ] Split large files in tools crate
- **Status:** ⏸️ paused (high effort)

---

## Documentation Coverage

| Metric | Value |
|--------|-------|
| Total modules in core | 62 |
| Documented modules | 29 (47%) |
| Undocumented modules | 33 (53%) |

---

## Completed Work Summary

### Files Created
1. `opencode-rust/docs/ARCHITECTURE.md` - Architecture documentation

### Files Modified
1. `crates/cli/src/cmd/files.rs` - Fixed 2 clippy warnings
2. 29 modules in `crates/core/src/` - Added module documentation

---

## Validation Results
| Test | Result |
|------|--------|
| cargo build --workspace | ✓ PASSED |
| cargo clippy --workspace | ✓ 0 warnings |

---

## Notes
- Following "incremental, verifiable refactoring" approach
- Documentation improves AI-coding-agent understandability
- Major refactoring requires significant effort