# PRD: OpenCode Rust Port — Implementation Plan

## Scope

This document translates the PRD specification set into a phased implementation plan for the OpenCode Rust port.

It defines:

- implementation phases mapped to PRD ownership areas
- concrete deliverables per phase
- cross-crate dependencies
- test plan alignment
- release gates

This plan is read alongside:

- PRD specifications: `01`–`15`
- `16-test-plan.md` — validation strategy
- `17-rust-test-implementation-roadmap.md` — test phasing
- `18-crate-by-crate-test-backlog.md` — backlog by crate

---

## Guiding Principles

### Authority-first implementation

Implement core invariants, config authority, and API surface before building on top of them.

### Dependency ordering

No phase assumes a dependency is implemented until that dependency is actually complete. Subsystem implementations wait for stable runtime contracts.

### Test alignment

Each implementation phase has a corresponding test phase from the roadmap. A feature is not done simply because it compiles — it is done when its test suite passes.

### Convention enforcement

Architecture boundary rules from the PRDs are encoded as convention tests as they are implemented, not retrofitted at the end.

---

## Phase 0 — Project Foundation

**Prerequisites:** none

### Goals

Establish the Rust workspace, crate structure, build infrastructure, and shared test harness that all later phases depend on.

### Deliverables

#### Workspace setup

- [ ] Define `opencode-rust/Cargo.toml` workspace with initial crate list
- [ ] Establish crate naming and module layout matching PRD ownership areas
- [ ] Add `rust-toolchain.toml` with pinned Rust version
- [ ] Configure `cargo-fmt` / rustfmt / clippy in `.cargo/config.toml`
- [ ] Add workspace-level `Cargo.lock` hygiene

#### Crate scaffold

- [ ] `crates/core/` — entities, errors, shared types ( scaffold only, implementation in Phase 1 )
- [ ] `crates/config/` — config parsing/normalization ( scaffold only )
- [ ] `crates/storage/` — persistence layer ( scaffold only )
- [ ] `crates/server/` — HTTP API ( scaffold only )
- [ ] `crates/agent/` — agent runtime ( scaffold only )
- [ ] `crates/tools/` — tool registry ( scaffold only )
- [ ] `crates/plugin/` — plugin system ( scaffold only )
- [ ] `crates/tui/` — terminal UI ( scaffold only )
- [ ] `crates/mcp/` — MCP integration ( scaffold only )
- [ ] `crates/lsp/` — LSP integration ( scaffold only )
- [ ] `crates/llm/` — LLM provider abstractions ( scaffold only )
- [ ] `crates/git/` — VCS/Git integration ( scaffold only )

#### Test infrastructure scaffold

- [ ] `opencode-rust/tests/` workspace layout with `src/common/`
- [ ] `TempProject`, `MockServer`, `MockLLMProvider` helpers ported/added
- [ ] `integration_infrastructure.rs` wired into workspace test config
- [ ] `ratatui-testing/` integrated into TUI crate test workflow

#### CI skeleton

- [ ] `cargo fmt --check` in fast PR gate
- [ ] `cargo clippy -- -D warnings` in fast PR gate
- [ ] Unit test invocation per crate in PR gate
- [ ] Integration suite invocation in extended gate

### Exit Criteria

- `cargo build --all-features` produces a compilable (if skeleton-only) workspace
- `cargo test` runs harness-level tests
- `cargo clippy --all -- -D warnings` is clean
- [x] Convention test suite directory exists at `tests/conventions/` **`(done — 23 passing tests in tests/src/conventions/)`**

---

## Phase 1 — Authority Implementation

**Prerequisites:** Phase 0 complete

### Goals

Implement the three authority contracts that all other subsystems depend on:

1. **Core entity model** (`01`) — stable IDs, ownership tree, lifecycle invariants
2. **Config system** (`06`) — precedence, normalization, ownership boundaries
3. **HTTP API surface** (`07`) — route groups, auth, request validation

### Deliverables

