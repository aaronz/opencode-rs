# PRD: Skills System

## Overview

Skills are reusable behavior definitions loaded on-demand via `SKILL.md` files. This document defines the skill package format, deterministic discovery and resolution rules, loading semantics, and the permission model.

This document does **not** redefine configuration schema. For `permission` configuration syntax and precedence, see [06-configuration-system.md](./06-configuration-system.md).

---

## Scope

This document covers:

- Skill package format (`SKILL.md`)
- Skill discovery locations and precedence order
- Deterministic resolution when duplicate skill names exist
- Skill loading semantics
- Permission model for skill access
- Compatibility paths (Claude Code and Agent compatibility)
- Cross-references to other subsystem documents

This document does **not** cover:

- Permission configuration schema (see [06-configuration-system.md](./06-configuration-system.md))
- Permission rule evaluation logic (see [02-agent-system.md](./02-agent-system.md))
- Tool implementation specifics (see [03-tools-system.md](./03-tools-system.md))
- Plugin internal structure (see [08-plugin-system.md](./08-plugin-system.md))

---

## Skill Package Format

### File Structure

```
<skill-directory>/
  SKILL.md
```

### SKILL.md Format

```markdown
---
name: skill-name           # Required: 1-64 chars, lowercase + single hyphens
description: What it does # Required: 1-1024 chars
license?: MIT             # Optional
compatibility?: opencode  # Optional: target system hint
metadata?:                # Optional
  audience: maintainers
  workflow: github
---

## What I do

- Description of functionality

## When to use me

- Use case 1
- Use case 2
```

### Name Validation

Regex: `^[a-z0-9]+(-[a-z0-9]+)*$`

- Lowercase letters and numbers only
- Single hyphens allowed (not at start/end)
- No consecutive hyphens

### Compatibility Field

The `compatibility` field is an optional hint indicating which system the skill is designed for:

| Value | Meaning |
|-------|---------|
| `opencode` | OpenCode native |
| `claude` | Claude Code compatibility |
| `agent` | Agent compatibility |

This field does not affect discovery or loading behavior; it is informational metadata for skill authors.

---

## Skill Discovery Locations

### Location Scopes

Skills are discovered from three scope categories, evaluated in priority order:

| Priority | Scope | Base Path |
|----------|-------|-----------|
| 1 (highest) | Project | `.opencode/skills/<name>/SKILL.md` |
| 2 | Global | `~/.config/opencode/skills/<name>/SKILL.md` |
| 3 | Claude compat | `.claude/skills/<name>/SKILL.md` |
| 4 | Claude global | `~/.claude/skills/<name>/SKILL.md` |
| 5 | Agent compat | `.agents/skills/<name>/SKILL.md` |
| 6 (lowest) | Agent global | `~/.agents/skills/<name>/SKILL.md` |

**Note:** The Claude and Agent compatibility paths (3–6) are compatibility shims for skills written for other systems. They do not support remote discovery and are not primary skill locations.

### Project Scope Traversal

For project-scoped discovery (priority 1), OpenCode searches from the current working directory upward to the git worktree root. The search stops at the first directory that is not a valid project boundary (i.e., not inside a git worktree or project root).

Within each visited project directory, the full `.opencode/skills/` subtree is scanned for matching skill names.

---

## Discovery and Resolution Rules

### Deterministic Resolution

Resolution is deterministic following these rules:

1. **Scope priority is fixed** — The order in the table above is the only ordering. Higher priority scopes always take precedence over lower priority scopes.

2. **Within a scope, first-found wins** — When multiple `SKILL.md` files with the same skill name exist within the same scope (e.g., scanning a directory tree), the first one encountered during the scan wins.

3. **No merging or layering** — A skill is loaded entirely from a single `SKILL.md` file. There is no merging of partial definitions from multiple locations.

4. **No overriding between scopes** — A skill found in a higher-priority scope does not cause the search to skip lower-priority scopes for other skill names. Each skill is resolved independently.

### Resolution Algorithm

For a skill with name `<name>`:

```
for each scope in priority order:
  find SKILL.md at scope.path/<name>/SKILL.md
  if found:
    load and return skill content
    stop searching
if not found in any scope:
    return not found
```

### Duplicate Name Handling

If the same skill name appears multiple times within a single scope, the runtime must resolve the conflict deterministically. This PRD requires a stable traversal order and a stable winner for a given directory structure; implementations must not treat duplicate resolution as undefined.

---

## Loading Semantics

### Skill Loading Flow

1. Agent queries the `skill` tool for available skills
2. Available skills are enumerated from all discovery locations
3. Agent calls `skill({ name: "skill-name" })` to load a specific skill
4. The system resolves the skill using the discovery algorithm above
5. If found, the `SKILL.md` content is injected into the agent's context
6. If not found, an error is returned

### Skill Content in Context

When a skill is loaded, its `SKILL.md` content becomes part of the agent's instructions. The skill content is treated as guidance, not executable code.

### Disabling the Skill Tool

Skill loading can be restricted through the canonical permission/config system. This document does not introduce an alternative schema; see [06-configuration-system.md](./06-configuration-system.md) for config ownership.

---

## Permission Model

### Configuration Source

Skill loading/usage permission is part of the canonical permission/config system. See [06-configuration-system.md](./06-configuration-system.md) for the normative schema and [02-agent-system.md](./02-agent-system.md) for evaluation semantics.

### Permission Evaluation

Permission evaluation for skills follows the same runtime semantics as tool permission evaluation. See [02-agent-system.md](./02-agent-system.md) for the normative evaluation semantics.

### Per-Agent Override

Per-agent restrictions or allowances for skill usage should be expressed through the canonical config/runtime permission model rather than a skills-specific schema invented here.

---

## Compatibility Paths

### Claude Code Compatibility

Skills stored in Claude Code locations (`.claude/skills/` and `~/.claude/skills/`) are discoverable by OpenCode for compatibility. These are loaded using the same resolution rules described above, but at a lower priority than OpenCode-native locations.

### Agent Compatibility

Skills stored in Agent compatibility locations (`.agents/skills/` and `~/.agents/skills/`) are discoverable by OpenCode for compatibility, at the lowest priority scope.

### Remote Discovery

OpenCode does not support remote skill discovery. Skills must be present in a local filesystem location. There is no network-based skill registry or remote `SKILL.md` fetching defined in this version.

---

## Metadata

```yaml
---
name: git-release
description: Create releases and changelogs
metadata:
  audience: maintainers
  workflow: github
---
```

Metadata fields are optional and informational. They do not affect discovery, resolution, or loading behavior.

---

## Skill Example

```markdown
---
name: git-release
description: Create consistent releases and changelogs
license: MIT
---

## What I do

- Draft release notes from merged PRs
- Propose version bump
- Provide copy-pasteable `gh release create` command

## When to use me

Use when preparing a tagged release.
Ask clarifying questions if versioning scheme is unclear.
```

---

## Cross-References

| Topic | Document | Relationship |
|-------|----------|--------------|
| Permission config schema | [06-configuration-system.md](./06-configuration-system.md) | `permission` config is normative; 12 does not redefine it |
| Permission evaluation | [02-agent-system.md](./02-agent-system.md) | Evaluation semantics are owned by 02 |
| Tool system | [03-tools-system.md](./03-tools-system.md) | Skills may invoke tools; same pipeline applies |
| Plugin system | [08-plugin-system.md](./08-plugin-system.md) | Plugin system provides skill loading primitives |
| Core architecture | [01-core-architecture.md](./01-core-architecture.md) | Session context for skill loading |
