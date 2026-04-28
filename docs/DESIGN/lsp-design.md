# LSP / Code Intelligence Architecture for `opencode-rs`

## 1. Executive Summary

`opencode-rs` should **not** treat LSP as an IDE feature. It should treat LSP as one backend inside a broader **agent-oriented code intelligence subsystem**.

The recommended direction is:

| Decision                                  | Recommendation                                                                                                                                                                                                                       |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Reuse existing LSP servers?               | **Yes.** Reuse `rust-analyzer`, `typescript-language-server`, `gopls`, `pyright` / `ruff` / `jedi-language-server`, `clangd`, `jdtls`, etc.                                                                                          |
| Build an internal LSP client layer?       | **Yes.** `opencode-rs` should own a robust Rust LSP client runtime for process management, request routing, diagnostics, file sync, timeouts, and observability.                                                                     |
| Build abstraction above LSP?              | **Absolutely yes.** Agents should not call raw LSP directly. They should call stable tools like `find_symbol`, `impact_analysis`, `context_pack_for_task`, `safe_rename`, and `changed_file_diagnostics`.                            |
| Redesign from bottom up?                  | **Partially.** Do not rebuild language intelligence from scratch, but do redesign the orchestration, context packing, edit validation, ranking, caching, and agent-facing API.                                                       |
| Combine with tree-sitter/search/indexing? | **Yes.** LSP alone is insufficient for long-running, multi-file, repo-aware AI coding workflows. Combine LSP + tree-sitter + ripgrep-style search + symbol index + dependency graph + optional semantic index + runtime/MCP context. |

The ideal architecture is:

```text
Agent / Skills / Commands / Hooks / Subagents
        ↓
Agent Tool API
        ↓
Code Intelligence Service
        ↓
LSP Client Layer ─ Tree-sitter Layer ─ Search/Index Layer ─ Context Packer ─ Validation Layer
        ↓
Language Servers / Repository / File System / Git / Shell / MCP
```

Use existing Rust crates where they are strong:

* `lsp-types` for protocol types.
* `async-lsp` or a custom Tokio JSON-RPC transport for client implementation.
* Study Helix’s separation of `helix-lsp`, `helix-lsp-types`, editor core, and TUI layers as a useful architectural pattern. Helix explicitly has a `helix-lsp` language server client crate and `helix-lsp-types` type definitions, separated from its TUI/editor layers. ([GitHub][1])
* Use `tree-sitter`, `ignore`, `globset`, `walkdir`, `grep` / ripgrep crates, `tantivy` or SQLite/redb for indexing, and `tracing` for observability.
* Avoid `tower-lsp` as the primary LSP client foundation because it is mainly an LSP **server** abstraction, although it re-exports `lsp-types` and is useful if `opencode-rs` later implements its own server. ([Docs.rs][2])
* Consider `async-lsp` because it supports both Language Server and Language Client abstractions, middleware, tracing, lifecycle, cancellation, and request dispatch patterns. ([Docs.rs][3])

The most important design principle:

> LSP answers “what does the language server know?”
> Code intelligence for agents answers “what does the agent need to safely understand, change, validate, and explain this repository?”

---

# 2. Why Coding Agents Need More Than Traditional LSP

Traditional LSP usage is designed around a human editor:

```text
Human opens file → editor sends didOpen/didChange → user requests hover/completion/definition → editor renders UI
```

A coding agent has different needs:

```text
User gives task → planner explores repo → context is packed for LLM → implementer edits many files
→ diagnostics/tests/git diff are evaluated → reviewer validates result → final report is produced
```

## 2.1 IDE-Oriented LSP Usage

Traditional IDE LSP focuses on:

* Completion
* Hover
* Go to definition
* Find references
* Diagnostics
* Rename
* Formatting
* Code actions
* Semantic tokens
* Inlay hints
* Call/type hierarchy

The LSP 3.17 spec includes advanced features such as type hierarchy, inline values, inlay hints, notebook support, and a protocol meta model. ([Microsoft GitHub][4])

This is useful, but still editor-centered.

## 2.2 Agent-Oriented Code Intelligence Usage

AI coding agents need more than one-shot LSP calls.

They need:

### Multi-file reasoning

Agents need to understand:

* Entry points
* Related modules
* Tests
* Callers and callees
* Configuration files
* Generated files
* Dependency boundaries
* Ownership boundaries
* Cross-language edges

LSP can provide references and definitions, but it does not decide which files matter for a task.

### Symbol dependency exploration

A planner may need to ask:

```text
What symbols are related to AuthProvider?
Which files define it?
Which tests cover it?
Which commands instantiate it?
Which runtime config influences it?
Which other modules depend on it?
```

Raw LSP does not provide this complete task graph. `opencode-rs` needs to build it by combining LSP, tree-sitter, search, import graphs, test discovery, git history, and runtime/MCP metadata.

### Context packing for LLMs

An IDE renders information to a human. A coding agent must compress information into an LLM context window.

That means the system must rank and pack:

* Symbol definition
* Type signatures
* Relevant references
* Neighboring functions
* Tests
* Diagnostics
* Recent diffs
* Rules
* Project conventions
* Runtime logs
* Prior agent decisions

LSP provides data. The agent code-intel subsystem must transform it into **LLM-usable context**.

### Safe code modification

Agents need to edit code safely. Before editing, they need:

* Exact symbol range
* Current file version
* Edit conflict check
* Formatting expectations
* Whether symbol is generated or vendored
* Whether edits affect public API
* Whether tests should be run

After editing, they need:

* Changed-file diagnostics
* Project diagnostics
* Compile/test result correlation
* Rollback plan
* Final quality gate

Traditional LSP does not own this entire loop.

### Pre-edit validation

Before modifying a symbol, the agent should verify:

```text
Is this the symbol the user meant?
Is the file writable?
Is the range still current?
Is this generated code?
Does this symbol have references outside the edited file?
Is there a safer rename/refactor operation?
```

### Post-edit diagnostics

After edits, the agent should trigger:

* LSP diagnostics refresh
* Formatting
* Compiler check
* Test impact analysis
* Targeted test execution
* Git diff review

### Refactor planning

For refactors, the planner needs:

* Symbol graph
* File dependency graph
* Public API boundaries
* Tests likely affected
* Risk classification
* Step-by-step edit plan

LSP can provide `prepareRename`, references, call hierarchy, and type hierarchy, but the agent needs a higher-level refactor workflow. The LSP trait docs for `tower_lsp` also reflect that call hierarchy and type hierarchy are multi-step flows: first prepare an item, then resolve incoming/outgoing calls or supertypes/subtypes. ([Docs.rs][5])

### Cross-language repositories

Modern repositories include:

* Rust + TypeScript
* Go + Python
* Java + YAML
* SQL + protobuf
* Shell + Docker
* Terraform + CI config

A single LSP server cannot understand the entire repository. Helix’s language support list shows a practical ecosystem pattern: many languages have syntax support, tree-sitter support, and one or more default language servers, but support differs by language. ([Helix][6])

`opencode-rs` needs graceful degradation.

### Long-running agent sessions

In an agent session:

* Files change frequently.
* Some edits are unsaved or pending.
* LSP diagnostics may lag.
* The agent may perform shell commands outside LSP awareness.
* Several subagents may inspect the same repository.
* Context snapshots must remain reproducible.

The state model matters much more than in a basic CLI tool.

### Headless / TUI usage

`opencode-rs` is terminal-first. There may be no editor UI.

So the LSP subsystem must expose:

```bash
opencode-rs lsp status
opencode-rs lsp logs
opencode-rs code-intel explain path/to/file.rs:42
opencode-rs code-intel diagnostics --changed
opencode-rs code-intel context --task "refactor auth provider"
```

The LSP subsystem is not a UI feature. It is an agent runtime service.

---

# 3. Design Goals

## 3.1 Language-Agnostic Intelligence

Expose stable concepts:

```rust
Symbol
Location
Range
Diagnostic
Reference
Definition
CallEdge
DependencyEdge
ContextItem
EditPlan
ValidationResult
```

Agents should not depend on language-specific server quirks.

## 3.2 Low Latency for Interactive Agent Workflows

The system should support:

* Fast startup
* Lazy language server startup
* Immediate text search fallback
* Progressive indexing
* Partial answers
* Timeout-aware tools
* Cached diagnostics

## 3.3 Robustness Under Frequent File Edits

Agent edits are more bursty than human edits. The subsystem must handle:

* Atomic patch application
* Debounced LSP notifications
* Versioned document state
* Stale response rejection
* Revalidation after edit
* Rollback if diagnostics explode

## 3.4 Large Repository Support

Must handle:

* Monorepos
* Multi-root workspaces
* Generated code
* Vendor directories
* Large dependency trees
* Language-specific project roots
* Partial indexing

## 3.5 Progressive Indexing

Do not block startup on full indexing.

Recommended order:

```text
1. File discovery
2. Language detection
3. Text search ready
4. Tree-sitter parse hot files
5. LSP startup for active languages
6. Symbol index
7. Dependency index
8. Optional semantic index
```

## 3.6 Graceful Degradation

If LSP is missing:

```text
LSP unavailable → tree-sitter symbols + ripgrep + file index
```

If tree-sitter grammar is missing:

```text
tree-sitter unavailable → text search + file heuristics
```

If semantic index is unavailable:

```text
semantic unavailable → lexical ranking + symbol graph
```

## 3.7 Stable Tool Interface for Agents

Agents should use stable tools:

```text
find_symbol
go_to_definition
find_references
diagnostics
context_pack_for_task
safe_rename
impact_analysis
```

Not raw protocol calls like:

```text
textDocument/definition
workspace/symbol
textDocument/references
```

## 3.8 Reproducible Diagnostics and Context Retrieval

