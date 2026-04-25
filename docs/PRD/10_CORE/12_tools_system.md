# PRD: Tools System

> **User Documentation**: [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx)
>
> This document describes the tools system architecture from an implementation perspective. For user-facing documentation on configuring and using tools, see the user docs linked above.

## Overview

This document describes OpenCode's tool system architecture: how tools are categorized, registered, discovered, executed, and gated by permission configuration. It covers the runtime pipeline from tool registration through permission checking to execution and error handling.

This document does **not** redefine configuration schema. For permission configuration syntax and precedence, see [Configuration System](./06-configuration-system.md). For permission evaluation semantics, see [Agent System](./02-agent-system.md).

---

## Scope

This document covers:

- Tool categories (built-in, custom, MCP)
- Built-in tool model (what distinguishes a built-in)
- Custom tool model and discovery
- Tool registration and availability lifecycle
- Execution flow pipeline
- Permission gating (runtime, not config schema)
- Tool error model
- Result caching behavior

This document does **not** cover:

- Full list of built-in tools (tool catalog evolves; see implementation)
- Permission configuration schema (see [06](./06-configuration-system.md))
- Permission rule evaluation logic (see [02](./02-agent-system.md))
- MCP protocol details (see [04](./04-mcp-system.md))
- Plugin internal structure (see [08](./08-plugin-system.md))

---

## Tool Categories

OpenCode supports three tool categories:

| Category | Source | Discovery |
|----------|--------|-----------|
| **Built-in** | Core runtime | Always available (subject to permission) |
| **Custom** | `.opencode/tools/` or `~/.config/opencode/tools/` | File-based loader |
| **MCP** | MCP servers | Dynamic via MCP protocol |

All tools, regardless of category, pass through the same execution pipeline and are subject to the same permission gating.

---

## Built-in Tool Model

Built-in tools are implemented in the core runtime and registered with the runtime tool registry during startup. They are distinguished by:

- **Compiled into the runtime** — no file-based discovery required
- **Stable interface** — name, description, and argument schema are fixed
- **Subject to permission configuration** — like all tools, access is gated by the `permission` config

Built-in tools cover core file operations, shell execution, web access, and agent/runtime operations.

### Canonical Built-in Tool List (from user documentation)

| Tool | Description | Permission Key |
|------|-------------|----------------|
| `bash` | Execute shell commands in the project environment | `bash` |
| `edit` | Modify existing files via precise string replacement | `edit` |
| `write` | Create new files or overwrite existing files | `edit` |
| `read` | Read file contents from the codebase | `read` |
| `grep` | Search file contents using regular expressions | `grep` |
| `glob` | Find files by pattern matching | `glob` |
| `lsp` | (Experimental) Interact with LSP servers for code intelligence | `lsp` |
| `patch` | Apply patches to files | `edit` |
| `skill` | Load a skill (SKILL.md file) into the conversation | `skill` |
| `todowrite` | Manage todo lists within coding sessions | `todowrite` |
| `webfetch` | Fetch web page content | `webfetch` |
| `websearch` | Search the web for information (requires OpenCode provider or `OPENCODE_ENABLE_EXA=1`) | `websearch` |
| `question` | Ask the user questions during execution | `question` |

### Additional Tools via MCP

MCP servers expose tools dynamically. MCP tool names are typically server-qualified (e.g., `mcp_github_*`) to avoid collisions.

### Tool Description Schema

Each tool defines:
- **name**: Stable identifier string
- **description**: Human-readable description for LLM
- **args**: JSON Schema for argument validation

The set of built-in tools should be treated as stable identifiers. The system is designed so that adding new built-in tools does not require changes to this document.

---

## Custom Tool Model

Custom tools are user-defined tools loaded from the filesystem at runtime.

### Definition Format

Custom tools are defined in TypeScript/JavaScript files:

```typescript
import { tool } from "@opencode-ai/plugin"

export default tool({
  description: "Query the project database",
  args: {
    query: tool.schema.string().describe("SQL query to execute"),
  },
  async execute(args, context) {
    const { directory, worktree } = context
    return `Executed: ${args.query}`
  },
})
```

### Execution Context

```typescript
context: {
  agent: string      // agent type invoking the tool
  sessionID: string // current session identifier
  messageID: string  // current message identifier
  directory: string  // current working directory
  worktree: string   // git worktree root
}
```

### Discovery Locations

| Scope | Path |
|-------|------|
| Project | `.opencode/tools/` |
| Global | `~/.config/opencode/tools/` |

Tools are discovered by scanning these directories at runtime. A tool file may export a single default tool or multiple named exports.

### Naming

- Single tool per file: filename becomes the tool name (e.g., `database.ts` → tool `database`)
- Multiple tools: export named tools (e.g., `math.ts` with `add`, `multiply` → tools `math_add`, `math_multiply`)

### Override Behavior

Custom tools may collide with built-in tool names. Name resolution and precedence must be deterministic in the runtime implementation; this PRD does not require one override policy beyond requiring stable lookup behavior.

---

## Registration and Discovery

### Startup Sequence

1. **Core built-in tools** register with the runtime tool registry during initialization
2. **Custom tools** are discovered from configured tool directories and registered when valid
3. **MCP tools** are discovered dynamically when MCP integrations become available (see [04](./04-mcp-system.md))

### Tool Registry

The runtime tool registry is the central lookup surface for all available tools. It provides:

- **Registration** — tools register on startup
- **Discovery** — tools are looked up by name
- **Listing** — the agent system can enumerate available tools for capability reporting

After registration, a tool is **available** — meaning it exists in the runtime registry. Availability does not imply executability; that is determined by the permission check at execution time.

### MCP Tool Prefixing

MCP server tools are typically exposed with a server-qualified name to avoid collisions between different MCP integrations.

