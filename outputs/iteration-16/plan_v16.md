# OpenCode-RS 实施计划 v16 — 差距修复与完整性补全

**版本**: 16.0  
**日期**: 2026-04-07  
**基于**: gap-analysis.md + constitution_updates.md (v2.1) + plan_v15.md  
**状态**: 已发布  

---

## 1. 实施策略

### 1.1 优先级分层

| 阶段 | 优先级 | 需求范围 | 目标 |
|------|--------|----------|------|
| **阶段 1** | P1 关键修复 | 配置格式迁移/session工具/health端点/abort端点 | 消除阻塞 v1 发布的差距 |
| **阶段 2** | P2 完整性补全 | OpenAPI文档/LSP能力/Share服务/测试覆盖 | 达到 95%+ 完整性 |
| **阶段 3** | 技术债务清理 | README更新/权限路由统一/测试分布/文档完善 | 代码质量达标 |

### 1.2 实施原则

1. **P1 优先**: 消除所有 P1 差距后再处理 P2
2. **增量交付**: 每个阶段结束时进行 clippy + 编译验证
3. **最小侵入**: 配置迁移等改动最小化影响现有代码
4. **复用现有**: 最大化利用已有 storage crate、server 路由结构

---

## 2. 当前实现状态审计

### 2.1 整体完成度

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

### 2.2 关键差距 (来自 gap-analysis.md)

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

### 2.3 技术债务

| 技术债务 | 位置 | 风险 | 建议 |
|---------|------|------|------|
| **配置格式不一致** | `crates/core/src/config.rs` | 高 | 迁移到 JSONC 或明确文档说明差异 |
| **README 过时** | `rust-opencode-port/README.md` | 中 | 更新 README 反映 20+ providers |
| **权限回复路由不明确** | `crates/server/src/routes/` | 中 | 统一为 POST /sessions/{id}/permissions/{req_id}/reply |
| **TUI 权限确认非模态** | `crates/tui/src/right_panel.rs` | 低 | 评估 UX 影响，当前方案也可用 |
| **auth_layered 目录未纳入 lib.rs** | `crates/llm/src/auth_layered/` | 低 | 检查是否应整合进 auth.rs |
| **sap_aicore.rs 未在 PRD 中定义** | `crates/llm/src/sap_aicore.rs` | 低 | 确认是否保留 |
| **测试分布不均** | `crates/*/src/*_test.rs` | 中 | 补充 provider 和 streaming 测试 |

### 2.4 与 v15 计划的关系

- v15 计划 (TUI 完整实现) — 已 100% 完成 ✅
- v16 计划聚焦于 **差距修复**，不重复 v15 已完成的工作
- v15 中未完成的 TUI 相关任务（如 SQLite 会话、命令面板等）如已实现则标记为完成

---

## 3. 实施计划

### 阶段 1: P1 关键修复 (Week 1)

**目标**: 消除所有阻塞 v1 发布的 P1 差距

#### 1.1 配置格式迁移 (TOML → JSONC)

- [ ] 分析现有 TOML 配置结构，映射到 JSONC 格式
- [ ] 实现 JSONC 解析器 (使用 `jsonc-parser` 或 `serde_json` + 注释剥离)
- [ ] 实现多层配置合并 (全局 ~/.config/opencode/ + 项目 .opencode/)
- [ ] 实现 TOML → JSONC 自动迁移工具
- [ ] 更新 config.rs 配置加载逻辑
- [ ] 编写配置格式迁移测试
- [ ] 更新 Constitution C-056 合规性

#### 1.2 session_load / session_save 工具

- [ ] 实现 session_load 工具 (从 storage crate 加载会话)
- [ ] 实现 session_save 工具 (保存会话到 storage crate)
- [ ] 集成权限系统 (allow/ask/deny 检查)
- [ ] 编写工具单元测试
- [ ] 更新 Constitution C-024 合规性

#### 1.3 Server API 端点补全

- [ ] 实现 GET /health 健康检查端点
- [ ] 实现 POST /sessions/{id}/abort 中止端点
- [ ] 确认 POST /sessions/{id}/permissions/{req_id}/reply 权限回复端点
- [ ] 编写端点集成测试
- [ ] 更新 Constitution C-057 合规性

### 阶段 2: P2 完整性补全 (Week 2-3)

**目标**: 达到 95%+ 完整性，消除 P2 差距

#### 2.1 OpenAPI 文档生成

- [ ] 集成 utoipa crate 到 server
- [ ] 为所有端点添加 #[utoipa::path] 注解
- [ ] 生成 openapi.json 构建产物
- [ ] 添加 Swagger UI 路由 (可选)
- [ ] 集成 openapi-generator SDK 生成钩子

#### 2.2 LSP 能力补全

- [ ] 实现 textDocument/definition
- [ ] 实现 textDocument/references
- [ ] 实现 textDocument/hover
- [ ] 实现 textDocument/completion (可选)
- [ ] 编写 LSP 集成测试
- [ ] 更新 Constitution C-058 合规性

#### 2.3 Share 系统服务层

