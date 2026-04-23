# Task Plan: Implementing PRD Module Tasks

## Overview
Implement test cases from tasks.json files across all modules in `docs/PRD/modules/`

## Module Status Summary

| Module | Total | Completed | Not Started | Requires Code | Requires Infra |
|--------|-------|-----------|-------------|---------------|----------------|
| lsp | 11 | 11 | 0 | 0 | 0 |
| config | 20 | 14 | 0 | 4 | 2 |
| tool | 24 | 24 | 0 | 0 | 0 |
| provider | 18 | 18 | 0 | 0 | 0 |
| storage | 16 | 16 | 0 | 0 | 0 |
| auth | 12 | 8 | 4 | 0 | 0 |
| mcp | 7 | 7 | 0 | 0 | 0 |
| git | 14 | 5 | 9 | 0 | 0 |
| plugin | 11 | 9 | 0 | 2 | 0 |
| permission | 11 | 11 | 0 | 0 | 0 |
| session | 16 | 16 | 0 | 0 | 0 |
| agent | 18 | 18 | 0 | 0 | 0 |
| disaster_recovery | 13 | 0 | 13 | 0 | 0 |
| observability | 15 | 0 | 15 | 0 | 0 |
| chaos | 15 | 0 | 15 | 0 | 0 |
| performance | 10 | 0 | 10 | 0 | 0 |
| integration | 17 | 0 | 17 | 0 | 0 |
| project | 17 | 0 | 17 | 0 | 0 |
| util | 16 | 0 | 16 | 0 | 0 |
| acp | 10 | 0 | 10 | 0 | 0 |
| cli | 22 | 0 | 22 | 0 | 0 |
| server | 20 | 0 | 20 | 0 | 0 |
| file | 12 | 0 | 0 | 12 | 0 |
| shell | 13 | 3 | 0 | 10 | 0 |

## Current Focus: shell module
### Critical Tasks to Implement
1. shell_sec_001: Shell injection prevention (CRITICAL) - requires_code_change
2. shell_sec_002: Environment variable sanitization (CRITICAL) - requires_code_change
3. shell_e2e_001: Shell command with environment variables - requires_code_change

### Implementation Plan for shell_sec_001
The current `BashTool` directly passes commands to `sh -c` without sanitization. Need to:
1. Add command validation/sanitization to block shell metacharacters
2. Update test to verify injection attempts are actually blocked
3. Ensure error is returned (not just weak content check)

## Phases

### Phase 1: Already Completed Modules
Skip modules that are fully completed (lsp, tool, provider, storage, mcp, permission, session, agent)

### Phase 2: Partially Completed (auth, git, plugin, config, shell)
Work through remaining tasks in these modules

### Phase 3: Not Started Modules
- disaster_recovery (13 tasks)
- observability (15 tasks)
- chaos (15 tasks)
- performance (10 tasks)
- integration (17 tasks)
- project (17 tasks)
- util (16 tasks)
- acp (10 tasks)
- cli (22 tasks)
- server (20 tasks)
- file (12 tasks)

## Current Progress
- Starting with modules that have not_started status
- Will track each module completion

## Critical Tasks to Prioritize
1. Shell module - shell_sec_001 (CRITICAL: shell injection)
2. File module - filesystem_sec_001 (CRITICAL: path traversal)
3. Auth module - auth_oauth_001 (CRITICAL: CSRF prevention)
4. Git module - git_security_001 (CRITICAL: credentials in logs)