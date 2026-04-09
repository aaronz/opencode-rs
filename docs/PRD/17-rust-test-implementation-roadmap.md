# PRD: Rust Test Implementation Roadmap

## Scope

This document converts [16-test-plan.md](./16-test-plan.md) into an execution roadmap for implementing automated tests in the Rust monorepo.

It defines:

- test implementation phases
- crate/package ownership for major test areas
- recommended sequencing
- CI/release gates
- exit criteria for each milestone
- convention-test lane for enforcing repository and PRD structure

This roadmap assumes the monorepo structure described in `AGENTS.md`, especially:

- `opencode-rust/` — Rust implementation workspace
- `ratatui-testing/` — TUI testing library

---

## Goals

The roadmap must ensure the team builds tests in dependency order:

1. core invariants first
2. config/API authority second
3. runtime interactions third
4. user-visible workflows fourth
5. compatibility and convention gates next
6. non-functional gates last

This ordering reduces rework and ensures subsystem tests are built on stable authority-level contracts.

---

## Guiding Principles

- **Authority-first testing**: implement tests for `01`, `06`, and `07` before subsystem-heavy suites
- **Crate-local where possible**: unit tests should live near implementation crates
- **Integration at workspace boundary**: cross-crate workflows belong in integration test packages
- **Deterministic fixtures**: prefer `TempProject`, `MockServer`, and `MockLLMProvider` over ad hoc setup
- **TUI tests use `ratatui-testing`**: avoid manual terminal simulation if the shared library can cover it
- **Compatibility is explicit**: only build compatibility tests for behaviors intentionally preserved
- **Convention tests are structural**: use them to enforce ownership, layering, naming, and test-placement rules continuously

---

## Recommended Test Ownership by Workspace Area

### Core crates

| Area | Primary Location | Responsibility |
|---|---|---|
| Core entities, IDs, persistence invariants | `opencode-rust/crates/core/` | `Project`, `Session`, `Message`, `Part`, error types, serialization invariants |
| Config normalization and precedence | `opencode-rust/crates/core/` and/or config crate | config parsing, merging, normalization, variable expansion |
| Agent execution model | `opencode-rust/crates/agent/` | primary/subagent behavior, permission boundaries |
| Tool registry and execution pipeline | `opencode-rust/crates/tools/` | registry, validation, execution, caching, tool qualification |
| LSP integration | `opencode-rust/crates/lsp/` | server registration, diagnostics, custom config |
| Storage and snapshots | `opencode-rust/crates/storage/` | durability, recovery, checkpoint/revert behaviors |
| HTTP API | `opencode-rust/crates/server/` | route groups, auth, streaming, request validation |
| MCP integration | `opencode-rust/crates/mcp/` | server lifecycle, qualification, auth integration |
| Git integration | `opencode-rust/crates/git/` | worktree/session-adjacent VCS behaviors |
| TUI runtime | `opencode-rust/crates/tui/` | layout, commands, navigation, sidebar, plugin surfaces |

### Cross-workspace support

| Area | Primary Location | Responsibility |
|---|---|---|
| TUI rendering + interaction tests | `ratatui-testing/` + `opencode-rust/crates/tui/` | repeatable terminal UI assertions |
| End-to-end and workflow tests | `opencode-rust/tests/` | multi-crate flows, CLI/server/TUI integration |
| Shared test fixtures | `opencode-rust/tests/src/common/` | `TempProject`, `MockServer`, `MockLLMProvider`, future helpers |

---

## Phase Plan

## Phase 0 — Test Harness Foundation

### Purpose

Establish the shared testing infrastructure required by later phases.

### Work Items

- Standardize fixture helpers in `tests/src/common/`
- Add reusable builders for:
  - project/session/message graphs
  - config layering scenarios
  - permission-rule matrices
  - tool call/result fixtures
- Add server test harness helpers for authenticated and unauthenticated API requests
- Add plugin/MCP fake runtime harnesses if not already present
- Integrate `ratatui-testing` into TUI-focused test workflows

### Deliverables

- stable fixture modules
- mock provider/server helpers
- snapshot/checkpoint fixture helpers
- CI jobs that can run unit and integration suites separately

### Exit Criteria

- New tests can create a temp project/session with one helper path
- API tests can boot a test server in-process
- TUI tests can render and assert at least one simple screen transition

---

## Phase 1 — Authority Contract Tests

### Purpose

Lock down the most important invariants before subsystem behavior expands.

### PRD Coverage

- `01-core-architecture.md`
- `06-configuration-system.md`
- `07-server-api.md`

