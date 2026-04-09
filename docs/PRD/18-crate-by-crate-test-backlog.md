# PRD: Crate-by-Crate Test Backlog

## Scope

This document converts the phased roadmap in [17-rust-test-implementation-roadmap.md](./17-rust-test-implementation-roadmap.md) into a concrete backlog organized by crate/package.

Each backlog section includes:

- test scope
- concrete task slices
- dependencies
- acceptance criteria

This backlog is intended to be execution-oriented and should be used together with:

- [16-test-plan.md](./16-test-plan.md)
- [17-rust-test-implementation-roadmap.md](./17-rust-test-implementation-roadmap.md)

---

## Priority Model

| Priority | Meaning |
|---|---|
| P0 | Blocks authority-level correctness or shared harness usage |
| P1 | Blocks runtime subsystem completion |
| P2 | Important subsystem depth / compatibility coverage |
| P3 | Nice-to-have hardening or optimization coverage |

---

## Shared Test Infrastructure

### `rust-opencode-port/tests/src/common/`

**Purpose**
- Shared fixtures and helpers for all integration and end-to-end suites

**Backlog**

#### P0
- Add `TempProject` builder variants for:
  - git workspace
  - non-git workspace
  - nested directory execution
- Add `SessionFixture` builder for project/session/message/part graphs
- Add config fixture builder for:
  - global config
  - project config
  - inline config
  - invalid config cases
- Add permission matrix helper for `allow` / `ask` / `deny` patterns
- Add API auth helper for authenticated vs unauthenticated requests

#### P1
- Add snapshot/checkpoint fixture helpers
- Add plugin test harness helpers
- Add MCP fake server helpers
- Add provider mock helpers beyond `MockLLMProvider` as needed

**Dependencies**
- none; this should start first

**Acceptance criteria**
- any integration test can set up a temp project and session in <10 lines
- any API test can create an authenticated request with one helper

---

## Authority Crates

### `rust-opencode-port/crates/core/`

**PRD coverage**
- `01-core-architecture.md`
- `06-configuration-system.md` (normalization portions)
- `02-agent-system.md` (shared types only)

**Backlog**

#### P0
- Add unit tests for stable ID/entity construction
- Add serialization/deserialization tests for project/session/message/part models
- Add ownership invariant tests:
  - message requires session
  - session requires project
  - part requires message
- Add ordering invariant tests for messages and parts
- Add fork lineage model tests

#### P0
- Add config parse tests for JSON and JSONC
- Add config precedence merge tests
- Add `tools` → `permission` normalization tests
- Add environment/file variable expansion tests
- Add invalid config rejection tests

#### P1
- Add permission glob matching tests if implemented in core/shared layer
- Add auth/secret storage structure tests if owned here

#### P2
- Add property-style tests for config merge determinism
- Add fuzz-like invalid payload tests for serialization boundaries

**Dependencies**
- shared fixture helpers for config/project/session builders

**Acceptance criteria**
- core model invariants and config normalization are fully automated
- no authority-level merge/model regression can bypass crate-local tests

---

### `rust-opencode-port/crates/storage/`

**PRD coverage**
- `01-core-architecture.md`

**Backlog**

#### P0
- Add persistence roundtrip tests for project/session/message state
- Add session recovery tests after simulated restart
- Add snapshot/checkpoint create/load tests
- Add revert-to-checkpoint tests

#### P1
- Add compaction-safe persistence tests
- Add corrupted storage handling tests
- Add concurrent session isolation tests

#### P2
- Add migration/backward-compat tests for persisted artifacts if versioning is introduced

**Dependencies**
- core entity fixtures

**Acceptance criteria**
- restart/recovery/revert are verified without relying on manual testing

---

### `rust-opencode-port/crates/server/`

**PRD coverage**
- `07-server-api.md`
- portions of `13-desktop-web-interface.md`

**Backlog**

#### P0
- Add route-group presence tests
- Add auth-required vs public endpoint tests
- Add request validation tests for session/project/message routes
- Add session lifecycle API tests:
  - create
  - list/get
  - append/send message
  - abort/fork/share/revert where implemented

#### P1
- Add provider/config endpoint tests
- Add permission-response endpoint tests
- Add streaming transport tests (SSE and/or websocket if supported)
- Add API error-shape consistency tests

#### P2
- Add compatibility adapter tests for legacy routes only if implemented
- Add concurrency tests for multiple active sessions

**Dependencies**
- shared server harness
- storage/core fixtures

**Acceptance criteria**
- canonical API resource groups are covered by automated integration tests
- unauthorized and malformed requests fail predictably

---

## Runtime Crates

### `rust-opencode-port/crates/agent/`

