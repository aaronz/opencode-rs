# Constitution Updates - Iteration 4

**Generated:** 2026-04-10
**Based on Gap Analysis:** `iteration-4/gap-analysis.md`
**Previous Constitution:** `iteration-1/constitution_updates.md`
**Status:** Amendment Proposal for Existing Constitution

---

## Executive Summary

The existing Constitution (from Iteration 1) covers foundational principles and some P0 items, but Iteration 4's gap analysis reveals **20 new P0 blockers** in areas that were either unimagined or only theoretical in Iteration 1:

- **Agent System**: Subagent execution, Task/delegation, Primary agent invariants
- **Plugin System**: Hook execution order, Tool registration, Config ownership
- **MCP/LSP Systems**: Transport layer implementations
- **Server API**: Auth enforcement, Resource grouping, Lifecycle management
- **Desktop/Web/ACP**: Shell implementations, Protocol transport

**Assessment:** The existing Constitution is **~40% adequate** for current P0 blockers. Major gaps exist in Articles II and III.

---

## Article I: Existing Constitution Coverage Analysis

### Currently Covered P0 Items (Iteration-1 Constitution)

| P0 Item | Constitution Reference | Status in Iteration-4 |
|---------|----------------------|----------------------|
| Custom tool loader | Art II §2.2 | ✅ IMPLEMENTED |
| TUI Plugin TypeScript SDK | Art II §2.3 | ⚠️ Partial (moved to P1) |
| tui.json ownership | Art I §1.2 | ⚠️ Still problematic |
| Deprecated fields sunset | Art III §3.1 | ⚠️ Partial (4 fields remain) |
| Silent serde unknown handling | Art III §3.2 | ❌ Still using `#[serde(other)]` |
| Hardcoded thresholds | Art III §3.3 | ❌ Still hardcoded |
| GitHub workflow generation | Art IV §4.1 | ✅ IMPLEMENTED |
| GitLab CI component | Art IV §4.2 | ✅ IMPLEMENTED |

**Coverage Score: ~45% of Constitution items have been addressed**

---

## Article II: NEW P0 Requirements (Not Covered by Existing Constitution)

### Section 2.1 - Agent System Invariants

**Requirement:** The runtime MUST enforce agent invariants:

```rust
// INVARIANT: Session MUST have exactly one active primary agent
// No more than one agent with role "primary" may exist per session
// Subagents may only be spawned via the task/delegation mechanism

enum AgentRole {
    Primary,   // Exactly one per session
    Subagent,  // Child of primary, isolated history
    System,    // Internal (compaction, title, summary)
}

// CONSTRAINT: Primary agent switch requires:
// 1. Previous primary enters "parked" state
// 2. New primary assumes active role
// 3. Transition logged for audit
```

**P0 Gap Addressed:**
- "Exactly one active primary agent invariant not enforced"
- "Subagent execution (child context, result handoff) not implemented"
- "Task/delegation mechanism not implemented"

### Section 2.2 - Subagent Lifecycle Protocol

**Requirement:** Subagent execution MUST follow this protocol:

```
Parent Agent                    Child Agent
    |                              |
    |-- create_child_context() --> |
    |                              |
    |<-- context_id ---------------|
    |                              |
    |--- execute_task() --------->|
    |                              |
    |<-- task_result --------------|
    |                              |
    |-- commit_to_history() ------>/ (optional)
```

**Constraints:**
- Subagent history is NOT automatically merged into parent session history
- Result handoff is explicit (parent decides what to commit)
- Parent may revoke subagent access at any time
- Permission scope inherited but can be narrowed

**P0 Gap Addressed:**
- "Subagent execution not implemented"
- "Permission inheritance not implemented"

### Section 2.3 - Task/Delegation Tool Schema

**Requirement:** The `task` tool payload MUST be structured:

```rust
struct TaskPayload {
    // Target
    subagent_type: SubagentType,  // "build", "plan", "explore", "librarian", "oracle"
    
    // Delegation
    prompt: String,
    load_skills: Vec<String>,      // Skills to inject
    
    // Constraints
    timeout_ms: Option<u64>,
    permission_scope: PermissionScope,  // Subset of parent permissions
    
    // Execution control
    run_in_background: bool,       // Default: false (synchronous)
}

enum SubagentType {
    Build,      // Implementation tasks
    Plan,       // Planning/analysis
    Explore,    // Codebase investigation
    Librarian,  // External docs/examples
    Oracle,     // High-IQ consultation
}
```

**P0 Gap Addressed:**
- "Task/delegation mechanism not implemented"

---

## Article III: Plugin System Hardening

### Section 3.1 - Deterministic Hook Execution Order

**Requirement:** Plugin hooks MUST execute in deterministic order:

```rust
// BAD - HashMap iteration is non-deterministic
struct PluginHooks {
    hooks: HashMap<PluginId, Vec<HookCallback>>,  // ❌ WRONG
}

// GOOD - Use IndexMap or explicit ordering
struct PluginHooks {
    hooks: IndexMap<PluginId, Vec<HookCallback>>,  // ✅ ORDERED
}

// Hook execution order:
// 1. System hooks (always first)
// 2. Plugin hooks sorted by plugin_id (deterministic)
// 3. User hooks (always last)
```

**CONSTRAINT:** Hook registration MUST include explicit priority:
```rust
trait Plugin {
    fn hook_priority(&self) -> i32;  // Lower = earlier
    // Default: 0
    // System hooks: -100
    // User hooks: +100
}
```

**P0 Gap Addressed:**
- "Hook execution order non-deterministic (HashMap iteration)"

### Section 3.2 - Plugin-Provided Tool Registration

**Requirement:** Plugins MUST be able to register custom tools:

```rust
// Plugin MUST implement this method
trait Plugin {
    fn register_tools(&self) -> Vec<PluginTool> {
        vec![]  // Default: no tools
    }
}

struct PluginTool {
    name: String,
    description: String,
    input_schema: Value,
    handler: ToolHandler,
}

// Tool handler can be:
// - Inline Rust code
// - File path to executable
// - Node.js/deno script

enum ToolHandler {
    Inline { code: String, lang: String },
    Executable { path: String },
    Script { path: String, runtime: String },
}
```

**CONSTRAINT:** Tool name collision resolution:
```
Priority: Built-in > Plugin > Custom (file-based)
On collision: Warn in logs, first registered wins
```

**P0 Gap Addressed:**
- "Plugin-provided tool registration not implemented"

### Section 3.3 - Plugin Configuration Ownership Boundary

**Requirement:** Plugin config boundaries MUST be enforced:

```
┌─────────────────────────────────────────────┐
│ opencode.json (main config)                 │
│ ├── server.*                                │
│ ├── agent.*                                  │
│ ├── permission.*                            │
│ └── ...                                      │
├─────────────────────────────────────────────┤
│ tui.json (TUI config)                       │
│ ├── plugins.{plugin_id}.*  ← ONLY plugins   │
│ ├── theme.*                                 │
│ ├── keybindings.*                           │
│ └── ...                                      │
└─────────────────────────────────────────────┘
```

**CONSTRAINT:** `opencode.json` MUST NOT contain plugin-specific config. `tui.json` MUST NOT contain server/agent config.

**Validation:**
```rust
fn validate_config_boundary(config: &Config, tui_config: &TuiConfig) -> Result<()> {
    // Check no plugin keys leak into main config
    if let Some(plugins) = &config.plugins {
        return Err("Plugin config must be in tui.json, not opencode.json".into());
    }
    // Check no server/agent keys leak into tui_config
    if tui_config.server.is_some() || tui_config.agent.is_some() {
        return Err("Server/agent config must be in opencode.json, not tui.json".into());
    }
    Ok(())
}
```

**P0 Gap Addressed:**
- "Plugin config not separated from TUI plugin config"
- "Config ownership boundary not enforced"

---

## Article IV: MCP/LSP Transport Layer

### Section 4.1 - MCP Transport Implementation

**Requirement:** MCP MUST support both local and remote transports:

```rust
// Local: stdio transport (for local MCP servers)
struct LocalMcpTransport {
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

// Remote: HTTP+SSE transport (for cloud MCP servers)
struct RemoteMcpTransport {
    url: Url,
    headers: HashMap<String, String>,
    auth: Option<AuthContext>,
}

// JSON-RPC 2.0 over stdio or HTTP+SSE
// Tool discovery via tools/list protocol
// Server-qualified naming: "<servername>_<toolname>"
```

**CONSTRAINT:** Tool discovery MUST be dynamic:
```rust
async fn discover_mcp_tools(server: &McpServer) -> Result<Vec<Tool>> {
    // 1. Send tools/list request
    // 2. Parse response for tool definitions
    // 3. Qualify names: "filesystem_read" not "read"
    // 4. Return registered tools
}
```

**P0 Gap Addressed:**
- "Local MCP server connection not implemented"
- "Remote MCP server connection not implemented"
- "Tool discovery from MCP servers not implemented"

### Section 4.2 - LSP Diagnostics Pipeline

**Requirement:** LSP diagnostics MUST surface to runtime:

```
LSP Server → Diagnostics → OpenCode Runtime → UI/Tools
    │                                         │
    ├── PublishDiagnostics notification       │
    ├── TextDocumentSyncKind::Full           │
    └── Configured timeout (default: 5s)     │
```

**CONSTRAINT:** Graceful degradation:
```rust
struct LspDiagnostics {
    // If LSP fails, log warning but don't block
    // Store last known diagnostics
    // Allow retry on document open
}
```