- [ ] 实现 self-hosted share server 基础架构
- [ ] 实现会话导出为可分享链接
- [ ] 实现 API Key 脱敏
- [ ] 实现分享链接过期机制
- [ ] 编写 Share 系统测试

#### 2.4 测试覆盖提升

- [ ] 按 TEST_MAPPING.md 补充 ~50 个缺失测试
- [ ] 补充 compaction 自动触发测试
- [ ] 补充 server HTTP API 端到端测试
- [ ] 补充 LSP client/server 集成测试
- [ ] 补充 provider 和 streaming 测试 (llm crate)
- [ ] 确保每个 crate 测试覆盖率 >= 50%
- [ ] 确保核心 crate (core, server, llm) 测试覆盖率 >= 70%
- [ ] 更新 Constitution C-055 合规性

### 阶段 3: 技术债务清理 (Week 4)

**目标**: 代码质量达标，文档完善

#### 3.1 文档更新

- [ ] 更新 README.md 反映 20+ providers 和真实能力
- [ ] 更新配置文档说明 JSONC 格式
- [ ] 添加 API 文档 (openapi.json 生成后)
- [ ] 补充 crate 级文档覆盖率 >= 80%

#### 3.2 代码清理

- [ ] 统一权限回复路由 (POST /sessions/{id}/permissions/{req_id}/reply)
- [ ] 检查 auth_layered 目录是否应整合
- [ ] 确认 sap_aicore.rs 是否保留
- [ ] clippy 警告清零
- [ ] 代码格式化 (rustfmt)

#### 3.3 测试分布均衡

- [ ] 补充 llm crate 测试 (当前仅 9 个)
- [ ] 补充 server crate 集成测试
- [ ] 确保测试分布差异不超过 2x
- [ ] 运行全量测试套件验证

---

## 4. 技术决策

### 4.1 配置格式迁移

**决策**: 从 TOML 迁移到 JSONC

**理由**:
- PRD 明确要求 JSONC 格式
- Constitution C-056 强制要求
- 与 OpenCode 生态兼容
- 用户配置可迁移

**实施策略**:
1. 保留 TOML 解析作为向后兼容 (deprecated 警告)
2. 优先加载 JSONC，降级到 TOML
3. 提供 `migrate-config` 命令自动转换
4. 迁移后删除旧 TOML 文件

### 4.2 OpenAPI 文档

**决策**: 使用 utoipa 生成 OpenAPI 3.1 文档

**理由**:
- Rust 生态最成熟的 OpenAPI 生成库
- 与 axum 集成良好
- 编译时生成，无运行时开销
- 支持 SDK 自动生成

### 4.3 session_load/session_save

**决策**: 复用 storage crate 实现

**理由**:
- storage crate 已有 session 模型和数据库操作
- 避免重复实现
- 保持数据一致性

---

## 5. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 配置迁移破坏现有用户配置 | 高 | 保留 TOML 向后兼容，提供迁移工具 |
| session_load/save 与现有 session 管理冲突 | 中 | 复用 storage crate，最小化新代码 |
| LSP 能力补全影响性能 | 中 | 异步处理 LSP 请求，缓存结果 |
| 测试补充工作量超预期 | 中 | 按优先级分批，P0/P1 路径优先 |

---

## 6. 验收标准

### 6.1 阶段 1 验收

- [ ] 配置文件支持 JSONC 格式，TOML 自动迁移
- [ ] session_load/session_save 工具可正常使用
- [ ] GET /health 返回 {status: "ok", version: "x.y.z"}
- [ ] POST /sessions/{id}/abort 可中止运行中会话
- [ ] 权限回复端点确认存在且工作正常

### 6.2 阶段 2 验收

- [ ] OpenAPI 3.1 文档生成 (openapi.json)
- [ ] LSP 支持 definition/references/hover
- [ ] Share 系统可生成分享链接
- [ ] 测试覆盖率 >= 85%
- [ ] 核心 crate 测试覆盖率 >= 70%

### 6.3 阶段 3 验收

- [ ] README.md 更新完成
- [ ] clippy 警告: 0
- [ ] 文档覆盖率 >= 80%
- [ ] 测试分布差异 <= 2x
- [ ] 全量测试套件通过

---

## 7. 追溯链

```
gap-analysis.md (P1/P2/P3 差距)
constitution_updates.md (v2.1: C-056~C-058, 扩展 C-024/C-055)
    │
    ├── 4 项 P1 差距 (配置/session工具/health/abort)
    ├── 5 项 P2 差距 (OpenAPI/LSP/Share/测试/GitHub)
    ├── 7 项 P3 差距 (provider验证/集成测试/企业控制面)
    ├── 7 项技术债务
    │
    ▼
plan_v16.md (本文档) — 3 阶段修复计划
    │
    ├── 阶段 1: P1 关键修复 (Week 1)
    ├── 阶段 2: P2 完整性补全 (Week 2-3)
    └── 阶段 3: 技术债务清理 (Week 4)
    │
    ▼
tasks_v16.md — 原子任务清单
```

---

**文档状态**: 已发布  
**下一步**: 创建 tasks_v16.md 原子任务清单
