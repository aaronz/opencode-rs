# opencode-rs CLI Product Requirements Document

## 1. Executive Summary

`opencode-rs` is a Rust-based AI coding agent system inspired by opencode-rs / Claude Code-like developer tools. The CLI is the primary control surface for coding sessions, repo-aware automation, agent execution, context management, provider configuration, MCP integration, Git workflows, debugging, validation, and future TUI/Desktop/CI integrations.

The CLI must not be a thin command launcher. It must behave as a structured, observable, safe, scriptable, repo-aware AI coding runtime.

Core design goals:

* Make simple AI coding tasks easy.
* Make advanced agent workflows transparent and controllable.
* Make automation and CI/CD reliable.
* Make context, LLM requests, tool calls, file edits, and validation inspectable.
* Make the CLI architecture reusable by future TUI, desktop UI, remote server, and platform integrations.
* Provide strong Rust-native reliability, performance, typed schemas, and testability.

---

## 2. Product Context

### 2.1 Product Definition

`opencode-rs` is a local-first AI coding agent system for developers and teams. It runs inside or against a code repository and coordinates:

* LLM providers
* Agents and subagents
* Skills
* User-defined commands
* Hooks
* Rules
* MCP servers
* Git operations
* File edits
* Shell/tool execution
* Build/test/validation loops
* Logs, traces, and replay

### 2.2 CLI Role

The CLI is responsible for:

1. Starting and managing sessions.
2. Building repo-aware context.
3. Executing AI coding tasks.
4. Showing plans, diffs, validations, and decisions.
5. Managing local and repo configuration.
6. Managing providers, models, authentication, MCP, agents, skills, hooks, and rules.
7. Supporting both human interactive usage and deterministic non-interactive automation.

### 2.3 Future Compatibility

The CLI must expose internal services in a way that can later power:

* TUI
* Desktop UI
* Remote control app
* Web dashboard
* CI/CD usage
* Team platform integrations
* Agent debugging UI

---

## 3. Goals and Non-Goals

### 3.1 Goals

| Goal                       | Description                                                                                              |
| -------------------------- | -------------------------------------------------------------------------------------------------------- |
| Terminal-native AI coding  | Provide excellent terminal-first interaction for daily coding work.                                      |
| Repo-aware execution       | Understand project structure, Git state, build/test commands, rules, and context.                        |
| Scriptable automation      | Support stable command behavior, JSON/NDJSON output, exit codes, and dry-run mode.                       |
| Transparent agent runtime  | Make planning, context, LLM calls, tool calls, edits, and validation inspectable.                        |
| Safe by default            | Require approval for dangerous edits, shell commands, network calls, Git mutations, and sensitive files. |
| Extensible command system  | Support agents, skills, custom commands, hooks, rules, and MCP servers.                                  |
| Rust-native implementation | Use strongly typed config, async execution, structured errors, tracing, and robust testing.              |
| CI/CD compatibility        | Provide non-interactive mode, policy enforcement, machine-readable output, and stable failures.          |
| Compatibility path         | Support migration from original opencode-rs where practical.                                                |

### 3.2 Non-Goals

| Non-Goal                             | Explanation                                                                             |
| ------------------------------------ | --------------------------------------------------------------------------------------- |
| Fully autonomous uncontrolled coding | The system should support autonomy, but with explicit permission and policy boundaries. |
| Replacing Git, CI, or IDEs           | The CLI integrates with them instead of replacing them.                                 |
| Provider lock-in                     | The system should support multiple providers and custom proxies.                        |
| Desktop-first UX                     | Desktop UI is future work; CLI architecture must enable it.                             |
| Hidden agent execution               | Agent behavior must be observable and debuggable.                                       |

---

## 4. Target Users and Personas

### 4.1 Individual Developer

**Profile**

* Works in local repositories.
* Wants fast help for coding, debugging, refactoring, tests, and explanations.
* Prefers simple commands and predictable behavior.

**Needs**

* `opencode-rs init`
* `opencode-rs chat`
* `opencode-rs run`
* `opencode-rs fix`
* `opencode-rs test`
* `opencode-rs diff`
* `opencode-rs apply`
* Easy provider setup.
* Clear approval prompts.
* Simple logs and session resume.

**Success Criteria**

* Can install, configure provider, initialize repo, and run first useful coding task within minutes.
* Can inspect and approve changes before applying.
* Can resume previous sessions.

---

### 4.2 AI Coding Power User

**Profile**

* Uses custom prompts, rules, skills, commands, hooks, and subagents.
* Wants precise context control.
* Wants to inspect LLM requests, tool calls, and context payloads.

**Needs**

* `opencode-rs context show`
* `opencode-rs context export`
* `opencode-rs trace show`
* `opencode-rs agent run`
* `opencode-rs skill run`
* `opencode-rs hook validate`
* `opencode-rs rule validate`
* Custom permission modes.
* Replay and diagnostics.

**Success Criteria**

* Can define reusable coding workflows.
* Can debug why an agent made a decision.
* Can reproduce failed runs.

---

### 4.3 Team / Enterprise Developer

**Profile**

* Works in team repositories with shared conventions.
* Needs policy enforcement, auditability, secret handling, reproducible workflows.

**Needs**

* Repo-level `.opencode/config.toml`
* Shared `.opencode/rules/`
* Shared `.opencode/skills/`
* Policy files.
* Audit logs.
* Secret redaction.
* CI-compatible output.
* Provider routing through enterprise proxy.

**Success Criteria**

* Team can share standard rules and workflows.
* Sensitive files and secrets are protected.
* CI can enforce AI-generated change validation.

---

### 4.4 Toolchain / Platform Engineer

**Profile**

* Integrates `opencode-rs` into CI/CD, repo templates, internal dev platforms, and automation.

**Needs**

* Stable CLI contracts.
* JSON/NDJSON output.
* Stable exit codes.
* Headless execution.
* Config migration.
* Provider/proxy templates.
* Diagnostics export.

**Success Criteria**

* Can integrate CLI into pipelines without brittle text parsing.
* Can enforce policy and collect telemetry.
* Can template repo initialization.

---

### 4.5 Debugger / Maintainer

**Profile**

* Investigates provider failures, agent behavior, MCP errors, hook failures, context bugs, and replay issues.

**Needs**

* `opencode-rs logs`
* `opencode-rs trace`
* `opencode-rs replay`
* `opencode-rs diagnostics export`
* LLM request/response inspection with redaction.
* Tool call history.
* Context snapshots.
* Provider/MCP latency.

**Success Criteria**

* Can diagnose “model not found”, missing context, failed tool calls, and invalid config quickly.
* Can export a diagnostic bundle for issue reproduction.

---

## 5. Product Principles

| Principle                          | Requirement                                                                                                                |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Terminal-native first              | CLI must feel natural in terminal, support pipes, shell scripting, TTY detection, colors, pagers, editors, and completion. |
| Local-first and repo-aware         | Default execution should use local repo state, Git state, config, tests, build scripts, and rules.                         |
| Scriptable and automation-friendly | Every important workflow must support non-interactive mode and machine-readable output.                                    |
| Transparent agent execution        | Plans, context, prompts, tool calls, diffs, validations, and errors must be inspectable.                                   |
| Deterministic where possible       | Config resolution, command behavior, exit codes, and validation flow should be stable.                                     |
| Safe by default                    | Default permission mode should ask before writes, shell commands, network calls, and Git mutations.                        |
| Composable command model           | Small command groups should compose into workflows.                                                                        |
| Rust-native reliability            | Use typed data models, structured errors, async runtime, strong tests, and predictable performance.                        |
| Configurable without chaos         | Config layering must be explicit and explainable.                                                                          |
| Beginner-friendly, expert-powerful | Common workflows should be simple; advanced control should be available through flags/config.                              |
| Observable and debuggable          | Logs, traces, replay, diagnostics, and redaction are first-class features.                                                 |
| Future UI compatible               | CLI runtime should expose reusable services for TUI/Desktop/remote control.                                                |
| CI/CD compatible                   | Headless mode, JSON output, stable exit codes, and policies are mandatory.                                                 |

---

## 6. CLI Command Information Architecture

### 6.1 Top-Level Command Shape

```bash
opencode-rs [GLOBAL_FLAGS] <COMMAND> [COMMAND_FLAGS] [ARGS]
```

### 6.2 Global Flags

| Flag                        | Description                                               |
| --------------------------- | --------------------------------------------------------- |
| `--config <path>`           | Use explicit config file.                                 |
| `--repo <path>`             | Run against a specific repository path.                   |
| `--session <id>`            | Attach command to existing session.                       |
| `--provider <name>`         | Override provider.                                        |
| `--model <name>`            | Override model.                                           |
| `--agent <name>`            | Override default agent.                                   |
| `--output text/json/ndjson` | Select output format.                                     |
| `--quiet`                   | Suppress non-essential human output.                      |
| `--verbose`                 | Show detailed progress.                                   |
| `--trace`                   | Enable trace-level output and trace capture.              |
| `--no-color`                | Disable ANSI colors.                                      |
| `--yes`                     | Auto-confirm safe prompts according to permission policy. |
| `--dry-run`                 | Plan and preview without applying changes.                |
| `--ci`                      | Enable CI-safe non-interactive behavior.                  |
| `--permission-mode <mode>`  | Override permission mode.                                 |

