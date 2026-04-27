You are a senior Rust systems architect, AI coding agent engineering expert, and large-scale refactoring specialist.

I am working on an open-source Rust port of opencode / Claude-Code-like AI coding agent tooling. Please deeply inspect and refactor this project so that it becomes significantly easier for future coding agents and human developers to understand, modify, test, and extend.

This is not a superficial cleanup task. The goal is to improve the long-term maintainability, modularity, and engineering quality of the project.

## Core Refactoring Goal

Refactor the current opencode-rs project according to Rust ecosystem best practices and AI-coding-friendly engineering principles.

The refactored project should have:

- High cohesion inside each module
- Low coupling between modules
- Clear crate/module boundaries
- Explicit ownership of responsibilities
- Predictable file and directory organization
- Minimal circular dependencies
- Clean public APIs between subsystems
- Better separation between domain logic, infrastructure, UI, CLI, TUI, provider integration, config, logging, persistence, and testing utilities
- A structure that is easy for coding agents to navigate and safely modify

## Important Context

This project is a Rust implementation / port of opencode-like functionality.

It may include areas such as:

- CLI commands and subcommands
- TUI interaction flow
- LLM provider integration
- Model selection
- Authentication and key validation
- Config loading and persistence
- Logging and diagnostics
- Session management
- Agent execution
- Tool execution
- MCP integration
- Skills / commands / hooks / rules
- Repository and workspace awareness
- Testing infrastructure
- Documentation and examples

You must analyze the actual repository before making changes. Do not assume the current structure is correct.

## Required Refactoring Principles

Apply Rust project organization best practices, including but not limited to:

1. Crate and module design

- Keep modules focused on one responsibility.
- Avoid large `mod.rs` or catch-all utility modules.
- Prefer explicit module names that reflect business/domain concepts.
- Use `lib.rs` to expose stable internal APIs where appropriate.
- Keep `main.rs` thin and delegate real work to library crates/modules.
- Consider whether the project should be organized as a Cargo workspace.
- If a workspace is appropriate, propose and implement a clean crate structure.

Possible crate boundaries may include:

- `opencode-rs-cli`
- `opencode-rs-tui`
- `opencode-rs-core`
- `opencode-rs-config`
- `opencode-rs-provider`
- `opencode-rs-agent`
- `opencode-rs-tooling`
- `opencode-rs-mcp`
- `opencode-rs-testing`

Only create crates if they genuinely reduce coupling and improve maintainability.

2. Layering and dependency direction

Establish a clear dependency direction.

For example:

- CLI/TUI should depend on application services.
- Application services should depend on domain/core abstractions.
- Provider implementations should depend on provider traits, not the other way around.
- Config, logging, and persistence should be infrastructure concerns.
- Domain logic should not directly depend on UI, terminal rendering, filesystem layout, or specific provider SDKs.
- Test utilities should not leak into production modules.

Avoid dependency cycles and hidden global state.

3. Domain-driven module boundaries

Separate major responsibilities into clear domains, for example:

- `agent`
- `session`
- `provider`
- `model`
- `auth`
- `config`
- `workspace`
- `tool`
- `mcp`
- `skill`
- `command`
- `hook`
- `rule`
- `logging`
- `diagnostics`
- `ui`
- `cli`
- `tui`
- `storage`
- `testing`

Each domain should expose a small, intentional API.

4. AI-coding-agent-friendly structure

Optimize the repository so that future coding agents can work on it reliably.

This means:

- Clear directory and file names
- Smaller files with focused responsibilities
- Minimal implicit behavior
- Explicit interfaces and contracts
- Good error messages
- Good debug logging boundaries
- Clear entry points for common changes
- Consistent naming conventions
- Localized changes for feature work
- Well-documented architectural decisions
- A short `ARCHITECTURE.md` explaining the module structure and dependency rules
- A `CONTRIBUTING.md` or developer guide explaining where to add new commands, providers, TUI screens, config fields, tests, and docs

5. Rust idioms and quality

Apply idiomatic Rust practices:

