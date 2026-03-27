## ADDED Requirements

### Requirement: e2e-project-tests
Tests for project discovery and worktree handling.

#### Scenario: Project discovers git repository
- **WHEN** loading a directory that's a git repo
- **THEN** project is identified with correct ID

#### Scenario: Project handles non-git directory
- **WHEN** loading a directory without git
- **THEN** returns global project

#### Scenario: Project worktree handling
- **WHEN** loading from a git worktree
- **THEN** worktree is properly identified
