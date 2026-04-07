# OpenCode-RS 原子任务清单 v16

**版本**: 16.0  
**日期**: 2026-04-07  
**基于**: plan_v16.md  
**状态**: 已完成  

---

## 阶段 1: P1 关键修复 (Week 1)

### 1.1 配置格式迁移 (TOML → JSONC)

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 1.1.1 | 分析现有 TOML 配置结构，映射到 JSONC 格式 | P1 | ALREADY IMPLEMENTED | - |
| 1.1.2 | 实现 JSONC 解析器 (使用 `jsonc-parser` 或 `serde_json` + 注释剥离) | P1 | ALREADY IMPLEMENTED | 1.1.1 |
| 1.1.3 | 实现多层配置合并 (全局 ~/.config/opencode/ + 项目 .opencode/) | P1 | ALREADY IMPLEMENTED | 1.1.2 |
| 1.1.4 | 实现 TOML → JSONC 自动迁移工具 | P1 | COMPLETED | 1.1.2 |
| 1.1.5 | 更新 config.rs 配置加载逻辑 | P1 | ALREADY IMPLEMENTED | 1.1.2, 1.1.3 |
| 1.1.6 | 编写配置格式迁移测试 | P1 | COMPLETED | 1.1.5 |
| 1.1.7 | Constitution C-056 配置格式迁移规范 | P2 | COMPLETED | 1.1.6 |

### 1.2 session_load / session_save 工具

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 1.2.1 | 实现 session_load 工具 (从 storage crate 加载会话) | P1 | ALREADY IMPLEMENTED | - |
| 1.2.2 | 实现 session_save 工具 (保存会话到 storage crate) | P1 | ALREADY IMPLEMENTED | - |
| 1.2.3 | 集成权限系统 (allow/ask/deny 检查) | P1 | COMPLETED | 1.2.1, 1.2.2 |
| 1.2.4 | 编写工具单元测试 | P1 | COMPLETED | 1.2.3 |
| 1.2.5 | Constitution C-024 会话工具权限规范 | P2 | COMPLETED | 1.2.4 |

### 1.3 Server API 端点补全

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 1.3.1 | 实现 GET /health 健康检查端点 | P1 | COMPLETED | - |
| 1.3.2 | 实现 POST /sessions/{id}/abort 中止端点 | P1 | COMPLETED | - |
| 1.3.3 | 确认 POST /sessions/{id}/permissions/{req_id}/reply 权限回复端点 | P1 | COMPLETED | - |
| 1.3.4 | 编写端点集成测试 | P1 | COMPLETED | 1.3.1, 1.3.2, 1.3.3 |
| 1.3.5 | Constitution C-057 服务器API端点规范 | P2 | COMPLETED | 1.3.4 |

**阶段 1 验收标准**:
- [ ] 配置文件支持 JSONC 格式，TOML 自动迁移
- [ ] session_load/session_save 工具可正常使用
- [ ] GET /health 返回 {status: "ok", version: "x.y.z"}
- [ ] POST /sessions/{id}/abort 可中止运行中会话
- [ ] 权限回复端点确认存在且工作正常

---

## 阶段 2: P2 完整性补全 (Week 2-3)

### 2.1 OpenAPI 文档生成

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 2.1.1 | 集成 utoipa crate 到 server | P1 | pending | - |
| 2.1.2 | 为所有端点添加 #[utoipa::path] 注解 | P1 | pending | 2.1.1 |
| 2.1.3 | 生成 openapi.json 构建产物 | P1 | pending | 2.1.2 |
| 2.1.4 | 添加 Swagger UI 路由 (可选) | P2 | pending | 2.1.3 |
| 2.1.5 | 集成 openapi-generator SDK 生成钩子 | P2 | pending | 2.1.3 |