### 6.3 Command Groups

| Command Group         | Purpose                                                   |
| --------------------- | --------------------------------------------------------- |
| `opencode-rs init`       | Initialize repo configuration and local metadata.         |
| `opencode-rs chat`       | Start or resume interactive AI coding chat.               |
| `opencode-rs run`        | Execute one-shot natural language coding task.            |
| `opencode-rs plan`       | Generate an implementation plan without applying changes. |
| `opencode-rs apply`      | Apply generated or approved changes.                      |
| `opencode-rs review`     | Review code, diffs, branch, or PR.                        |
| `opencode-rs test`       | Generate, run, or fix tests.                              |
| `opencode-rs fix`        | Diagnose and fix errors, tests, builds, or lint issues.   |
| `opencode-rs explain`    | Explain code, architecture, errors, or diffs.             |
| `opencode-rs refactor`   | Perform controlled refactoring.                           |
| `opencode-rs agent`      | Manage and run agents/subagents.                          |
| `opencode-rs skill`      | Manage reusable skills.                                   |
| `opencode-rs command`    | Manage custom user/team commands.                         |
| `opencode-rs hook`       | Manage lifecycle hooks.                                   |
| `opencode-rs rule`       | Manage coding and policy rules.                           |
| `opencode-rs mcp`        | Manage MCP servers, tools, and resources.                 |
| `opencode-rs provider`   | Manage LLM providers.                                     |
| `opencode-rs model`      | Manage model metadata and tests.                          |
| `opencode-rs session`    | Manage persisted sessions.                                |
| `opencode-rs context`    | Build, inspect, and debug context.                        |
| `opencode-rs repo`       | Inspect repo metadata and project detection.              |
| `opencode-rs git`        | Git-aware AI workflows.                                   |
| `opencode-rs diff`       | Show and manage generated diffs.                          |
| `opencode-rs validate`   | Run validation gates.                                     |
| `opencode-rs doctor`     | Check environment health.                                 |
| `opencode-rs config`     | Inspect, edit, validate, migrate config.                  |
| `opencode-rs auth`       | Manage provider authentication.                           |
| `opencode-rs logs`       | Inspect logs.                                             |
| `opencode-rs trace`      | Inspect structured traces.                                |
| `opencode-rs replay`     | Replay failed or previous runs.                           |
| `opencode-rs serve`      | Start local service for TUI/Desktop/remote integrations.  |
| `opencode-rs tui`        | Launch future terminal UI.                                |
| `opencode-rs upgrade`    | Upgrade CLI and internal schemas.                         |
| `opencode-rs completion` | Generate shell completions.                               |

---

## 7. Core Workflows

## 7.1 First-Time Setup

### User Story

As a new developer, I want to install and configure `opencode-rs` so that I can run my first coding task safely.

### Flow

```bash
cargo install opencode-rs
opencode-rs doctor
opencode-rs provider add openai-compatible \
  --base-url https://api.example.com/v1 \
  --api-key-env OPENCODE_API_KEY
opencode-rs model list --provider openai-compatible
opencode-rs auth status
cd my-repo
opencode-rs init
opencode-rs run "Explain this repository and identify the main entry points"
```

### Expected Behavior

1. CLI checks terminal, Git, config directory, keychain availability, provider reachability.
2. Provider is saved in user config.
3. API key is not written to plaintext config if keychain or env var is used.
4. `opencode-rs init` creates `.opencode/`.
5. First `run` creates a session, builds context, calls model, stores trace and logs.

### Acceptance Criteria

* New user can complete setup without editing config manually.
* `doctor` reports missing provider/auth clearly.
* Secrets are redacted from logs and traces.
* First session can be resumed.

---

## 7.2 Repo Initialization

```bash
opencode-rs init
opencode-rs init --template rust
opencode-rs init --force
opencode-rs init --dry-run
```

### Steps

1. Detect Git root.
2. Detect language/framework/package manager.
3. Detect build/test/lint commands.
4. Create `.opencode/` structure.
5. Generate default config.
6. Generate default rules.
7. Generate default hooks.
8. Index repo files.
9. Create initial context summary.
10. Validate generated config.

### Generated Structure

```text
.opencode/
  config.toml
  agents/
    default.toml
    reviewer.toml
  skills/
  commands/
  hooks/
  rules/
    coding.md
    testing.md
    security.md
  sessions/
  logs/
  traces/
  mcp.json
  context/
    index.json
    summaries/
```

### Acceptance Criteria

* Does not overwrite existing files unless `--force`.
* Shows a dry-run diff when requested.
* Produces valid config.
* Detects Rust, Node, Python, Java, Go, and generic repos.

---

## 7.3 One-Shot Coding Task

```bash
opencode-rs run "Add CLI flag --json to the report command"
```

### Steps

1. Resolve repo and config.
2. Create session.
3. Build context.
4. Load agent.
5. Generate plan.
6. Request approval if required.
7. Execute edits.
8. Show diff.
9. Run validation.
10. Ask to apply/commit if interactive.
11. Persist session, trace, logs.

### Non-Interactive CI Example

```bash
opencode-rs run "Fix clippy warnings" \
  --ci \
  --yes \
  --output ndjson \
  --validate \
  --max-iterations 3
```

### Acceptance Criteria

* In interactive mode, file edits require approval unless policy allows.
* In CI mode, no prompt is emitted.
* Exit code reflects final result.
* Diff is inspectable before apply unless `--yes` and policy permit.

---

## 7.4 Interactive Chat Task

```bash
opencode-rs chat
opencode-rs chat --session latest
opencode-rs chat --include src/main.rs
```

### Capabilities

* Persistent conversation.
* File references.
* Tool calls.
* Inline diff preview.
* Approval prompts.
* Slash commands.

### Example Slash Commands

```text
/context show
/include src/cli.rs
/exclude target/
/diff
/validate
/apply
/session fork
/trace show
```

### Acceptance Criteria

* Chat can resume previous state.
* User can reference files explicitly.
* Tool calls and file edits are visible.
* Session history is persisted.

---

## 7.5 Plan-First Workflow

```bash
opencode-rs plan "Refactor provider abstraction to support fallback routing"
opencode-rs apply --plan .opencode/sessions/abc123/plan.json
opencode-rs validate
```

### Requirements

* Plan is generated as both human-readable Markdown and machine-readable JSON.
* Plan contains files affected, risks, validation strategy, and rollback notes.
* Apply requires explicit approval unless policy allows.

### Acceptance Criteria

* Plan can be reviewed before execution.
* Plan can be exported.
* Apply can resume from plan.

---

## 7.6 Multi-Agent Workflow

```bash
opencode-rs run "Implement streaming NDJSON output" \
  --workflow planner,implementer,reviewer,tester
```

### Agents

| Agent             | Responsibility                                    |
| ----------------- | ------------------------------------------------- |
| Planner           | Break task into implementation plan.              |
| Implementer       | Modify code.                                      |
| Reviewer          | Review diff and architecture impact.              |
| Tester            | Run/generate tests.                               |
| Security Reviewer | Check secrets, unsafe commands, risky code paths. |

### Conflict Resolution

If agents disagree:

1. CLI records conflicting recommendations.
2. Reviewer summarizes tradeoffs.
3. User approves one path, or policy selects conservative path.
4. Trace links all agent outputs.

### Acceptance Criteria

* Each agent has separate trace span.
* User can inspect agent-specific context and decisions.
* Agent permissions are scoped by role.

---

## 7.7 Skill-Based Workflow

```bash
opencode-rs skill list
opencode-rs skill run rust-cli-hardening --input task.md
opencode-rs skill install ./skills/openapi-test-generator
```

### Flow

1. Discover local and repo skills.
2. Validate skill schema.
3. Resolve inputs.
4. Execute skill.
5. Skill may emit commands, hooks, rules, or agent instructions.
6. Store result in session.

### Acceptance Criteria

* Skills are versioned.
* Untrusted skills require approval.
* Skill execution is logged and traceable.

---

## 7.8 MCP Workflow

```bash
opencode-rs mcp add filesystem --command "npx @modelcontextprotocol/server-filesystem ."
opencode-rs mcp doctor
opencode-rs mcp tools filesystem
opencode-rs run "Use runtime logs to diagnose the failing checkout flow" --mcp logs
```

### Acceptance Criteria

* MCP server health is visible.
* Tool/resource permissions are enforced.
* MCP failures produce actionable errors.
* MCP calls appear in trace.

