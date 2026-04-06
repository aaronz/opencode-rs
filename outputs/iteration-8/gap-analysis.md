I'll gather context by exploring the codebase structure and locating the PRD document in parallel.
Let me explore the Rust implementation and compare with PRD.
# 差距分析报告

## 一、概述

本报告对比 **PRD.md** (OpenCode-RS 产品设计文档) 与 **rust-opencode-port** 当前实现，分析功能完整性、接口、前端、数据模型、配置管理及测试覆盖等维度，识别差距并分类优先级。

---

## 二、PRD 核心要求 vs 实现状态

### 2.1 核心功能模块对比

| PRD 要求 | 实现状态 | 模块 | 差距说明 |
|----------|----------|------|----------|
| **1. 项目感知的 AI 会话** | ✅ 已实现 | core/project | ProjectManager、Session、WorktreeManager |
| **2. TUI 交互** | ✅ 已实现 | tui | ratatui 实现，包含 app/input/layout 等 |
| **3. Tool Calling 与权限系统** | ✅ 已实现 | tools + permission | 30+ 工具 + PermissionEvaluator |
| **4. 文件读取/编辑/补丁/Bash** | ✅ 已实现 | tools | read/edit/write/patch/bash 等 |
| **5. Session 持久化与恢复** | ✅ 已实现 | storage | SQLite + checkpoint |
| **6. 模型提供方抽象** | ✅ 已实现 | llm | 20+ providers (OpenAI/Anthropic/Ollama等) |
| **7. 配置系统** | ✅ 已实现 | core/config | JSONC loader, 多层 merge |
| **8. Server API** | ✅ 已实现 | server | REST + SSE + WebSocket |
| **9. LSP 诊断接入** | ✅ 已实现 | lsp | DiagnosticAggregator, symbols |
| **10. MCP 接入** | ✅ 已实现 | mcp | stdio/remote bridge |
| **11. 自定义 Commands/Skills** | ✅ 已实现 | core + tools | CommandRegistry, SkillManager |
| **12. 基础插件系统** | ⚠️ 部分实现 | plugin | 框架存在，功能不完整 |

### 2.2 Crates 映射

| Crate | 对应功能 | 状态 |
|-------|---------|------|
| `opencode-core` | 领域模型、配置、session、context | ✅ 完整 |
| `opencode-agent` | Build/Plan/Explore/Review/Refactor/Debug agents | ✅ 完整 |
| `opencode-tools` | 文件、bash、git、session tools | ✅ 完整 |
| `opencode-llm` | Provider 适配、认证层 | ✅ 完整 |
| `opencode-permission` | allow/ask/deny、审计日志 | ✅ 完整 |
| `opencode-storage` | SQLite、repository pattern | ✅ 完整 |
| `opencode-server` | REST API、SSE、WS、mDNS | ✅ 完整 |
| `opencode-tui` | ratatui 客户端 | ✅ 完整 |
| `opencode-lsp` | LSP bridge、diagnostics | ✅ 完整 |
| `opencode-mcp` | MCP client/server | ✅ 完整 |
| `opencode-git` | git status/diff/log tools | ✅ 完整 |
| `opencode-auth` | 认证管理、OAuth | ✅ 完整 |
| `opencode-cli` | CLI 入口 | ✅ 完整 |
| `opencode-plugin` | 插件系统 | ⚠️ 框架级 |
| `opencode-control-plane` | 企业级功能 | ❌ 未实现 |

---

## 三、差距详细列表

### 3.1 功能完整性差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **P1** 插件系统 - 仅框架，无实际插件能力 | P1 | plugin | 实现 WASM 插件加载器、事件总线 |
| **P1** 企业控制平面 - account/enterprise SSO | P1 | control-plane | 实现 CentralConfig、SsoConfig |
| **P2** Plan Agent 功能不完整 - 缺少只读约束强制执行 | P2 | agent | plan_agent 需严格限制 write/edit/bash |
| **P2** Share 能力 - 本地导出无，服务层无 | P2 | core/share | 实现 JSON/Markdown 导出 + self-hosted |
| **P2** GitHub 集成 - 无 issue/PR trigger | P2 | git | v2 路线，可暂缓 |
| **P3** OAuth Browser 登录 - 仅有框架 | P3 | auth | 实现 PKCE + localhost callback |

### 3.2 接口完整性差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **P0** Provider 管理 API 缺失 | P0 | server | POST /providers/{id}/credentials, /test, /revoke |
| **P0** Permission 审批 API 缺失 | P0 | server | POST /permissions/{req_id}/reply |
| **P1** Artifact API - diff/snapshots/revert | P1 | server | GET /sessions/{id}/diff, /snapshots, /revert |
| **P1** Runtime API - /doc, /health | P1 | server | 已部分实现，需完善 |