Each agent run should be able to record:

* File versions
* Tool calls
* LSP server version
* Diagnostics snapshot
* Context items included
* Why each context item was selected

## 3.9 Easy Integration with MCP / Skills / Commands / Rules / Hooks

The code intelligence layer should be callable by:

* Planner
* Implementer
* Reviewer
* Skills
* Slash commands
* Hooks
* MCP-backed runtime tools
* TUI commands

## 3.10 Good Observability

Every LSP request should be traceable:

```text
request_id
language_server
method
file
duration_ms
timeout
result_count
error
redaction_status
agent_task_id
```

---

# 4. Non-Goals

Initial versions should **not** try to be:

1. A full IDE.
2. A replacement for `rust-analyzer`, `gopls`, `clangd`, `pyright`, etc.
3. A perfect compiler.
4. A universal semantic analyzer for all languages.
5. A heavy centralized indexer that blocks startup.
6. A cloud-only semantic intelligence system.
7. A magical cross-language compiler graph.
8. A UI completion engine.
9. A real-time collaborative editor.
10. A custom language server for every language.

`opencode-rs` should be a **consumer, orchestrator, normalizer, and agent-aware intelligence layer**, not a new IDE platform at the start.

---

# 5. Proposed High-Level Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│ Agent Runtime                                                │
│ Planner │ Implementer │ Reviewer │ Skills │ Commands │ Hooks │
└───────────────────────────────┬─────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────┐
│ 5.1 Agent Tool Interface Layer                               │
│ Stable tools: find_symbol, diagnostics, context_pack, rename │
└───────────────────────────────┬─────────────────────────────┘
                                │
┌───────────────────────────────▼─────────────────────────────┐
│ 5.2 Code Intelligence Service Layer                          │
│ Routing │ Normalization │ Ranking │ Cache │ Versioning       │
└──────┬──────────────┬──────────────┬──────────────┬──────────┘
       │              │              │              │
┌──────▼──────┐ ┌─────▼──────┐ ┌─────▼──────┐ ┌─────▼──────────┐
│ 5.3 LSP     │ │ 5.4 Syntax │ │ 5.5 Search │ │ 5.6 Context    │
│ Client      │ │ Structure  │ │ Index      │ │ Packaging      │
└──────┬──────┘ └─────┬──────┘ └─────┬──────┘ └─────┬──────────┘
       │              │              │              │
┌──────▼──────────────▼──────────────▼──────────────▼──────────┐
│ 5.7 Edit Validation Layer                                    │
│ Pre-edit checks │ diagnostics gates │ tests │ rollback       │
└──────────────────────────────────────────────────────────────┘
```

---

## 5.1 Agent Tool Interface Layer

This is the public interface exposed to:

* Planner agent
* Implementer agent
* Reviewer agent
* Subagents
* Skills
* Commands
* Hooks
* MCP tools
* CLI/TUI commands

Example tools:

```text
find_symbol
go_to_definition
find_references
hover
workspace_symbols
document_symbols
diagnostics
semantic_search
dependency_graph
call_hierarchy
type_hierarchy
prepare_rename
safe_rename
impact_analysis
changed_file_diagnostics
explain_symbol
context_pack_for_task
```

Design principle:

```text
Agent-facing tools should be task-oriented, not protocol-oriented.
```

Bad agent API:

```json
{
  "method": "textDocument/definition",
  "params": {}
}
```

Good agent API:

```json
{
  "tool": "go_to_definition",
  "input": {
    "workspace": "repo",
    "file": "src/auth/provider.rs",
    "position": { "line": 42, "character": 17 },
    "include_context": true,
    "fallback": ["tree_sitter", "text_search"]
  }
}
```

The agent should not need to know whether the result came from:

* LSP
* tree-sitter
* symbol index
* ripgrep
* semantic search
* generated cache

---

## 5.2 Code Intelligence Service Layer

This is the central orchestrator.

Responsibilities:

### Request Routing

Route each tool request to the best backend:

| Request                 | Primary Backend           | Fallback                     |
| ----------------------- | ------------------------- | ---------------------------- |
| `go_to_definition`      | LSP                       | tree-sitter + search         |
| `find_references`       | LSP                       | text search + symbol index   |
| `document_symbols`      | LSP or tree-sitter        | tree-sitter                  |
| `workspace_symbols`     | LSP + symbol index        | file index                   |
| `diagnostics`           | LSP                       | compiler/test output         |
| `semantic_search`       | embedding index           | text search                  |
| `context_pack_for_task` | hybrid                    | hybrid                       |
| `impact_analysis`       | symbol graph + references | search heuristics            |
| `safe_rename`           | LSP rename                | generated edits + validation |

### Result Normalization

Normalize different LSP server outputs into internal structs.

Example:

```rust
pub struct NormalizedSymbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub language: LanguageId,
    pub file: PathBuf,
    pub range: TextRange,
    pub selection_range: TextRange,
    pub container: Option<String>,
    pub visibility: Option<Visibility>,
    pub backend: BackendKind,
    pub confidence: Confidence,
}
```

### Ranking

Rank results by:

* Exact name match
* Symbol kind
* Current file proximity
* Import relationship
* Test/source relationship
* Recently changed files
* Diagnostics relevance
* Agent task query similarity
* Project rules
* Historical successful context

### Caching

Cache:

* File index
* Symbol index
* Parsed tree-sitter AST summaries
* LSP capabilities
* Diagnostics snapshots
* Context snapshots
* Tool traces

Do **not** blindly cache:

* Raw LSP server responses without version tags
* Diagnostics without file version
* Context packs without trace metadata

### File Version Tracking

Every request should know:

```rust
WorkspaceId
DocumentId
FileVersion
ContentHash
LspVersion
GitHead
DirtyState
```

### Partial Failure Handling

Example:

```text
typescript-language-server timed out
→ return tree-sitter + search result
→ mark confidence = degraded
→ record warning in tool output
```

---

## 5.3 LSP Client Layer

This layer manages real language servers.

### Responsibilities

```text
Process lifecycle
Initialization
Workspace folders
Capability negotiation
Request/response handling
Notification handling
Diagnostics stream
File open/change/save synchronization
Restart policy
Timeout policy
Per-language configuration
Multi-root repositories
Multiple language servers in one repository
```

### Recommended Internal Components

```text
lsp/
  client.rs          # typed LSP request API
  manager.rs         # manages all server sessions
  transport.rs       # stdio JSON-RPC framing
  lifecycle.rs       # initialize/shutdown/restart
  diagnostics.rs     # publish/pull diagnostics handling
  capabilities.rs    # capability registry
  sync.rs            # didOpen/didChange/didSave/didClose
  router.rs          # request routing by file/language/root
  config.rs          # per-language server config
  trace.rs           # request/response tracing
```

### Process Lifecycle

Each server session:

```rust
pub struct LspServerSession {
    pub id: LspServerId,
    pub language_id: LanguageId,
    pub root: WorkspaceRoot,
    pub command: LspCommand,
    pub process: ChildProcessHandle,
    pub capabilities: ServerCapabilities,
    pub state: LspServerState,
    pub started_at: Instant,
    pub last_health_check: Instant,
}
```

States:

```text
Discovered
Starting
Initializing
Ready
Degraded
Restarting
Stopped
Crashed
Disabled
```

### Initialization Flow

```text
1. Detect language/project root
2. Resolve server command
3. Spawn process
4. Send initialize
5. Receive capabilities
6. Send initialized
7. Register workspace folders
8. Open hot documents
9. Subscribe to diagnostics
10. Mark Ready
```

### Capability Negotiation

The manager should record:

```text
definitionProvider
referencesProvider
hoverProvider
documentSymbolProvider
workspaceSymbolProvider
renameProvider
prepareProvider
callHierarchyProvider
typeHierarchyProvider
codeActionProvider
documentFormattingProvider
diagnosticProvider
semanticTokensProvider
```

Not every server supports every method. The agent API must check capabilities before invoking backend operations.

### Request / Response Handling

Design requirements:

* Unique request IDs
* Timeout per method
* Cancellation support
* Stale response detection
* Structured error mapping
* Request deduplication where safe
* Backpressure for expensive methods

### Notification Handling

Handle:

```text
window/logMessage
window/showMessage
textDocument/publishDiagnostics
$/progress
client/registerCapability
client/unregisterCapability
workspace/configuration
workspace/workspaceFolders
```

### Diagnostics Stream

Diagnostics must be versioned:

```rust
pub struct DiagnosticSnapshot {
    pub workspace: WorkspaceId,
    pub file: PathBuf,
    pub file_version: FileVersion,
    pub source: DiagnosticSource,
    pub diagnostics: Vec<NormalizedDiagnostic>,
    pub received_at: DateTime<Utc>,
    pub server_id: Option<LspServerId>,
}
```

### File Sync

For agent edits:

```text
read file
apply patch
increment internal version
send didChange
debounce diagnostics
wait for diagnostics or timeout
run formatter if configured
run changed-file diagnostics gate
```

### Restart Policy

Recommended:

```text
Crash once → restart immediately
Crash repeatedly → exponential backoff
Crash after bad file → isolate file if possible
Server unavailable → degrade to tree-sitter/search
```

### Timeout Policy

Example defaults:

| Method            | Timeout |
| ----------------- | ------: |
| hover             |      2s |
| definition        |      3s |
| references        |      8s |
| workspace symbols |      8s |
| call hierarchy    |     10s |
| rename            |     15s |
| diagnostics wait  |     10s |
| initialize        |     20s |

### Multiple Language Servers

One repository may need:

```text
Rust: rust-analyzer
TypeScript: typescript-language-server + eslint
Python: pyright + ruff
YAML: yaml-language-server
Dockerfile: docker-langserver
```

The LSP manager should support multiple servers per file and per workspace.

Example:

```rust
pub enum ServerRole {
    PrimarySemantic,
    DiagnosticsOnly,
    Formatter,
    Linter,
    ConfigLanguage,
}
```

---

## 5.4 Syntax / Structure Layer

Use tree-sitter as the fast, local structural intelligence layer.

Tree-sitter is not semantically equivalent to LSP, but it is excellent for:

* File parsing
* Function/class/module boundaries
* Symbol extraction
* Import extraction
* Changed-region detection
* Chunking
* Language-aware search
* Fallback when LSP is unavailable
* Fast startup indexing

### Responsibilities

```text
Parse files
Extract symbols
Extract imports
Identify test functions
Identify public API declarations
Generate code chunks
Track changed regions
Support language-specific query files
```

### Recommended Structure

```text
syntax/
  tree_sitter.rs
  registry.rs
  queries/
    rust/
      symbols.scm
      imports.scm
      chunks.scm
      tests.scm
    typescript/
    python/
    go/
  symbols.rs
  imports.rs
  chunks.rs
  changed_regions.rs
