# PRD: Agent System

> **User Documentation**: [agents.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/agents.mdx)
>
> This document describes the agent system architecture from an implementation perspective. For user-facing documentation on configuring and using agents, see the user docs linked above.

## Scope

This document describes the agent system architecture for OpenCode: agent roles, the primary/subagent execution model, agent/session relationships, and permission boundaries.

This document is authoritative for:

- Agent roles and their execution semantics
- Primary vs. subagent execution model
- Agent/session and session/subsession relationships
- Agent/tool interaction model
- Permission boundaries and enforcement

This document is **not** authoritative for:

- Configuration schema and precedence (see [06-configuration-system.md](./06-configuration-system.md))
- Tool implementation specifics (see [03-tools-system.md](./03-tools-system.md))
- Session persistence and lifecycle (see [01-core-architecture.md](./01-core-architecture.md))

---

## Agent Roles

### Primary Agents

Primary agents are the main assistants users interact with. A session always has exactly one active primary agent at any given time. Users can cycle primary agents via the **Tab** key or `switch_agent` keybinding.

Primary agents have the following characteristics:

- Can be cycled/rotated during a session
- Carry session-level execution context (model, prompt, permissions)
- May invoke subagents via the `task` tool
- Can be hidden (e.g., Compaction, Title, Summary agents) or visible in the agent switcher

**Built-in primary agents:**

| Agent | Description | Default Mode | User Config Key |
|-------|-------------|--------------|-----------------|
| `build` | Full tool access (default). Standard agent for development work requiring full file operations and system commands | Visible | `agent.build` |
| `plan` | Read-only analysis; all modifications (file edits, bash commands) set to `ask` by default. Designed for code analysis and planning without making changes | Visible | `agent.plan` |
| `compaction` | Hidden system agent; automatically compresses long contexts into smaller summaries | Hidden | (automatic) |
| `title` | Hidden system agent; automatically generates short session titles | Hidden | (automatic) |
| `summary` | Hidden system agent; automatically creates session summaries | Hidden | (automatic) |

### User-Facing Agent Descriptions (from user docs)

**Build Agent**: The default primary agent with full tool access. Users can switch to Plan mode via Tab key when they want analysis without modifications.

**Plan Agent**: A restricted agent for planning and analysis. Default permissions for `file edits` and `bash` are set to `ask` - the user must approve any modifications. Use this when you want the LLM to analyze code, suggest changes, or create plans without making actual changes to the codebase.

### Subagents

Subagents are specialized assistants invoked by primary agents or manually by users via `@` mention. Unlike primary agents, subagents execute within the context of a parent agent's session without changing the active primary agent.

Subagents have the following characteristics:

- Invoked via `task` tool or `@` mention syntax
- Execute in a child session derived from the parent's context
- Return results to the parent agent for incorporation
- May be hidden from `@` autocomplete

**Built-in subagents:**

| Agent | Description | Default Tool Access | User Invocation |
|-------|-------------|--------------------|-----------------|
| `general` | Full tool access for complex multi-step tasks. Can modify files (except todo), run parallel work units | write, edit, bash, read, grep, glob, list | `@general` |
| `explore` | Read-only code exploration. Fast agent for finding files by pattern, searching code keywords, or answering questions about the codebase | read, grep, glob, list | `@explore` |

### User-Facing Subagent Descriptions (from user docs)

**General Subagent**: A general-purpose agent for researching complex issues and executing multi-step tasks. Has full tool access (except todo), so it can modify files when needed. Can be used for running multiple work units in parallel.

**Explore Subagent**: A fast read-only agent for exploring the codebase. Cannot modify files. Use when you need to quickly find files by pattern, search for keywords in code, or answer questions about the codebase.

### Subagent Session Navigation

When a subagent creates its own subsessions, users can navigate between parent and all child sessions using:
- **\<Leader>+Right** (or `session_child_cycle` keybinding): Cycle forward: parent → child1 → child2 → ... → parent
- **\<Leader>+Left** (or `session_child_cycle_reverse` keybinding): Cycle backward: parent ← child1 ← child2 ← ... ← parent

---

## Primary vs. Subagent Execution Model

### Primary Agent Execution

A primary agent owns the top-level session execution loop:

1. User submits a prompt to the session
2. The session's active primary agent processes the prompt
3. The agent may emit tool calls, which are checked against permission rules
4. Tool results are fed back to the agent for continued reasoning
5. The agent produces a final response, which is persisted as a session message

### Subagent Execution

Subagents execute in a subsession with a derived execution context:

```
Parent Session
├── Message 1 (user prompt)
├── Message 2 (assistant)
│   └── Subsession 1 (subagent task)
│       ├── Message 1 (subagent prompt + context)
│       └── Message 2 (subagent response)
└── Message 3 (parent assistant; incorporates subagent result)
```