---

## Execution Flow

The tool execution pipeline is:

```
LLM generates tool call
        │
        ▼
┌─────────────────────┐
│  Tool name lookup   │  ← Resolve name to registered tool
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Permission check   │  ← Gate: allow / ask / deny
└─────────────────────┘
        │
   ┌────┴────┐
   │ denied? │──yes──→ return error
   └────┬────┘
        │no
        ▼
┌─────────────────────┐
│  Argument validation│  ← Validate args against tool schema
└─────────────────────┘
        │
   ┌────┴────┐
   │ invalid?│──yes──→ return error
   └────┬────┘
        │no
        ▼
┌─────────────────────┐
│  Tool execution     │  ← Call tool implementation
└─────────────────────┘
        │
        ▼
┌─────────────────────┐
│  Result or error    │  ← Structured response to LLM
└─────────────────────┘
```

### Registration → Availability

A tool is **registered** when it has been added to the `ToolRegistry`. A tool is **available** when it is registered and not explicitly removed. The agent system can query the registry for available tools to include in system prompts or capability reports.

### Permission Check (Runtime)

The permission check is a runtime gate, not a configuration declaration. At execution time, the system evaluates the `permission` configuration (see [06](./06-configuration-system.md)) for the current tool name and returns:

- **`allow`** — execute immediately
- **`ask`** — prompt the user for approval; execution proceeds if granted
- **`deny`** — return an error; no execution occurs

For fine-grained rule evaluation (glob patterns, precedence, etc.), see [02](./02-agent-system.md). That document is the normative reference for how permission rules are evaluated.

### Argument Validation

Each tool defines an argument schema. Before execution, the system validates the LLM-supplied arguments against this schema. Invalid arguments result in a structured error rather than silent failure or undefined behavior.

### Result

On success, the tool returns a structured result to the LLM. On failure, the tool returns a structured error. Results may be cached by the implementation (see [Caching](#caching--optimization)).

---

## Permission and Availability Flow

This section describes the runtime relationship between tool availability and permission gating. For configuration syntax, see [06](./06-configuration-system.md). For evaluation semantics, see [02](./02-agent-system.md).

### Availability vs. Permission

- **Availability** is a property of the tool itself (it exists in the registry)
- **Permission** is a runtime gate that controls whether an available tool can execute

A tool can be registered but denied, or registered and allowed. The permission check is evaluated per-invocation based on the current session's permission configuration.

### Permission Patterns

Permission rules support glob patterns on tool names. For example:

```json
{
  "permission": {
    "mcp_github_*": "ask",
    "bash": "allow",
    "edit": "deny"
  }
}
```

**Pattern Evaluation**: Last matching rule wins. More specific patterns should come after general wildcards.

**User-Facing Configuration** (from user docs):
- `permission.<tool>`: Control built-in tools
- `permission.<pattern>`: Control tool groups via glob patterns

**Example - Disable all MCP tools except specific ones**:
```json
{
  "permission": {
    "mcp_*": "deny",
    "mcp_mytrusted_*": "allow"
  }
}
```

### Interaction with Agent Mode

Agents may impose additional restrictions. For example, the `plan` agent may disable write operations entirely regardless of permission configuration. These restrictions are applied at the agent level before the permission check is reached.

---

## Error Model

Tool execution errors return structured responses:

```typescript
{
  error: {
    name: "ToolError"       // error type
    message: string          // human-readable description
    tool?: string            // tool that produced the error
    retryable?: boolean      // hint: may succeed on retry
  }
}
```

Error categories:

| Category | Description | Retryable |
|----------|-------------|-----------|
| **NotFound** | Tool name does not exist in registry | No |
| **PermissionDenied** | Tool denied by permission config | No |
| **ValidationError** | Arguments failed schema validation | No |
| **ExecutionError** | Tool implementation threw an error | Depends |
| **TimeoutError** | Tool exceeded its time budget | May be |

---

## Caching and Optimization

Tool results may be cached to avoid redundant operations:

| Operation | Caching Strategy |
|-----------|-----------------|
| File reads | Cached by path + mtime; invalidated on file change |
| Glob | Cached by pattern; invalidated on file system changes |
| Bash commands | **Not cached** (side effects) |
| Web fetches | Cached by URL; subject to HTTP caching headers |

Caching is transparent to the execution flow; the pipeline above is unchanged for cached results.

---

## Ignore File Support

File-discovering tools may respect both VCS ignore rules and host-specific ignore extensions. The exact ignore-file semantics should follow the selected upstream implementation baseline.

---

## Cross-References

| Topic | Document | User Docs | Relationship |
|-------|----------|-----------|--------------|
| Permission config schema | [06-configuration-system.md](./06-configuration-system.md) | [permissions.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/permissions.mdx) | `permission` config is normative; 03 does not redefine it |
| Permission evaluation | [02-agent-system.md](./02-agent-system.md) | [permissions.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/permissions.mdx) | Evaluation semantics are owned by 02 |
| MCP tools | [04-mcp-system.md](./04-mcp-system.md) | [mcp-servers.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/mcp-servers.mdx) | MCP registration and protocol |
| Custom tool loading | [08-plugin-system.md](./08-plugin-system.md) | [plugins.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/plugins.mdx) | Plugin system provides tool loading primitives |
| Agent tool access | [02-agent-system.md](./02-agent-system.md) | [agents.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/agents.mdx) | Agent-level tool restrictions applied before permission check |
| Skills system | [12-skills-system.md](./12-skills-system.md) | [skills.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/skills.mdx) | Skills may invoke tools; same pipeline applies |
| Tool module implementation | [../modules/tool.md](../modules/tool.md) | N/A | Rust `Tool` trait, built-in tool implementations |