```

### Chunking for LLM Context

Instead of naive line chunks, use structural chunks:

```text
Module chunk
Class chunk
Function chunk
Impl block chunk
Test case chunk
Config section chunk
```

Each chunk should include:

```rust
pub struct CodeChunk {
    pub id: ChunkId,
    pub file: PathBuf,
    pub language: LanguageId,
    pub range: TextRange,
    pub symbol_ids: Vec<SymbolId>,
    pub imports: Vec<ImportRef>,
    pub text_hash: String,
    pub token_estimate: usize,
    pub chunk_kind: ChunkKind,
}
```

### Fallback Example

If `rust-analyzer` is unavailable:

```text
find_symbol("AuthProvider")
→ tree-sitter symbol index
→ exact text search
→ import graph expansion
→ return degraded confidence result
```

---

## 5.5 Search and Index Layer

This layer combines:

* File index
* Text search
* Symbol index
* Dependency index
* Optional embedding/semantic index
* Generated metadata cache

### File Index

Tracks:

```text
path
language
size
hash
mtime
git status
ignored/generated/vendor/test/source classification
```

Use:

* `ignore` for `.gitignore`-aware walking
* `walkdir` for directory traversal when needed
* `globset` for ignore/include patterns
* ripgrep/`grep` crates for fast text search

### Symbol Index

Sources:

* LSP document symbols
* LSP workspace symbols
* tree-sitter extraction
* language-specific heuristics

Persistent fields:

```rust
pub struct IndexedSymbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub file: PathBuf,
    pub range: TextRange,
    pub container: Option<String>,
    pub language: LanguageId,
    pub source: SymbolSource,
    pub confidence: Confidence,
    pub updated_at_hash: String,
}
```

### Dependency Index

Start simple:

```text
imports
module references
Cargo.toml package dependency
package.json dependency
go.mod dependency
pyproject dependency
Maven/Gradle dependency
Docker/CI references
```

Later:

```text
call graph
type graph
test coverage graph
runtime dependency graph from MCP
```

### Semantic Index

Optional, not Phase 1.

Use for:

* Natural language task search
* Similar code search
* Rule matching
* Architecture pattern detection

But keep privacy/local-first default.

### Persistent vs Ephemeral

| Data                  | Persistent?           | Notes                    |
| --------------------- | --------------------- | ------------------------ |
| File index            | Yes                   | Hash-based invalidation  |
| Symbol index          | Yes                   | Rebuild per changed file |
| Tree-sitter summaries | Yes/optional          | Useful for large repos   |
| Raw AST               | No                    | Too heavy initially      |
| Diagnostics           | Ephemeral + snapshots | Version-sensitive        |
| Context packs         | Yes                   | For reproducibility      |
| LSP capabilities      | Yes                   | Per server version       |
| Semantic embeddings   | Optional              | Config-controlled        |
| Tool traces           | Yes                   | Debugging and audit      |

---

## 5.6 Context Packaging Layer

This is one of the most important agent-specific layers.

LSP can tell you where a definition is. It does not decide what to put into an LLM prompt.

### Responsibilities

```text
Context budget management
Symbol-centered context expansion
Definition + references + tests + diagnostics packing
Ranking
Deduplication
Staleness checks
Traceability
Context snapshots
```

### Context Packer Input

```rust
pub struct ContextPackRequest {
    pub task: String,
    pub focus_files: Vec<PathBuf>,
    pub focus_symbols: Vec<SymbolId>,
    pub changed_files: Vec<PathBuf>,
    pub token_budget: usize,
    pub include_tests: bool,
    pub include_diagnostics: bool,
    pub include_git_diff: bool,
    pub include_rules: bool,
}
```

### Context Packer Output

```rust
pub struct ContextPack {
    pub id: ContextSnapshotId,
    pub items: Vec<ContextItem>,
    pub token_estimate: usize,
    pub omitted_items: Vec<OmittedContextItem>,
    pub trace: ContextPackTrace,
    pub file_versions: Vec<FileVersionRef>,
}
```

### Ranking Rules

Prioritize:

1. Directly mentioned files/symbols.
2. Definitions of target symbols.
3. Current diagnostics.
4. Recently changed files.
5. Tests covering target symbols.
6. Direct references.
7. Callers/callees.
8. Imports and config.
9. Project rules and conventions.
10. Runtime/MCP evidence.

### Traceability

Each context item should explain why it was included:

```json
{
  "file": "src/auth/provider.rs",
  "range": "L20-L88",
  "reason": [
    "defines target symbol AuthProvider",
    "referenced by failing diagnostic in src/auth/mod.rs",
    "changed in current git diff"
  ],
  "backend": ["lsp", "tree_sitter", "git_diff"],
  "confidence": "high"
}
```

### Snapshot Reproducibility

Each context pack should record:

```text
git head
file hashes
dirty file versions
tool calls used
ranking parameters
token budget
timestamp
agent task id
```

---

## 5.7 Edit Validation Layer

This layer makes agent edits safe.

### Pre-Edit Lookup

Before edit:

```text
Resolve target symbol
Check file version
Check generated/vendor status
Check public API status
Check references
Check diagnostics baseline
Create rollback snapshot
```

### Edit Range Validation

Before applying patch:

```text
Range still matches expected text?
File version unchanged since context pack?
Patch applies cleanly?
No overlapping edits?
```

### Formatting

After edit:

```text
Run LSP formatting if available
Else run language formatter command
Else skip with warning
```

### Changed-File Diagnostics

After edit:

```text
Notify LSP didChange
Wait for diagnostics
Return diagnostics only for changed files
Classify new vs existing diagnostics
```

### Project Diagnostics

For larger edits:

```text
cargo check
npm test -- affected
go test ./...
pytest targeted tests
mvn test
```

### Rollback Support

Every tool-applied edit should support:

```text
rollback by patch
rollback by file snapshot
rollback by git checkout if clean
```

### Conflict Detection

Detect:

* File changed externally
* LSP stale version
* Patch mismatch
* Formatter changed unrelated regions
* Generated code edited
* Concurrent subagent edit conflict

### Final Quality Gates

Recommended gates:

```text
No new changed-file diagnostics
Formatter completed
Targeted tests pass
Git diff reviewed
No sensitive files touched
No generated/vendor files modified unless allowed
```

---

# 6. Agent Runtime Integration

## 6.1 Planner Agent

Planner uses code intelligence to:

```text
Understand task
Find relevant symbols
Build dependency graph
Locate tests
Estimate risk
Create edit plan
```

Example planner calls:

```text
workspace_symbols("AuthProvider")
dependency_graph(symbol="AuthProvider", depth=2)
find_references(symbol="AuthProvider")
context_pack_for_task(task="refactor auth provider")
```

## 6.2 Implementer Agent

Implementer uses:

```text
go_to_definition
document_symbols
safe_edit
changed_file_diagnostics
format_file
```

The implementer should not blindly edit files from text search results. It should resolve symbols and validate ranges.

## 6.3 Reviewer Agent

Reviewer uses:

```text
git diff
diagnostics
test results
impact_analysis
rules
context snapshots
```

It checks whether the implementation satisfies the task and does not introduce regressions.

## 6.4 Tool Executor

The tool executor enforces:

```text
Timeouts
Permissions
Workspace trust
Logging
Redaction
Rollback
Tool-call trace recording
```

## 6.5 File Editing System

The file editing system and LSP must share document versioning.

Flow:

```text
FileEditService applies patch
→ CodeIntel updates document state
→ LSP sync sends didChange
→ Diagnostics collector waits
→ Validation layer reports result
```

## 6.6 Git Subsystem

Git provides:

```text
Changed files
Diff hunks
Rollback baseline
Branch information
Ignored files
Generated/vendor detection hints
Final report
```

## 6.7 Shell Runner

Shell output can become diagnostics-like evidence:

```text
cargo check error
pytest failure
npm test failure
tsc error
```

Normalize shell errors into:

```rust
ExternalDiagnostic {
    source: "cargo check",
    file,
    range,
    message,
    severity,
}
```

## 6.8 MCP Subsystem

MCP can provide:

* Runtime logs
* API schemas
* Database schema
* Observability data
* Issue tracker context
* Deployment metadata

But MCP data should be treated as separate trust boundary.

## 6.9 Skills / Commands / Rules / Hooks

Examples:

| Extension Point                     | Code Intelligence Usage                               |
| ----------------------------------- | ----------------------------------------------------- |
| Skill: `rust-refactor`              | Uses symbol graph, `cargo check`, changed diagnostics |
| Command: `/explain-symbol`          | Calls `explain_symbol`                                |
| Rule: “Do not edit generated files” | Enforced by validation layer                          |
| Hook: `before_edit`                 | Runs pre-edit validation                              |
| Hook: `after_edit`                  | Runs formatting + diagnostics                         |
| Hook: `before_final`                | Runs final quality gate                               |

## 6.10 TUI / CLI Progress Display

Show high-level progress:

```text
Code intelligence:
  Rust analyzer: Ready
  TypeScript server: Starting
  File index: 12,430 files
  Symbol index: 8,920 symbols
  Diagnostics: 3 changed-file errors
  Context pack: 18 items, 21k tokens
