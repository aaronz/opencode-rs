You are a senior Rust systems architect, AI coding agent architect, CLI/TUI expert, and engineering reviewer.

I am working on `opencode-rs`, a Rust-based implementation of an AI coding agent system inspired by opencode / Claude Code-like tools.

A local Ollama server is already running and may be used for testing real LLM-related flows.

Your task is to review the core modules of `opencode-rs` with special attention to BIG issues: architectural flaws, broken core flows, incomplete product capabilities, incorrect abstractions, reliability risks, state-management problems, and major gaps compared with intended opencode-like behavior.

This is not a formatting or minor cleanup task. Do not spend time on small stylistic issues unless they indicate a larger engineering problem.

==================================================
1. Review Priority
==================================================

Focus on high-impact issues only:

- Core flows that are broken or unreliable
- Architectural coupling that blocks future development
- Missing or incomplete core abstractions
- Incorrect state management
- Config/auth/provider/model persistence problems
- TUI lifecycle bugs that break user workflows
- CLI commands that do not actually work end to end
- LLM request/response flow bugs
- Async hangs, deadlocks, cancellation issues, or missing timeout behavior
- Logging/diagnostics failures that make debugging impossible
- Path/config conflicts with upstream opencode or other tools
- Security risks such as leaking API keys into logs
- Missing test harness coverage for critical user journeys
- Major capability gaps compared with opencode behavior

Ignore or defer:

- Pure formatting preferences
- Small naming improvements
- Micro-optimizations
- Cosmetic refactors
- Low-risk clippy suggestions
- Large rewrites that are not necessary to fix core behavior

==================================================
2. Core Modules to Review
==================================================

Inspect the repository and identify the actual core modules. Prioritize:

- CLI entrypoint and command dispatch
- TUI app state, event loop, dialogs, and screen transitions
- Provider/model registry and selection flow
- API key validation and persistence
- Config loading, migration, saving, and path resolution
- Agent/session orchestration
- LLM client abstraction and request lifecycle
- MCP integration boundary
- Skills / commands / hooks / rules loading if implemented
- Logging, tracing, and diagnostic output
- Test harness and integration/E2E tests

==================================================
3. Big-Issue Review Questions
==================================================

For each core area, answer:

- Can this feature work end to end for a real user?
- Is the current abstraction strong enough for multiple providers, agents, sessions, and tools?
- Is there a single source of truth for state, config, provider selection, and credentials?
- Are errors observable, actionable, and routed to the right place?
- Can failures be debugged without corrupting the TUI?
- Are async operations cancellable or timeout-protected?
- Are critical state transitions explicit and testable?
- Is the system safe from accidental secret leakage?
- Are core behaviors covered by meaningful tests rather than shallow mocks?
- Does the current implementation make future opencode parity easier or harder?

==================================================
4. Required Work Process
==================================================

Follow this process:

1. Inspect the repository structure.
2. Identify the core modules and summarize their responsibilities.
3. Build the project and run the existing test suite.
4. Execute or inspect the most important end-to-end flows:
   - start CLI
   - open TUI
   - configure provider
   - validate API key
   - persist API key/config
   - select model
   - start a session
   - send a prompt
   - receive or mock an LLM response
   - exit and restart with config preserved
5. Use the local Ollama server for real LLM smoke testing where possible.
6. Search for high-risk patterns:
   - broken state transitions
   - duplicated config sources
   - hardcoded paths
   - direct stdout/stderr writes during TUI rendering
   - blocking calls in async/UI paths
   - unbounded waits
   - swallowed errors
   - secret logging
   - incomplete placeholder logic
   - tests that pass without validating real behavior
7. Identify the biggest issues first.
8. Fix only the high-impact issues directly.
9. Add regression tests or E2E tests for the fixed issues.
10. Re-run formatting, linting, tests, and relevant manual flows.
11. Produce a final report ranked by severity.

==================================================
5. Local Ollama Testing Requirements
==================================================

A local Ollama server is already running.

Assume the local endpoint is:

```bash
http://localhost:11434

First inspect available local models:

ollama list

Use one available lightweight model for testing. If a specific model is needed but not installed, do not pull large models automatically unless explicitly approved. Prefer an already-installed small model.

Use Ollama testing to verify:

provider registration
provider selection
model discovery or model configuration
API key bypass behavior if Ollama does not require a key
session creation
prompt sending
streaming and non-streaming response handling
timeout behavior
cancellation behavior
error reporting when model/provider is unavailable
config persistence after restart
TUI flow from provider selection to model selection to session start

Recommended smoke test prompt:

Reply with exactly: opencode-rs-ollama-smoke-ok

Expected behavior:

The request reaches the local Ollama server.
The selected local model returns a response.
The response is rendered correctly in CLI/TUI.
Logs are written to log files, not into the TUI screen.
No fake API key is required for Ollama unless the project explicitly models one.
No secret or credential-like value is printed to logs.

If Ollama testing fails, classify the failure by severity:

P0 if the local provider flow cannot complete end to end.
P1 if the flow works only through manual config hacks or has poor diagnostics.
P2 if the flow works but lacks good tests or has minor UX problems.

Document:

Ollama endpoint used
Model selected
Commands run
Whether request/response worked
Relevant logs, with secrets redacted
Any code fixes made
Remaining gaps
==================================================
6. Fixing Rules

When fixing issues:

Prioritize correctness of core user journeys over cosmetic improvement.
Prefer targeted fixes that stabilize the architecture.
Do not perform broad rewrites unless a core flow cannot be fixed otherwise.
Do not hide major design problems behind shallow patches.
Avoid changing unrelated files.
Preserve public behavior unless it is clearly wrong.
Improve observability where debugging is currently impossible.
Ensure logs go to proper log files, not into the TUI screen.
Ensure secrets are redacted from logs and error output.
Ensure config paths are deterministic and opencode-rs specific.
Ensure provider/model/auth state has one clear source of truth.
Ensure validation dialogs, model selection dialogs, and session state transitions close/open correctly.
Ensure long-running provider validation or LLM calls have timeout/cancellation handling where appropriate.
Ensure every meaningful fix has a test.
==================================================
7. Severity Model

Classify findings by severity:

P0 — Core flow broken:

App cannot start
Provider setup unusable
API key cannot be saved
Model cannot be selected
Session cannot send prompt
TUI hangs indefinitely
Secrets leak into logs
Local Ollama provider flow cannot complete end to end

P1 — Major architecture or reliability issue:

Multiple conflicting config sources
Incorrect provider/model abstraction
Async task lifecycle is unsafe
Error handling prevents debugging
TUI state machine is fragile
Major opencode parity gap
Critical flows lack tests
Ollama flow only works through manual config hacks

P2 — Important but not blocking:

Incomplete diagnostics
Weak test coverage for secondary flows
Minor config migration gaps
Non-critical command behavior inconsistent
Ollama flow works but lacks good automated coverage

Do not spend time on P3 cosmetic issues unless they are caused by or reveal P0/P1 issues.

==================================================
8. Required Verification Commands

Run at minimum:

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

Also run relevant CLI/TUI smoke tests and local Ollama smoke tests.

If a command fails, do not ignore it. Determine whether it is related to your changes, pre-existing, or caused by a bigger issue. Document the result clearly.

==================================================
9. Final Report Format

Return a concrete engineering report:

# opencode-rs Core Module Big-Issue Review

## Executive Summary
- Overall status:
- Biggest risks:
- Core flows verified:
- Core flows still broken:

## Local Ollama Test Summary
- Endpoint:
- Model used:
- Smoke prompt:
- Result:
- Logs reviewed:
- Issues found:

## Core Modules Reviewed
| Module | Responsibility | Review Result | Risk Level |
|---|---|---|---|

## P0 Findings
| Finding | Evidence | Fix Implemented | Verification |
|---|---|---|---|

## P1 Findings
| Finding | Evidence | Fix Implemented | Verification |
|---|---|---|---|

## P2 Findings
| Finding | Evidence | Fix Implemented / Deferred | Verification |
|---|---|---|---|

## Fixes Implemented
- ...

## Tests Added or Updated
- ...

## Commands Run
| Command | Result | Notes |
|---|---|---|

## Remaining Risks
- ...

## Recommended Next Steps
- ...
==================================================
10. Hard Constraints
Focus on big issues, not cosmetic cleanup.
Use local Ollama for real LLM flow testing where possible.
Do not pull large Ollama models automatically without approval.
Do not only review; fix high-impact bugs directly.
Do not perform unnecessary rewrites.
Do not use a real API key unless explicitly provided.
Do not leak secrets in logs, snapshots, tests, or reports.
Do not change unrelated files.
Do not claim a flow works unless you verified it.
Be explicit about anything that remains unresolved.

Start by inspecting the repository structure, then check local Ollama availability, then proceed through the core flows.