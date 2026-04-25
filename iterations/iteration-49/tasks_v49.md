# Task List v49 — CLI Module Implementation

**Document Version:** 49.1
**Updated:** 2026-04-26
**Based on:** CLI Module Specification v1.0 (Spec Version 49)

---

## P0 — Must Fix (Blockers)

### Task 1: Implement `run --format ndjson/json`
- **FR:** FR-006
- **Priority:** P0
- **Gap:** `cmd/run.rs:228-255` — placeholder output
- **Actions:**
  - [ ] Load model from Config (not hardcoded "gpt-4o")
  - [ ] Initialize LLM provider from config
  - [ ] Send prompt to LLM with streaming
  - [ ] Write events to stdout via NdjsonSerializer
  - [ ] Handle tool calls and results in output
- **Verify:** `cargo test -p opencode-cli run_command`

### Task 2: Implement `config set`
- **FR:** FR-005
- **Priority:** P0
- **Gap:** `cmd/config.rs:199-202` — always errors
- **Actions:**
  - [ ] Parse dot-notation key (e.g., `agent.model`)
  - [ ] Validate value type matches expected type
  - [ ] Persist to config file via Config system
  - [ ] Reload config to apply changes
- **Verify:** `opencode config set agent.model gpt-4o && opencode config get agent.model`

### Task 3: Implement `agent run`
- **FR:** FR-004
- **Priority:** P0
- **Gap:** `cmd/agent.rs:48` — only prints debug
- **Actions:**
  - [ ] Query AgentRegistry for requested agent type
  - [ ] Initialize LLM provider from Config
  - [ ] Execute agent with provided prompt
  - [ ] Stream output via NdjsonSerializer if `--format` set
- **Verify:** `cargo test -p opencode-cli agent_command`

---

## P1 — High Priority

### Task 4: Implement `agent list`
- **FR:** FR-011
- **Priority:** P1
- **Gap:** `cmd/agent.rs:48` — not implemented
- **Status:** ✅ Done
- **Actions:**
  - [x] Query AgentRegistry for all registered agents
  - [x] Display agent name, description, capabilities
  - [x] Support `--verbose` flag for detailed info
- **Verify:** `opencode agent list`

### Task 5: Implement `account login/logout/status`
- **FR:** FR-007
- **Priority:** P1
- **Gap:** `cmd/account.rs` — returns "not_implemented"
- **Actions:**
  - [ ] Integrate with `opencode-auth` crate
  - [ ] Implement browser-based OAuth flow
  - [ ] Store credentials securely
  - [ ] Display authentication status
- **Verify:** `opencode account status`

### Task 6: Implement `attach` command
- **FR:** FR-008
- **Priority:** P1
- **Gap:** `cmd/attach.rs:83-90` — only prints URL/id
- **Actions:**
  - [ ] Connect via ACP protocol to remote session
  - [ ] Handle local session attachment
  - [ ] Manage session state transfer
- **Verify:** `opencode attach --session-id <id>`

### Task 7: Implement `mcp add`
- **FR:** FR-009
- **Priority:** P1
- **Gap:** `cmd/mcp.rs` — Add variant missing
- **Actions:**
  - [x] Add `Add` variant to `McpAction` enum
  - [x] Validate MCP server command exists
  - [x] Persist MCP server configuration
  - [x] Initialize MCP client connection
- **Verify:** `opencode mcp add --name my-server --command npx --args '["mcp", "serve"]'`
- **Status:** ✅ Done

### Task 8: Implement `session review/diff`
- **FR:** FR-010
- **Priority:** P1
- **Gap:** `cmd/session.rs:991-1011` — stubs only
- **Actions:**
  - [x] Load session messages and context
  - [x] Implement file diff using session state changes
  - [x] Show review of modified files
  - [x] Format output for terminal display
- **Verify:** `opencode session review <id> --file <path>`
- **Status:** ✅ Done

### Task 9: Implement `session fork` TUI integration
- **FR:** FR-014
- **Priority:** P1
- **Gap:** `cmd/session.rs:673-698` — prints JSON only
- **Actions:**
  - [ ] Create fork via SessionSharing
  - [ ] Transfer session context properly
  - [ ] Integrate with TUI for session management
- **Verify:** `opencode session fork <id>`

### Task 10: Implement `github install` persistence
- **FR:** FR-015
- **Priority:** P1
- **Gap:** `cmd/github.rs:189-211` — workflow not saved
- **Actions:**
  - [ ] Write workflow to `.opencode/` directory
  - [ ] Persist to workspace config
  - [ ] Verify installation on subsequent runs