```

---

## 6.11 Example Lifecycle: Refactor

```text
1. User:
   "Refactor AuthProvider to support OAuth and API key auth."

2. Planner:
   - find_symbol("AuthProvider")
   - find_references(AuthProvider)
   - dependency_graph(AuthProvider)
   - find_tests(AuthProvider)
   - context_pack_for_task(...)

3. Planner creates edit plan:
   - modify provider trait
   - update implementations
   - update tests
   - update config docs

4. Implementer:
   - go_to_definition(AuthProvider)
   - apply structured edits
   - send LSP didChange
   - format changed files

5. Diagnostics:
   - collect changed-file diagnostics
   - classify new errors

6. Shell:
   - run targeted tests
   - run cargo check

7. Reviewer:
   - inspect git diff
   - compare diagnostics baseline
   - verify tests
   - produce final report
```

---

# 7. Recommended Rust Implementation Strategy

## 7.1 `lsp-types`

### Good For

* Strongly typed LSP protocol structs.
* Shared by multiple Rust LSP implementations.
* Avoids hand-writing JSON types.

### Suitable for `opencode-rs`?

Yes. Use it directly in the LSP client layer.

### Complexity

Low.

### Risk

Some server-specific extensions may need custom structs.

### Recommendation

Use as the canonical protocol type crate.

---

## 7.2 `tower-lsp`

`tower-lsp` provides an LSP **server** abstraction for Tower and re-exports `lsp-types`. Its docs show server-side `LanguageServer`, `LspService`, and `Server` abstractions. ([Docs.rs][2])

### Good For

* Implementing your own language server.
* Building internal custom LSP-compatible services.
* Learning LSP server architecture.

### Suitable for `opencode-rs`?

Not as the primary client layer.

### Complexity

Medium if misused for client workflows.

### Risk

You may build around a server-oriented abstraction when you need a client.

### Recommendation

Do not use as the main LSP client. Consider it only if `opencode-rs` later exposes its own LSP server to editors.

---

## 7.3 `lsp-server`

### Good For

* Low-level JSON-RPC/LSP message handling.
* Commonly used in Rust LSP ecosystems.
* Useful if you want explicit control.

### Suitable for `opencode-rs`?

Potentially yes, especially if building a custom client transport.

### Complexity

Medium.

### Risk

You must implement lifecycle, routing, timeout, cancellation, and diagnostics logic yourself.

### Recommendation

Use if you want maximum control. Otherwise evaluate `async-lsp`.

---

## 7.4 `async-lsp`

`async-lsp` is an asynchronous LSP framework based on Tower. Its docs state that it centers on an `LspService` trait for either Language Servers or Language Clients, with a `MainLoop`, middleware for concurrency, panic handling, tracing, lifecycle, router dispatch, and support for custom middleware such as timeout and metering. ([Docs.rs][3])

### Good For

* Async LSP client/server architecture.
* Middleware-based lifecycle.
* Tracing integration.
* Request routing.
* Cancellation/concurrency patterns.

### Suitable for `opencode-rs`?

Yes, worth serious evaluation.

### Complexity

Medium.

### Risk

May still require custom wrapping for process lifecycle, multi-server routing, diagnostics normalization, and agent tool integration.

### Recommendation

Use as the likely base for the LSP transport/client loop if it fits your codebase. Wrap it behind your own `LspClient` trait so you can replace it later.

---

## 7.5 Helix-Inspired Implementation

Helix is a Rust editor with a separated architecture. Its architecture documentation lists distinct crates such as `helix-lsp` for the language server client, `helix-lsp-types` for LSP types, `helix-core` for editing primitives, `helix-view`, `helix-term`, and `helix-tui`. ([GitHub][1])

### Good For

* Inspiration for modular architecture.
* LSP client management ideas.
* Multi-language support configuration.
* TUI separation.
* Rope/snapshot ideas.

### Suitable for `opencode-rs`?

As architectural reference, yes.

### Complexity

High if directly copying.

### Risk

Editor assumptions differ from agent runtime assumptions.

### Recommendation

Study Helix’s separation of concerns. Do not copy its UI/editor assumptions wholesale.

---

## 7.6 Zed / Lapce / Other Editor Architectures

### Good For

* High-performance editor code intelligence patterns.
* Incremental indexing ideas.
* Multi-buffer/document state.
* Diagnostics UI patterns.

### Suitable for `opencode-rs`?

Useful as reference, not as direct dependency.

### Recommendation

Borrow concepts, not implementation.

---

## 7.7 `tree-sitter`

### Good For

* Fast parsing.
* Symbol extraction.
* Structural chunking.
* Changed-region detection.
* Language-aware fallback.

### Suitable for `opencode-rs`?

Yes, core dependency.

### Complexity

Medium.

### Risks

* Query maintenance per language.
* Syntax-level, not semantic.
* Grammar version drift.

### Recommendation

Use in Phase 1.

---

## 7.8 `ignore`, `walkdir`, `globset`

### Good For

* Repository scanning.
* `.gitignore` support.
* Include/exclude rules.
* Large repo traversal.

### Suitable for `opencode-rs`?

Yes.

### Recommendation

Use `ignore` as the default walker because respecting `.gitignore` is essential for developer tools.

---

## 7.9 `grep` / ripgrep crates

### Good For

* Fast text search.
* Regex search.
* Fallback for symbol/reference search.
* Agent exploration.

### Suitable for `opencode-rs`?

Yes.

### Recommendation

Use for Phase 1 search.

---

## 7.10 `tantivy`

### Good For

* Full-text search index.
* Ranked search.
* Large repositories.
* Persistent search index.

### Suitable for `opencode-rs`?

Phase 2 or 3.

### Complexity

Medium.

### Risk

Index freshness and storage overhead.

### Recommendation

Start without it. Add when repos become large enough.

---

## 7.11 `sled`, `redb`, SQLite

### `sled`

Good embedded KV store, but consider project maturity and operational behavior.

### `redb`

Good embedded Rust-native database option for local caches.

### SQLite

Excellent for inspectable, stable metadata cache.

### Recommendation

For `opencode-rs`, I would choose:

```text
Phase 1: JSON/MessagePack cache + SQLite optional
Phase 2: SQLite for metadata and traces
Phase 3: Tantivy for full-text index if needed
Phase 4: Optional vector index for semantic search
```

SQLite is especially good because users can debug it.

---

## 7.12 `tokio`

### Good For

* Process management.
* Async stdin/stdout.
* Timeouts.
* Concurrent server sessions.
* Background indexing.

### Suitable?

Yes. Core runtime dependency.

---

## 7.13 `serde`

### Good For

* Config.
* Protocol serialization.
* Tool input/output schema.
* Snapshot storage.

### Suitable?

Yes.

---

## 7.14 `tracing`

### Good For

* Structured logs.
* Request spans.
* Diagnostics timeline.
* Tool-call correlation.
* TUI debug mode.

### Suitable?

Yes. Mandatory.

---

# 8. Build vs Reuse Decision

## Option A: Minimal LSP Integration

Use existing LSP servers only for diagnostics and definitions.

| Dimension             | Evaluation                |
| --------------------- | ------------------------- |
| Development cost      | Low                       |
| Capability            | Low to medium             |
| Reliability           | Medium                    |
| Latency               | Good                      |
| Language coverage     | Good if servers installed |
| Maintenance burden    | Low                       |
| Fit for `opencode-rs` | Good only for early MVP   |
| Recommended phase     | Phase 1 only              |

### Pros

* Quick win.
* Diagnostics and definitions become available.
* Low complexity.

### Cons

* Not enough for agent planning.
* No strong context packing.
* No safe refactor workflow.
* Weak fallback behavior.

---

## Option B: Agent-Oriented LSP Client

Build a robust Rust LSP client and expose agent tools over it.

| Dimension             | Evaluation               |
| --------------------- | ------------------------ |
| Development cost      | Medium                   |
| Capability            | Medium to high           |
| Reliability           | High if implemented well |
| Latency               | Good with lazy startup   |
| Language coverage     | Good                     |
| Maintenance burden    | Medium                   |
| Fit for `opencode-rs` | Strong                   |
| Recommended phase     | Phase 2                  |

### Pros

* Good foundation.
* Reuses language servers.
* Agent API becomes stable.
* Diagnostics and references become reliable.

### Cons

* Still lacks deep context packing and repository graph.
* LSP server inconsistencies must be normalized.

---

## Option C: Full Code Intelligence Platform

Combine LSP, tree-sitter, search, symbol graph, semantic index, and context packing.

| Dimension             | Evaluation          |
| --------------------- | ------------------- |
| Development cost      | High                |
| Capability            | Very high           |
| Reliability           | High if incremental |
| Latency               | Good if progressive |
| Language coverage     | Strong              |
| Maintenance burden    | Medium to high      |
| Fit for `opencode-rs` | Best long-term fit  |
| Recommended phase     | Phase 3+            |

### Pros

* Designed for AI agents.
* Strong context retrieval.
* Supports refactoring workflows.
* Works even when LSP is partial.
* Enables impact analysis and safe editing.

### Cons

* More engineering work.
* Needs careful cache invalidation.
* Requires observability from day one.

---

## Option D: Custom Language Intelligence From Scratch

Avoid LSP and build everything internally.

| Dimension             | Evaluation                            |
| --------------------- | ------------------------------------- |
| Development cost      | Extremely high                        |
| Capability            | Low initially, maybe high years later |
| Reliability           | Low initially                         |
| Latency               | Depends                               |
| Language coverage     | Poor                                  |
| Maintenance burden    | Very high                             |
| Fit for `opencode-rs` | Bad                                   |
| Recommended phase     | Not recommended                       |

### Pros

* Maximum control.
* Potentially agent-native.

### Cons

* Reinvents language servers.
* Poor language coverage.
* Very expensive.
* Cannot match `rust-analyzer`, `gopls`, `clangd`, etc.

## Final Decision

Recommended path:

```text
Phase 1: Option A + tree-sitter/search foundation
Phase 2: Option B
Phase 3: Option C
Avoid Option D
```

---

# 9. Proposed Module Design for `opencode-rs`

```text
crates/
  code-intel/
    Cargo.toml
    src/
      lib.rs
      service.rs
      tool_api.rs
      types.rs
      error.rs

      lsp/
        mod.rs
        client.rs
        manager.rs
        transport.rs
        lifecycle.rs
        diagnostics.rs
        capabilities.rs
        sync.rs
        router.rs
        config.rs
        trace.rs

      syntax/
        mod.rs
        tree_sitter.rs
        registry.rs
        symbols.rs
        imports.rs
        chunks.rs
        changed_regions.rs
        queries/
          rust/
          typescript/
          python/
          go/

      search/
        mod.rs
        text_search.rs
        file_index.rs
        symbol_index.rs
        dependency_index.rs
        semantic_index.rs

      context/
        mod.rs
        packer.rs
        ranking.rs
        budget.rs
        dedupe.rs
        snapshot.rs
        explanation.rs

      validation/
        mod.rs
        edit_validator.rs
        diagnostics_gate.rs
        formatter.rs
        test_impact.rs
        rollback.rs
        quality_gate.rs

      cache/
        mod.rs
        store.rs
        schema.rs
        invalidation.rs

      config.rs
      telemetry.rs
      security.rs