**PRD coverage**
- `02-agent-system.md`

**Backlog**

#### P1
- Add tests for one-active-primary-agent semantics
- Add subagent child-context creation tests
- Add parent/subagent result handoff tests
- Add permission inheritance tests
- Add tighter-runtime-restriction tests for subagents

#### P2
- Add hidden-vs-visible agent selection tests
- Add session agent swap tests where supported

**Dependencies**
- core session fixtures
- tools mock/stub harnesses

**Acceptance criteria**
- parent/child agent execution behavior is deterministic and isolated

---

### `rust-opencode-port/crates/tools/`

**PRD coverage**
- `03-tools-system.md`

**Backlog**

#### P1
- Add registry registration tests
- Add built-in tool lookup tests
- Add custom tool discovery tests
- Add permission-before-execution tests
- Add argument validation failure tests
- Add structured tool error tests

#### P1
- Add common pipeline tests for:
  - built-in tools
  - custom tools
  - MCP-qualified tools

#### P2
- Add deterministic collision/name resolution tests
- Add cache behavior tests for cacheable tools
- Add non-cacheable behavior tests for shell/side-effecting tools
- Add ignore-file semantics tests if implementation supports them

**Dependencies**
- config permission fixtures
- plugin/MCP fake integrations

**Acceptance criteria**
- all tool categories share one verified execution pipeline

---

### `rust-opencode-port/crates/plugin/`

**PRD coverage**
- `08-plugin-system.md`

**Backlog**

#### P1
- Add server/runtime plugin source loading tests
- Add plugin hook registration and execution-order tests
- Add plugin-provided tool registration tests
- Add plugin failure containment tests

#### P2
- Add plugin config ownership boundary tests vs TUI plugins
- Add plugin unload/cleanup tests where supported

**Dependencies**
- tools registry harness
- config fixtures

**Acceptance criteria**
- plugin failures cannot silently corrupt runtime state

---

### `rust-opencode-port/crates/tui/`

**PRD coverage**
- `09-tui-system.md`
- `15-tui-plugin-api.md`

**Backlog**

#### P1
- Add slash command tests
- Add input model tests:
  - multiline input
  - history
  - autocomplete
  - file references
  - shell prefix handling
- Add sidebar visibility tests
- Add session/message navigation tests

#### P1
- Add TUI plugin registration tests:
  - commands
  - routes
  - dialogs
  - slots
  - themes
  - events

#### P1
- Add `plugin_enabled` activation/deactivation tests
- Add plugin cleanup/disposal tests

#### P2
- Add theme persistence/switching tests
- Add message/tool detail rendering tests
- Add child-session navigation tests if implemented

**Dependencies**
- `ratatui-testing/`
- shared session/config fixtures

**Acceptance criteria**
- TUI behavior is testable without manual terminal interaction
- TUI plugin lifecycle is repeatable and deterministic

---

### `ratatui-testing/`

**Purpose**
- Shared TUI testing substrate used by `crates/tui/`

**Backlog**

#### P0
- Add/assert stable screen snapshot helpers if missing
- Add key-event simulation helpers needed by current TUI PRDs
- Add support for dialog/route/sidebar assertions needed by plugin/TUI tests

#### P1
- Add plugin lifecycle assertion helpers
- Add async rendering/wait helpers for dynamic updates

#### P2
- Add richer diff output for failing UI assertions

**Dependencies**
- none; should progress in parallel with early TUI testing needs

**Acceptance criteria**
- TUI crate tests do not need brittle custom terminal harnesses

---

## Infrastructure / Integration Crates

### `rust-opencode-port/crates/mcp/`

**PRD coverage**
- `04-mcp-system.md`

**Backlog**

#### P2
- Add local server connection tests
- Add remote server connection tests
- Add per-server OAuth config tests
- Add timeout and unavailable-server tests
- Add tool qualification/naming tests
- Add permission-gated execution tests

**Dependencies**
- tools registry integration
- server/client mocks

**Acceptance criteria**
- MCP behavior is reliable under local, remote, timeout, and auth scenarios

---

### `rust-opencode-port/crates/lsp/`

**PRD coverage**
- `05-lsp-system.md`

**Backlog**

#### P2
- Add built-in server selection tests
- Add custom server config tests
- Add diagnostics retrieval tests
- Add disabled-server tests
- Add missing-server / startup-failure tests

#### P3
- Add experimental LSP tool tests if feature-gated behavior exists

**Dependencies**
- config fixtures
- mock language server harnesses

**Acceptance criteria**
- LSP integration fails gracefully and reports diagnostics correctly

---

### Provider-related crates (`crates/llm/`, `crates/auth/`, related packages)