#### Core entities (`crates/core/`)

Implement and test:

- [ ] `Project` type — stable ID, root path, VCS/worktree tracking
- [ ] `Session` type — stable ID, parent lineage, status machine (`idle` → `running` → terminal)
- [ ] `Message` type — ordered history within session, append-only mutation model
- [x] `Part` type — extensible content parts with versioned surface **`(done — crates/core/src/part.rs)`**
- [ ] Ownership tree invariants — `Project → Session → Message → Part`, acyclic
- [ ] Fork model — new child session, parent history intact
- [ ] Snapshot / checkpoint types — metadata for revert/compaction
- [ ] Error types in correct ranges per `AGENTS.md` conventions

Reference: `01-core-architecture.md`

#### Storage layer (`crates/storage/`)

Implement and test:

- [ ] Project/session/message persistence
- [ ] Session recovery after restart
- [ ] Snapshot create/load
- [ ] Revert to checkpoint
- [ ] Compaction with preserved resumability/shareability

Reference: `01-core-architecture.md` persistence model

#### Config system (`crates/config/` or `crates/core/`)

Implement and test:

- [ ] JSON and JSONC parsing
- [ ] Config precedence: remote → global → custom → project → `.opencode` → inline
- [ ] Variable expansion: `{env:VAR}` and `{file:PATH}`
- [ ] `tools` legacy alias normalization into `permission`
- [ ] Config ownership boundary: main config does not own TUI settings
- [ ] Permission rule type with glob pattern support
- [ ] Auth/secret storage paths

Reference: `06-configuration-system.md`

#### HTTP API (`crates/server/`)

Implement and test:

- [ ] Route registration by canonical resource group:
  - Global / runtime
  - Project
  - Session
  - Message
  - Permission / approval
  - Files and search
  - Config / provider / model
  - MCP external integration
  - Streaming transport
- [ ] Auth enforcement per endpoint
- [ ] Request validation
- [ ] Session/message lifecycle endpoints
- [ ] Streaming endpoints (SSE / websocket as supported)
- [ ] API error shape consistency

Reference: `07-server-api.md`

### Test Plan Alignment

| This phase implements | Test roadmap phase | Test backlog items |
|---|---|---|
| Core entities | Phase 1 | `crates/core/` P0 items |
| Storage/snapshots | Phase 1 | `crates/storage/` P0 items |
| Config system | Phase 1 | `crates/core/` config P0 items |
| HTTP API | Phase 1 | `crates/server/` P0 items |

### Exit Criteria

- All Phase 1 unit and integration tests pass
- `tests/authority/` covers core ownership invariants, config precedence, and API lifecycle
- Convention tests for architecture boundaries are added to `tests/conventions/`
- No PR can merge that breaks Phase 1 authority contracts

---

## Phase 2 — Runtime Core

**Prerequisites:** Phase 1 complete and stable

### Goals

Implement the runtime components that power agent execution:

1. **Agent system** (`02`) — primary/subagent model, permission boundaries
2. **Tools system** (`03`) — registry, execution pipeline, permission gate
3. **Plugin system** (`08`) — server/runtime plugin hooks and loading
4. **TUI plugin API** (`15`) — plugin surface for the terminal UI

### Deliverables

#### Agent system (`crates/agent/`)

Implement and test:

- [x] Primary agent execution loop **`(done — crates/agent/src/runtime.rs)`**
- [ ] Exactly one active primary agent per session invariant
- [ ] Hidden vs visible agent behavior (build, plan, compaction, title, summary)
- [ ] Subagent execution — child context, result handoff, parent history intact
- [ ] Task/delegation mechanism (exact payload TBD from implementation)
- [ ] Permission inheritance from parent to subagent
- [ ] Runtime restriction of subagent permissions

Reference: `02-agent-system.md`

#### Tools system (`crates/tools/`)

Implement and test:

- [ ] Tool registry — registration, lookup, listing
- [ ] Built-in tool interface — stable name/description/args schema
- [ ] Custom tool discovery from `.opencode/tools/` and `~/.config/opencode/tools/` **`(PARTIAL — DirectoryScanner::scan_tools() exists but: (1) scans TOOL.md not .ts/.js per PRD, (2) discovered tools recorded in config but NOT registered with ToolRegistry)`**
- [ ] Execution pipeline:
  1. name lookup
  2. [x] permission gate (`allow` / `ask` / `deny`) **`(done — AgentExecutor in crates/core/src/executor.rs)`**
  3. argument validation
  4. execution
  5. structured result or error
- [ ] MCP tool qualification (server-qualified naming)
- [ ] Deterministic collision resolution for custom vs built-in names
- [ ] Result caching for safe tools

Reference: `03-tools-system.md`

#### Server/runtime plugin system (`crates/plugin/`)

Implement and test:

- [ ] Plugin source loading from configured paths
- [x] Server/runtime plugin hooks: **`(done — on_init, on_start, on_tool_call, on_message, on_session_end added to Plugin trait in crates/plugin/src/lib.rs)`**
  - `on_init` ✅
  - `on_start` ✅
  - `on_tool_call` ✅
  - `on_message` ✅
  - `on_session_end` ✅
- [ ] Hook execution order **`(PARTIAL — hooks execute in HashMap iteration order (non-deterministic); on_tool_call blocks on first error, others continue)`**
- [ ] Plugin-provided tool registration through standard tool registry **`(NOT STARTED — Plugin trait has no register_tool() method)`**
- [x] Failure containment — plugin errors do not crash runtime **`(done — hooks log warnings but don't panic; on_tool_call returns Err which caller handles)`**
- [ ] Server/runtime plugin config ownership (not mixed with TUI plugin config) **`(NOT STARTED — config ownership split not enforced)`**

Reference: `08-plugin-system.md`

#### TUI plugin API (`crates/tui/`)

Implement and test:

- [ ] `tui.json` plugin configuration ownership
- [ ] Plugin identity — runtime ID resolution, file vs npm
- [ ] Plugin deduplication before activation
- [ ] `plugin_enabled` semantics — enable/disable at runtime without uninstall
- [ ] TUI plugin module interface:
  - `commands.register()`
  - `routes.register()`
  - `dialogs.register()`
  - `slots.register()`
  - `themes.install()` / `themes.set()`
  - `events.on()`
  - `state.get()` / `state.set()`
  - `onDispose` lifecycle
- [ ] Runtime `api.plugins.activate()` / `api.plugins.deactivate()`
- [ ] Bounded cleanup with `AbortSignal`
- [ ] Theme auto-sync on install

Reference: `15-tui-plugin-api.md`

### Test Plan Alignment

| This phase implements | Test roadmap phase | Test backlog items |
|---|---|---|
| Agent system | Phase 2 | `crates/agent/` P1 items |
| Tools system | Phase 2 | `crates/tools/` P1 items |
| Plugin system | Phase 2 | `crates/plugin/` P1 items |
| TUI plugin API | Phase 2 | `crates/tui/` P1 items + `ratatui-testing/` |

### Exit Criteria

- All Phase 2 runtime tests pass
- `tests/runtime/` covers agent → tool → plugin cross interactions
- Convention tests for plugin boundaries and permission-before-execution are added
- Parent/child agent workflows are deterministic and isolated

---

## Phase 3 — Infrastructure Subsystems

**Prerequisites:** Phase 2 stable (agent + tools contracts established)

### Goals

Implement the infrastructure-facing subsystems that extend the runtime:

1. **MCP** (`04`) — local/remote MCP server integration
2. **LSP** (`05`) — language server protocol integration
3. **Providers / models** (`10`) — LLM provider abstraction and model selection
4. **Formatters** (`11`) — code formatting pipeline
5. **Skills** (`12`) — skill discovery and loading
6. **TUI** (`09`) — terminal UI behavior

### Deliverables

#### MCP integration (`crates/mcp/`)

