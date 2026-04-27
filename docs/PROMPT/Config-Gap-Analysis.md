You are a senior Rust engineer, configuration system architect, developer tooling expert, and product analyst.

I am building `opencode-rs`, a Rust-based implementation of opencode. I want `opencode-rs` to eventually reach feature parity with the original opencode project.

Your task is to compare the current `opencode-rs` configuration schema with the official opencode configuration schema:

Official opencode config schema:
https://opencode.ai/config.json

You must perform a detailed gap analysis and produce a professional specification document that can guide implementation.

==================================================
1. Objective
==================================================

Compare:

1. The current `opencode-rs` config schema and config implementation
2. The official opencode config schema from:
   https://opencode.ai/config.json

The goal is to identify major capability gaps so that `opencode-rs` can support equivalent configuration capabilities as opencode.

This is not only a field-by-field comparison. You must analyze configuration semantics, runtime behavior, validation rules, defaults, user experience, and migration impact.

==================================================
2. Required Investigation
==================================================

Please inspect the `opencode-rs` codebase deeply and identify:

- Where the config schema is defined
- Where config files are loaded
- Where config files are saved
- Where defaults are applied
- Where validation is performed
- Where config values are consumed at runtime
- How provider/model/agent/tool/session-related config is handled
- How config paths and config precedence are implemented
- How errors are reported to CLI/TUI users
- Whether documentation and examples match the actual implementation

Then compare these findings against the official opencode schema.

==================================================
3. Comparison Scope
==================================================

Please compare at least the following areas:

1. Top-level config structure
   - Supported top-level fields
   - Missing fields
   - Extra non-compatible fields
   - Deprecated or incorrectly modeled fields

2. Provider and model configuration
   - Provider definitions
   - Model selection
   - API keys and authentication
   - Base URL support
   - Environment variable support
   - Custom provider support
   - Model aliases
   - Default provider/model behavior

3. Agent configuration
   - Default agents
   - Named agents
   - Agent model assignment
   - Agent-specific tools
   - Agent-specific prompts/instructions
   - Permission and behavior settings
   - Compatibility with opencode agent config semantics

4. Tools configuration
   - Tool enable/disable rules
   - Tool permissions
   - Built-in tools
   - MCP tools
   - Shell/command execution behavior
   - File operation permissions

5. MCP configuration
   - MCP server definitions
   - Command-based MCP servers
   - Environment variables
   - Remote MCP servers if supported by opencode
   - Enable/disable behavior
   - Runtime lifecycle handling

6. Commands, skills, rules, and hooks
   - Whether these concepts exist in the official schema
   - Whether `opencode-rs` supports them
   - How they are represented
   - Major missing lifecycle integration points

7. Session and project configuration
   - Project-level config
   - User-level config
   - Workspace-level config
   - Config inheritance and merge behavior
   - Config precedence rules

8. TUI/CLI behavior affected by config
   - `/connect`
   - Provider selection
   - Model selection
   - Config persistence after successful validation
   - Error display
   - Logging interaction
   - Whether config-driven behavior works in real E2E scenarios

9. Validation and schema compatibility
   - JSON schema compatibility
   - Type compatibility
   - Required vs optional fields
   - Default values
   - Error messages
   - Backward and forward compatibility

10. Documentation and examples
   - Existing docs
   - Sample config files
   - README references
   - Mismatches between docs and implementation

==================================================
4. Output Requirements
==================================================

Produce a complete gap analysis specification document in Markdown.

The document must include:

## 1. Executive Summary

Summarize whether `opencode-rs` is currently compatible, partially compatible, or significantly incompatible with the official opencode config schema.

## 2. Current opencode-rs Config Architecture

Describe the current config implementation in `opencode-rs`:

- Main files/modules
- Data structures
- Load/save flow
- Validation flow
- Runtime usage
- Config path strategy
- Known risks

## 3. Official opencode Config Schema Overview

Summarize the major capabilities exposed by the official opencode schema.

Do not copy the whole schema blindly. Extract and explain the important configuration domains.

## 4. Field-by-Field Compatibility Matrix

Create a table with columns:

- Official opencode field/path
- Official meaning
- Supported in `opencode-rs`?
- Current `opencode-rs` equivalent
- Gap type
- Severity
- Implementation notes

Use the following gap types:

- Missing
- Partially supported
- Incorrect semantics
- Wrong type
- Wrong default
- Unsupported runtime behavior
- Documentation mismatch
- Extra incompatible field

Use severity levels:

- P0: Blocks core compatibility
- P1: Major feature gap
- P2: Important but not blocking
- P3: Nice to have

## 5. Major Capability Gaps

Group the most important gaps by domain:

- Provider/model config gaps
- Agent config gaps
- Tool permission gaps
- MCP config gaps
- Commands/skills/rules/hooks gaps
- Config path and precedence gaps
- Validation/schema gaps
- CLI/TUI persistence gaps
- Documentation gaps

