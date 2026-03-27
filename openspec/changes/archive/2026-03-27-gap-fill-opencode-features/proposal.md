## Why

The mycode project aims to replicate the opencode project's .opencode configuration. Currently, mycode has 4 openspec commands and 4 skills, while the target opencode project has 7 commands (changelog, spellcheck, commit, issues, ai-deps, rmslop, learn) and 4 agents (translator, duplicate-pr, docs, triage). This gap prevents mycode from providing the same user experience and functionality as the target project.

## What Changes

1. **Add 7 missing commands to .opencode/command/**: changelog, spellcheck, commit, issues, ai-deps, rmslop, learn
2. **Add 4 missing agents to .opencode/agent/**: translator, duplicate-pr, docs, triage
3. **Set up test infrastructure** to run and pass target project tests

## Capabilities

### New Capabilities
- `command-changelog`: Generate changelogs from git history
- `command-spellcheck`: Spellcheck markdown file changes
- `command-commit`: Git commit with push and prefix conventions
- `command-issues`: Find issues on GitHub via gh cli
- `command-ai-deps`: AI-powered dependency management
- `command-rmslop`: Remove slop from code
- `command-learn`: Extract learnings to AGENTS.md
- `agent-translator`: Translation agent for multilingual docs
- `agent-duplicate-pr`: Detect duplicate PRs
- `agent-docs`: Documentation writing agent
- `agent-triage`: GitHub issue triaging agent with labeling rules

### Modified Capabilities
- None. All new capabilities.

## Impact

- `.opencode/command/` directory: 7 new command files
- `.opencode/agent/` directory: 4 new agent definition files
- Test configuration: Need to understand target project's test structure