**Subsession characteristics:**

- Inherits parent session context (project, working directory, agent config defaults)
- Has its own message sequence
- Does not alter parent session state directly; results are returned to parent
- Subagent executions may run under additional runtime restrictions relative to the parent context
- Parent session history remains intact after subagent completes

### Task Tool Invocation

Primary agents invoke subagents using the task/delegation mechanism.

At the architectural level, invocation semantics are:

- The parent agent creates a child execution context for the subagent
- Relevant session/project context is passed into that child execution
- The parent receives the child result as structured task output

The exact task tool payload shape is owned by the implementation/runtime API, not by this document.

---

## Agent/Session Relationship

A session carries execution context for its active primary agent:

- **Model**: which LLM model to use for this session
- **Agent config**: the active primary agent's configuration (prompt, temperature, etc.)
- **Permission scope**: the effective permission rules for the session

Changing the active primary agent does not create a new session; it swaps the execution context in place. Forking a session (for parallel exploration, branching reasoning, etc.) creates a new child session with a reference to the parent lineage.

See [01-core-architecture.md](./01-core-architecture.md) for full session lifecycle invariants and the canonical ownership tree (Project → Session → Message → Part).

---

## Agent/Tool Interaction Model

When an agent emits a tool call, the system performs a permission check before executing the tool:

1. **Agent emits tool call** — agent reasoning produces a named tool and arguments
2. **Permission check** — the tool name is matched against active permission rules
3. **Execution or denial** — `allow` executes immediately; `ask` prompts the user; `deny` blocks with an error
4. **Result returned** — tool output is fed back to the agent for continued reasoning

Permission rules are defined in config (see [06-configuration-system.md](./06-configuration-system.md)) and can use glob patterns for fine-grained control:

```json
{
  "permission": {
    "bash": "allow",
    "bash:rm *": "deny",
    "edit": "ask",
    "mcp_github_*": "ask"
  }
}
```

---

## Permission Boundaries

### Agent Permission Scope

Each agent operates within a permission scope defined by:

1. **Global permission rules** — defaults from config
2. **Agent-level permission overrides** — per-agent constraints from config/runtime
3. **Runtime context** — subsession boundaries may further restrict inherited permissions

### Permission Inheritance

Subagents inherit the parent's effective permission scope at invocation time, subject to any tighter runtime restrictions imposed by the host.

### User-Facing Permission Configuration

Users configure permissions in `opencode.json` under the `permission` key:

```json
{
  "permission": {
    "edit": "ask",     // Prompt before file modifications
    "bash": "allow",   // Allow all bash commands
    "mcp_github_*": "ask"  // MCP tools matching pattern
  }
}
```

Permission values:
- `"allow"` — Execute immediately without prompting
- `"ask"` — Prompt user for approval before execution
- `"deny"` — Block execution entirely

Permission patterns support glob matching for fine-grained control.

### Boundary Enforcement

- Permission checks happen at the tool call boundary, before any tool implementation runs
- File-path-based permission rules (e.g., `edit:*.md`) are enforced by the tool implementation when path information is available
- MCP tool permissions follow the same glob-pattern matching as built-in tools

---

## Cross-References

| Topic | Document | User Docs | Notes |
|-------|----------|-----------|-------|
| Core entities & session lifecycle | [01-core-architecture.md](./01-core-architecture.md) | (conceptual) | Session ownership tree, lifecycle invariants |
| Configuration schema | [06-configuration-system.md](./06-configuration-system.md) | [config.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/config.mdx) | `AgentConfig` schema, `permission` rule type, precedence |
| Tool implementation | [03-tools-system.md](./03-tools-system.md) | [tools.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/tools.mdx) | Built-in tool list, custom tool format, MCP integration |
| Permission evaluation | [06-configuration-system.md](./06-configuration-system.md) | [permissions.mdx](https://github.com/anomalyco/opencode/blob/main/packages/web/src/content/docs/zh-cn/permissions.mdx) | `PermissionRule` type, pattern matching semantics |
| Agent module implementation | [../modules/agent.md](../modules/agent.md) | N/A | Rust `Agent` trait, concrete agent implementations |

## Implementation Notes

### Agent Switching (User-Facing)

Users switch between primary agents using:
1. **Tab key** — Cycles through primary agents (Build ↔ Plan)
2. **`switch_agent` keybinding** — Customizable shortcut

### Task Tool Invocation (Implementation)

Primary agents invoke subagents using the task/delegation mechanism:

```rust
// From the agent's perspective:
runtime.invoke_subagent(
    &ExploreAgent,
    context_messages,
    provider,
    tool_registry
).await?
```

The parent agent receives a `SubagentResult` containing:
- The subagent's response
- Child session ID (for navigation)
- Effective permission scope (may be restricted relative to parent)