Implement and test:

- [ ] Local MCP server connection and initialization
- [ ] Remote MCP server connection with URL
- [ ] Per-server OAuth configuration (nested under server entry, not top-level)
- [ ] Tool discovery from MCP servers
- [ ] Tool naming with server qualification
- [ ] Permission gating for MCP tools
- [ ] Timeout and unavailable-server handling
- [ ] Context cost warnings

Reference: `04-mcp-system.md`

#### LSP integration (`crates/lsp/`)

Implement and test:

- [ ] Built-in LSP server detection by language/file type
- [ ] Custom LSP server registration via config
- [ ] Diagnostics retrieval and surfacing
- [ ] LSP failure handling (missing server, startup failure)
- [ ] Experimental LSP tool behavior if feature-gated

Reference: `05-lsp-system.md`

#### Provider / model system (`crates/llm/` and/or provider crates)

Implement and test:

- [ ] Provider abstraction — registration, credential lookup
- [ ] Default model selection
- [ ] Per-agent model override
- [ ] Provider credential resolution (env, file, secret store)
- [ ] Local model provider support
- [ ] Variant / reasoning budget handling
- [ ] Unavailable-provider fallback behavior

Reference: `10-provider-model-system.md`

#### Formatters (`crates/tools/` or dedicated formatter crate)

Implement and test:

- [x] Formatter detection by file type **`(done — FormatterEngine in crates/core/src/formatter.rs)`**
- [x] Project config-based formatter selection **`(done — FormatterEngine::match_formatters())`**
- [x] Disable-all and per-formatter disable **`(done — FormatterConfig::Disabled, FormatterEntry.disabled)`**
- [x] Custom formatter command invocation **`(done — async Command execution with env vars)`**
- [ ] Automatic formatting on write/edit **`(not surveyed — likely in TUI)`**
- [x] Formatter absence / error handling **`(done — non-fatal, logs warnings)`**

Reference: `11-formatters.md`

#### Skills system (implementation area TBD — `crates/tools/`, `crates/core/`, or dedicated module)

Implement and test:

- [x] SKILL.md format support **`(done — SkillManager with frontmatter parsing in crates/core/src/skill.rs)`**
- [x] Discovery precedence: project → global → compat paths **`(done — global ~/.config/opencode/skills/ + project .opencode/skills/)`**
- [x] Deterministic duplicate resolution within a scope **`(done — priority-based ordering)`**
- [ ] Compatibility path loading for Claude/Agent-style directories **`(not surveyed)`**
- [x] Skill loading into runtime context **`(done — inject_into_prompt())`**
- [ ] Permission restrictions for skill usage **`(not surveyed)`**

Reference: `12-skills-system.md`

#### TUI (`crates/tui/`)

Implement and test:

- [ ] Session view — markdown rendering, syntax highlighting, diff display, tool details
- [ ] Slash commands — `/command` syntax and execution
- [ ] Input model:
  - multiline input
  - history navigation
  - autocomplete for commands, `@` file references, tool names
  - shell prefix handling
- [ ] Sidebar — file tree, MCP/LSP status, diagnostics, todo items
- [ ] Keybinding system — leader key, category organization
- [ ] Session/message navigation
- [ ] Home view, mode indicators

Reference: `09-tui-system.md`

### Test Plan Alignment

| This phase implements | Test roadmap phase | Test backlog items |
|---|---|---|
| MCP | Phase 3 | `crates/mcp/` P2 items |
| LSP | Phase 3 | `crates/lsp/` P2 items |
| Providers/models | Phase 3 | provider crate P2 items |
| Formatters | Phase 3 | formatter area P2 items |
| Skills | Phase 3 | skills area P2 items |
| TUI | Phase 3 | `crates/tui/` P1+P2 items |

### Exit Criteria

- Each subsystem has both positive and negative automated test coverage
- `tests/subsystem/` covers MCP, LSP, formatter, and skill workflows
- Subsystems can be exercised independently in CI

---