```

## Module Responsibilities

### `lib.rs`

Public crate entry.

Exports:

```rust
CodeIntelService
CodeIntelConfig
AgentToolApi
```

### `service.rs`

Central orchestration.

Owns:

```rust
LspManager
SyntaxService
SearchIndex
ContextPacker
ValidationService
CacheStore
```

### `tool_api.rs`

Agent-facing tools.

Should expose stable typed APIs.

### `types.rs`

Shared normalized types:

```rust
Symbol
Location
Range
Diagnostic
ContextPack
EditPlan
ImpactAnalysis
```

### `error.rs`

Unified error model:

```rust
CodeIntelError
BackendUnavailable
Timeout
StaleDocument
UnsupportedLanguage
PermissionDenied
```

---

## `lsp/`

### `client.rs`

Typed client operations:

```rust
definition()
references()
hover()
document_symbols()
workspace_symbols()
rename()
formatting()
```

### `manager.rs`

Manages all server sessions.

### `transport.rs`

JSON-RPC stdio transport.

### `lifecycle.rs`

Initialize, shutdown, restart.

### `diagnostics.rs`

Diagnostics collection and normalization.

### `capabilities.rs`

Capability registry.

### `sync.rs`

Document synchronization.

### `router.rs`

Maps file/language/root to server sessions.

### `trace.rs`

Logs raw/structured LSP traffic with redaction.

---

## `syntax/`

### `tree_sitter.rs`

Parser wrapper.

### `registry.rs`

Language grammar registry.

### `symbols.rs`

Extracts syntax-level symbols.

### `imports.rs`

Extracts import/module dependencies.

### `chunks.rs`

Creates structural chunks.

### `changed_regions.rs`

Computes changed regions for targeted validation/context.

---

## `search/`

### `text_search.rs`

ripgrep-like search.

### `file_index.rs`

Workspace file index.

### `symbol_index.rs`

Persistent symbol index.

### `dependency_index.rs`

Import/module/dependency graph.

### `semantic_index.rs`

Optional embedding search.

---

## `context/`

### `packer.rs`

Builds LLM-ready context packs.

### `ranking.rs`

Ranks files/symbols/chunks.

### `budget.rs`

Token budget allocation.

### `dedupe.rs`

Avoids repeated content.

### `snapshot.rs`

Stores reproducible context snapshots.

### `explanation.rs`

Explains why context was included.

---

## `validation/`

### `edit_validator.rs`

Pre-edit and range validation.

### `diagnostics_gate.rs`

Changed-file/project diagnostics gate.

### `formatter.rs`

LSP or command-based formatting.

### `test_impact.rs`

Finds tests likely affected.

### `rollback.rs`

Rollback snapshots and patch reversal.

### `quality_gate.rs`

Final validation gate.

---

# 10. Public Agent Tool API

Below are core tools exposed to agents.

## 10.1 `find_symbol`

### Purpose

Find symbols by name or fuzzy query.

### Input

```json
{
  "query": "AuthProvider",
  "kind": ["trait", "struct", "function"],
  "workspace": "default",
  "limit": 20,
  "include_tests": false
}
```

### Output

```json
{
  "symbols": [
    {
      "name": "AuthProvider",
      "kind": "trait",
      "file": "src/auth/provider.rs",
      "range": {"start": [12,0], "end": [30,1]},
      "backend": ["lsp", "symbol_index"],
      "confidence": "high"
    }
  ]
}
```

### Failure Modes

* No results.
* LSP unavailable.
* Index stale.

### Preferred Backend

Hybrid: symbol index + LSP workspace symbols + tree-sitter.

---

## 10.2 `go_to_definition`

### Purpose

Resolve a symbol usage to its definition.

### Input

```json
{
  "file": "src/auth/mod.rs",
  "position": {"line": 42, "character": 18},
  "fallback": true
}
```

### Output

```json
{
  "definitions": [
    {
      "file": "src/auth/provider.rs",
      "range": {"start": [12,0], "end": [30,1]},
      "preview": "pub trait AuthProvider { ... }",
      "backend": "lsp",
      "confidence": "high"
    }
  ]
}
```

### Failure Modes

* LSP timeout.
* Ambiguous definition.
* Unsupported file type.

### Preferred Backend

LSP first, tree-sitter/search fallback.

---

## 10.3 `find_references`

### Purpose

Find references to a symbol.

### Input

```json
{
  "symbol": {
    "file": "src/auth/provider.rs",
    "position": {"line": 12, "character": 10}
  },
  "include_declaration": false,
  "limit": 200
}
```

### Output

```json
{
  "references": [
    {
      "file": "src/auth/oauth.rs",
      "range": {"start": [8,5], "end": [8,17]},
      "line": "impl AuthProvider for OAuthProvider {",
      "backend": "lsp"
    }
  ],
  "truncated": false
}
```

### Failure Modes

* LSP missing references provider.
* Too many results.
* Timeout.

### Preferred Backend

LSP first, text search fallback.

---

## 10.4 `hover`

### Purpose

Get type/documentation information.

### Input

```json
{
  "file": "src/auth/provider.rs",
  "position": {"line": 14, "character": 8}
}
```

### Output

```json
{
  "contents": "trait AuthProvider\n\nProvides authentication behavior.",
  "range": {"start": [12,0], "end": [30,1]},
  "backend": "lsp"
}
```

### Failure Modes

* No hover provider.
* Empty hover.

### Preferred Backend

LSP.

---

## 10.5 `document_symbols`

### Purpose

List symbols in a file.

### Input

```json
{
  "file": "src/auth/provider.rs"
}
```

### Output

```json
{
  "symbols": [
    {
      "name": "AuthProvider",
      "kind": "trait",
      "range": {"start": [12,0], "end": [30,1]},
      "children": []
    }
  ]
}
```

### Failure Modes

* Parse failure.
* Unsupported language.

### Preferred Backend

LSP or tree-sitter.

---

## 10.6 `diagnostics`

### Purpose

Retrieve diagnostics for workspace or files.

### Input

```json
{
  "scope": "changed_files",
  "severity": ["error", "warning"],
  "include_existing": false
}
```

### Output

```json
{
  "diagnostics": [
    {
      "file": "src/auth/oauth.rs",
      "range": {"start": [55,10], "end": [55,20]},
      "severity": "error",
      "message": "method not found",
      "source": "rust-analyzer",
      "is_new": true
    }
  ],
  "snapshot_id": "diag_123"
}
```

### Failure Modes

* Diagnostics stale.
* LSP not ready.
* Timeout waiting for diagnostics.

### Preferred Backend

LSP + compiler/test normalized diagnostics.

---

## 10.7 `context_pack_for_task`

### Purpose

Build LLM-ready context for a task.

### Input

```json
{
  "task": "Refactor AuthProvider to support OAuth and API key auth",
  "focus_symbols": ["AuthProvider"],
  "token_budget": 24000,
  "include_tests": true,
  "include_diagnostics": true,
  "include_git_diff": true
}
```

### Output

```json
{
  "snapshot_id": "ctx_456",
  "items": [
    {
      "file": "src/auth/provider.rs",
      "range": {"start": [1,0], "end": [80,0]},
      "reason": ["defines target symbol AuthProvider"],
      "token_estimate": 1200
    }
  ],
  "token_estimate": 18400,
  "omitted": []
}
```

### Failure Modes

* Budget exceeded.
* Stale file version.
* Missing symbol.

### Preferred Backend

Hybrid.

---

## 10.8 `prepare_rename`

### Purpose

Check whether a symbol can be renamed safely.

### Input

```json
{
  "file": "src/auth/provider.rs",
  "position": {"line": 12, "character": 10},
  "new_name": "CredentialProvider"
}
```

### Output

```json
{
  "can_rename": true,
  "symbol_range": {"start": [12,10], "end": [12,22]},
  "affected_files": 5,
  "backend": "lsp"
}
```

### Failure Modes

* Server does not support prepare rename.
* Invalid new name.
* Symbol not renameable.

### Preferred Backend

LSP.

---

## 10.9 `safe_rename`

### Purpose

Perform rename with validation and rollback.

### Input

```json
{
  "file": "src/auth/provider.rs",
  "position": {"line": 12, "character": 10},
  "new_name": "CredentialProvider",
  "run_diagnostics": true,
  "rollback_on_error": true
}
```

### Output

```json
{
  "applied": true,
  "changed_files": [
    "src/auth/provider.rs",
    "src/auth/oauth.rs",
    "src/auth/api_key.rs"
  ],
  "diagnostics": [],
  "rollback_id": "rb_789"
}
```

### Failure Modes

* Rename edit conflict.
* Diagnostics introduced.
* Formatter failed.
* Rollback failed.

### Preferred Backend

LSP rename + validation layer.

---

## 10.10 `impact_analysis`

### Purpose

Estimate what a change may affect.

### Input

```json
{
  "changed_files": ["src/auth/provider.rs"],
  "changed_symbols": ["AuthProvider"],
  "include_tests": true,
  "depth": 2
}
```

### Output

```json
{
  "affected_symbols": [],
  "affected_files": [],
  "recommended_tests": [],
  "risk": "medium",
  "reasoning_trace": []
}
```

### Failure Modes

* Incomplete references.
* Missing test mapping.
* Large result truncated.

### Preferred Backend

Hybrid: references + dependency graph + search + test heuristics.

---

## 10.11 `call_hierarchy`

### Purpose

Find incoming/outgoing calls for a function/method.

### Input

```json
{
  "file": "src/auth/service.rs",
  "position": {"line": 32, "character": 8},
  "direction": "both",
  "depth": 2
}
```

### Output

```json
{
  "incoming": [],
  "outgoing": [],
  "backend": "lsp",
  "partial": false
}
```

### Failure Modes

* Server lacks call hierarchy.
* Timeout.
* Recursive graph too large.

### Preferred Backend

LSP, fallback to syntax/search heuristics.

---

## 10.12 `explain_symbol`

### Purpose

Generate a structured explanation of a symbol for an agent/user.

### Input

```json
{
  "symbol": "AuthProvider",
  "include_references": true,
  "include_tests": true,
  "token_budget": 8000
}
```

### Output

```json
{
  "summary": "...",
  "definition": {},
  "key_references": [],
  "tests": [],
  "diagnostics": [],
  "context_snapshot_id": "ctx_999"
}
```

### Failure Modes

* Symbol ambiguous.
* Context too large.

### Preferred Backend

Hybrid + LLM summarization.

---

# 11. State Model

## 11.1 Repository Workspace

```rust
pub struct WorkspaceState {
    pub id: WorkspaceId,
    pub root: PathBuf,
    pub git_head: Option<String>,
    pub workspace_folders: Vec<PathBuf>,
    pub language_roots: Vec<LanguageRoot>,
    pub trust: WorkspaceTrust,
    pub config: CodeIntelConfig,
}
```

## 11.2 Open Documents

```rust
pub struct OpenDocument {
    pub path: PathBuf,
    pub language: LanguageId,
    pub version: FileVersion,
    pub content_hash: String,
    pub dirty: bool,
    pub text: Rope,
    pub opened_by: OpenReason,
}
```

Helix’s architecture notes that its core uses Rope-backed buffers, with cheap clones useful for text snapshots. ([GitHub][1]) This is a useful pattern for `opencode-rs` because agents need reproducible snapshots before and after edits.

## 11.3 File Versions

Use monotonic versions per open document:

```text
FileVersion = internal counter
ContentHash = hash of text
LspVersion = version sent to LSP
DiskMtime = external modification detection
```

## 11.4 Unsaved Edits

Even though `opencode-rs` is not a full editor, it may need transient edits before commit.

```rust
pub struct PendingEditState {
    pub base_version: FileVersion,
    pub proposed_version: FileVersion,
    pub edits: Vec<TextEdit>,
    pub validation_status: ValidationStatus,
}
```

## 11.5 LSP Server Sessions

```rust
pub struct LspServerState {
    pub server_id: LspServerId,
    pub status: ServerStatus,
    pub capabilities: ServerCapabilities,
    pub open_documents: HashSet<DocumentId>,
    pub pending_requests: HashMap<RequestId, PendingRequest>,
    pub diagnostics: DiagnosticStore,
}
```

## 11.6 Diagnostics

Diagnostics are not timeless facts. They are attached to versions.

```rust
pub struct VersionedDiagnostics {
    pub file: PathBuf,
    pub file_version: FileVersion,
    pub diagnostics: Vec<Diagnostic>,
    pub source: String,
    pub received_at: Instant,
}
```

## 11.7 Symbol Index

```rust
pub struct SymbolIndexState {
    pub index_version: u64,
    pub file_hashes: HashMap<PathBuf, String>,
    pub symbols: SymbolStore,
    pub stale_files: HashSet<PathBuf>,
}
```

## 11.8 Context Snapshots

```rust
pub struct ContextSnapshot {
    pub id: ContextSnapshotId,
    pub task_id: AgentTaskId,
    pub git_head: Option<String>,
    pub file_versions: Vec<FileVersionRef>,
    pub items: Vec<ContextItem>,
    pub trace: ContextPackTrace,
}
```

## 11.9 Tool Call Traces

```rust
pub struct ToolCallTrace {
    pub id: ToolCallId,
    pub agent_id: AgentId,
    pub tool_name: String,
    pub input_hash: String,
    pub output_summary: String,
    pub backends_used: Vec<BackendKind>,
    pub duration_ms: u64,
    pub warnings: Vec<String>,
}
```

## 11.10 Consistency While Editing

Rules:

1. Every edit has a base file version.
2. Patch application fails if base text mismatches.
3. LSP `didChange` uses the next version.
4. Diagnostics older than current version are marked stale.
5. Context snapshots record exact file versions.
6. Tool calls can request either latest state or snapshot state.
7. External disk changes invalidate affected document/index state.

---

# 12. Execution Lifecycle

## 12.1 Startup

```text
1. Load config
2. Establish workspace root
3. Determine trust level
4. Start file discovery
5. Initialize cache
6. Prepare code-intel service
7. Return control to CLI/TUI quickly
```

## 12.2 Repository Scan

```text
Use ignore-aware walker
Classify files
Detect languages
Detect project roots
Detect generated/vendor/build directories
Build file index
```

## 12.3 Language Detection

Inputs:

```text
file extension
shebang
known config files
Cargo.toml/package.json/go.mod/pyproject.toml
tree-sitter grammar availability
configured overrides
```

## 12.4 LSP Server Discovery

Sources:

```text
Project config
User config
PATH lookup
Known defaults
MCP-provided toolchain info
```

Example:

```text
Rust → rust-analyzer
TypeScript → typescript-language-server
Go → gopls
Python → pyright/ruff/jedi
C/C++ → clangd
Java → jdtls
```

## 12.5 LSP Initialization

Lazy by default:

```text
Initialize server when:
- agent needs semantic operation
- file of language is edited
- diagnostics are requested
- user runs lsp status/start
```

## 12.6 Progressive Indexing

```text
Immediate: file index
Fast: text search ready
Background: tree-sitter symbols
Later: dependency graph
Optional: semantic embeddings
```

## 12.7 Agent Request Handling

```text
Agent calls tool
Service checks state
Routes to backend
Applies timeout
Normalizes result
Ranks result
Stores trace
Returns structured output
```

## 12.8 File Edit Handling

```text
Receive edit request
Validate range/version
Apply edit
Update document state
Send LSP didChange
Update tree-sitter parse for file
Mark symbol/dependency indexes stale
Debounce diagnostics refresh
Run validation gate if requested
```

## 12.9 Diagnostics Refresh

```text
Wait for publishDiagnostics or pull diagnostics if supported
Merge diagnostics from multiple sources
Classify new/existing/resolved
Return changed-file result
```

## 12.10 Shutdown

```text
Flush traces
Persist cache
Send shutdown to LSP servers
Send exit
Kill orphan processes if needed
Close DB/index handles
```

---

## 12.11 Failure and Recovery

### LSP Server Crash

```text
Mark server crashed
Record logs
Restart with backoff
Reopen documents
Invalidate diagnostics
Fallback to tree-sitter/search
```

### Unsupported Language

```text
Use file index + text search
Use generic tree-sitter if available
Return degraded confidence
```

### Slow Response

```text
Timeout
Cancel request
Return partial fallback
Record slow request metric
```

### Invalid Diagnostics

```text
Reject diagnostics for stale file version
Normalize invalid ranges
Record warning
```

### Massive Repository

```text
Limit indexing
Respect ignore patterns
Index active files first
Use lazy language server startup
Avoid workspace-wide LSP calls by default
```

### Corrupted Cache

```text
Detect schema/hash mismatch
Move cache to quarantine
Rebuild progressively
```

### Stale File Versions

```text
Reject edits
Ask file system state to refresh
Re-run context pack if needed
```

---

# 13. Configuration Design

## 13.1 Configuration Areas

```text
Enabled languages
Language server command
Initialization options
File ignore patterns
Indexing limits
Diagnostics policy
Context packing policy
Tool timeouts
Cache location
Per-project overrides
Security/trust policy
```

## 13.2 Example Config

```toml
[code_intel]
enabled = true
cache_dir = ".opencode-rs/cache/code-intel"
progressive_indexing = true
lazy_lsp_start = true
max_workspace_files = 50000
max_file_size_kb = 1024