---

## 7.9 Git Workflow

```bash
opencode-rs git status
opencode-rs git branch create ai/add-json-output
opencode-rs run "Implement JSON output for report command"
opencode-rs diff show
opencode-rs validate
opencode-rs git commit --generate-message
opencode-rs git pr --generate-description
```

### Acceptance Criteria

* CLI refuses destructive Git operations without approval.
* Commit message generation uses diff and session summary.
* PR description includes summary, tests, risks, and validation results.

---

## 7.10 Debugging Workflow

```bash
opencode-rs logs --session latest
opencode-rs trace list
opencode-rs trace show latest
opencode-rs context export --session latest
opencode-rs replay latest --from-step before-tool-call
opencode-rs diagnostics export --session latest
```

### Acceptance Criteria

* User can inspect context, LLM request, response, tool calls, diffs, validation.
* Secrets are redacted by default.
* Replay can run with mocked provider responses.

---

## 7.11 CI/CD Workflow

```bash
opencode-rs run "Review this diff for security and test gaps" \
  --ci \
  --output json \
  --permission-mode auto-readonly \
  --fail-on finding:high
```

### Acceptance Criteria

* No interactive prompts.
* Stable JSON schema.
* Stable exit codes.
* Policy violations fail the build.
* Logs and diagnostics can be archived.

---

## 8. Command Specifications

### 8.1 Command Table

| Command               | Purpose                   | Example                                  | Key Flags                                     | Interactive Behavior                            | Non-Interactive Behavior            | Exit Codes           | Acceptance Criteria          |
| --------------------- | ------------------------- | ---------------------------------------- | --------------------------------------------- | ----------------------------------------------- | ----------------------------------- | -------------------- | ---------------------------- |
| `opencode-rs init`       | Initialize repo config    | `opencode-rs init --template rust`          | `--template`, `--force`, `--dry-run`          | Shows generated files and asks before overwrite | Writes files or outputs plan        | `0`, `2`, `10`       | Creates valid `.opencode/`   |
| `opencode-rs chat`       | Start interactive session | `opencode-rs chat`                          | `--session`, `--include`, `--agent`           | Opens REPL-like chat                            | Fails unless TTY or prompt provided | `0`, `2`, `20`       | Session persists             |
| `opencode-rs run`        | One-shot task             | `opencode-rs run "fix tests"`               | `--validate`, `--max-iterations`, `--dry-run` | Shows plan/diff/approval                        | Runs headless with policy           | `0`, `1`, `30`, `40` | Produces trace and result    |
| `opencode-rs plan`       | Generate plan             | `opencode-rs plan "refactor config"`        | `--output`, `--save`                          | Shows plan and risks                            | Emits JSON/Markdown                 | `0`, `30`            | No file edits                |
| `opencode-rs apply`      | Apply plan/diff           | `opencode-rs apply --plan plan.json`        | `--plan`, `--diff`, `--yes`                   | Asks approval                                   | Applies if policy allows            | `0`, `41`            | Safe patch application       |
| `opencode-rs review`     | Review code/diff          | `opencode-rs review --staged`               | `--staged`, `--branch`, `--security`          | Shows findings                                  | Emits structured findings           | `0`, `50`            | Findings have severity       |
| `opencode-rs test`       | Test workflows            | `opencode-rs test run`                      | `--generate`, `--fix`, `--watch`              | Shows commands and results                      | Stable output                       | `0`, `60`            | Uses detected test commands  |
| `opencode-rs fix`        | Fix failing command       | `opencode-rs fix -- cargo test`             | `--cmd`, `--validate`                         | Asks before edits                               | Iterates up to limit                | `0`, `61`            | Captures command output      |
| `opencode-rs explain`    | Explain code/error        | `opencode-rs explain src/main.rs`           | `--symbol`, `--error`, `--diff`               | Human explanation                               | JSON explanation optional           | `0`                  | No mutation                  |
| `opencode-rs refactor`   | Controlled refactor       | `opencode-rs refactor "split cli module"`   | `--scope`, `--validate`                       | Approval-heavy                                  | Applies only with policy            | `0`, `40`            | Preserves behavior           |
| `opencode-rs agent`      | Manage agents             | `opencode-rs agent list`                    | subcommands                                   | Interactive edit optional                       | JSON support                        | `0`, `70`            | Validates agent definitions  |
| `opencode-rs skill`      | Manage skills             | `opencode-rs skill run test-gen`            | subcommands                                   | Prompts for inputs                              | Inputs via file/stdin               | `0`, `71`            | Versioned skill execution    |
| `opencode-rs command`    | Custom commands           | `opencode-rs command run release-review`    | subcommands                                   | Prompt support                                  | Scriptable                          | `0`, `72`            | Resolves command definitions |
| `opencode-rs hook`       | Manage hooks              | `opencode-rs hook test before_file_edit`    | subcommands                                   | Shows hook effects                              | Deterministic                       | `0`, `73`            | Timeout/failure policy works |
| `opencode-rs rule`       | Manage rules              | `opencode-rs rule validate`                 | subcommands                                   | Shows rule issues                               | JSON findings                       | `0`, `74`            | Rule injection works         |
| `opencode-rs mcp`        | Manage MCP                | `opencode-rs mcp doctor`                    | subcommands                                   | Shows health                                    | JSON health                         | `0`, `80`            | MCP failures actionable      |
| `opencode-rs provider`   | Manage providers          | `opencode-rs provider test openai`          | subcommands                                   | Prompts for secrets                             | Env/keychain support                | `0`, `90`            | Provider test works          |
| `opencode-rs model`      | Manage models             | `opencode-rs model list`                    | subcommands                                   | Human table                                     | JSON array                          | `0`, `91`            | Capabilities visible         |
| `opencode-rs session`    | Manage sessions           | `opencode-rs session resume latest`         | subcommands                                   | Select session                                  | Explicit ID required                | `0`, `100`           | Session lifecycle works      |
| `opencode-rs context`    | Manage context            | `opencode-rs context show`                  | subcommands                                   | Preview context                                 | Export JSON                         | `0`, `110`           | Budget explainable           |
| `opencode-rs repo`       | Inspect repo              | `opencode-rs repo inspect`                  | `--json`                                      | Human summary                                   | JSON metadata                       | `0`, `120`           | Project detection works      |
| `opencode-rs git`        | Git workflows             | `opencode-rs git commit --generate-message` | subcommands                                   | Approval for mutations                          | Policy-controlled                   | `0`, `130`           | Git safety enforced          |
| `opencode-rs diff`       | Show generated diff       | `opencode-rs diff show`                     | `--session`, `--format`                       | Pager display                                   | Patch/JSON                          | `0`, `140`           | Diff snapshots stored        |
| `opencode-rs validate`   | Run gates                 | `opencode-rs validate`                      | `--tests`, `--lint`, `--policy`               | Shows progress                                  | CI output                           | `0`, `150`           | Validation history stored    |
| `opencode-rs doctor`     | Health check              | `opencode-rs doctor`                        | `--fix`, `--json`                             | Suggests fixes                                  | Machine-readable                    | `0`, `160`           | Actionable diagnostics       |
| `opencode-rs config`     | Config ops                | `opencode-rs config explain`                | subcommands                                   | Opens editor optionally                         | JSON/TOML                           | `0`, `170`           | Precedence explainable       |
| `opencode-rs auth`       | Auth ops                  | `opencode-rs auth login kimi`               | subcommands                                   | Browser/keychain                                | Env/headless                        | `0`, `180`           | Secrets not leaked           |
| `opencode-rs logs`       | Logs                      | `opencode-rs logs --tail`                   | `--session`, `--level`                        | Tail/pager                                      | JSON logs                           | `0`, `190`           | Redacted logs                |
| `opencode-rs trace`      | Trace inspection          | `opencode-rs trace show latest`             | subcommands                                   | Tree view                                       | JSON spans                          | `0`, `191`           | Trace spans linked           |
| `opencode-rs replay`     | Replay run                | `opencode-rs replay latest`                 | `--from-step`, `--mock-provider`              | Confirmation                                    | Deterministic replay                | `0`, `192`           | Reproducible failure         |
| `opencode-rs serve`      | Local service             | `opencode-rs serve --port 8765`             | `--host`, `--port`                            | Shows endpoint                                  | Daemon mode                         | `0`, `200`           | API serves sessions          |
| `opencode-rs tui`        | Launch TUI                | `opencode-rs tui`                           | `--session`                                   | Full TUI                                        | Requires TTY                        | `0`, `210`           | Future phase                 |
| `opencode-rs upgrade`    | Upgrade CLI/schema        | `opencode-rs upgrade`                       | `--check`, `--migrate`                        | Prompts                                         | Headless check                      | `0`, `220`           | Safe migration               |
| `opencode-rs completion` | Shell completion          | `opencode-rs completion zsh`                | shell name                                    | Prints script                                   | Prints script                       | `0`                  | Valid completions            |