### Primary Ownership

- `crates/core/`
- `crates/storage/`
- `crates/server/`
- integration tests in `tests/`

### Work Items

#### Core architecture
- entity ownership tree tests
- identity stability tests
- ordering invariants for messages and parts
- fork lineage tests
- snapshot/revert model tests

#### Configuration
- precedence merge tests
- JSON vs JSONC parsing tests
- env/file expansion tests
- `tools` → `permission` normalization tests
- config ownership boundary tests (`opencode.json` vs `tui.json`)

#### Server API
- route-group presence tests
- auth-required vs public route tests
- request validation tests
- session/message API lifecycle tests
- compatibility adapter tests only if legacy routes are implemented

### Exit Criteria

- All authority tests pass in CI
- No PR touching core/config/api can merge without these suites

---

## Phase 2 — Runtime Interaction Tests

### Purpose

Validate the main runtime architecture on top of locked authority contracts.

### PRD Coverage

- `02-agent-system.md`
- `03-tools-system.md`
- `08-plugin-system.md`
- `15-tui-plugin-api.md`

### Primary Ownership

- `crates/agent/`
- `crates/tools/`
- `crates/plugin/`
- `crates/tui/`
- `tests/`

### Work Items

#### Agent system
- active-primary-agent invariants
- subagent/child-session behavior
- task result roundtrip tests
- permission inheritance/restriction tests

#### Tools system
- registry tests
- argument validation tests
- permission-before-execution tests
- built-in/custom/MCP common pipeline tests
- deterministic name resolution tests

#### Server/runtime plugins
- source loading tests
- hook invocation order tests
- plugin-provided tool registration tests
- failure containment tests

#### TUI plugin API
- plugin identity and dedupe tests
- `plugin_enabled` activation tests
- command/route/dialog/slot registration tests
- activate/deactivate and cleanup tests

### Exit Criteria

- parent/child agent workflows are stable
- plugin and tool failures are contained
- TUI plugin lifecycle can be tested non-interactively and repeatably

---

## Phase 3 — Subsystem Functional Tests

### Purpose

Expand coverage to user-visible and infrastructure-facing subsystems.

### PRD Coverage

- `04-mcp-system.md`
- `05-lsp-system.md`
- `09-tui-system.md`
- `10-provider-model-system.md`
- `11-formatters.md`
- `12-skills-system.md`

### Primary Ownership

- `crates/mcp/`
- `crates/lsp/`
- `crates/tui/`
- provider-related crates
- `crates/tools/`
- `tests/`

### Work Items

#### MCP
- local/remote connection tests
- timeout tests
- per-server OAuth tests
- permission gating tests
- unavailable-server tests

#### LSP
- built-in server selection tests
- custom server config tests
- diagnostics integration tests
- disabled/missing-server tests

#### TUI
- slash command tests
- input model tests
- sidebar and navigation tests
- message/tool detail rendering tests

#### Providers/models
- model selection precedence tests
- provider credential resolution tests
- local provider tests
- variant/reasoning configuration tests

#### Formatters
- detection tests by file type/config
- disable behavior tests
- custom formatter tests
- formatter failure tests

#### Skills
- discovery precedence tests
- compatibility path tests
- deterministic duplicate resolution tests
- skill loading/denial tests

### Exit Criteria

- Each subsystem has both positive and negative coverage
- Each subsystem can be exercised independently in CI

---

## Phase 4 — Interface and Workflow Tests

### Purpose

Validate real user workflows across process boundaries.

### PRD Coverage

- `13-desktop-web-interface.md`
- `14-github-gitlab-integration.md`
- cross-links to `07`, `09`, and `15`

### Primary Ownership

- `crates/server/`
- `crates/tui/`
- interface-specific crates if present
- `tests/`

### Work Items

#### Desktop / web / ACP
- startup and attach flows
- auth-protected interface access
- session sharing between interfaces where supported
- ACP startup/handshake tests

#### GitHub / GitLab
- workflow fixture validation
- trigger parsing tests
- CI secret-loading tests
- environment-dependent GitLab flows marked optional/skipped unless environment is present

### Exit Criteria

- At least one smoke workflow exists for each supported interface mode
- External integration tests are clearly partitioned into required vs environment-dependent

---

## Phase 5 — Compatibility and Regression Hardening

### Purpose

Make compatibility behavior explicit and prevent regressions as features expand.

### Work Items