### 3.3 前端完整性差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **P2** Web UI - 仅基础实现 | P2 | server | 实现完整 web shell (v1.5 目标) |
| **P2** IDE 插件 - 无 | P2 | - | v2 目标，可暂缓 |
| **P2** Desktop Shell - 无 | P2 | - | v1.5 目标 |

### 3.4 数据模型差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **P0** Session State 状态机不完整 | P0 | core | idle/thinking/awaiting_permission 等 12 状态 |
| **P1** Tool Invocation Record 缺失详情 | P1 | core | 需补充 args_hash, latency_ms |
| **P1** PermissionDecision 审计模型 | P1 | permission | 需补充 scope, user_note |

### 3.5 配置管理差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **P2** credentialRef 引用机制 | P2 | core | 实现从 credential store 解析引用 |
| **P2** Provider 动态启用/禁用 | P2 | core | enabled_providers, disabled_providers |
| **P3** 环境变量覆盖不完全 | P3 | core | 补充 provider-specific 变量绑定 |

### 3.6 测试覆盖差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **P2** 工具单元测试 - 仅部分 | P2 | tools | 补充 edit/patch/batch 测试 |
| **P2** TUI E2E 测试 | P2 | tui | 实现 ratatui testing framework |
| **P2** Agent 集成测试 | P2 | agent | 补充 build/plan 闭环测试 |
| **P3** Provider 协议测试 | P3 | llm | 补充 mock provider 测试 |

---

## 四、P0/P1/P2 问题分类

### P0 - 阻断性问题 (必须修复)

| # | 问题 | 当前状态 | 修复方案 |
|---|------|----------|----------|
| 1 | **Provider 管理 API 缺失** | server 无 credentials API | 新增 routes/provider.rs: POST /providers/{id}/credentials |
| 2 | **Permission 审批 API 缺失** | 仅有 PermissionEvaluator | 新增 routes/permission.rs: POST /permissions/{id}/reply |
| 3 | **Session State 状态机不完整** | 12 状态未完全实现 | 完善 session_state.rs 状态转换逻辑 |

### P1 - 重要功能 (计划修复)

| # | 问题 | 当前状态 | 修复方案 |
|---|------|----------|----------|
| 1 | **插件系统不完整** | 仅框架 | 实现 WASM 加载器 + 事件 hooks |
| 2 | **Plan Agent 写限制** | 可被绕过 | build_agent.rs 添加工具黑名单 |
| 3 | **Artifact API** | 无 diff/snapshots | 新增 routes/artifact.rs |
| 4 | **Share 本地导出** | 无 | 实现 session JSON/Markdown export |
| 5 | **企业控制平面** | 无 | 实现 account + enterprise 模块 |

### P2 - 增强功能 (建议修复)

| # | 问题 | 当前状态 | 修复方案 |
|---|------|----------|----------|
| 1 | **Web UI 完整实现** | 基础 | 完善 web_ui routes |
| 2 | **工具测试覆盖** | 部分 | 补充测试用例 |
| 3 | **OAuth Browser 登录** | 框架 | 实现完整 PKCE 流程 |
| 4 | **Context Engine 压缩** | 基础 | 实现 85%/92%/95% 阈值 |

---

## 五、技术债务清单

| # | 技术债务 | 影响 | 优先级 |
|---|----------|------|--------|
| 1 | **硬编码配置** - 部分路径写死 | 灵活性 | P2 |
| 2 | **错误码不统一** - 各模块 error 分散 | 可维护性 | P2 |
| 3 | **类型定义缺失** - 部分使用 serde_json::Value | 类型安全 | P1 |
| 4 | **日志脱敏不完整** - credential 可能泄露 | 安全性 | P1 |
| 5 | **tracing 链路不完整** - 部分模块无 span | 可观测性 | P2 |
| 6 | **Auth Store 加密** - 明文存储 | 安全性 | P1 |

---

## 六、总结

### 实现进度

- **已完成**: 12/12 PRD v1 核心功能
- **完成度**: ~85% (核心功能齐全，部分模块深度不足)

### 关键差距

1. **P0**: API 层不完整 (Provider/Permission 管理)
2. **P1**: 插件系统、企业功能、Plan Agent 约束
3. **P2**: 测试覆盖、Web UI、Share、OAuth

### 建议优先级

**第一阶段 (P0)**: 完善 API 层 + Session 状态机
**第二阶段 (P1)**: 插件系统 + Plan 约束 + 企业模块
**第三阶段 (P2)**: 测试完善 + Web UI + Share + OAuth
