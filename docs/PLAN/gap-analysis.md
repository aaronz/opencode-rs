# Gap Analysis: Test Design vs Implementation

**Date**: 2026-04-24
**Design**: `docs/DESIGN/test-design.md`

## Status: Partially Implemented

### What's Implemented ✅

| Area | Status |
|------|--------|
| Unit tests (`#[cfg(test)]`) | ✅ Everywhere |
| Integration tests (`tests/`) | ✅ Organized by feature |
| Async tests (`#[tokio::test]`) | ✅ |
| CLI tests (`assert_cmd`) | ✅ |
| Mock helpers | ✅ `MockServer`, `MockLLMProvider`, `TempProject` |
| Benchmarks | ✅ `opencode-benches/` with Criterion |
| TUI testing | ✅ `ratatui-testing/` with `DialogRenderTester` |
| Coverage | ✅ `cargo llvm-cov` with 80% gate |
| Workspace structure | ✅ Proper Cargo.toml layout |

### What's Missing ❌

| Area | Severity | Notes |
|------|----------|-------|
| Property/Fuzz Testing | **P0** | No `proptest`, `quickcheck`, or `cargo-fuzz` |
| CI Staged Jobs | **P0** | Single `cargo test --all`, no nextest |
| Feature Matrix | **P1** | No `--all-features` / `--no-default-features` testing |
| Mutation Testing | **P1** | No `cargo-mutants` |
| Snapshot/Golden Workflow | **P1** | No `insta`, manual snapshots only |
| `test-support` Crate | **P1** | Using `tests/src/common/` instead |
| Fixtures Organization | **P1** | Scattered, no unified `fixtures/` |
| Slow Test Isolation | **P1** | No `#[ignore]`, no DB container isolation |
| Platform Matrix | **P2** | macOS only in CI |
| DB/Container Tests | **P2** | No structured testcontainers setup |
| Nightly Jobs | **P2** | No scheduled fuzz/mutation runs |
| Doc Test Enforcement | **P2** | `cargo test --doc` not in CI |

## Priority Summary

**P0 (Blocking)**:
1. Add `cargo nextest` for faster CI
2. Add feature matrix testing
3. Add platform matrix (Linux/macOS/Windows)

**P1 (Quality)**:
4. Add `proptest` property tests
5. Add `cargo-fuzz` fuzzing
6. Add `insta` snapshot testing
7. Add `cargo-mutants` mutation testing
8. Create `crates/test-support/` crate
9. Consolidate fixtures
10. Isolate slow tests

**P2 (Nice to have)**:
11. Platform matrix expansion
12. DB/container test infrastructure
13. Nightly scheduled jobs
14. Explicit doc test CI

## Quick Wins

1. **Add `cargo nextest`** - 2-10x faster test execution
2. **Add `--all-features` and `--no-default-features`** to existing CI
3. **Add `proptest`** workspace dep and write 5 property tests

## Detailed Analysis

See full plan in `docs/PLAN/README.md`