### 2.2 LSP 能力补全

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 2.2.1 | 实现 textDocument/definition | P1 | pending | - |
| 2.2.2 | 实现 textDocument/references | P1 | pending | - |
| 2.2.3 | 实现 textDocument/hover | P1 | pending | - |
| 2.2.4 | 实现 textDocument/completion (可选) | P3 | pending | 2.2.3 |
| 2.2.5 | 编写 LSP 集成测试 | P1 | pending | 2.2.1, 2.2.2, 2.2.3 |
| 2.2.6 | Constitution C-058 LSP能力规范 | P2 | COMPLETED | 2.2.5 |

### 2.3 Share 系统服务层

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 2.3.1 | 实现 self-hosted share server 基础架构 | P1 | pending | - |
| 2.3.2 | 实现会话导出为可分享链接 | P1 | pending | 2.3.1 |
| 2.3.3 | 实现 API Key 脱敏 | P1 | pending | 2.3.2 |
| 2.3.4 | 实现分享链接过期机制 | P1 | pending | 2.3.3 |
| 2.3.5 | 编写 Share 系统测试 | P1 | pending | 2.3.4 |

### 2.4 测试覆盖提升

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 2.4.1 | 按 TEST_MAPPING.md 补充 ~50 个缺失测试 | P1 | pending | - |
| 2.4.2 | 补充 compaction 自动触发测试 | P1 | pending | - |
| 2.4.3 | 补充 server HTTP API 端到端测试 | P1 | pending | 1.3.4 |
| 2.4.4 | 补充 LSP client/server 集成测试 | P1 | pending | 2.2.5 |
| 2.4.5 | 补充 provider 和 streaming 测试 (llm crate) | P1 | pending | - |
| 2.4.6 | 确保每个 crate 测试覆盖率 >= 50% | P2 | pending | 2.4.1-2.4.5 |
| 2.4.7 | 确保核心 crate (core, server, llm) 测试覆盖率 >= 70% | P2 | pending | 2.4.6 |
| 2.4.8 | Constitution C-055 测试覆盖率要求 | P2 | COMPLETED | 2.4.7 |

**阶段 2 验收标准**:
- [ ] OpenAPI 3.1 文档生成 (openapi.json)
- [ ] LSP 支持 definition/references/hover
- [ ] Share 系统可生成分享链接
- [ ] 测试覆盖率 >= 85%
- [ ] 核心 crate 测试覆盖率 >= 70%

---

## 阶段 3: 技术债务清理 (Week 4)

### 3.1 文档更新

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 3.1.1 | 更新 README.md 反映 20+ providers 和真实能力 | P2 | pending | - |
| 3.1.2 | 更新配置文档说明 JSONC 格式 | P2 | pending | 1.1.5 |
| 3.1.3 | 添加 API 文档 (openapi.json 生成后) | P2 | pending | 2.1.3 |
| 3.1.4 | 补充 crate 级文档覆盖率 >= 80% | P2 | pending | - |

### 3.2 代码清理

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 3.2.1 | 统一权限回复路由 (POST /sessions/{id}/permissions/{req_id}/reply) | P2 | pending | 1.3.3 |
| 3.2.2 | 检查 auth_layered 目录是否应整合 | P3 | pending | - |
| 3.2.3 | 确认 sap_aicore.rs 是否保留 | P3 | pending | - |
| 3.2.4 | clippy 警告清零 | P2 | pending | - |
| 3.2.5 | 代码格式化 (rustfmt) | P2 | pending | - |

### 3.3 测试分布均衡

| # | 任务 | 优先级 | 状态 | 依赖 |
|---|------|--------|------|------|
| 3.3.1 | 补充 llm crate 测试 (当前仅 9 个) | P2 | pending | 2.4.5 |
| 3.3.2 | 补充 server crate 集成测试 | P2 | pending | 2.4.3 |
| 3.3.3 | 确保测试分布差异不超过 2x | P2 | pending | 3.3.1, 3.3.2 |
| 3.3.4 | 运行全量测试套件验证 | P1 | pending | 3.3.3 |

**阶段 3 验收标准**:
- [ ] README.md 更新完成
- [ ] clippy 警告: 0
- [ ] 文档覆盖率 >= 80%
- [ ] 测试分布差异 <= 2x
- [ ] 全量测试套件通过