- legacy config alias regression tests
- compatibility skill path regression tests
- plugin dedupe and ownership boundary regression tests
- optional legacy route adapter tests if implemented
- serialization backward-compatibility tests for persisted session artifacts where applicable

### Exit Criteria

- Intended compatibility behaviors are codified in dedicated suites
- Unsupported compatibility behavior fails explicitly, not accidentally

---

## Phase 5A — Convention Test Hardening

### Purpose

Continuously enforce structural conventions so the implementation does not drift away from the PRD ownership model or repository testing conventions.

### Work Items

- add architecture-boundary convention tests
- add config-ownership convention tests
- add route/resource-group convention tests
- add test-placement convention tests
- add TUI-testing convention checks to ensure UI interaction tests use `ratatui-testing`
- add naming/layering checks where they can be automated safely

### Exit Criteria

- convention suite is green in CI
- new tests and new crates cannot bypass agreed structural rules silently

---

## Phase 6 — Non-Functional and Release Qualification

### Purpose

Establish the final release gates required for shipping.

### Work Items

- performance baselines for session creation, tool execution, config loading, and API responsiveness
- crash/restart recovery tests
- snapshot/revert durability under interruption
- plugin cleanup reliability tests
- secret-handling and auth enforcement checks
- concurrency/isolation tests for multi-session workflows

### Exit Criteria

- baseline performance thresholds are recorded
- recovery/security suites are green
- release candidates cannot ship without full authority + runtime + subsystem + non-functional gates

---

## Recommended Test Suite Layout

```text
opencode-rust/
  crates/
    core/
      src/
      tests/
    agent/
      tests/
    tools/
      tests/
    storage/
      tests/
    server/
      tests/
    mcp/
      tests/
    lsp/
      tests/
    tui/
      tests/
  tests/
    src/
      common/
    authority/
    runtime/
    subsystem/
    interfaces/
    compatibility/
    conventions/
    nonfunctional/

ratatui-testing/
  tests/
```

---

## CI Pipeline Roadmap

### Stage 1 — Fast PR Gate

- `cargo fmt --all -- --check`
- `cargo clippy --all -- -D warnings`
- crate-local unit tests for touched crates

### Stage 2 — Required Integration Gate

- authority test suite
- runtime interaction suite
- selected subsystem integration suites

### Stage 3 — Extended Gate

- full workspace integration suite
- TUI rendering/plugin suite
- compatibility suite
- convention suite

### Stage 4 — Release Qualification Gate

- non-functional suite
- recovery suite
- full end-to-end smoke workflows

---

## Milestone-to-PRD Mapping

| Milestone | PRDs Covered | Must Be Green Before Next Phase |
|---|---|---|
| Phase 0 | harness support | yes |
| Phase 1 | `01`, `06`, `07` | yes |
| Phase 2 | `02`, `03`, `08`, `15` | yes |
| Phase 3 | `04`, `05`, `09`, `10`, `11`, `12` | yes |
| Phase 4 | `13`, `14` | yes |
| Phase 5 | compatibility overlays | yes |
| Phase 5A | convention enforcement | yes |
| Phase 6 | release qualification | release gate |

---

## Team Execution Recommendations

### Parallelization Opportunities

- One engineer owns authority tests (`01`, `06`, `07`)
- One engineer owns runtime tests (`02`, `03`)
- One engineer owns plugin/TUI tests (`08`, `09`, `15`, `ratatui-testing`)
- One engineer owns infra subsystems (`04`, `05`, `10`, `11`, `12`)
- One engineer owns interface/integration and CI qualification (`13`, `14`, non-functional)

### Sequencing Constraints

- Do not start broad TUI/plugin e2e before authority tests are stable
- Do not finalize external integration workflows before server/auth behavior is stable
- Do not add compatibility suites until canonical behavior is already passing

---

## Definition of Done

The roadmap is complete when:

1. Every PRD area in [16-test-plan.md](./16-test-plan.md) maps to implemented automated tests
2. Every implemented subsystem has negative tests
3. Authority contract regressions are blocked in CI
4. Release candidates pass full qualification gates
5. Test ownership is clear by crate/package and suite

---

## Cross-References

- [16-test-plan.md](./16-test-plan.md)
- [01-core-architecture.md](./01-core-architecture.md)
- [06-configuration-system.md](./06-configuration-system.md)
- [07-server-api.md](./07-server-api.md)
- [02-agent-system.md](./02-agent-system.md)
- [03-tools-system.md](./03-tools-system.md)
- [08-plugin-system.md](./08-plugin-system.md)
- [15-tui-plugin-api.md](./15-tui-plugin-api.md)