- **Verify:** `opencode github install <workflow> --workspace .`

### Task 11: Extend `providers login` multi-provider
- **FR:** FR-016
- **Priority:** P1
- **Gap:** `cmd/providers.rs:142-145` — OpenAI only
- **Actions:**
  - [ ] Extend browser auth to Anthropic
  - [ ] Add Google AI Studio support
  - [ ] Add Azure OpenAI support
  - [ ] Add custom provider support
- **Verify:** `opencode providers login --provider anthropic`

### Task 12: Implement `acp handshake` session storage
- **FR:** FR-017
- **Priority:** P1
- **Gap:** `cmd/acp.rs:250-292` — session not stored
- **Actions:**
  - [ ] Store ACP session after successful handshake
  - [ ] Implement session recovery on reconnect
  - [ ] Handle session expiration
- **Verify:** `opencode acp handshake --client-id <id> --capabilities chat,tasks`

### Task 13: Fix `config migrate`
- **FR:** FR-018
- **Priority:** P1
- **Gap:** `cmd/config.rs:204-208` — stub errors
- **Actions:**
  - [ ] Implement TOML→JSONC migration OR
  - [ ] Remove flag with deprecation notice
- **Verify:** `opencode config migrate` (or deprecation warning)

### Task 14: Parse environment variables
- **FR:** FR-019
- **Priority:** P1
- **Gap:** `main.rs` — not parsed before App init
- **Actions:**
  - [ ] Parse `OPENCODE_AUTO_SHARE`
  - [ ] Parse `OPENCODE_CONFIG`
  - [ ] Parse `OPENCODE_CONFIG_DIR`
  - [ ] Parse `OPENCODE_DISABLE_AUTOUPDATE`
  - [ ] Parse `OPENCODE_ENABLE_EXA`
  - [ ] Parse `OPENCODE_SERVER_PASSWORD`
  - [ ] Apply before Config loading
- **Verify:** Check env vars are applied at startup

### Task 15: Move model visibility to Config
- **FR:** FR-020
- **Priority:** P1
- **Gap:** `cmd/models.rs:238-267` — flat JSON file
- **Actions:**
  - [ ] Update Config schema for model visibility
  - [ ] Migrate existing visibility settings
  - [ ] Deprecate flat JSON file
- **Verify:** `opencode models visibility --model <id> --visible true`

### Task 16: Load default model from Config
- **FR:** FR-021
- **Priority:** P1
- **Gap:** `cmd/run.rs:226` — hardcoded "gpt-4o"
- **Actions:**
  - [ ] Replace hardcoded "gpt-4o" with Config lookup
  - [ ] Use `config.get("agent.model")` with fallback
- **Verify:** `opencode run "test"` uses Config default

---

## P2 — Enhancement

### Task 17: Implement `auth login`
- **FR:** FR-012
- **Priority:** P2
- **Actions:**
  - [ ] Support multiple providers (OpenAI, Anthropic)
  - [ ] Browser-based OAuth flow
  - [ ] Secure credential storage
  - [ ] Display login status
- **Verify:** `opencode auth login --provider openai`

### Task 18: Implement `agent create`
- **FR:** FR-013
- **Priority:** P2
- **Actions:**
  - [ ] Create agent configuration
  - [ ] Register with AgentRegistry
  - [ ] Persist to config
- **Verify:** `opencode agent create --name my-agent --type custom`

---

## Technical Debt Tasks

### Task 19: Fix provider browser unwrap
- **Location:** `cmd/providers.rs:67-88`
- **Actions:** Replace `unwrap()` with proper error propagation
- **Verify:** `cargo clippy -p opencode-cli -- -D warnings`

### Task 20: Consolidate `load_config()`
- **Location:** Multiple cmd files
- **Actions:** Create shared helper for config loading
- **Verify:** All commands still work after refactor

### Task 21: Remove hardcoded "cmd+k"
- **Location:** `cmd/config.rs:217-218`
- **Actions:** Move keybinds to Config system
- **Verify:** Keybinds still work from Config

### Task 22: Make GitHub API URL configurable
- **Location:** `cmd/github.rs:5-8`
- **Actions:** Read from Config instead of hardcode
- **Verify:** `OPENCODE_GITHUB_API_URL` env var works

---

## Completion Criteria

- [ ] All P0 tasks complete and verified
- [ ] All P1 tasks complete and verified
- [ ] `cargo build -p opencode-cli --all-features` passes
- [ ] `cargo test -p opencode-cli` passes
- [ ] `cargo clippy -p opencode-cli -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