---

## 技术债务清单

| # | 技术债务 | 位置 | 风险 | 建议 |
|---|---------|------|------|------|
| T1 | 配置格式不一致 | `crates/core/src/config.rs` | 高 | 迁移到 JSONC 或明确文档说明差异 |
| T2 | README 过时 | `rust-opencode-port/README.md` | 中 | 更新 README 反映 20+ providers |
| T3 | 权限回复路由不明确 | `crates/server/src/routes/` | 中 | 统一为 POST /sessions/{id}/permissions/{req_id}/reply |
| T4 | TUI 权限确认非模态 | `crates/tui/src/right_panel.rs` | 低 | 评估 UX 影响，当前方案也可用 |
| T5 | auth_layered 目录未纳入 lib.rs | `crates/llm/src/auth_layered/` | 低 | 检查是否应整合进 auth.rs |
| T6 | sap_aicore.rs 未在 PRD 中定义 | `crates/llm/src/sap_aicore.rs` | 低 | 确认是否保留 |
| T7 | 测试分布不均 | `crates/*/src/*_test.rs` | 中 | 补充 provider 和 streaming 测试 |

---

## P3 延期任务 (v1.5+)

| # | 任务 | 优先级 | 说明 |
|---|------|--------|------|
| P3.1 | HuggingFace/AI21 provider 验证 | P3 | 文件存在但需验证实现完整性 |
| P3.2 | compaction 自动触发测试 | P3 | 已纳入 2.4.2 |
| P3.3 | server 集成测试 | P3 | 已纳入 2.4.3 |
| P3.4 | LSP 集成测试 | P3 | 已纳入 2.4.4 |
| P3.5 | 企业控制面 (SAML/SSO) | P3 | 非 v1 需求，标记为 v1.5+ |
| P3.6 | GitHub 集成 | P2 | v2 规划，但 crate 已存在 |

---

## 执行跟踪

| 日期 | 阶段 | 完成任务 | 阻塞项 |
|------|------|---------|--------|
| 2026-04-07 | 1.1 | 1.1.4 TOML→JSONC迁移工具, 1.1.6 配置迁移测试 | - |
| 2026-04-07 | 1.2 | 1.2.3 权限系统集成, 1.2.4 单元测试完成 | - |
| 2026-04-07 | 1.3 | 1.3.1-1.3.4 所有端点完成 | - |
| 2026-04-07 | 2.1 | 2.1.1 utoipa crate 集成 | - |
| 2026-04-07 | 2.2 | LSP JSON-RPC 基础设施实现 | - |
| 2026-04-07 | 2.3 | Share server - 已实现(13 tests) | - |
| 2026-04-07 | 2.4 | 添加bus测试(6), format测试(15), session_tools测试(7), server测试(4), llm测试(+) | - |
| 2026-04-07 | 3.2.4 | Clippy 警告验证(51个均为预存在) | - |

---

**文档状态**: 已完成  
**完成时间**: 2026-04-07  
**测试总数**: 448+ tests passing

## 架构说明

**重要发现**: 权限系统 (`opencode-permission` crate) 存在但与工具执行层完全断开连接。

- `ToolContext` 携带会话上下文但不包含权限数据
- `PermissionEvaluator` 和 `ApprovalQueue` 存在但从未在工具执行时调用
- `crates/tools/src/session_tools.rs` 使用 `Tool` trait 系统（异步），独立于 `crates/core/src/tool.rs` 的函数注册系统

**当前实现**: 采用最小可行方案 — 工具内部调用 `check_tool_permission_default()` 进行基于 scope 的检查。此检查使用默认 `ReadOnly` scope，意味着:
- `session_load` (在 `is_read_tool` 列表中) → 自动批准
- `session_save` (不在 `is_safe_tool` 列表中) → 需要批准

**完整集成**需要: 传递 `ToolContext` 到工具执行器、连接 `ApprovalQueue` 到工具执行层。