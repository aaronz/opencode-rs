# Task Plan: Unify OpenCode Documentation with PRD Documents

## Goal

Create a unified documentation structure that combines OpenCode user documentation (zh-cn) with PRD implementation documents for Rust development guidance.

## Source Documents

| Source | Location | Type |
|--------|----------|------|
| OpenCode User Docs (zh-cn) | `/Users/aaronzh/Documents/GitHub/opencode/packages/web/src/content/docs/zh-cn/` | User-facing documentation |
| OpenCode-RS PRD | `/Users/aaronzh/Documents/GitHub/opencode-rs/docs/PRD/` | Rust implementation guides |

## Target Location

`/Users/aaronzh/Documents/GitHub/opencode-rs/docs/`

## Current Phase

Phase 3: Enhancement (in progress)

## Phase Status

| Phase | Status |
|-------|--------|
| Phase 1: Requirements & Discovery | complete |
| Phase 2: Create Unified Structure | complete |
| Phase 3: Enhance PRDs with User Docs | in_progress |
| Phase 4: Verification | pending |

## Mapping Analysis

### PRD System Docs → User Docs (zh-cn)

| PRD System Doc | User Doc (zh-cn) | Content Area |
|----------------|------------------|--------------|
| 01-core-architecture.md | (conceptual - no direct doc) | Core entities: Project, Session, Message, Part |
| 02-agent-system.md | agents.mdx | Agent roles, primary/subagent model, permissions |
| 03-tools-system.md | tools.mdx | Tool categories, execution pipeline |
| 04-mcp-system.md | mcp-servers.mdx | MCP protocol integration |
| 05-lsp-system.md | lsp.mdx | LSP integration |
| 06-configuration-system.md | config.mdx | Configuration schema |
| 07-server-api.md | server.mdx, web.mdx | HTTP API |
| 08-plugin-system.md | plugins.mdx | Plugin system |
| 09-tui-system.md | tui.mdx | Terminal UI |
| 10-provider-model-system.md | providers.mdx, models.mdx | AI providers |
| 11-formatters.md | formatters.mdx | Code formatters |
| 12-skills-system.md | skills.mdx | Skills system |
| 13-desktop-web-interface.md | web.mdx | Desktop/web interface |
| 14-github-gitlab-integration.md | github.mdx, gitlab.mdx | VCS integration |
| 15-tui-plugin-api.md | (TUI plugin API) | Dialog rendering |
| 16-test-plan.md | (internal) | Test strategy |
| 17-25-*.md | (internal) | Implementation tracking |

### PRD Modules → User Docs

| PRD Module | Category | Related User Docs |
|------------|----------|-------------------|
| agent.md | Core | agents.mdx |
| session.md | Core | (session concepts in agents.mdx) |
| tool.md | Core | tools.mdx, custom-tools.mdx |
| provider.md | Core | providers.mdx |
| cli.md | Infrastructure | cli.mdx |
| server.md | Infrastructure | server.mdx |
| storage.md | Infrastructure | (internal implementation) |
| lsp.md | Integration | lsp.mdx |
| mcp.md | Integration | mcp-servers.mdx |
| plugin.md | Integration | plugins.mdx |
| auth.md | Integration | providers.mdx (auth) |
| project.md | Integration | (project detection - internal) |
| acp.md | Integration | acp.mdx |
| file.md | Utility | tools.mdx (read/write) |
| git.md | Utility | (git operations in bash) |
| config.md | Utility | config.mdx |
| shell.md | Utility | (bash tool in tools.mdx) |
| pty.md | Utility | (internal terminal) |
| sync.md | Utility | (SSE streaming) |
| skill.md | Utility | skills.mdx |
| format.md | Utility | formatters.mdx |
| ide.md | Utility | ide.mdx |
| share.md | Utility | share.mdx |
| permission.md | Utility | permissions.mdx |
| question.md | Utility | (question tool) |
| account.md | Utility | (user account - web) |
| control-plane.md | Utility | enterprise.mdx |
| installation.md | Utility | (install docs in index.mdx) |

## Unified Structure Plan

```
docs/
├── README.md                    # Documentation index
├── user/                        # User-facing docs (from zh-cn)
│   ├── index.mdx
│   ├── agents.mdx
│   ├── tools.mdx
│   └── ...
├── prd/                         # PRD documents
│   ├── system/                  # System-level PRDs
│   │   ├── 01-core-architecture.md
│   │   ├── 02-agent-system.md
│   │   └── ...
│   └── modules/                 # Module-level PRDs
│       ├── README.md
│       ├── agent.md
│       └── ...
└── mapping.md                  # Mapping between docs
```

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Keep source docs in place | Don't duplicate, reference directly |
| Create mapping doc | Links PRD to user docs |
| Create unified index | Single entry point |

## Progress

- [x] Analyze PRD system docs
- [x] Analyze PRD modules
- [x] Map PRD to user docs
- [x] Create unified index (MAPPING.md)
- [x] Enhance system PRDs with user doc links (02, 03, 06)
- [x] Enhance module PRDs with user doc links (cli.md)
- [ ] Enhance remaining system PRDs
- [ ] Enhance remaining module PRDs
- [ ] Verify completeness

## Enhancement Methodology

The PRD documents are enhanced by merging user documentation content with implementation guidance:

### Enhancement Pattern for System PRDs

1. **Add user doc header** at the top:
   ```markdown
   > **User Documentation**: [link-to-user-doc]
   >
   > Brief description of how this doc relates to user-facing docs.
   ```

2. **Add user-facing context** to feature descriptions:
   - Include user-facing descriptions from user docs
   - Add user config keys where applicable
   - Reference user invocation patterns

3. **Enhance tables** with user doc columns:
   - Add "User Doc" column to cross-reference tables
   - Add "User Config Key" where applicable

4. **Add user-facing examples**:
   - Include JSON config examples from user docs
   - Add CLI command examples

### Enhancement Pattern for Module PRDs

1. **Add user doc header** at the top with link to corresponding user doc

2. **Add user-facing command reference table**:
   - List user-facing commands/CLI commands
   - Link to user doc sections

3. **Add user config examples** where relevant

### Enhanced PRD Documents

| Document | Enhancements |
|---------|--------------|
| `system/02-agent-system.md` | Added user doc header, user-facing agent descriptions, permission config examples, user doc links in cross-refs |
| `system/03-tools-system.md` | Added user doc header, built-in tool list from user docs, permission pattern examples |
| `system/06-configuration-system.md` | Added user doc header, user doc links in cross-refs |
| `modules/cli.md` | Added user doc header, user-facing CLI commands reference table, environment variables table |
| `modules/README.md` | Added user doc header with link to zh-cn docs |