For each gap, include:

- Problem
- Why it matters
- Current behavior
- Expected opencode-compatible behavior
- Suggested implementation approach
- Related files/modules to modify
- Suggested tests

## 6. Recommended Target Config Model for opencode-rs

Propose the target Rust config model.

Include:

- Rust struct design
- Serde compatibility strategy
- Optional vs required fields
- Default handling
- Unknown field handling
- Versioning strategy
- Compatibility with official opencode schema
- Extensibility for opencode-rs-specific fields

Use Rust-style pseudocode where useful.

## 7. Config Loading and Precedence Specification

Define the expected config loading behavior, including:

- Global user config
- Project config
- Workspace/repo config
- Environment variables
- CLI flags
- Runtime overrides
- Merge strategy
- Conflict resolution
- Error handling

## 8. Migration Plan

Create a phased implementation plan:

### Phase 1: Schema alignment foundation
### Phase 2: Provider/model compatibility
### Phase 3: Agent/tool/MCP compatibility
### Phase 4: Commands/skills/rules/hooks compatibility
### Phase 5: CLI/TUI persistence and UX hardening
### Phase 6: Documentation and examples

For each phase, include:

- Goal
- Tasks
- Files likely affected
- Acceptance criteria
- Tests required

## 9. Test Strategy

Define a concrete test strategy:

- Unit tests for config parsing
- Golden tests using official opencode-compatible config examples
- Schema validation tests
- Migration tests
- Config precedence tests
- CLI E2E tests
- TUI E2E tests
- Provider validation persistence tests
- Regression tests for previously observed bugs

Include specific test cases, especially:

- Valid opencode config should parse in `opencode-rs`
- Unknown fields should be handled intentionally
- Provider selected in `/connect` should be persisted correctly
- Validated API key should be written to the correct opencode-rs config location
- Logs should not render inside the TUI screen
- Validation dialog should close after successful validation
- Model selection dialog should appear after provider validation
- Config path should not conflict with original opencode

## 10. Implementation Task Breakdown

Generate a `task.json`-style task breakdown that an AI coding agent can execute.

Use this simplified structure:

```json
{
  "tasks": [
    {
      "id": "CONFIG-GAP-001",
      "title": "Compare official opencode schema with current opencode-rs config structs",
      "priority": "P0",
      "type": "analysis",
      "description": "...",
      "acceptance_criteria": [
        "..."
      ],
      "files_to_inspect": [
        "..."
      ],
      "files_to_modify": [
        "..."
      ],
      "tests": [
        "..."
      ]
    }
  ]
}
==================================================
5. Engineering Requirements

When doing the analysis:

Do not only inspect type definitions; trace actual runtime usage.
Do not assume a field is supported just because a similarly named struct exists.
Verify whether values are actually loaded, merged, validated, saved, and consumed.
Pay special attention to CLI/TUI flows that mutate config.
Pay attention to provider validation and persistence behavior.
Pay attention to config path conflicts with original opencode.
Prefer compatibility with official opencode unless there is a strong reason to diverge.
Any opencode-rs-specific extension should be namespaced or clearly documented.
Make sure the final spec is actionable by an AI coding agent.
==================================================
6. Final Output Format

Output only the Markdown gap analysis specification document.

Do not write generic advice.

Do not skip implementation details.

Do not simply say “compare the schemas”; actually produce the detailed comparison, gap list, implementation plan, and test plan.


A shorter version you can use when you want the coding agent to act faster:

```text
Compare the current `opencode-rs` config schema and implementation with the official opencode config schema at https://opencode.ai/config.json.

Goal: identify all major gaps preventing `opencode-rs` from having equivalent configuration capability as opencode.

Please inspect the codebase deeply, including config structs, loading, saving, defaults, validation, path resolution, CLI/TUI usage, provider/model persistence, MCP config, agent config, tools, commands, skills, rules, hooks, documentation, and tests.

Produce a Markdown gap analysis spec with:

1. Executive summary
2. Current `opencode-rs` config architecture
3. Official opencode schema capability overview
4. Field-by-field compatibility matrix
5. Major gaps grouped by domain
6. Recommended target Rust config model
7. Config loading and precedence rules
8. Migration plan
9. Concrete test strategy
10. `task.json` implementation breakdown for an AI coding agent

For each gap, include:

- Problem
- Current behavior
- Expected opencode-compatible behavior
- Severity: P0/P1/P2/P3
- Suggested implementation approach
- Files to inspect/modify
- Tests to add

Pay special attention to:
- Provider/model config
- `/connect` flow
- API key validation and persistence
- Config path conflicts with original opencode
- TUI behavior after validation
- MCP config
- Agent/tool permissions
- Schema validation
- Documentation mismatch

Output only the final Markdown spec.