# PRD: Agent System

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

| Agent | Description | Default Mode |
|-------|-------------|--------------|
| `build` | Full tool access (default) | Visible |
| `plan` | Read-only analysis; all modifications disabled | Visible |
| `compaction` | Hidden; context compression | Hidden |
| `title` | Hidden; session title generation | Hidden |
| `summary` | Hidden; session summarization | Hidden |

### Subagents

Subagents are specialized assistants invoked by primary agents or manually by users via `@` mention. Unlike primary agents, subagents execute within the context of a parent agent's session without changing the active primary agent.

Subagents have the following characteristics:

- Invoked via `task` tool or `@` mention syntax
- Execute in a child session derived from the parent's context
- Return results to the parent agent for incorporation
- May be hidden from `@` autocomplete

**Built-in subagents:**

| Agent | Description | Default Tool Access |
|-------|-------------|--------------------|
| `general` | Full tool access for complex multi-step tasks | write, edit, bash, read, grep, glob, list |
| `explore` | Read-only code exploration | read, grep, glob, list |

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

### Boundary Enforcement

- Permission checks happen at the tool call boundary, before any tool implementation runs
- File-path-based permission rules (e.g., `edit:*.md`) are enforced by the tool implementation when path information is available
- MCP tool permissions follow the same glob-pattern matching as built-in tools

---

## Cross-References

| Topic | Document | Notes |
|-------|----------|-------|
| Core entities & session lifecycle | [01-core-architecture.md](./01-core-architecture.md) | Session ownership tree, lifecycle invariants |
| Configuration schema | [06-configuration-system.md](./06-configuration-system.md) | `AgentConfig` schema, `permission` rule type, precedence |
| Tool implementation | [03-tools-system.md](./03-tools-system.md) | Built-in tool list, custom tool format, MCP integration |
| Permission evaluation | [06-configuration-system.md](./06-configuration-system.md) | `PermissionRule` type, pattern matching semantics |