---

## 9. Configuration System

## 9.1 Config Layers and Precedence

Highest precedence first:

1. CLI flags
2. Environment variables
3. Session config
4. Workspace config
5. Repo config
6. User config
7. Global/system config
8. Built-in defaults

### Example

```text
--model claude-sonnet
OPENCODE_MODEL
.opencode/sessions/<id>/session.toml
.opencode/workspace.toml
.opencode/config.toml
~/.config/opencode-rs/config.toml
/etc/opencode-rs/config.toml
built-in defaults
```

## 9.2 File Locations

| Scope         | Location                            |
| ------------- | ----------------------------------- |
| User config   | `~/.config/opencode-rs/config.toml` |
| User secrets  | OS keychain or env vars             |
| Repo config   | `.opencode/config.toml`             |
| Agents        | `.opencode/agents/`                 |
| Skills        | `.opencode/skills/`                 |
| Commands      | `.opencode/commands/`               |
| Hooks         | `.opencode/hooks/`                  |
| Rules         | `.opencode/rules/`                  |
| MCP           | `.opencode/mcp.json`                |
| Sessions      | `.opencode/sessions/`               |
| Logs          | `.opencode/logs/`                   |
| Traces        | `.opencode/traces/`                 |
| Context cache | `.opencode/context/`                |

## 9.3 User Config Example

```toml
version = 1
default_provider = "openrouter"
default_model = "anthropic/claude-sonnet"

[ui]
color = "auto"
pager = "auto"
editor = "code --wait"
output = "text"

[permissions]
mode = "ask"
allow_network = false
allow_shell = "ask"
allow_file_write = "ask"
allow_git_mutation = "ask"

[providers.openrouter]
kind = "openai-compatible"
base_url = "https://openrouter.ai/api/v1"
api_key_env = "OPENROUTER_API_KEY"

[providers.ollama]
kind = "ollama"
base_url = "http://localhost:11434"

[models."anthropic/claude-sonnet"]
provider = "openrouter"
context_window = 200000
output_limit = 8192
supports_tools = true
supports_vision = true
```

## 9.4 Repo Config Example

```toml
version = 1
project_name = "opencode-rs"
default_agent = "default"

[repo]
language = "rust"
package_manager = "cargo"
source_roots = ["src", "crates"]
test_roots = ["tests"]
exclude = ["target", ".git", "node_modules"]

[commands]
build = "cargo build --workspace"
test = "cargo test --workspace"
lint = "cargo clippy --workspace --all-targets -- -D warnings"
format = "cargo fmt --all"

[context]
max_tokens = 60000
include = ["src/**/*.rs", "crates/**/*.rs", "Cargo.toml"]
exclude = ["target/**", "*.lock"]

[validation]
default = ["format", "lint", "test"]

[policy]
permission_mode = "ask"
require_diff_approval = true
block_sensitive_files = true
```

## 9.5 Environment Variables

| Variable                   | Purpose                   |
| -------------------------- | ------------------------- |
| `OPENCODE_CONFIG`          | Explicit config path      |
| `OPENCODE_REPO`            | Repo path                 |
| `OPENCODE_PROVIDER`        | Default provider override |
| `OPENCODE_MODEL`           | Default model override    |
| `OPENCODE_API_KEY`         | Generic API key           |
| `OPENCODE_PERMISSION_MODE` | Permission mode           |
| `OPENCODE_OUTPUT`          | Output mode               |
| `NO_COLOR`                 | Disable color             |
| `OPENCODE_LOG`             | Logging level             |

## 9.6 Config Validation

```bash
opencode-rs config validate
opencode-rs config explain
opencode-rs config migrate
```

Validation must check:

* Schema version.
* Unknown keys.
* Invalid provider kind.
* Missing model/provider references.
* Insecure plaintext secrets.
* Invalid command templates.
* Invalid permission mode.
* Invalid hook event names.
* Invalid MCP server definitions.

---

## 10. Session Model

## 10.1 Session Lifecycle

```text
created -> active -> waiting_approval -> executing -> validating -> completed
                                     -> failed
                                     -> archived
```

## 10.2 Session Directory

```text
.opencode/sessions/2026-04-26T10-30-00_abc123/
  session.toml
  conversation.jsonl
  tool_calls.jsonl
  edits.jsonl
  diffs/
    001.patch
    002.patch
  validations.jsonl
  context/
    snapshot-001.json
  plan.md
  plan.json
  summary.md
```

## 10.3 Session Metadata

```rust
struct SessionMetadata {
    id: SessionId,
    title: String,
    repo_root: PathBuf,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    status: SessionStatus,
    provider: String,
    model: String,
    default_agent: String,
    permission_mode: PermissionMode,
    git_branch: Option<String>,
    git_commit: Option<String>,
    tags: Vec<String>,
}
```

## 10.4 Session Commands

| Command           | Purpose         | Example                                       |
| ----------------- | --------------- | --------------------------------------------- |
| `session list`    | List sessions   | `opencode-rs session list`                       |
| `session show`    | Show metadata   | `opencode-rs session show latest`                |
| `session resume`  | Resume session  | `opencode-rs session resume abc123`              |
| `session fork`    | Fork session    | `opencode-rs session fork abc123`                |
| `session archive` | Archive session | `opencode-rs session archive abc123`             |
| `session delete`  | Delete session  | `opencode-rs session delete abc123`              |
| `session export`  | Export session  | `opencode-rs session export abc123 --format zip` |

## 10.5 Privacy and Retention

Config options:

```toml
[sessions]
retention_days = 30
store_llm_requests = true
store_llm_responses = true
redact_secrets = true
store_context_snapshots = true
```

Acceptance criteria:

* User can disable request/response capture.
* Redaction is enabled by default.
* Export bundle clearly marks redacted fields.

---

## 11. Agent System Integration

## 11.1 Agent Concepts

| Concept               | Description                                                |
| --------------------- | ---------------------------------------------------------- |
| Default agent         | Agent used when no explicit agent is selected.             |
| Agent registry        | Collection of built-in, user, and repo agents.             |
| Agent profile         | Prompt, tools, model, permission, context policy.          |
| Agent role            | Planner, implementer, reviewer, tester, security reviewer. |
| Subagent              | Specialized child agent invoked by a parent workflow.      |
| Agent routing         | Selection of agent based on command/task/rules.            |
| Agent permissions     | File, shell, network, MCP, Git permissions.                |
| Agent memory boundary | Session, repo, user, or none.                              |

## 11.2 Agent Commands

```bash
opencode-rs agent list
opencode-rs agent show default
opencode-rs agent create reviewer
opencode-rs agent edit reviewer
opencode-rs agent run reviewer "review this diff"
opencode-rs agent validate reviewer
opencode-rs agent doctor reviewer
```

## 11.3 Agent Definition Example

```toml
version = 1
name = "rust-implementer"
role = "implementer"
description = "Implements Rust code changes with tests."

model = "anthropic/claude-sonnet"
temperature = 0.2

[permissions]
file_read = "allow"
file_write = "ask"
shell = "ask"
git = "ask"
network = "deny"
mcp = "ask"

[context]
include_rules = ["coding", "testing"]
max_tokens = 80000

[prompts]
system = """
You are a senior Rust engineer.
Prefer small cohesive modules, explicit errors, and testable interfaces.
Always update tests when behavior changes.
"""

[tools]
enabled = ["file_read", "file_edit", "shell", "git_diff"]
```

## 11.4 Agent Execution Lifecycle

1. Load config.
2. Resolve agent.
3. Resolve model/provider.
4. Build context.
5. Load rules.
6. Execute before-agent hooks.
7. Send LLM request.
8. Parse response.
9. Execute tool calls.
10. Apply edits subject to permission.
11. Validate.
12. Store trace.
13. Emit result.

Acceptance criteria:

* Agent execution is represented as trace spans.
* Permission failures are explainable.
* Agent definitions are schema-validated.

---

## 12. Skills / Commands / Hooks / Rules

## 12.1 Skills

### Purpose

A skill is a reusable packaged AI workflow or expert procedure.

Examples:

* Rust CLI refactor skill.
* OpenAPI test generation skill.
* Security review skill.
* Idempotency inspection skill.
* Dependency upgrade skill.

### Structure

```text
.opencode/skills/rust-cli-hardening/
  skill.toml
  README.md
  prompt.md
  examples/
  rules/
  commands/
  hooks/
```

### Skill Schema

```toml
version = 1
name = "rust-cli-hardening"
description = "Review and harden Rust CLI UX, errors, and tests."
version_tag = "0.1.0"

[inputs]
task = { type = "string", required = true }
scope = { type = "string", required = false }

[execution]
agent = "rust-reviewer"
permission_mode = "auto-readonly"

[outputs]
format = "markdown"
```

### Commands

