Let me read the PRD files and explore the implementation directories in parallel.
Three explore agents launched. Let me also gather more context on the build system and scripts while waiting.
I detect **investigation/analysis** intent - you want a comprehensive gap analysis between PRD requirements and current implementation. My approach: fire parallel explore agents for deep codebase analysis, then synthesize findings into a structured report.

Background agents launched for:
1. **Rust crate structure & maturity** (bg_fcc03dfc)
2. **TUI implementation coverage** (bg_7f37713c)
3. **Server/API implementation** (bg_03ef0dd4)

Already gathered:
- 3 PRD documents (main PRD, TUI PRD, Config PRD)
- Existing gap analysis from 2026-03-31 (claims 85-90% completeness)
- Workspace structure: 15 crates + tests + benchmarks
- Test mapping showing ~50 tests not yet implemented

Let me collect the agent results now.
All agents complete. Let me do a few final targeted checks to fill remaining gaps.
Now I have comprehensive data from all agents. Let me synthesize the full gap analysis.

---

# 差距分析报告

## 概览

| 维度 | PRD 要求 | 实现状态 | 完成度 |
|------|---------|---------|--------|
| 核心领域模型 | Session/Message/Tool/Permission | ✅ 完整 | 100% |
| Agent 系统 | 10 种 Agent | ✅ 完整 | 100% |
| Tool 系统 | 35+ 工具 | ⚠️ 33/35 | 94% |
| LLM Provider | 15+ Provider | ✅ 20+ | 111% |
| 权限系统 | allow/ask/deny + scope | ✅ 完整 | 100% |
| TUI | 消息/输入/权限/差异/会话 | ✅ 完整 | 100% |
| Server API | REST/WS/SSE | ⚠️ 缺 2 端点 | 95% |
| 配置系统 | JSONC + 多层合并 | ⚠️ TOML 替代 | 80% |
| MCP | 本地/远程/工具桥接 | ✅ 完整 | 100% |
| LSP | 诊断/符号 | ⚠️ 基础实现 | 70% |
| 插件系统 | WASM + 事件 | ✅ 完整 | 100% |
| 测试覆盖 | ~100 TS 测试等价 | ⚠️ 154/200+ | 75% |
| Share 系统 | 本地导出 + 服务层 | ⚠️ 部分 | 60% |
| GitHub 集成 | v2 功能 | ❌ 未实现 | 0% |
| 企业控制面 | SAML/SSO | ⚠️ 框架存在 | 40% |

## 差距列表

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|---------|------|---------|
| session_load / session_save 工具缺失 | P1 | tools | 补充两个工具实现，连接 storage crate |
| 缺少 `/health` 健康检查端点 | P1 | server | 添加简单 GET /health 路由 |
| 缺少 session abort 端点 | P1 | server | 添加 POST /sessions/{id}/abort |
| 配置格式使用 TOML 而非 JSONC | P1 | config | PRD 要求 JSONC/JSON，实现使用 TOML，需迁移 |
| 缺少 OpenAPI/Swagger 文档生成 | P2 | server | 集成 utoipa 生成 OpenAPI 3.1 文档 |
| 缺少 SDK 生成钩子 | P2 | server | 添加 openapi-generator 集成 |
| LSP 仅基础实现，缺 definition/references | P2 | lsp | 补充 LSP v1.1 能力 |
| Share 系统未完整实现服务层 | P2 | core/share | 实现 self-hosted share server |
| GitHub 集成完全缺失 | P2 | git | v2 规划，但 crate 已存在 |
| ~50 个测试未实现 | P2 | tests | 按 TEST_MAPPING.md 优先级补充 |
| HuggingFace/AI21 provider 标记但未验证 | P3 | llm | 文件存在但需验证实现完整性 |
| 缺少 compaction 自动触发测试 | P3 | core | 补充 session 压缩自动化测试 |
| 缺少 server 集成测试 | P3 | tests | 添加 HTTP API 端到端测试 |
| 缺少 LSP 集成测试 | P3 | tests | 添加 LSP client/server 测试 |
| 企业控制面 (SAML/SSO) 仅框架 | P3 | control-plane | 非 v1 需求，标记为 v1.5+ |
| 权限回复端点可能缺失 | P3 | server | 确认 tool permission reply 路由 |