[code_intel.ignore]
patterns = [
  "target/**",
  "node_modules/**",
  "dist/**",
  "build/**",
  ".git/**",
  "**/*.lock"
]

[code_intel.context]
default_token_budget = 24000
include_tests = true
include_diagnostics = true
include_git_diff = true
dedupe = true
snapshot = true
max_reference_files = 30

[code_intel.diagnostics]
changed_files_required = true
project_diagnostics_on_large_refactor = true
wait_timeout_ms = 10000
classify_new_vs_existing = true

[code_intel.timeouts]
hover_ms = 2000
definition_ms = 3000
references_ms = 8000
workspace_symbols_ms = 8000
rename_ms = 15000
initialize_ms = 20000

[code_intel.security]
workspace_trust_required_for_lsp = true
allow_language_server_processes = true
redact_lsp_logs = true
deny_env = ["OPENAI_API_KEY", "ANTHROPIC_API_KEY", "AWS_SECRET_ACCESS_KEY"]
sensitive_patterns = [
  ".env",
  ".env.*",
  "**/secrets/**",
  "**/*.pem",
  "**/*.key"
]

[code_intel.languages.rust]
enabled = true
extensions = ["rs"]
language_server = "rust-analyzer"
command = ["rust-analyzer"]
root_markers = ["Cargo.toml", "rust-project.json"]
tree_sitter = true