```bash
opencode-rs skill list
opencode-rs skill show rust-cli-hardening
opencode-rs skill run rust-cli-hardening --input task.md
opencode-rs skill validate rust-cli-hardening
opencode-rs skill install ./skill-dir
opencode-rs skill remove rust-cli-hardening
```

---

## 12.2 Custom Commands

### Purpose

Custom commands are named workflows defined by users or teams.

Example:

```yaml
name: release-review
description: Review branch before release
steps:
  - run: "opencode-rs review --branch main...HEAD --security"
  - run: "opencode-rs validate"
  - run: "opencode-rs git pr --generate-description"
```

Commands:

```bash
opencode-rs command list
opencode-rs command run release-review
opencode-rs command validate release-review
```

Acceptance criteria:

* Commands support arguments.
* Commands can call built-in commands.
* Commands have dry-run mode.

---

## 12.3 Hooks

### Purpose

Hooks are lifecycle automation points.

### Required Hook Events

| Event                  | Trigger                            |
| ---------------------- | ---------------------------------- |
| `before_session_start` | Before session creation.           |
| `after_session_start`  | After session metadata is created. |
| `before_context_build` | Before context collection.         |
| `after_context_build`  | After context snapshot.            |
| `before_llm_request`   | Before provider request.           |
| `after_llm_response`   | After provider response.           |
| `before_tool_call`     | Before tool execution.             |
| `after_tool_call`      | After tool execution.              |
| `before_file_edit`     | Before file mutation.              |
| `after_file_edit`      | After file mutation.               |
| `before_validation`    | Before validation command.         |
| `after_validation`     | After validation command.          |
| `before_git_commit`    | Before Git commit.                 |
| `after_git_commit`     | After Git commit.                  |
| `on_error`             | On recoverable or fatal error.     |
| `on_session_end`       | Before session closes.             |

### Hook Definition Example

```toml
version = 1
name = "block-sensitive-files"
event = "before_file_edit"
enabled = true
timeout_ms = 3000
failure_behavior = "block"

[match]
paths = [".env", "*.pem", "secrets/**"]

[action]
kind = "builtin"
name = "block_edit"

[logging]
capture_stdout = true
capture_stderr = true
```

### Hook Failure Behavior

| Mode     | Description             |
| -------- | ----------------------- |
| `ignore` | Log but continue.       |
| `warn`   | Warn user but continue. |
| `block`  | Stop current operation. |
| `ask`    | Ask user interactively. |

### Hook Commands

```bash
opencode-rs hook list
opencode-rs hook show block-sensitive-files
opencode-rs hook test before_file_edit --file .env
opencode-rs hook validate
opencode-rs hook enable block-sensitive-files
opencode-rs hook disable block-sensitive-files
```

---

## 12.4 Rules

### Purpose

Rules are persistent instructions and policy constraints injected into context or enforced by validation.

Examples:

* Rust style rules.
* Architecture boundaries.
* Testing requirements.
* Security constraints.
* Enterprise compliance policy.

### Rule Example

```markdown
---
name: rust-error-handling
type: coding
severity: warning
scope:
  include:
    - "src/**/*.rs"
---

# Rust Error Handling Rule

- Use `thiserror` for library/domain errors.
- Use `anyhow` only at application boundaries.
- Do not use `unwrap()` in production code unless justified.
- Add tests for error branches when behavior is user-visible.
```

Commands:

```bash
opencode-rs rule list
opencode-rs rule show rust-error-handling
opencode-rs rule validate
opencode-rs rule explain rust-error-handling
```

Acceptance criteria:

* Rules can be injected into context.
* Rules can be validated structurally.
* Rules can have severity and scope.

---

## 13. MCP Integration

## 13.1 MCP Server Definition

```json
{
  "version": 1,
  "servers": {
    "filesystem": {
      "enabled": true,
      "transport": "stdio",
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "."],
      "permissions": {
        "tools": "ask",
        "resources": "allow"
      }
    },
    "runtime-logs": {
      "enabled": false,
      "transport": "http",
      "url": "http://localhost:9000/mcp",
      "auth": {
        "type": "env",
        "env": "RUNTIME_LOGS_TOKEN"
      }
    }
  }
}
```

## 13.2 MCP Commands

| Command                  | Purpose             |
| ------------------------ | ------------------- |
| `opencode-rs mcp add`       | Add server.         |
| `opencode-rs mcp list`      | List servers.       |
| `opencode-rs mcp show`      | Show server config. |
| `opencode-rs mcp remove`    | Remove server.      |
| `opencode-rs mcp enable`    | Enable server.      |
| `opencode-rs mcp disable`   | Disable server.     |
| `opencode-rs mcp doctor`    | Health check.       |
| `opencode-rs mcp tools`     | List tools.         |
| `opencode-rs mcp resources` | List resources.     |
| `opencode-rs mcp logs`      | Show MCP logs.      |

## 13.3 MCP Permission Model

| Permission         | Default                    |
| ------------------ | -------------------------- |
| List tools         | allow                      |
| Read resources     | ask                        |
| Execute tool       | ask                        |
| Network MCP server | ask                        |
| Write through MCP  | ask/block depending policy |

## 13.4 MCP Failure Handling

| Failure             | Behavior                                   |
| ------------------- | ------------------------------------------ |
| Server not found    | Show command/path and install hint.        |
| Startup timeout     | Show timeout and captured stderr.          |
| Tool schema invalid | Disable tool and warn.                     |
| Tool call failure   | Record trace and ask retry if interactive. |
| Auth failure        | Show auth source without leaking secret.   |

---

## 14. Provider / Model / Auth Design

## 14.1 Provider Types

| Provider Type        | Examples                                   |
| -------------------- | ------------------------------------------ |
| OpenAI-compatible    | OpenAI, OpenRouter, internal proxies, vLLM |
| Anthropic-compatible | Anthropic, compatible proxies              |
| Local model          | Ollama, LM Studio                          |
| Chinese providers    | Kimi, GLM, MiniMax, Qwen                   |
| Enterprise proxy     | Internal gateway with routing/policy       |

## 14.2 Provider Commands

```bash
opencode-rs provider add openrouter --kind openai-compatible --base-url https://openrouter.ai/api/v1
opencode-rs provider list
opencode-rs provider show openrouter
opencode-rs provider test openrouter
opencode-rs provider remove openrouter
```

## 14.3 Model Commands

```bash
opencode-rs model list
opencode-rs model list --provider openrouter
opencode-rs model show qwen-plus
opencode-rs model test qwen-plus
```

## 14.4 Auth Commands

```bash
opencode-rs auth login openrouter
opencode-rs auth logout openrouter
opencode-rs auth status
```

## 14.5 Provider Schema

```rust
struct ProviderConfig {
    name: String,
    kind: ProviderKind,
    base_url: Option<String>,
    api_key_env: Option<String>,
    credential_ref: Option<String>,
    default_headers: BTreeMap<String, String>,
    timeout_ms: u64,
    retry: RetryPolicy,
}
```

## 14.6 Model Schema

```rust
struct ModelConfig {
    name: String,
    provider: String,
    context_window: u32,
    output_limit: u32,
    supports_tools: bool,
    supports_vision: bool,
    supports_json_mode: bool,
    supports_streaming: bool,
    cost: Option<ModelCost>,
}
```

## 14.7 Error Mapping

| Provider Error      | CLI Error                |
| ------------------- | ------------------------ |
| 401/403             | `AUTH_FAILED`            |
| 404 model           | `MODEL_NOT_FOUND`        |
| 429                 | `RATE_LIMITED`           |
| timeout             | `PROVIDER_TIMEOUT`       |
| invalid tool schema | `TOOL_SCHEMA_REJECTED`   |
| context too large   | `CONTEXT_LIMIT_EXCEEDED` |

Acceptance criteria:

* Provider errors are normalized.
* Raw provider response can be inspected with trace mode.
* Secrets and auth headers are redacted.

---

## 15. Context Engine

## 15.1 Context Sources

| Source           | Description                                    |
| ---------------- | ---------------------------------------------- |
| Repo files       | Source code, configs, docs.                    |
| Git state        | Branch, diff, staged files, recent commits.    |
| Rules            | Coding, testing, security, architecture rules. |
| Skills           | Skill-specific prompts and examples.           |
| MCP              | Runtime logs, external tools, resources.       |
| User input       | Task prompt, explicit includes, pasted files.  |
| Session history  | Previous conversation and edits.               |
| Dependency graph | Imports, modules, crate/package graph.         |
| Symbol index     | Functions, structs, classes, modules.          |

## 15.2 Commands

```bash
opencode-rs context build
opencode-rs context show
opencode-rs context explain
opencode-rs context include src/cli.rs
opencode-rs context exclude target/**
opencode-rs context index
opencode-rs context refresh
opencode-rs context stats
opencode-rs context export --session latest
```

## 15.3 Context Budget Strategy