## Phase 4 — Interface Implementations

**Prerequisites:** Phase 2 stable (server/API, plugins); Phase 3 partial (TUI)

### Goals

Implement the external-facing interfaces:

1. **Desktop / web** (`13`) — desktop app, web interface, ACP
2. **GitHub / GitLab** (`14`) — CI/CD integration

### Deliverables

#### Desktop / web / ACP (`crates/server/`, interface crates)

Implement and test:

- [ ] Desktop app startup flow (shell integration) **`(NOT STARTED — stubs in crates/cli/)`**
- [ ] Web server mode **`(NOT STARTED — stub in crates/cli/src/cmd/web.rs)`**
- [ ] Auth-protected interface access **`(NOT STARTED)`**
- [ ] Session sharing between interface modes **`(NOT STARTED)`**
- [ ] ACP startup/handshake for editor integration **`(NOT STARTED — AcpAgentEvent structs exist in crates/control-plane/src/acp_stream.rs but no transport)`**
- [ ] Sharing behavior in managed vs restricted deployments **`(NOT STARTED)`**

Reference: `13-desktop-web-interface.md`

#### GitHub / GitLab integration (implementation area TBD)

Implement and test:

- [ ] GitHub workflow trigger examples
- [ ] Comment / PR trigger parsing
- [ ] CI secret loading for GitHub Actions
- [ ] GitLab CI component support (conditional, environment-dependent)
- [ ] GitLab Duo behavior marked explicitly as experimental/environment-dependent

Reference: `14-github-gitlab-integration.md`

### Test Plan Alignment

| This phase implements | Test roadmap phase | Test backlog items |
|---|---|---|
| Desktop/web/ACP | Phase 4 | `tests/interfaces/` P2+P3 items |
| GitHub/GitLab | Phase 4 | `tests/interfaces/` P3 items |

### Exit Criteria

- At least one smoke workflow per supported interface mode
- External integration tests clearly partitioned: required vs environment-dependent

---

## Phase 5 — Hardening

**Prerequisites:** Phases 1–4 substantially complete

### Goals

Lock in compatibility behavior, enforce conventions, and establish the final pre-release quality bar.

### Sub-phase 5a — Compatibility hardening

Implement and test:

- [ ] `tools` legacy alias regression suite
- [ ] Compatibility skill path regression suite
- [ ] Plugin ownership boundary regression suite
- [ ] Legacy route adapter tests (only if implemented)
- [ ] Persisted artifact compatibility tests (only if format versioning exists)

Reference: `tests/compatibility/`

### Sub-phase 5b — Convention enforcement

Implement and test:

- [x] Architecture boundary convention tests **`(done — 5 tests in tests/src/conventions/architecture_boundaries.rs)`**
- [x] Config ownership convention tests **`(done — 4 tests in tests/src/conventions/config_ownership.rs)`**
- [x] Route/resource-group convention tests **`(done — 4 tests in tests/src/conventions/route_conventions.rs)`**
- [x] Test-placement convention tests **`(done — 5 tests in tests/src/conventions/test_layout.rs)`**
- [x] TUI convention checks (requires `ratatui-testing`) **`(done — 5 tests in tests/src/conventions/tui_conventions.rs)`**
- [x] Plugin boundary convention tests **`(done — included in architecture_boundaries.rs)`**

Reference: `tests/conventions/`

### Exit Criteria

- `tests/compatibility/` covers all intended compatibility behaviors
- `tests/conventions/` fails with actionable messages on convention violations
- No PR can introduce silent architecture drift

---

## Phase 6 — Release Qualification

**Prerequisites:** Phases 1–5 complete

### Goals

Establish non-functional baselines and release gates.

### Deliverables

#### Non-functional testing

- [ ] Session creation / message append latency baselines
- [ ] Tool execution overhead measurements
- [ ] Config load/merge overhead baselines
- [ ] Server responsiveness under concurrent sessions
- [ ] Crash/restart recovery validation
- [ ] Snapshot/revert durability under interruption
- [ ] Secret redaction and auth enforcement checks
- [ ] Plugin cleanup reliability validation
- [ ] Observability — structured error reporting, diagnostic visibility