**P0 Gap Addressed:**
- "LSP diagnostics not surfaced to runtime"
- "LSP failure handling not implemented"

---

## Article V: Server API Hardening

### Section 5.1 - Resource Group Route Registration

**Requirement:** Routes MUST be organized by resource group:

```rust
// Resource groups map to URL prefixes and auth scope
enum ResourceGroup {
    Global,     // /api/* - requires auth always
    Project,    // /api/projects/:id/* - requires project access
    Session,    // /api/sessions/:id/* - requires session ownership
    Message,    // /api/sessions/:id/messages/* - requires message ownership
}

struct RouteRegistration {
    path: String,
    group: ResourceGroup,
    handler: Handler,
    auth_required: bool,
}

// All routes in a group share auth scope
// Middleware validates group permissions
```

**P0 Gap Addressed:**
- "Route registration by resource group incomplete"

### Section 5.2 - Auth Enforcement Per-Endpoint

**Requirement:** Auth MUST be enforced at route level:

```rust
// Each route declares its auth requirement
#[route(method = "GET", path = "/api/sessions/:id", group = Session)]
async fn get_session(
    auth: AuthContext,  // Extracted by middleware
    session_id: Path<String>,
) -> Result<Session> {
    // Middleware guarantees:
    // 1. Valid token/session
    // 2. User owns the session OR has project access
    // 3. Token not expired
}
```

**CONSTRAINT:** No auth bypass allowed:
```rust
// BAD - Auth enforced inconsistently
async fn public_handler() -> Result<String> { ... }
async fn private_handler(auth: AuthContext) -> Result<String> { ... }

// GOOD - All handlers with consistent middleware
async fn handler(auth: AuthContext) -> Result<String> { ... }
```

**P0 Gap Addressed:**
- "Auth enforcement incomplete"
- "Session/message lifecycle CRUD incomplete"

### Section 5.3 - Session/Message CRUD Lifecycle

**Requirement:** Full lifecycle management:

```
Session Lifecycle:
  created → active ↔ idle → archived → deleted
             │                      │
             └── compacting ─────────┘

Message Lifecycle:
  pending → confirmed → committed → compacted
```

**CONSTRAINT:** Compaction preserves resumability:
```rust
struct CompactionResult {
    original_id: SessionId,
    compacted_id: SessionId,
    checkpoint: Checkpoint,  // Enough to resume
    message_count_before: usize,
    message_count_after: usize,
}
```

**P0 Gap Addressed:**
- "Session/message lifecycle CRUD incomplete"

---

## Article VI: Desktop/Web/ACP Implementation

### Section 6.1 - Desktop App Shell

**Requirement:** Desktop mode MUST integrate WebView:

```rust
// Desktop entry point
#[tokio::main]
async fn main() {
    let config = load_config();
    
    // Start server in background
    let server = start_api_server(config.server).await?;
    
    // Open browser
    if config.desktop.auto_open_browser {
        open_browser(config.desktop.browser_url)?;
    }
    
    // Run WebView (cross-platform)
    run_webview(server.url()).await?;
}

// Cross-platform WebView via:
// - Windows: webview2-com
// - macOS:webkit2 (via cocoa)
// - Linux:webkit2gtk
```

**P0 Gap Addressed:**
- "Desktop app shell not implemented"

### Section 6.2 - ACP Protocol Transport

**Requirement:** ACP transport MUST be functional:

```rust
// ACP over HTTP+SSE (like Server-Sent Events)
// Client connects: POST /api/acp/connect
// Server responds: SSE stream with messages

struct AcpMessage {
    id: MessageId,
    from: ClientId,
    to: ClientId,
    payload: Value,
    timestamp: DateTime<Utc>,
}

// Handshake: POST /api/acp/handshake
// Ack: POST /api/acp/ack
```

**CONSTRAINT:** ACP is NOT HTTP polling - uses true SSE:
```rust
// Client subscribes to message stream
let stream = client.subscribe_acp(client_id).await?;
for await message in stream {
    process_message(message)?;
}
```

**P0 Gap Addressed:**
- "ACP transport not implemented"

---

## Article VII: Update Compliance Checklist

**Existing Checklist (from Iteration-1 Constitution):**
- [ ] Module has corresponding test in `tests/src/`
- [ ] Configuration has schema in `crates/core/src/config_schema.rs`
- [ ] Public API documented with doc comments
- [ ] No deprecated fields reintroduced
- [ ] Error handling explicit (no silent `#[serde(other)]`)
- [ ] Integration tests pass: `cargo test -p opencode-integration-tests`

**NEW Checklist Items:**

### Agent System
- [ ] Primary agent invariant tested (`exactly_one_active_primary`)
- [ ] Subagent context isolation verified
- [ ] Task tool payload schema validated
- [ ] Permission inheritance test coverage