Context builder should prioritize:

1. Explicit user includes.
2. Current Git diff.
3. Files mentioned in task.
4. Relevant symbols.
5. Relevant rules.
6. Relevant tests.
7. Repo summary.
8. Session history summary.
9. MCP resources.

## 15.4 Context Explain Example

```bash
opencode-rs context explain --task "add JSON output to report command"
```

Output:

```text
Context budget: 58,200 / 80,000 tokens

Included:
- src/cli/report.rs: directly matches "report command"
- src/output.rs: contains output format abstraction
- tests/cli_report.rs: likely validation target
- .opencode/rules/testing.md: repo testing policy

Excluded:
- target/**: ignored by repo config
- docs/old-report.md: low relevance
```

Acceptance criteria:

* Context inclusion is explainable.
* Context can be exported.
* Context snapshots are attached to sessions.

---

## 16. Terminal UX and Output Design

## 16.1 Output Modes

| Mode              | Use Case                             |
| ----------------- | ------------------------------------ |
| `--output text`   | Human-readable terminal output.      |
| `--output json`   | Single machine-readable JSON result. |
| `--output ndjson` | Streaming machine-readable events.   |
| `--quiet`         | Minimal output.                      |
| `--verbose`       | Detailed progress.                   |
| `--trace`         | Debug-level trace output.            |

## 16.2 NDJSON Event Example

```json
{"type":"session_started","session_id":"abc123"}
{"type":"context_built","tokens":58200}
{"type":"llm_request_started","provider":"openrouter","model":"anthropic/claude-sonnet"}
{"type":"file_edit_proposed","path":"src/output.rs"}
{"type":"validation_started","command":"cargo test --workspace"}
{"type":"completed","status":"success"}
```

## 16.3 Human Output Style

Requirements:

* Clear headings.
* Minimal noise by default.
* Color only when TTY supports it.
* `NO_COLOR` respected.
* Long output uses pager.
* Diffs are syntax-aware where possible.
* Errors include cause, fix, and debug command.

### Error Example

```text
Error: model not found

Provider: axonhub
Model: minimax-m2.5-free

What happened:
The provider returned a model-not-found response before any tool call started.

Try:
  opencode-rs provider test axonhub
  opencode-rs model list --provider axonhub
  opencode-rs config explain model

Debug:
  opencode-rs trace show latest
```

## 16.4 Approval Prompts

Example:

```text
Agent wants to edit 3 files:

  modified src/output.rs
  modified src/cli/report.rs
  added    tests/report_json.rs

Approve?
  [y] yes
  [n] no
  [d] show diff
  [e] edit patch
  [a] approve all safe edits in this session
```

Non-interactive behavior:

* If approval required and no TTY: fail with `APPROVAL_REQUIRED`.
* If `--yes` is provided: approve only if policy permits.

---

## 17. Safety, Security, and Permissions

## 17.1 Permission Modes

| Mode                       | Behavior                                                                      |
| -------------------------- | ----------------------------------------------------------------------------- |
| `ask`                      | Ask before writes, shell, network, Git mutation, MCP tool calls.              |
| `auto-readonly`            | Auto-allow read-only actions only.                                            |
| `auto-safe`                | Auto-allow safe edits/commands defined by policy.                             |
| `dangerously-auto-approve` | Auto-approve all actions; must show warning and require explicit config/flag. |

## 17.2 Protected Operations

| Operation            | Default   |
| -------------------- | --------- |
| Read source files    | allow     |
| Read sensitive files | ask/block |
| Edit source files    | ask       |
| Edit secrets         | block     |
| Run shell command    | ask       |
| Network access       | ask       |
| Git commit           | ask       |
| Git push             | ask/block |
| MCP tool call        | ask       |
| Install skill/hook   | ask       |
| Run untrusted hook   | ask/block |

## 17.3 Sensitive File Detection

Patterns:

```text
.env
.env.*
*.pem
*.key
id_rsa
id_ed25519
secrets/**
credentials/**
*.p12
*.pfx
```

## 17.4 Policy Example

```toml
[policy]
permission_mode = "ask"
block_sensitive_files = true
redact_secrets = true
allow_shell_commands = ["cargo test", "cargo fmt", "cargo clippy"]
deny_shell_patterns = ["rm -rf", "curl * | sh", "sudo", "chmod 777"]
require_approval_for_git_push = true
```

## 17.5 Prompt Injection Protection

Requirements:

* Treat repo content as untrusted.
* Separate system/developer/user/repo/MCP context layers.
* Mark external content origin.
* Do not let file content override policies.
* MCP tool descriptions must not override permissions.
* Hooks cannot silently weaken policy.

Acceptance criteria:

* Dangerous commands are blocked or require approval.
* Sensitive content is redacted from logs.
* Policy explain command shows why an action was blocked.

---

## 18. Observability and Debugging

## 18.1 Logs

```bash
opencode-rs logs
opencode-rs logs --session latest
opencode-rs logs --level debug
opencode-rs logs --tail
```

Log fields:

```rust
struct LogEvent {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    session_id: Option<String>,
    target: String,
    message: String,
    fields: BTreeMap<String, Value>,
}
```

## 18.2 Traces

Trace spans:

* CLI command.
* Config resolution.
* Context build.
* Agent execution.
* LLM request.
* LLM response.
* Tool call.
* File edit.
* Hook execution.
* Validation command.
* Git operation.
* MCP call.

Commands:

```bash
opencode-rs trace list
opencode-rs trace show latest
opencode-rs trace export latest --format json
```

## 18.3 Request / Response Capture

Default:

* Store metadata.
* Store redacted request/response if enabled.
* Never store raw auth headers.
* Allow disabling in config.

## 18.4 Diagnostics Bundle

```bash
opencode-rs diagnostics export --session latest
```

Bundle:

```text
diagnostics.zip
  manifest.json
  config.redacted.toml
  session/
  logs/
  traces/
  context/
  diffs/
  provider-test.json
  mcp-health.json
```

Acceptance criteria:

* Bundle is redacted.
* Bundle is enough to reproduce most failures.
* Export explains what is included.

---

## 19. Rust Implementation Architecture

## 19.1 Workspace Structure

```text
opencode-rs/
  Cargo.toml
  crates/
    opencode-cli/
    opencode-core/
    opencode-config/
    opencode-session/
    opencode-agent/
    opencode-provider/
    opencode-context/
    opencode-mcp/
    opencode-hooks/
    opencode-policy/
    opencode-git/
    opencode-diff/
    opencode-observability/
    opencode-testkit/
```

## 19.2 Suggested Libraries

| Area            | Libraries                                                   |
| --------------- | ----------------------------------------------------------- |
| CLI parser      | `clap`                                                      |
| Async runtime   | `tokio`                                                     |
| Serialization   | `serde`, `serde_json`, `toml`, `serde_yaml`                 |
| Errors          | `thiserror`, `anyhow`                                       |
| HTTP            | `reqwest`                                                   |
| Logging/tracing | `tracing`, `tracing-subscriber`                             |
| Git             | `git2` or shell-based wrapper                               |
| Terminal        | `crossterm`, `ratatui`, `dialoguer`, `inquire`              |
| Diff            | `similar`                                                   |
| File walking    | `ignore`, `walkdir`                                         |
| Watch           | `notify`                                                    |
| Directories     | `directories`                                               |
| Secrets         | `keyring`                                                   |
| JSON schema     | `schemars`                                                  |
| Tests           | `assert_cmd`, `predicates`, `insta`, `tempfile`, `wiremock` |

## 19.3 Major Modules

### `opencode-cli`

Responsibility:

* Parse commands.
* Dispatch to core services.
* Render output.
* Handle TTY/non-TTY behavior.

Public interface:

```rust
async fn run_cli(args: impl IntoIterator<Item = String>) -> CliResult;
```

Testing:

* CLI contract tests.
* Snapshot output tests.
* Exit code tests.

---

### `opencode-config`

Responsibility:

* Load layered config.
* Validate schema.
* Explain precedence.
* Migrate config.

Core types:

```rust
struct ResolvedConfig {
    user: Option<UserConfig>,
    repo: Option<RepoConfig>,
    session: Option<SessionConfig>,
    effective: EffectiveConfig,
    sources: Vec<ConfigSource>,
}
```

Testing:

* Precedence tests.
* Migration tests.
* Invalid config tests.

---

### `opencode-provider`

Responsibility:

* Provider abstraction.
* Streaming/non-streaming requests.
* Tool-call support.
* Error normalization.
* Retry/fallback.

Trait:

```rust
#[async_trait]
trait LlmProvider {
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, ProviderError>;
    async fn stream(&self, request: LlmRequest) -> Result<LlmStream, ProviderError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, ProviderError>;
    async fn health_check(&self) -> Result<ProviderHealth, ProviderError>;
}
```

Testing:

* Mock provider.
* HTTP wiremock tests.
* Error mapping tests.

---

### `opencode-agent`

Responsibility:

* Agent lifecycle.
* Tool execution.
* Planning/execution loops.
* Multi-agent orchestration.

Core types:

```rust
struct AgentRuntime {
    provider: Arc<dyn LlmProvider>,
    tools: ToolRegistry,
    policy: PolicyEngine,
    hooks: HookRunner,
}
```

Testing:

* Mock LLM response tests.
* Tool-call tests.
* Permission tests.
* Replay tests.

---

### `opencode-context`

Responsibility:

* Repo scanning.
* File indexing.
* Context selection.
* Context budget.
* Context explanation.

Testing:

* Fixture repos.
* Ignore rules.
* Budget selection.
* Explicit include/exclude.

---

### `opencode-session`

Responsibility:

* Session persistence.
* Conversation history.
* Tool/edit/validation history.
* Export/import.
* Replay data.

Testing:

* Create/resume/fork/archive/delete.
* Corrupt session recovery.
* Export/import compatibility.

---

### `opencode-policy`

Responsibility:

* Permission mode.
* Approval gates.
* Sensitive file detection.
* Dangerous command detection.
* Enterprise policy enforcement.

Testing:

* Permission matrix.
* Dangerous command cases.
* Sensitive file patterns.

---

### `opencode-mcp`

Responsibility:

* MCP server config.
* Transport.
* Tool/resource discovery.
* MCP execution.
* MCP health/logs.

Testing:

* Mock MCP server.
* Startup failure.
* Tool schema validation.
* Timeout handling.

---

### `opencode-observability`

Responsibility:

* Structured logs.
* Traces.
* Redaction.
* Diagnostics export.

Testing:

* Redaction tests.
* Trace schema tests.
* Bundle contents tests.

---

## 20. Data Models and Schemas

## 20.1 Task Request

```rust
struct TaskRequest {
    id: TaskId,
    session_id: SessionId,
    prompt: String,
    repo_root: PathBuf,
    agent: String,
    provider: String,
    model: String,
    mode: TaskMode,
    validation: ValidationPolicy,
    permission_mode: PermissionMode,
}
```

## 20.2 Plan

```rust
struct Plan {
    id: String,
    task_id: String,
    summary: String,
    steps: Vec<PlanStep>,
    affected_files: Vec<PathBuf>,
    risks: Vec<Risk>,
    validation_plan: Vec<ValidationStep>,
    rollback_strategy: Option<String>,
}
```

## 20.3 File Edit

```rust
struct FileEdit {
    id: String,
    path: PathBuf,
    operation: FileOperation,
    before_hash: Option<String>,
    after_hash: Option<String>,
    patch: String,
    approved: bool,
}
```

## 20.4 Tool Call

```rust
struct ToolCall {
    id: String,
    name: String,
    input: serde_json::Value,
    output: Option<serde_json::Value>,
    status: ToolCallStatus,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
}
```

## 20.5 Trace Span

```rust
struct TraceSpan {
    id: String,
    parent_id: Option<String>,
    name: String,
    kind: TraceKind,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    attributes: BTreeMap<String, serde_json::Value>,
    redacted: bool,
}
```

---

## 21. Error Handling and Exit Codes

## 21.1 Error Style

Every user-facing error should include:

1. Short error title.
2. What happened.
3. Why it likely happened.
4. Suggested fix.
5. Debug command.
6. Exit code.

## 21.2 Exit Codes

|  Code | Meaning                      |
| ----: | ---------------------------- |
|   `0` | Success                      |
|   `1` | General failure              |
|   `2` | Invalid CLI usage            |
|  `10` | Initialization failure       |
|  `20` | Interactive mode unavailable |
|  `30` | Agent execution failure      |
|  `40` | File edit/apply failure      |
|  `41` | Approval required            |
|  `50` | Review found blocking issue  |
|  `60` | Test failure                 |
|  `61` | Fix loop exhausted           |
|  `70` | Agent config error           |
|  `71` | Skill error                  |
|  `72` | Command definition error     |
|  `73` | Hook error                   |
|  `74` | Rule error                   |
|  `80` | MCP error                    |
|  `90` | Provider error               |
|  `91` | Model error                  |
| `100` | Session error                |
| `110` | Context error                |
| `120` | Repo detection error         |
| `130` | Git error                    |
| `140` | Diff error                   |
| `150` | Validation failure           |
| `160` | Doctor check failed          |
| `170` | Config error                 |
| `180` | Auth error                   |
| `190` | Log error                    |
| `191` | Trace error                  |
| `192` | Replay error                 |
| `200` | Serve error                  |

---

## 22. Testing Strategy

## 22.1 Test Levels

| Test Type               | Purpose                                                    |
| ----------------------- | ---------------------------------------------------------- |
| Unit tests              | Validate pure logic and module behavior.                   |
| Integration tests       | Validate CLI commands against fixture repos.               |
| Snapshot tests          | Validate human-readable output.                            |
| Golden output tests     | Validate JSON/NDJSON contracts.                            |
| CLI contract tests      | Validate flags, args, exit codes.                          |
| Config precedence tests | Validate config layering.                                  |
| Mock provider tests     | Validate provider behavior without real LLM calls.         |
| Mock MCP tests          | Validate MCP integration.                                  |
| Hook lifecycle tests    | Validate hook event order and failure modes.               |
| Permission model tests  | Validate safe/default behavior.                            |
| Git workflow tests      | Validate branch/diff/commit flows.                         |
| Session replay tests    | Validate deterministic replay.                             |
| Cross-platform tests    | macOS, Linux, Windows.                                     |
| Performance tests       | Large repo scan, context build, session load.              |
| Compatibility tests     | Compare behavior against original opencode-rs where feasible. |

## 22.2 Fixture Layout

```text
tests/
  fixtures/
    repos/
      rust-basic/
      rust-workspace/
      node-basic/
      python-basic/
      dirty-git-repo/
      sensitive-files/
    providers/
      openai-compatible/
      anthropic-compatible/
    mcp/
      mock-filesystem/
      mock-failing-server/
    configs/
      precedence/
      invalid/
  cli/
    init_test.rs
    run_test.rs
    config_test.rs
    session_test.rs
    provider_test.rs
    mcp_test.rs
    git_test.rs
    output_contract_test.rs
```

## 22.3 Example Test Cases

### Config Precedence

```text
Given:
- User config model = model-a
- Repo config model = model-b
- Env OPENCODE_MODEL=model-c
- CLI flag --model model-d

Expect:
- Effective model = model-d
- config explain shows all sources
```

### Permission Test

```text
Given:
- permission mode = auto-readonly
When:
- agent proposes file edit
Expect:
- edit blocked
- exit code = 41
- error includes approval guidance
```

### NDJSON Contract Test

```text
When:
- opencode-rs run "explain repo" --output ndjson
Expect:
- every line is valid JSON
- required event fields exist
- final event has type completed or failed
```

### Mock Provider Test

```text
Given:
- mock provider returns tool call to edit src/main.rs
When:
- opencode-rs run "change greeting"
Expect:
- file edit proposal is created
- diff snapshot exists
- trace contains provider request and tool call
```

---

## 23. Compatibility With Original opencode

## 23.1 Compatibility Goals

| Area               | Strategy                                                           |
| ------------------ | ------------------------------------------------------------------ |
| Common commands    | Provide similar behavior where it helps adoption.                  |
| Config migration   | Offer migration command for known config patterns.                 |
| Agents             | Support adapter/import where schema can be mapped.                 |
| Skills/rules/hooks | Provide compatibility layer if original formats are stable enough. |
| Sessions           | Best-effort import/export only.                                    |
| Provider config    | Support OpenAI-compatible and custom proxy patterns.               |
| Testing parity     | Build independent parity test harness.                             |

## 23.2 Intentional Differences

| Difference                 | Reason                                               |
| -------------------------- | ---------------------------------------------------- |
| Strong typed config        | Rust implementation should prefer schema validation. |
| Explicit permission model  | Safer enterprise/team usage.                         |
| First-class trace/replay   | Better debugging and reliability.                    |
| JSON/NDJSON contracts      | Stronger automation support.                         |
| Modular crate architecture | Support future TUI/Desktop/server reuse.             |

## 23.3 Migration Commands

```bash
opencode-rs config import --from opencode-rs --path ~/.config/opencode
opencode-rs agent import --from opencode-rs .opencode/agents
opencode-rs skill import --from opencode-rs ./skills
opencode-rs doctor --compat opencode
```

## 23.4 Compatibility Matrix

| Feature           | MVP        | V1                    | V2             |
| ----------------- | ---------- | --------------------- | -------------- |
| Basic chat/run    | Partial    | Strong                | Strong         |
| Provider config   | Strong     | Strong                | Strong         |
| Agent definitions | Basic      | Strong                | Strong         |
| Skills            | No         | Strong                | Strong         |
| Hooks             | No         | Strong                | Strong         |
| MCP               | No         | Strong                | Strong         |
| Sessions          | Own format | Import/export attempt | Better tooling |
| TUI               | No         | Experimental          | Strong         |
| Multi-agent       | No         | Partial               | Strong         |