**PRD coverage**
- `10-provider-model-system.md`

**Backlog**

#### P2
- Add default model resolution tests
- Add per-agent model override tests
- Add provider credential lookup tests
- Add local provider tests
- Add variant/reasoning setting tests

#### P3
- Add unavailable-provider and fallback tests where supported
- Add managed/curated offering tests if implemented in this repo

**Dependencies**
- config fixtures
- mock provider implementations

**Acceptance criteria**
- model/provider selection is deterministic and credential handling is covered

---

### Formatter-related implementation area (`crates/tools/` and/or dedicated formatter crate)

**PRD coverage**
- `11-formatters.md`

**Backlog**

#### P2
- Add formatter selection/detection tests
- Add disable-all and per-formatter disable tests
- Add custom formatter command tests
- Add post-write/post-edit formatting trigger tests
- Add formatter failure-path tests

**Dependencies**
- file fixture helpers
- tool execution harness

**Acceptance criteria**
- formatter behavior is automated for both success and failure paths

---

### Skills-related implementation area (`crates/tools/`, `crates/core/`, or dedicated skills module)

**PRD coverage**
- `12-skills-system.md`

**Backlog**

#### P2
- Add project/global/compat-path discovery tests
- Add precedence-order tests
- Add deterministic duplicate resolution tests
- Add skill load success/failure tests
- Add permission denial tests for skill usage

**Dependencies**
- fixture directories on disk
- config/permission helpers

**Acceptance criteria**
- skill resolution is deterministic across supported paths

---

## Cross-Crate Integration Suites

### `rust-opencode-port/tests/authority/`

**Backlog**

#### P0
- end-to-end authority coverage spanning core + storage + server
- config ownership boundary tests across main config and TUI config
- session lifecycle API + persistence integration tests

**Acceptance criteria**
- authority contracts are enforced across crate boundaries, not only inside unit tests

---

### `rust-opencode-port/tests/runtime/`

**Backlog**

#### P1
- agent + tools + permissions integration suite
- plugin-provided tools through standard execution pipeline
- parent/subagent workflow scenarios

**Acceptance criteria**
- runtime behavior matches the PRD interaction model under real crate composition

---

### `rust-opencode-port/tests/subsystem/`

**Backlog**

#### P2
- MCP integration workflows
- LSP diagnostics workflows
- provider/model integration workflows
- formatter-trigger workflows
- skills discovery/loading workflows

**Acceptance criteria**
- each subsystem has at least one positive and one negative integration workflow

---

### `rust-opencode-port/tests/interfaces/`

**Backlog**

#### P2
- TUI smoke workflows
- local server/web interface smoke workflows
- ACP startup/attach workflow tests if supported

#### P3
- GitHub workflow fixture tests
- environment-gated GitLab flow tests

**Acceptance criteria**
- at least one end-to-end happy-path workflow exists per supported interface mode

---

### `rust-opencode-port/tests/compatibility/`

**Backlog**

#### P2
- `tools` alias regression tests
- compat skill path regression tests
- plugin ownership boundary regressions

#### P3
- legacy route adapter tests only if implemented
- persisted artifact compatibility tests only if format versioning exists

**Acceptance criteria**
- intended compatibility behavior is explicit and locked by tests

---

### `rust-opencode-port/tests/nonfunctional/`

**Backlog**

#### P3
- session creation/message append performance benchmarks
- server responsiveness under concurrent sessions
- crash/restart recovery tests
- snapshot/revert durability tests under interruption
- secret redaction/auth enforcement checks
- plugin cleanup reliability tests

**Acceptance criteria**
- release qualification metrics are captured and enforced

---

## Recommended Delivery Order

1. `tests/src/common/`
2. `crates/core/`
3. `crates/storage/`
4. `crates/server/`
5. `tests/authority/`
6. `ratatui-testing/`
7. `crates/tools/`
8. `crates/agent/`
9. `crates/plugin/`
10. `crates/tui/`
11. `tests/runtime/`
12. `crates/mcp/`
13. `crates/lsp/`
14. provider/formatter/skills areas
15. `tests/subsystem/`
16. `tests/interfaces/`
17. `tests/compatibility/`
18. `tests/nonfunctional/`

---

## Backlog Completion Rules

An item is complete only when:

- the automated test exists
- it runs in the intended suite
- it has a clear positive or negative assertion
- it is traceable to a PRD area
- it does not duplicate ownership already covered by another suite unless it is explicitly cross-crate integration coverage

---

## Cross-References

- [16-test-plan.md](./16-test-plan.md)
- [17-rust-test-implementation-roadmap.md](./17-rust-test-implementation-roadmap.md)