[code_intel.languages.rust.initialization_options]
checkOnSave = { command = "clippy" }

[code_intel.languages.typescript]
enabled = true
extensions = ["ts", "tsx", "js", "jsx"]
language_server = "typescript-language-server"
command = ["typescript-language-server", "--stdio"]
root_markers = ["package.json", "tsconfig.json"]
tree_sitter = true

[code_intel.languages.python]
enabled = true
extensions = ["py"]
language_servers = ["pyright", "ruff"]
root_markers = ["pyproject.toml", "setup.py", "requirements.txt"]
tree_sitter = true

[code_intel.languages.python.servers.pyright]
command = ["pyright-langserver", "--stdio"]
role = "primary_semantic"

[code_intel.languages.python.servers.ruff]
command = ["ruff", "server"]
role = "diagnostics_formatter"
```

---

# 14. Observability and Debugging

## 14.1 Structured Logs

Use `tracing` spans:

```text
code_intel.tool_call
lsp.request
lsp.response
lsp.notification
lsp.diagnostics
context.pack
validation.gate
index.refresh
```

Example fields:

```text
task_id
agent_id
workspace
file
language
server_id
method
duration_ms
status
result_count
timeout
fallback_used
```

## 14.2 LSP Request / Response Tracing

Support levels:

```text
off
summary
headers
body_redacted
body_full
```

Default should be `summary` or `body_redacted`.

## 14.3 Redaction Policy

Redact:

* API keys
* `.env`
* private keys
* credentials
* tokens
* sensitive file contents unless explicitly allowed

## 14.4 Timing Metrics

Track:

```text
server startup time
initialize duration
request latency by method
diagnostics latency after edit
indexing throughput
context pack duration
cache hit rate
fallback rate
```

## 14.5 Diagnostics Timeline

Store timeline:

```text
T0 baseline diagnostics
T1 edit applied
T2 didChange sent
T3 diagnostics received
T4 formatter ran
T5 tests ran
T6 final diagnostics
```

## 14.6 Server Lifecycle Events

Log:

```text
server discovered
server started
initialize sent
capabilities received
server ready
server crashed
server restarted
server disabled
```

## 14.7 Context Pack Explanation

Command:

```bash
opencode-rs code-intel explain-context ctx_456
```

Output:

```text
Included src/auth/provider.rs because:
- defines target symbol AuthProvider
- referenced by changed file src/auth/service.rs
- has diagnostics related to trait implementation

Omitted src/auth/legacy.rs because:
- matched text search but no symbol/reference relationship
- lower ranking than tests under token budget
```

## 14.8 TUI Status Display

Show:

```text
Code Intelligence
  File index: ready, 12,430 files
  Symbol index: indexing, 64%
  rust-analyzer: ready, 3 diagnostics
  tsserver: starting
  Fallback mode: disabled
  Last context pack: 18 files / 21k tokens
```

## 14.9 Debug Commands

```bash
opencode-rs lsp status
opencode-rs lsp start rust
opencode-rs lsp restart rust
opencode-rs lsp logs --server rust-analyzer
opencode-rs lsp trace --method textDocument/definition

opencode-rs code-intel index status
opencode-rs code-intel index rebuild
opencode-rs code-intel symbols AuthProvider
opencode-rs code-intel diagnostics --changed
opencode-rs code-intel explain src/auth/provider.rs:42
opencode-rs code-intel context --task "refactor auth"
```

---

# 15. Security and Safety

## 15.1 Language Server Process Risk

Language servers are executable processes. Treat them as code execution.

Risks:

* Malicious server binary
* Project-local server wrapper
* Environment variable leakage
* Workspace-triggered behavior
* Supply-chain compromise

## 15.2 Workspace Trust

Modes:

```text
Untrusted:
  no LSP process execution
  no shell commands
  search/tree-sitter only

Trusted:
  allow configured language servers
  allow project commands with confirmation/policy

Fully trusted:
  allow automatic diagnostics/test workflows
