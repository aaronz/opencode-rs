# PRD: GitHub and GitLab Integration

## Overview

This document describes repository-hosted integration patterns for GitHub and GitLab.

GitHub coverage in this PRD is stronger and example-driven. GitLab coverage should be treated as more conditional unless explicitly backed by the chosen upstream implementation for the Rust port.

---

## GitHub Integration

### GitHub App Setup

```bash
opencode github install
```

Creates:
- GitHub App installation
- Workflow file at `.github/workflows/opencode.yml`
- Required secrets

### Manual Setup

1. Install [github.com/apps/opencode-agent](https://github.com/apps/opencode-agent)
2. Add workflow file:

```yaml
name: opencode

on:
  issue_comment:
    types: [created]
  pull_request_review_comment:
    types: [created]

jobs:
  opencode:
    if: contains(github.event.comment.body, '/oc') || contains(github.event.comment.body, '/opencode')
    runs-on: ubuntu-latest
    permissions:
      id-token: write
    steps:
      - uses: actions/checkout@v6
        with:
          fetch-depth: 1
          persist-credentials: false
      - uses: anomalyco/opencode/github@latest
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        with:
          model: anthropic/claude-sonnet-4-20250514
```

### Trigger Events

| Event | Trigger | Context |
|-------|---------|---------|
| `issue_comment` | `/oc` or `/opencode` | Full issue thread |
| `pull_request_review_comment` | `/oc` or `/opencode` | File + line + diff |
| `issues` | Create/edit with `prompt` input | Issue content |
| `pull_request` | Open/sync/reopen | PR diff |
| `schedule` | Cron with `prompt` input | No issue context |
| `workflow_dispatch` | Manual with `prompt` input | No issue context |

### Configuration

| Input | Required | Description |
|-------|---------|-------------|
| `model` | Yes | Provider/model format |
| `agent` | No | Agent type (default: `build`) |
| `share` | No | Share session (default: true for public) |
| `prompt` | No | Custom prompt |
| `token` | No | GitHub token (default: App token) |

---

## GitLab Integration

### CI Component (Conditional)

```yaml
include:
  - component: $CI_SERVER_FQDN/nagyv/gitlab-opencode/opencode@2
    inputs:
      config_dir: ${CI_PROJECT_DIR}/opencode-config
      auth_json: $OPENCODE_AUTH_JSON
      command: optional-command
      message: "Your prompt"
```

### GitLab Duo (Experimental / Environment-dependent)

Support status: **Experimental / environment-dependent**. Availability depends on the GitLab product tier, deployment setup, and whichever upstream integration path is adopted for the Rust port.

1. Configure GitLab Duo in GitLab settings (if available in your GitLab tier)
2. Use `@opencode` mention in MRs/issues (if supported by GitLab Duo configuration)

---

## Example Workflows

### Issue Triage

```yaml
on:
  issues:
    types: [opened]

jobs:
  triage:
    if: github.event.issue.user.created_at > 30.days
    steps:
      - uses: anomalyco/opencode/github@latest
        with:
          model: anthropic/claude-sonnet-4-20250514
          prompt: |
            Review this issue and provide documentation links
            or error handling guidance if it's a bug.
```

### PR Review

```yaml
on:
  pull_request:
    types: [opened, synchronize, reopened]

jobs:
  review:
    steps:
      - uses: anomalyco/opencode/github@latest
        with:
          model: anthropic/claude-sonnet-4-20250514
          prompt: |
            Review this PR:
            - Code quality
            - Potential bugs
            - Suggestions
```

### Scheduled Review

```yaml
on:
  schedule:
    - cron: "0 9 * * 1"

jobs:
  review:
    steps:
      - uses: anomalyco/opencode/github@latest
        with:
          model: anthropic/claude-sonnet-4-20250514
          prompt: |
            Review codebase for TODO comments
            and create issues for important ones.
```

---

## Cross-References

| Document | Topic |
|----------|-------|
| [Configuration System](./06-configuration-system.md) | Config ownership, `share` settings |
| [07-server-api.md](./07-server-api.md) | Sharing API endpoints |