## P0/P1/P2 问题分类

### P0 - 无 (无阻塞发布的问题)

### P1 - 高优先级 (影响 v1 核心功能)

| 问题 | 影响 | 工作量 |
|------|------|--------|
| **配置格式不匹配** - PRD 要求 JSONC，实现用 TOML | 与 OpenCode 生态不兼容，用户配置无法迁移 | 中 |
| **session_load/session_save 工具缺失** | Agent 无法主动加载/保存会话上下文 | 小 |
| **缺少 /health 端点** | 无法做服务健康检查/容器探针 | 小 |
| **缺少 session abort 端点** | PRD 明确要求，客户端无法中止运行中会话 | 小 |

### P2 - 中优先级 (影响完整性和可维护性)

| 问题 | 影响 | 工作量 |
|------|------|--------|
| OpenAPI 文档缺失 | 无法自动生成 SDK，开发者体验差 | 中 |
| LSP 能力不完整 | 仅 diagnostics，缺 definition/references/hover | 中 |
| Share 服务层缺失 | 只能本地导出，无法远程分享 | 中 |
| 测试覆盖 75% | 关键路径 (session/server/LSP) 缺测试 | 大 |

### P3 - 低优先级 (v1.5+ 或非阻塞)

| 问题 | 影响 | 工作量 |
|------|------|--------|
| GitHub 集成 | v2 功能，不影响 v1 | 大 |
| 企业控制面 | v1.5+ 功能 | 大 |
| 部分 provider 验证 | 非核心 provider | 小 |

## 技术债务清单

| 技术债务 | 位置 | 风险 | 建议 |
|---------|------|------|------|
| **配置格式不一致** | `crates/core/src/config.rs` | 高 - 与 PRD 定义的 JSONC 格式冲突，用户无法使用 OpenCode 官方配置 | 迁移到 JSONC 或明确文档说明差异 |
| **README 过时** | `rust-opencode-port/README.md` | 中 - 只提到 3 个 provider，实际有 20+ | 更新 README 反映真实能力 |
| **权限回复路由不明确** | `crates/server/src/routes/` | 中 - permission reply 可能散落在 MCP 而非独立端点 | 统一为 POST /sessions/{id}/permissions/{req_id}/reply |
| **TUI 权限确认非模态** | `crates/tui/src/right_panel.rs` | 低 - PRD 要求 modal，实现为 panel tab | 评估 UX 影响，当前方案也可用 |
| **auth_layered 目录未纳入 lib.rs** | `crates/llm/src/auth_layered/` | 低 - 可能存在未导出的认证层代码 | 检查是否应整合进 auth.rs |
| **sap_aicore.rs 未在 PRD 中定义** | `crates/llm/src/sap_aicore.rs` | 低 - 超出 PRD 范围的 provider | 确认是否保留 |
| **测试分布不均** | `crates/*/src/*_test.rs` | 中 - core 126 测试，llm 仅 9 测试 | 补充 provider 和 streaming 测试 |

## 与上次差距分析 (2026-03-31) 对比

| 指标 | 上次 | 本次 | 变化 |
|------|------|------|------|
| 实现完成度 | 85-90% | ~88% | 持平 |
| 缺失关键功能 | 0 | 0 | 持平 |
| 测试覆盖 | ~154 tests | ~154 tests | 持平 |
| 新增发现 | - | 配置格式冲突 | 🔴 新问题 |
| 新增发现 | - | API 缺 2 端点 | 🟡 细化 |

**上次声称已解决的问题验证**:
- ✅ 10/10 Agents - 确认完整
- ✅ WebSocket/SSE - 确认完整
- ✅ MCP 协议 - 确认完整
- ✅ 33/35 Tools - 确认，缺 session_load/save
- ⚠️ "all endpoints complete" - **不准确**，缺 /health 和 /abort
