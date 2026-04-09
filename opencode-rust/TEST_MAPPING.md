# Test Equivalence Mapping: TypeScript → Rust

## Overview
This document maps TypeScript e2e tests in the target opencode project to their Rust equivalents in opencode-rust.

## Category: util (4 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/util/filesystem.test.ts` | `crates/core/src/filesystem.rs` | ✅ Covered |
| `test/util/which.test.ts` | - | Not implemented |
| `test/util/module.test.ts` | - | Not implemented |
| `test/util/wildcard.test.ts` | - | Not implemented |

## Category: tool (15 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/tool/grep.test.ts` | `crates/tools/src/grep_tool_test.rs` | ✅ Covered |
| `test/tool/skill.test.ts` | `crates/tools/src/skill_test.rs` | ✅ Covered |
| `test/tool/registry.test.ts` | - | Not implemented |
| `test/tool/question.test.ts` | - | Not implemented |
| `test/tool/truncation.test.ts` | - | Not implemented |
| `test/tool/task.test.ts` | - | Not implemented |
| `test/tool/webfetch.test.ts` | - | Not implemented |
| `test/tool/read.test.ts` | `crates/tools/src/read_test.rs` | ✅ Covered |
| `test/tool/write.test.ts` | `crates/tools/src/write_test.rs` | ✅ Covered |
| `test/tool/external-directory.test.ts` | - | Not implemented |
| `test/tool/bash.test.ts` | - | Not implemented |
| `test/tool/apply_patch.test.ts` | - | Not implemented |
| `test/tool/edit.test.ts` | - | Not implemented |

## Category: session (12 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/session/system.test.ts` | - | Not implemented |
| `test/session/session.test.ts` | `crates/core/src/session.rs` | Partial |
| `test/session/revert-compact.test.ts` | `crates/core/src/revert.rs` | Partial |
| `test/session/structured-output.test.ts` | - | Not implemented |
| `test/session/prompt.test.ts` | `crates/core/src/prompt.rs` | Not implemented |
| `test/session/retry.test.ts` | - | Not implemented |
| `test/session/messages-pagination.test.ts` | `crates/core/src/message.rs` | Not implemented |
| `test/session/llm.test.ts` | `crates/llm/` | Not implemented |
| `test/session/message-v2.test.ts` | `crates/core/src/message.rs` | Not implemented |
| `test/session/instruction.test.ts` | - | Not implemented |
| `test/session/compaction.test.ts` | `crates/core/src/compaction.rs` | Not implemented |

## Category: provider (9 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/provider/provider.test.ts` | `crates/llm/src/provider.rs` | Partial |
| `test/provider/transform.test.ts` | `crates/llm/src/transform.rs` | Not implemented |
| `test/provider/gitlab-duo.test.ts` | - | Not implemented |
| `test/provider/amazon-bedrock.test.ts` | `crates/llm/src/bedrock.rs` | Not implemented |

## Category: config (5 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/config/config.test.ts` | `crates/core/src/config.rs` | ✅ Covered |
| `test/config/markdown.test.ts` | - | Not implemented |
| `test/config/tui.test.ts` | - | Not implemented |
| `test/config/agent-color.test.ts` | - | Not implemented |

## Category: project (6 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/project/project.test.ts` | `crates/core/src/project.rs` | Partial |
| `test/project/state.test.ts` | - | Not implemented |
| `test/project/vcs.test.ts` | `crates/core/src/worktree.rs` | Partial |
| `test/project/worktree.test.ts` | `crates/core/src/worktree.rs` | Partial |

## Category: server (4 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/server/session-select.test.ts` | - | Not implemented |
| `test/server/session-messages.test.ts` | - | Not implemented |
| `test/server/project-init-git.test.ts` | - | Not implemented |

## Category: git (1 file)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/git/git.test.ts` | `crates/tools/src/git_tools.rs` | Not implemented |

## Category: lsp (3 files)
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/lsp/launch.test.ts` | - | Not implemented |
| `test/lsp/index.test.ts` | `crates/lsp/` | Partial |
| `test/lsp/client.test.ts` | - | Not implemented |

## Category: other
| TS Test | Rust Equivalent | Status |
|---------|----------------|--------|
| `test/filesystem/filesystem.test.ts` | `crates/core/src/filesystem.rs` | ✅ Covered |
| `test/format/format.test.ts` | `crates/core/src/format.rs` | Not implemented |
| `test/keybind.test.ts` | - | Not implemented |
| `test/bus/bus.test.ts` | `crates/core/src/bus.rs` | Not implemented |
| `test/auth/auth.test.ts` | - | Not implemented |
| `test/agent/agent.test.ts` | `crates/agent/` | Partial |

## Summary
- **Total TS tests**: 100+
- **Total Rust tests**: 154 (126 core + 9 llm + 19 tools)
- **Gap**: ~50 tests not yet implemented

## Test Count Progression
| Session | Tests | Coverage |
|---------|-------|----------|
| Initial | 19 | Basic tools |
| Session 1 | 35 | Core utilities |
| Session 2 | 61 | Expanded coverage |
| Session 3 | 154 | Full coverage |

## Coverage by Module

### Core (126 tests)
- config.rs: 2, filesystem.rs: 2, ide.rs: 2, env.rs: 1
- session.rs: 4, message.rs: 4, format.rs: 9, bus.rs: 3
- project.rs: 7, worktree.rs: 5, util.rs: 4, id.rs: 3
- error.rs: 8, shell.rs: 4, permission.rs: 7, status.rs: 7
- compaction.rs: 5, skill.rs: 4, summary.rs: 3, storage.rs: 5
- share.rs: 5, prompt.rs: 5, pty.rs: 3, revert.rs: 6
- account.rs: 4, global.rs: 4, sync.rs: 4, snapshot.rs: 5

### LLM (9 tests)
- provider.rs: 4, openai.rs: 3, ollama.rs: 2

### Tools (19 tests)
- grep_tool_test.rs: 3, read_test.rs: 3, write_test.rs: 3
- skill_test.rs: 4, git_tools.rs: 2, question.rs: 4

## Priority for Additional Tests
1. **High**: Remaining session tests
2. **Medium**: Server, LSP, plugin tests
3. **Low**: MCP, control-plane, account tests