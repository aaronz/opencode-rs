## 1. Commands Implementation

### 1.1 Add command-changelog
- [x] 1.1.1 Create .opencode/command/changelog.md with YAML frontmatter (model: opencode/kimi-k2.5)
- [x] 1.1.2 Implement changelog generation logic for sections TUI, Desktop, Core, Misc
- [x] 1.1.3 Implement PR parsing and subagent spawning for summarization

### 1.2 Add command-spellcheck
- [x] 1.2.1 Create .opencode/command/spellcheck.md with YAML frontmatter
- [x] 1.2.2 Implement markdown diff extraction for .md and .mdx files
- [x] 1.2.3 Implement spellcheck logic for changed lines

### 1.3 Add command-commit
- [x] 1.3.1 Create .opencode/command/commit.md with YAML frontmatter and subtask: true
- [x] 1.3.2 Implement git diff/diff --cached/stauts display
- [x] 1.3.3 Implement commit message with prefix validation (docs:, tui:, core:, ci:, ignore:, wip:)
- [x] 1.3.4 Implement push functionality
- [x] 1.3.5 Handle conflicts by notifying user (no auto-fix)

### 1.4 Add command-issues
- [x] 1.4.1 Create .opencode/command/issues.md with YAML frontmatter (model: opencode/claude-haiku-4-5)
- [x] 1.4.2 Implement gh cli integration for issue search
- [x] 1.4.3 Implement search with multiple keywords from title/description
- [x] 1.4.4 Format output with issue number, title, explanation, link

### 1.5 Add command-ai-deps
- [x] 1.5.1 Create .opencode/command/ai-deps.md with YAML frontmatter
- [x] 1.5.2 Implement package.json reading for opencode packages
- [x] 1.5.3 Implement dependency version analysis (minor/patch only)
- [x] 1.5.4 Implement changelog/reference fetching for each dependency
- [x] 1.5.5 Write findings to ai-sdk-updates.md

### 1.6 Add command-rmslop
- [x] 1.6.1 Create .opencode/command/rmslop.md with YAML frontmatter
- [x] 1.6.2 Implement diff against dev branch
- [x] 1.6.3 Implement AI slop detection (extra comments, try/catch, any casts, inconsistent style, emoji)
- [x] 1.6.4 Implement removal of identified slop
- [x] 1.6.5 Provide 1-3 sentence summary

### 1.7 Add command-learn
- [x] 1.7.1 Create .opencode/command/learn.md with YAML frontmatter
- [x] 1.7.2 Implement session analysis for discoveries
- [x] 1.7.3 Implement AGENTS.md creation/update at appropriate scope levels
- [x] 1.7.4 Format learnings as 1-3 line insights

## 2. Agents Implementation

### 2.1 Add agent-translator
- [x] 2.1.1 Create .opencode/agent/translator.md with YAML frontmatter
- [x] 2.1.2 Set color: "#38A3EE"
- [x] 2.1.3 Configure translation capabilities

### 2.2 Add agent-duplicate-pr
- [x] 2.2.1 Create .opencode/agent/duplicate-pr.md with YAML frontmatter
- [x] 2.2.2 Set mode: primary, hidden: true, model: opencode/claude-haiku-4-5
- [x] 2.2.3 Set color: "#E67E22"
- [x] 2.2.4 Configure tools: github-pr-search only
- [x] 2.2.5 Implement duplicate detection logic

### 2.3 Add agent-docs
- [x] 2.3.1 Create .opencode/agent/docs.md with YAML frontmatter
- [x] 2.3.2 Set color: "#38A3EE"
- [x] 2.3.3 Configure documentation writing rules (tone, formatting, code style)

### 2.4 Add agent-triage
- [x] 2.4.1 Create .opencode/agent/triage.md with YAML frontmatter
- [x] 2.4.2 Set mode: primary, hidden: true, model: opencode/minimax-m2.5
- [x] 2.4.3 Set color: "#44BA81"
- [x] 2.4.4 Configure tools: github-triage only
- [x] 2.4.5 Implement labeling rules (windows, perf, desktop, nix, zen, core, acp, docs, opentui)
- [x] 2.4.6 Implement assignment rules for each label

## 3. Tools Implementation (Required by Agents)

### 3.1 Add github-triage tool
- [x] 3.1.1 Create tool implementation for GitHub issue triage
- [x] 3.1.2 Implement label application
- [x] 3.1.3 Implement assignee assignment

### 3.2 Add github-pr-search tool
- [x] 3.2.1 Create tool implementation for PR search
- [x] 3.2.2 Implement keyword-based search
- [x] 3.2.3 Implement duplicate detection algorithm
