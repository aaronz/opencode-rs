# Workspace bootstrap instructions

This repository contains a Rust AI coding runtime plus supporting workspace metadata.

## What this workspace is

- `rust-opencode-port/` is the primary Rust workspace and runtime.
- The root repository also contains documentation under `docs/` and a small `.opencode/` metadata directory for plugin/skill/command tooling.
- The Rust workspace is the main delivery target; use it as the default working area for code and runtime changes.

## Key entrypoints

- Root build wrapper: `./build.sh`
- Rust workspace root: `rust-opencode-port/`
- Rust workspace manifest: `rust-opencode-port/Cargo.toml`
- Runtime README: `rust-opencode-port/README.md`

## Build and test

Preferred quick commands:

```bash
./build.sh         # build release binary
./build.sh --debug # build debug binary
./build.sh --test  # build and run tests
```

Direct Rust commands:

```bash
cd rust-opencode-port
cargo build --release
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Primary binary and runtime

- The runtime binary is built as `rust-opencode-port/target/release/opencode-rs`.
- The expected runtime command is `opencode-rs` from the `rust-opencode-port` directory.

## Important structural conventions

- `rust-opencode-port/` is a Cargo workspace with these crates:
  - `crates/core`
  - `crates/cli`
  - `crates/llm`
  - `crates/tools`
  - `crates/tui`
  - `crates/agent`
  - `crates/lsp`
  - `crates/storage`
  - `crates/server`
  - `crates/permission`
  - `crates/auth`
  - `crates/control-plane`
  - `crates/plugin`
  - `crates/git`

- `docs/` contains product/design documents, PRDs, and TUI design notes. Treat them as source of truth for architecture and user-facing design decisions.
- `.opencode/` at repository root is workspace metadata for plugin/command tooling and should not be assumed to be part of the Rust runtime implementation.

## What to prioritize

- For runtime changes, focus on `rust-opencode-port/` and its crates.
- For project-level design or product decisions, prefer the docs in `docs/PRD.md` and `docs/design-tui.md`.
- Avoid duplicating design content from `docs/` into code comments or instructions unless adding implementation-specific notes.

## Agent guidance

- Start from `rust-opencode-port/Cargo.toml` and `rust-opencode-port/README.md`.
- Use `./build.sh` or the workspace `cargo` commands; do not assume a top-level `package.json` or Node build for the Rust runtime.
- When modifying runtime behavior, confirm whether the work belongs in `crates/core`, `crates/agent`, `crates/tools`, or `crates/tui`.
- Keep feature work isolated to the Rust workspace unless the task explicitly involves `.opencode/` plugin/skill metadata.

## Notes for new contributors

- There is no existing `.github/copilot-instructions.md`; this file is the bootstrap anchor for AI/assistant workflows.
- If you need to add workspace-specific agent rules later, use `AGENTS.md` only when the repository has multiple distinct agent modes or cross-project workflows.
