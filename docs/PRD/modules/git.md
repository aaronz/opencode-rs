# PRD: git Module

## Module Overview

**Module Name:** `git`
**Type:** Utility
**Source:** `/packages/opencode/src/git/`

## Purpose

Git operations wrapper. Provides Git functionality for the agent.

## Functionality

### Core Features

1. **Git Operations**
   - Repository detection
   - Branch operations
   - Commit operations
   - Diff handling

## Acceptance Criteria

1. Git operations work correctly
2. Repository state is detected
3. Errors are handled properly

## Rust Implementation Guidance

The Rust equivalent should:
- Use `git2` crate for Git operations
- Use `tokio` for async when needed
- Handle errors properly

## Test Design

### Integration Tests
- `repo_operations`: Initialize a temp git repo, make a commit, and use the module to read branches and status.

### Rust Specifics
- Use the `git2` crate testing patterns on local `tempfile` directories.