- Prefer strong types over stringly typed data
- Use enums for finite state machines and workflow states
- Use traits for stable boundaries and dependency inversion
- Avoid unnecessary trait objects unless dynamic dispatch is needed
- Avoid excessive generics when they reduce readability
- Use `thiserror` / structured errors where appropriate
- Use `tracing` for structured logs instead of ad-hoc printing
- Avoid leaking UI logs into TUI rendering
- Keep async boundaries clear
- Avoid blocking work on async runtimes
- Prefer `Arc` only where shared ownership is actually needed
- Keep configuration structs serializable and versionable
- Make tests deterministic and isolated

6. Engineering directories

Also review and improve engineering-related directories and files, including:

- `.github/`
- CI workflows
- `scripts/`
- `docs/`
- `examples/`
- `tests/`
- `benches/`
- `fixtures/`
- `crates/`
- `xtask/`
- `.cargo/`
- config templates
- development setup docs
- release or packaging scripts
- test data organization

The goal is not only to refactor source code, but also to make the whole repository easier to build, test, debug, release, and evolve.

## Required Work Process

Please follow this process carefully.

### Phase 1: Repository analysis

First inspect the current repository structure and identify:

- Current crate/module layout
- Main entry points
- Major subsystems
- Dependency direction problems
- Overly large files or modules
- Mixed responsibilities
- Duplicated logic
- Global state or hidden coupling
- Config and logging inconsistencies
- Testing structure issues
- Engineering directory problems

Output a concise but concrete refactoring diagnosis before making large changes.

### Phase 2: Refactoring plan

Create a concrete refactoring plan that includes:

- Target directory structure
- Target crate/module boundaries
- Dependency direction rules
- Migration steps
- Risk areas
- Test strategy
- Rollback or incremental safety strategy

Do not perform a massive unsafe rewrite in one step. Prefer incremental, verifiable refactoring.

### Phase 3: Implementation

Implement the refactoring in small, safe steps.

For each major change:

- Move code without changing behavior first when possible.
- Preserve public behavior.
- Update imports and module exports.
- Remove obsolete modules only after successful compilation.
- Add or update tests.
- Improve error handling and logging where the existing structure is clearly wrong.
- Keep CLI/TUI behavior compatible unless explicitly fixing a bug.
- Avoid introducing new dependencies unless strongly justified.

### Phase 4: Validation

After implementation, run the appropriate checks:

- `cargo fmt`
- `cargo clippy --all-targets --all-features`
- `cargo test --all-features`
- CLI smoke tests
- TUI-related tests if available
- Provider/config/auth-related tests if relevant
- Any existing CI-equivalent local command

If some checks cannot run, explain exactly why and what remains unverified.

### Phase 5: Documentation

Update or create documentation:

- `ARCHITECTURE.md`
- Developer guide for adding new features
- Module/crate responsibility map
- Config/logging path conventions
- Testing guide
- Any changed README sections

The documentation should help both humans and future coding agents understand how to work with the project.

## Expected Output

At the end, provide a clear summary containing:

1. What repository problems were found
2. What refactoring was implemented
3. New or updated directory structure
4. New dependency/layering rules
5. Important files changed
6. Tests and validation results
7. Any remaining risks or follow-up tasks

## Constraints

- Do not rewrite everything from scratch.
- Do not change user-facing behavior unless necessary.
- Do not remove features.
- Do not introduce unnecessary abstractions.
- Do not create many crates just for aesthetic reasons.
- Do not hide errors.
- Do not make TUI logs print into the middle of the UI.
- Do not mix test fixtures, runtime config, and production code.
- Do not rely on undocumented implicit conventions.

## Refactoring Success Criteria

The refactoring is successful only if:

- The project still builds and tests pass.
- A new coding agent can quickly understand where to modify CLI, TUI, provider, config, logging, session, agent, tool, and test logic.
- Each major module has a clear responsibility.
- Cross-module dependencies are reduced and intentional.
- The code structure follows common Rust project conventions.
- Engineering directories are organized and documented.
- Future features can be added with localized changes instead of touching many unrelated files.