---

## 24. MVP / V1 / V2 Roadmap

| Phase | Scope                                                                                                  | Out of Scope                                                 | User Value                                      | Engineering Work                                                                                      | Acceptance Criteria                                                          | Risks                                             |
| ----- | ------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------ | ----------------------------------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | ------------------------------------------------- |
| MVP   | Provider config, auth, repo init, basic chat/run, diff/apply, session persistence, logs, doctor        | Skills, hooks, MCP, multi-agent, TUI                         | Developers can use CLI for real coding tasks    | CLI parser, config, provider abstraction, session store, basic agent loop, file edit/diff, validation | Can initialize repo, run task, inspect diff, apply, validate, resume session | Provider variability, unsafe edits, poor context  |
| V1    | Skills, commands, rules, hooks, MCP, Git workflow, advanced context, structured output, CI mode        | Full multi-agent orchestration, desktop UI, remote execution | Power users and teams can standardize workflows | Skill engine, hook runner, MCP client, policy engine, JSON/NDJSON contracts, Git workflows            | CI can run non-interactively; rules/hooks/MCP work                           | Hook security, MCP instability, config complexity |
| V2    | Multi-agent workflows, replay/debug UI, TUI, remote execution, team policy, audit, desktop integration | Marketplace, full cloud service                              | Enterprise/team-scale AI coding operations      | Orchestration engine, local service API, TUI, audit store, remote protocol                            | Multi-agent workflows traceable; TUI usable; audit export works              | Scope creep, UI/runtime coupling, performance     |

---

## 25. Risks and Open Questions

## 25.1 Risks

| Risk                                    | Impact                    | Mitigation                                                       |
| --------------------------------------- | ------------------------- | ---------------------------------------------------------------- |
| CLI becomes too complex                 | Poor adoption             | Keep defaults simple; hide advanced features behind subcommands. |
| Provider APIs vary widely               | Fragile integrations      | Normalize provider abstraction and error mapping.                |
| Unsafe agent edits                      | Data loss/security issues | Approval gates, backups, diffs, sensitive file blocking.         |
| Context quality is poor                 | Bad agent output          | Context explain, explicit include/exclude, indexing, tests.      |
| Hooks/skills supply-chain risk          | Security issue            | Trust model, signatures later, sandboxing, approval.             |
| MCP instability                         | Failed workflows          | Health checks, timeouts, logs, graceful degradation.             |
| Original opencode-rs compatibility unclear | Migration friction        | Best-effort import and independent parity harness.               |
| Long-running tasks hard to debug        | User frustration          | Trace, replay, diagnostics export.                               |

## 25.2 Open Questions

1. Should Git operations use `git2` or shell out to system Git for maximum compatibility?
2. Should skills be local-only initially, or support registries later?
3. Should session storage use plain files only, SQLite, or both?
4. Should provider request/response capture be opt-in or opt-out?
5. What compatibility level with original opencode-rs is required for first release?
6. Should TUI share the same command service API from day one?
7. How strict should CI policy defaults be?

---

## 26. Acceptance Criteria

## 26.1 Product-Level Acceptance Criteria

* CLI can initialize a repo and create valid `.opencode/` configuration.
* CLI can configure at least one OpenAI-compatible provider.
* CLI can run a one-shot coding task and produce a diff.
* CLI can ask for approval before applying file edits.
* CLI can run validation commands.
* CLI can persist and resume sessions.
* CLI can show logs and traces.
* CLI can run in non-interactive CI mode.
* CLI can output stable JSON and NDJSON.
* CLI can protect sensitive files by default.
* CLI has a documented error and exit code model.
* CLI has meaningful automated test coverage.

## 26.2 MVP Acceptance Criteria

```bash
opencode-rs doctor
opencode-rs provider add
opencode-rs provider test
opencode-rs init
opencode-rs chat
opencode-rs run
opencode-rs diff show
opencode-rs apply
opencode-rs validate
opencode-rs session list
opencode-rs session resume
opencode-rs logs
```

All above commands must:

* Have help text.
* Have typed config.
* Have tests.
* Support `--output json` where useful.
* Return stable exit codes.
* Avoid leaking secrets.

---

## 27. Appendix: Example Commands

```bash
# Setup
opencode-rs doctor
opencode-rs provider add local --kind ollama --base-url http://localhost:11434
opencode-rs model list --provider local

# Repo initialization
opencode-rs init --template rust
opencode-rs repo inspect

# Basic usage
opencode-rs run "Add JSON output support to the report command"
opencode-rs chat
opencode-rs explain src/main.rs
opencode-rs fix -- cargo test

# Plan-first
opencode-rs plan "Split provider module into provider-core and provider-http"
opencode-rs apply --plan .opencode/sessions/latest/plan.json

# Context
opencode-rs context build
opencode-rs context show
opencode-rs context explain --task "fix provider model not found"
opencode-rs context export --session latest

# Git
opencode-rs git branch create ai/json-output
opencode-rs diff show
opencode-rs validate
opencode-rs git commit --generate-message
opencode-rs git pr --generate-description

# Debug
opencode-rs logs --session latest
opencode-rs trace show latest
opencode-rs replay latest
opencode-rs diagnostics export --session latest

# CI
opencode-rs run "review this diff for security issues" \
  --ci \
  --output json \
  --permission-mode auto-readonly
```

---

## 28. Appendix: Example Config Files

## 28.1 `.opencode/config.toml`

```toml
version = 1
project_name = "opencode-rs"
default_agent = "default"

[repo]
language = "rust"
source_roots = ["src", "crates"]
test_roots = ["tests"]
exclude = ["target", ".git"]

[commands]
build = "cargo build --workspace"
test = "cargo test --workspace"
lint = "cargo clippy --workspace --all-targets -- -D warnings"
format = "cargo fmt --all"

[context]
max_tokens = 80000
include = ["src/**/*.rs", "crates/**/*.rs", "Cargo.toml"]
exclude = ["target/**"]

[permissions]
mode = "ask"
block_sensitive_files = true

[validation]
default = ["format", "lint", "test"]
```

## 28.2 `.opencode/mcp.json`

```json
{
  "version": 1,
  "servers": {
    "filesystem": {
      "enabled": true,
      "transport": "stdio",
      "command": "npx",
      "args": ["@modelcontextprotocol/server-filesystem", "."]
    }
  }
}
```

---

## 29. Appendix: Example Agent / Skill / Hook / Rule Definitions

## 29.1 Agent Definition

```toml
version = 1
name = "default"
role = "generalist"
description = "Default repo-aware coding agent."

model = "anthropic/claude-sonnet"
temperature = 0.2

[permissions]
file_read = "allow"
file_write = "ask"
shell = "ask"
network = "ask"
git = "ask"
mcp = "ask"

[context]
max_tokens = 80000
include_rules = ["coding", "testing", "security"]

[prompts]
system = """
You are opencode-rs, a repo-aware AI coding agent.
Work safely, explain plans, show diffs, and validate changes.
Prefer minimal, testable, maintainable changes.
"""
```

## 29.2 Skill Definition

```toml
version = 1
name = "rust-test-generator"
description = "Generate Rust tests for changed behavior."
version_tag = "0.1.0"

[inputs]
scope = { type = "string", required = false }
task = { type = "string", required = true }

[execution]
agent = "rust-implementer"
permission_mode = "ask"

[outputs]
format = "markdown"
```

## 29.3 Hook Definition

```toml
version = 1
name = "run-format-before-validation"
event = "before_validation"
enabled = true
timeout_ms = 10000
failure_behavior = "block"

[action]
kind = "shell"
command = "cargo fmt --all"

[permissions]
shell = "allow"
```

## 29.4 Rule Definition

```markdown
---
name: rust-cli-ux
type: coding
severity: warning
---

# Rust CLI UX Rule

- Every command must have clear help text.
- Every error must include a suggested fix.
- Non-interactive mode must never prompt.
- JSON output must be stable and schema-tested.
- Use structured exit codes.
```

---

# Final Implementation Guidance

The recommended implementation order is:

1. Build `opencode-cli`, `opencode-config`, `opencode-provider`, and `opencode-session`.
2. Implement `doctor`, `provider`, `model`, `auth`, `config`, and `init`.
3. Implement minimal `run`, `chat`, `diff`, `apply`, and `validate`.
4. Add structured logs and trace storage early.
5. Add permission policy before enabling file writes.
6. Add JSON/NDJSON output contracts before CI support.
7. Add skills, hooks, rules, and MCP in V1.
8. Add multi-agent orchestration, replay UI, TUI, and remote service in V2.