Reference: `tests/nonfunctional/`

### Release Gate

| Gate | Criteria |
|---|---|
| Authority | Phases 1 authority tests green |
| Runtime | Phase 2 tests green |
| Subsystems | Phase 3 tests green |
| Interfaces | Phase 4 smoke workflows pass |
| Compatibility | Phase 5a suite green |
| Conventions | Phase 5b suite green |
| Non-functional | Phase 6 baselines recorded, thresholds met |

---

## Implementation Dependencies Map

```
Phase 0
  └── workspace, harness scaffold

Phase 1
  ├── core entities ──────────────────┐
  ├── storage ───────────────────────┐ │
  ├── config system ───────────────┐ │ │
  └── HTTP API ───────────────────┘ │ │
        │                           │ │
Phase 2 │                           │ │
  ├── agent ───────────────────────┘ │
  ├── tools ────────────────────────┘
  ├── plugin ──────────────────────── all of Phase 1
  └── TUI plugin API ─────────────────┘

Phase 3
  ├── MCP ─────────────────────────── tools (Phase 2)
  ├── LSP ─────────────────────────── config (Phase 1)
  ├── providers ─────────────────────── config (Phase 1)
  ├── formatters ───────────────────── tools (Phase 2)
  ├── skills ───────────────────────── tools (Phase 2)
  └── TUI ──────────────────────────── TUI plugin API (Phase 2)

Phase 4
  ├── desktop/web/ACP ─────────────── server (Phase 1)
  └── GitHub/GitLab ───────────────── server (Phase 1), config (Phase 1)

Phase 5a ─────────────────────────── all prior phases
Phase 5b ─────────────────────────── all prior phases
Phase 6 ──────────────────────────── all prior phases
```

---

## Crate Ownership Summary

| Crate | Phase | PRD |
|---|---|---|
| `crates/core/` | 1 | `01`, `06` |
| `crates/storage/` | 1 | `01` |
| `crates/config/` | 1 | `06` |
| `crates/server/` | 1, 4 | `07`, `13` |
| `crates/agent/` | 2 | `02` |
| `crates/tools/` | 2, 3 | `03`, `11` |
| `crates/plugin/` | 2 | `08` |
| `crates/tui/` | 2, 3 | `09`, `15` |
| `crates/mcp/` | 3 | `04` |
| `crates/lsp/` | 3 | `05` |
| `crates/llm/` / provider crates | 3 | `10` |
| skills area | 3 | `12` |
| git area | TBD | `01` |
| `ratatui-testing/` | 2, 3 | `09`, `15` |

---

## Cross-References

- [01-core-architecture.md](./01-core-architecture.md)
- [02-agent-system.md](./02-agent-system.md)
- [03-tools-system.md](./03-tools-system.md)
- [04-mcp-system.md](./04-mcp-system.md)
- [05-lsp-system.md](./05-lsp-system.md)
- [06-configuration-system.md](./06-configuration-system.md)
- [07-server-api.md](./07-server-api.md)
- [08-plugin-system.md](./08-plugin-system.md)
- [09-tui-system.md](./09-tui-system.md)
- [10-provider-model-system.md](./10-provider-model-system.md)
- [11-formatters.md](./11-formatters.md)
- [12-skills-system.md](./12-skills-system.md)
- [13-desktop-web-interface.md](./13-desktop-web-interface.md)
- [14-github-gitlab-integration.md](./14-github-gitlab-integration.md)
- [15-tui-plugin-api.md](./15-tui-plugin-api.md)
- [16-test-plan.md](./16-test-plan.md)
- [17-rust-test-implementation-roadmap.md](./17-rust-test-implementation-roadmap.md)
- [18-crate-by-crate-test-backlog.md](./18-crate-by-crate-test-backlog.md)