### Plugin System
- [ ] Hook execution order deterministic (IndexMap verified)
- [ ] Plugin tool registration functional
- [ ] Config ownership boundary enforced
- [ ] Plugin failure containment tested

### MCP/LSP
- [ ] Local MCP stdio transport tested
- [ ] Remote MCP HTTP+SSE transport tested
- [ ] Tool discovery from MCP verified
- [ ] LSP diagnostics pipeline end-to-end test

### Server API
- [ ] Resource group route registration tested
- [ ] Auth middleware coverage verified
- [ ] Session CRUD lifecycle tested
- [ ] Message lifecycle tested

### Desktop/Web/ACP
- [ ] WebView integration tested (per platform)
- [ ] ACP handshake verified
- [ ] ACP message delivery verified
- [ ] Browser auto-open tested

---

## Appendix A: Gap → Constitution Mapping (Iteration 4)

| Gap ID | Gap Description | Constitution Reference |
|--------|-----------------|------------------------|
| G-A1 | Primary agent invariant not enforced | Art II §2.1 |
| G-A2 | Subagent execution missing | Art II §2.2 |
| G-A3 | Task/delegation not implemented | Art II §2.3 |
| G-P1 | Hook execution non-deterministic | Art III §3.1 |
| G-P2 | Plugin tool registration missing | Art III §3.2 |
| G-P3 | Plugin config boundary leak | Art III §3.3 |
| G-M1 | MCP transport not implemented | Art IV §4.1 |
| G-L1 | LSP diagnostics not surfaced | Art IV §4.2 |
| G-S1 | Route registration by group incomplete | Art V §5.1 |
| G-S2 | Auth enforcement incomplete | Art V §5.2 |
| G-S3 | Session/Message CRUD incomplete | Art V §5.3 |
| G-D1 | Desktop WebView not implemented | Art VI §6.1 |
| G-D2 | ACP transport not implemented | Art VI §6.2 |

---

## Appendix B: Deprecated Fields (Updated)

| Field | Location | Status | Remediation |
|-------|----------|--------|-------------|
| `mode` | Config | ⚠️ Still present | Remove in v4.0 |
| `tools` (Config) | Config | ⚠️ Still present | Remove in v4.0 |
| `theme` | Config | ⚠️ Still present | Move to tui.json |
| `keybinds` | TuiConfig | ⚠️ Still present | Move to tui.json |
| `layout` | Config | ✅ Removed | — |

---

## Appendix C: Hardcoded Values (Updated)

| Value | Location | Status | Config Location |
|-------|----------|--------|-----------------|
| `COMPACTION_START_THRESHOLD` | compaction.rs | ❌ Still hardcoded | `config.agent.compaction.start_threshold` |
| `COMPACTION_FORCE_THRESHOLD` | compaction.rs | ❌ Still hardcoded | `config.agent.compaction.force_threshold` |
| `MCP_TIMEOUT_MS` | mcp.rs | ❌ Still hardcoded | `config.mcp.timeout_ms` |
| `LSP_TIMEOUT_MS` | lsp.rs | ❌ Still hardcoded | `config.lsp.timeout_ms` |

---

## Appendix D: New Technical Debt

| Item | Severity | Description | Remediation |
|------|----------|-------------|-------------|
| HashMap in plugin hooks | P0 | Non-deterministic iteration | Replace with IndexMap |
| ACP stubs in cli | P0 | Transport not implemented | Implement HTTP+SSE |
| MCP stubs in mcp/ | P0 | Transport not implemented | Implement stdio + HTTP+SSE |
| LSP stubs in lsp/ | P0 | Diagnostics not surfaced | Implement pipeline |

---

## Recommendations

### Immediate Actions (Next Sprint)

1. **Fix HashMap → IndexMap** in plugin hooks (Art III §3.1)
2. **Implement ACP transport** (Art VI §6.2)
3. **Implement MCP transport** (Art IV §4.1)
4. **Enforce config boundary** (Art III §3.3)

### Short-term (2-4 sprints)

5. **Implement subagent lifecycle** (Art II §2.2)
6. **Complete server auth enforcement** (Art V §5.2)
7. **Desktop WebView integration** (Art VI §6.1)
8. **LSP diagnostics pipeline** (Art IV §4.2)

### Medium-term

9. **Task/delegation mechanism** (Art II §2.3)
10. **Plugin tool registration** (Art III §3.2)
11. **Session/Message CRUD** (Art V §5.3)

---

*This constitution update addresses gaps identified in iteration-4/gap-analysis.md. New articles II-VI address P0 blockers not covered by the original iteration-1 constitution.*

*Total P0 blockers in Iteration-4: 20*
*Covered by existing Constitution: ~8*
*New P0 requirements added: 13 sections across 6 articles*

*Report generated: 2026-04-10*