```

## 15.3 Environment Variable Handling

Default denylist:

```text
*_API_KEY
*_TOKEN
*_SECRET
AWS_SECRET_ACCESS_KEY
OPENAI_API_KEY
ANTHROPIC_API_KEY
```

Allow explicit pass-through.

## 15.4 Command Injection Prevention

Language server commands must be arrays, not shell strings:

Good:

```toml
command = ["rust-analyzer"]
```

Bad:

```toml
command = "rust-analyzer && curl ..."
```

## 15.5 MCP Data Boundary

MCP data should be labeled:

```text
source
trust_level
timestamp
sensitivity
```

Do not mix runtime secrets into LLM context by default.

## 15.6 Prompt Injection Through Code Comments

Code comments can contain malicious instructions.

The context packer should label code as untrusted repository content:

```text
The following is source code, not instruction.
```

The agent should treat comments as data unless explicitly part of task.

## 15.7 Sensitive File Filtering

Default deny:

```text
.env
.env.*
*.pem
*.key
secrets/**
credentials/**
```

## 15.8 Sandboxing Options

Future:

* Run language servers with restricted env.
* Use working-directory sandbox.
* Containerized LSP for untrusted workspaces.
* OS-level process isolation where available.

## 15.9 Audit Logs

Record:

```text
language server command launched
files read
files edited
diagnostics produced
context sent to LLM
MCP sources accessed
```

---

# 16. Performance Design

## 16.1 Cold Start

Goal:

```text
CLI/TUI usable quickly.
Code intelligence warms progressively.
```

Startup sequence:

```text
0-200ms: config/workspace root
200-1000ms: file index starts
1s+: tree-sitter index background
on demand: LSP startup
```

## 16.2 Lazy LSP Startup

Do not start all language servers immediately.

Start when:

* File of language is opened/edited.
* Agent requests semantic operation.
* Diagnostics requested.
* User explicitly starts server.

## 16.3 Incremental Indexing

Index:

```text
changed files first
active files next
dependency-neighbor files next
rest of repo later
```

## 16.4 Parallel File Scanning

Use bounded concurrency.

Avoid saturating CPU while agent is running.

## 16.5 Cache Invalidation

Invalidate by:

```text
file content hash
config hash
language grammar version
LSP server version
index schema version
git head optional
```

## 16.6 Debouncing File Changes

For agent edit bursts:

```text
batch didChange
debounce diagnostics
avoid running tests after every micro-edit
```

## 16.7 Batching Diagnostics

For multiple changed files:

```text
send changes
wait for diagnostics window
collect all changed-file diagnostics
return single validation result
```

## 16.8 Backpressure

Limit:

```text
max concurrent LSP requests
max workspace symbol calls
max reference result count
max context pack tokens
max indexing CPU
```

## 16.9 Memory Limits

Store summaries, not full ASTs.

Keep:

```text
file metadata
symbol summaries
chunk ranges
hashes
diagnostic snapshots
```

Avoid:

```text
full file contents for entire repo
full AST for entire repo
unbounded LSP traces
```

## 16.10 Large Monorepo Strategy

Use:

```text
workspace folders
language roots
project root detection
partial indexes
active-area indexing
git sparse awareness
configurable limits
```

---

# 17. Example Workflows

## Workflow 1: Explain a Symbol

### User

```text
Explain AuthProvider and how it is used.
```

### Flow

```text
1. Agent calls find_symbol("AuthProvider")
2. CodeIntel queries symbol index + LSP workspace symbols
3. Agent chooses best symbol
4. CodeIntel calls go_to_definition
5. CodeIntel calls find_references
6. CodeIntel finds tests and related config
7. Context packer builds symbol-centered context
8. Agent summarizes:
   - definition
   - responsibilities
   - key implementations
   - callers
   - tests
```

### Backends

```text
LSP: definition/references/hover
tree-sitter: chunk boundaries
search: test/config discovery
context packer: LLM-ready context
```

---

## Workflow 2: Safe Rename

### User

```text
Rename AuthProvider to CredentialProvider.
```

### Flow

```text
1. prepare_rename at symbol location
2. find_references
3. risk check:
   - public API?
   - generated files?
   - cross-language references?
4. LSP rename returns workspace edit
5. Edit validator checks file versions
6. Apply edits
7. Send didChange to LSP
8. Format changed files
9. Collect changed-file diagnostics
10. Run targeted tests if configured
11. If errors and rollback_on_error=true:
    rollback
12. Final report
```

### Critical Safety Rule

Never perform global text replacement as the first strategy if LSP rename is available.

---

## Workflow 3: Refactor a Module

### User

```text
Split auth/service.rs into smaller modules.
```

### Flow

```text
1. Planner calls document_symbols(auth/service.rs)
2. Planner calls dependency_graph(file)
3. Planner calls find_references for exported symbols
4. Context packer includes definitions, callers, tests
5. Planner proposes module split
6. Implementer creates new files and moves symbols
7. LSP receives didOpen/didChange
8. Import paths are updated
9. Formatter runs
10. Diagnostics gate runs
11. Targeted tests run
12. Reviewer inspects diff and diagnostics
```

---

## Workflow 4: Diagnose Compilation Error

### User

```text
Fix this cargo check error.
```

### Flow

```text
1. Shell runner captures cargo check output
2. Normalize compiler errors to diagnostics
3. Map errors to files/ranges
4. CodeIntel gets surrounding symbols
5. LSP hover/definition enriches type context
6. Context packer includes:
   - failing code
   - definition of missing type/method
   - related trait impl
   - recent diff
7. Agent proposes fix
8. Apply edit
9. Re-run cargo check
```

---

## Workflow 5: Cross-Language Repository Task

### User

```text
Add a new API field from backend Rust service to frontend TypeScript client.
```

### Flow

```text
1. Detect Rust + TypeScript roots
2. Start rust-analyzer for backend
3. Start TypeScript language server for frontend
4. Search OpenAPI/protobuf/schema files
5. Build cross-language context:
   - Rust DTO
   - API schema
   - generated TS client
   - frontend usage
   - tests
6. Implement backend change
7. Update schema/client if appropriate
8. Update frontend usage
9. Run Rust diagnostics
10. Run TS diagnostics
11. Run targeted tests
```

### Key Point

LSP is per-language. The code-intel service owns cross-language orchestration.

---

# 18. Phased Implementation Roadmap

## Phase 1: Minimal Useful Code Intelligence

### Scope

```text
File index
ripgrep search
tree-sitter symbols
basic LSP lifecycle
basic LSP diagnostics
go-to-definition
changed-file diagnostics
```

### Deliverables

* `crates/code-intel`
* Ignore-aware file index
* Language detection
* Tree-sitter symbol extraction for Rust/TS/Python initially
* LSP manager for one server per language
* Basic `definition`, `diagnostics`
* TUI status
* Structured logs

### Acceptance Criteria

```text
1. opencode-rs can index a Rust repo without blocking startup.
2. Agent can call find_symbol and get tree-sitter results.
3. rust-analyzer can be started lazily.
4. go_to_definition works for Rust files.
5. After an agent edit, changed-file diagnostics are collected.
6. If rust-analyzer is missing, search/tree-sitter fallback still works.
```

---

## Phase 2: Agent-Oriented LSP Tools

### Scope

```text
references
hover
workspace symbols
document symbols
call hierarchy
rename preparation
structured tool outputs
diagnostics gates
```

### Deliverables

* Stable agent tool API
* Capability-aware LSP routing
* Diagnostics baseline/new regression classification
* `prepare_rename`
* `safe_rename` MVP
* Tool traces

### Acceptance Criteria

```text
1. Agent can find references across a Rust project.
2. Agent can perform LSP-based rename with rollback.
3. Diagnostics gate can distinguish existing vs new diagnostics.
4. Tool outputs are structured and serializable.
5. LSP failures produce degraded but useful results.
```

---

## Phase 3: Context Packaging and Refactor Support

### Scope

```text
symbol graph
dependency-aware context packer
impact analysis
safe edit validation
context snapshots
test impact analysis
```

### Deliverables

* Context packer
* Context snapshot store
* Dependency graph
* Impact analysis tool
* Test discovery heuristics
* Refactor workflow helpers

### Acceptance Criteria

```text
1. Given a refactor task, context_pack_for_task selects definitions, references, tests, and diagnostics.
2. Every context pack explains why files were included.
3. Agent can run impact_analysis after edits.
4. Safe edit validator prevents stale-range edits.
5. Reviewer agent can use context snapshot + diagnostics + test result.
```

---

## Phase 4: Advanced Intelligence

### Scope

```text
semantic index
multi-agent code review support
cross-repository context
runtime MCP integration
adaptive ranking
architecture rule integration
```

### Deliverables

* Optional local embedding index
* MCP runtime data adapter
* Cross-repo context support
* Architecture rule checks
* Adaptive ranking based on agent outcomes
* Advanced debug UI

### Acceptance Criteria

```text
1. Natural language semantic search improves context retrieval.
2. Runtime MCP logs can be linked to source symbols.
3. Cross-repository symbol/context lookup works with configured projects.
4. Architecture rules can influence context ranking and validation.
5. Multi-agent reviewer can produce structured code intelligence evidence.
```

---

# 19. Key Engineering Trade-offs

## 19.1 LSP Accuracy vs Startup Cost

LSP gives semantic accuracy, but startup can be slow.

Recommendation:

```text
Lazy start LSP.
Use tree-sitter/search immediately.
Upgrade result confidence when LSP becomes ready.
```

## 19.2 Tree-sitter Speed vs Semantic Depth

Tree-sitter is fast and local, but not type-aware.

Recommendation:

```text
Use tree-sitter for structure, chunking, and fallback.
Use LSP for semantic resolution.
```

## 19.3 Persistent Index vs Freshness

Persistent index improves speed but risks staleness.

Recommendation:

```text
Hash-based invalidation.
Version every result.
Never use stale index silently.
```

## 19.4 Agent Autonomy vs Safety Gates

Too many gates slow agents. Too few gates cause broken code.

Recommendation:

```text
Use risk-based validation.
Small edit → changed-file diagnostics.
Refactor → diagnostics + tests + review.
Public API change → impact analysis required.
```

## 19.5 Generic Abstraction vs Language-Specific Features

A generic API is stable, but language-specific features matter.

Recommendation:

```text
Expose generic tools.
Allow language-specific extensions behind optional capability objects.
```

Example:

```json
{
  "language_specific": {
    "rust": {
      "cargo_target": "opencode-rs",
      "crate_name": "code-intel"
    }
  }
}
```

## 19.6 Local-First Privacy vs Remote Semantic Indexing

Remote embeddings can improve search but may leak code.

Recommendation:

```text
Local-first by default.
Remote semantic indexing opt-in only.
Respect sensitive file filters.
```

## 19.7 Simplicity vs IDE-Grade Intelligence

Trying to build a full IDE will slow `opencode-rs`.

Recommendation:

```text
Build agent-critical intelligence first:
find, understand, edit, validate, explain.
Do not prioritize completion UI.
```

---

# 20. Final Recommendation

## Should `opencode-rs` integrate existing Rust LSP libraries?

Yes.

Recommended:

```text
lsp-types: yes
async-lsp: evaluate strongly for client loop/middleware
lsp-server: consider if you need lower-level control
tower-lsp: not for primary client; useful only if building an LSP server
```

## Which layer should be custom?

Custom layers should be:

```text
Agent Tool API
Code Intelligence Service
Context Packaging
Edit Validation
Diagnostics Gate
Impact Analysis
Tool Tracing
Security/Trust Policy
```

These are agent-specific and should be owned by `opencode-rs`.

## Which layer should reuse existing language servers?

Reuse language servers for:

```text
definitions
references
hover
diagnostics
rename
formatting
call hierarchy
type hierarchy
workspace/document symbols
code actions where useful
```

Do not rebuild this from scratch.

## What should be implemented first?

Start with:

```text
1. File index
2. Text search
3. Tree-sitter symbol extraction
4. LSP manager for rust-analyzer
5. go_to_definition
6. diagnostics collection
7. changed-file diagnostics after edit
8. structured agent tool API
9. tracing and lsp status/debug commands
```

This gives immediate practical value without overbuilding.

## What should be avoided?

Avoid:

```text
Rebuilding rust-analyzer-like intelligence
Blocking startup on full indexing
Exposing raw LSP protocol directly to agents
Global text replacement for semantic refactors
Unversioned diagnostics
Untraceable context packing
Running arbitrary language server commands in untrusted workspaces
Building semantic embedding infrastructure before basic symbol/search/LSP works
```

## Ideal Long-Term Architecture

The long-term design should be:

```text
A local-first, agent-oriented code intelligence platform
that reuses existing language servers,
adds fast structural parsing through tree-sitter,
maintains repository-level search and symbol/dependency indexes,
packs traceable LLM context,
validates edits through diagnostics/tests/git,
and exposes stable, task-oriented tools to planners, implementers, reviewers, skills, commands, hooks, and subagents.
```

The most important architectural boundary is this:

```text
LSP Client Layer = protocol/backend integration
Code Intelligence Service = repository intelligence orchestration
Agent Tool API = stable AI-facing capability surface
Context + Validation Layers = what makes it a coding-agent subsystem, not just an editor feature
```

For `opencode-rs`, the best path is **not** “build an LSP client and stop.” The best path is:

```text
Build a robust Rust LSP client,
wrap it in an agent-native code intelligence service,
combine it with tree-sitter/search/indexing,
and make context packing + safe edit validation the core differentiator.
```

[1]: https://github.com/helix-editor/helix/blob/master/docs/architecture.md "helix/docs/architecture.md at master · helix-editor/helix · GitHub"
[2]: https://docs.rs/tower-lsp "tower_lsp - Rust"
[3]: https://docs.rs/async-lsp/latest/async_lsp/ "async_lsp - Rust"
[4]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/?utm_source=chatgpt.com "Language Server Protocol Specification - 3.17"
[5]: https://docs.rs/tower-lsp/latest/tower_lsp/trait.LanguageServer.html?utm_source=chatgpt.com "LanguageServer in tower_lsp - Rust"
[6]: https://docs.helix-editor.com/lang-support.html "Language support"
