# OpenCode-RS 实施计划 v8

**版本**: 8.0  
**日期**: 2026-04-06  
**基于**: spec_v8.md + gap-analysis.md  
**状态**: 已完成

---

## 1. 计划概述

### 1.1 背景

本计划基于 iteration-8 差距分析编写，旨在解决以下核心问题：

- **P0 (3项)**: Provider 管理 API 缺失 + Permission 审批 API 缺失 + Session State 状态机不完整
- **P1 (5项)**: Plan Agent 写限制 + Artifact API + Share 本地导出 + Enterprise 控制平面
- **P2 (3项)**: Web UI 完整实现 + 工具测试覆盖 + Context Engine 压缩阈值

### 1.2 目标

1. **立即目标**: 解决 3 个 P0 阻断性问题
2. **短期目标**: 完成 5 个 P1 核心功能
3. **中期目标**: 实现 3 个 P2 增强功能
4. **验收标准**: 满足 spec_v8.md 的所有 FR 要求

---

## 2. 实施阶段

### Phase 1: P0 阻断性问题 (最高优先级)

**目标**: 解决 3 个 P0 阻断性问题，API 层基础设施补全

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P0-1 | Provider 管理 API (credentials/test/revoke) | server | FR-089 | - | 2d |
| P0-2 | Permission 审批 API | server | FR-090 | - | 2d |
| P0-3 | Session State 状态机完整实现 | core | FR-091 | - | 3d |

#### P0-1: Provider 管理 API

**详细任务**:
1. 实现 GET /providers - 列出所有 provider、连接状态、默认模型、认证策略（脱敏）
2. 实现 POST /providers/{id}/credentials - 设置或更新 credential
3. 实现 POST /providers/{id}/test - 连通性与权限测试
4. 实现 DELETE /providers/{id}/credentials - 撤销当前绑定

**验收标准**:
- [ ] GET /providers 返回脱敏信息（不返回明文 credential）
- [ ] POST /credentials 正确保存
- [ ] POST /test 测试连接但不改变状态
- [ ] DELETE /credentials 使相关会话进入需重验状态

#### P0-2: Permission 审批 API

**详细任务**:
1. 实现 GET /permissions - 列出待审批的权限请求
2. 实现 GET /permissions/{req_id} - 获取特定请求详情
3. 实现 POST /permissions/{req_id}/reply - 批准/拒绝权限请求

**验收标准**:
- [ ] GET /permissions 返回 pending 列表
- [ ] POST /reply 可批准/拒绝请求
- [ ] decision 状态正确转换（pending → approved/denied）

#### P0-3: Session State 状态机完整实现

**详细任务**:
1. 实现完整的 12 状态枚举（Idle, Thinking, AwaitingPermission, ExecutingTool, Streaming, ApplyingChanges, Verifying, Summarizing, Aborted, Error, Completed, Paused）
2. 实现状态转换规则与 can_transition_to 验证
3. 实现事件触发与状态绑定
4. 验证状态转换正确性

**验收标准**:
- [ ] 12 种状态完整实现
- [ ] 状态转换规则正确（14 条规则）
- [ ] 事件触发正确（12 个事件）

---

### Phase 2: P1 核心功能

**目标**: 完成 5 个 P1 核心功能

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P1-1 | Plan Agent 工具写限制强制执行 | agent | FR-092 | - | 1d |
| P1-2 | Artifact API (diff/snapshots/revert) | server | FR-093 | - | 2d |
| P1-3 | Share 本地导出 (JSON/Markdown) | core | FR-094 | - | 2d |
| P1-4 | Enterprise 控制平面 | control-plane | FR-095 | - | 5d |
| P1-5 | Plugin WASM 运行时 (补充) | plugin | FR-072 | - | 2d |

#### P1-1: Plan Agent 工具写限制

**详细任务**:
1. 定义 PLAN_AGENT_DENIED_TOOLS 黑名单
2. 实现 validate_tool_for_agent 函数
3. 实现 ToolDeniedError 错误类型
4. TUI 反馈（禁用写工具按钮）

**验收标准**:
- [ ] edit/write/patch/move/delete/bash 被 Plan Agent 拒绝
- [ ] 错误消息清晰
- [ ] TUI 反馈正确

#### P1-2: Artifact API

**详细任务**:
1. 实现 GET /sessions/{id}/diff - 获取当前 diff
2. 实现 GET /sessions/{id}/snapshots - 获取快照列表
3. 实现 GET /sessions/{id}/snapshots/{snap_id} - 获取特定快照
4. 实现 POST /sessions/{id}/revert - 回滚到指定快照

**验收标准**:
- [ ] diff 正确生成 unified format
- [ ] snapshots 正确创建与列出
- [ ] revert 正确执行（soft/hard 模式）

#### P1-3: Share 本地导出

**详细任务**:
1. 实现 GET /export/sessions/{id} - JSON 导出
2. 实现 GET /export/sessions/{id}/transcript - Markdown 导出
3. 实现 GET /export/sessions/{id}/patch - Patch Bundle 导出
4. 实现 CLI 命令（opencode export）

**验收标准**:
- [ ] JSON 导出完整
- [ ] Markdown 格式正确
- [ ] 敏感信息脱敏

#### P1-4: Enterprise 控制平面

**详细任务**:
1. 创建 opencode-control-plane crate
2. 实现 account 模块（Account, User, Team）
3. 实现 enterprise 模块（Enterprise, Policy）
4. 实现 sso 模块（SAML/OIDC provider）
5. 实现 central_config 模块（Remote Config fetcher）

**验收标准**:
- [ ] Account/Enterprise 数据模型完整
- [ ] SSO 流程正确（OIDC 为例）
- [ ] Central Config 可用

#### P1-5: Plugin WASM 运行时

**详细任务**:
1. 集成 WASM 执行引擎（wasmer/wasmtime）
2. 实现 Plugin 加载/卸载/隔离
3. 实现事件 hooks

**验收标准**:
- [ ] WASM 插件可加载执行
- [ ] 插件隔离正确

---

### Phase 3: P2 完善性

**目标**: 完成 3 个 P2 增强功能

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P2-1 | Web UI 完整实现 | server | FR-096 | - | 3d |
| P2-2 | 工具单元测试覆盖 | tools | FR-097 | - | 3d |
| P2-3 | Context Engine 压缩阈值 (85%/92%/95%) | core | FR-098 | - | 2d |

#### P2-1: Web UI 完整实现

**详细任务**:
1. 完善页面路由（/, /sessions, /session/:id, /settings, /admin）
2. 实现 UI 组件（MessageList, InputArea, DiffViewer, FileTree, ProviderManager, PermissionQueue）
3. 实时通信（SSE/WebSocket）

**验收标准**:
- [ ] 页面路由完整
- [ ] UI 组件可用
- [ ] 实时通信正常

#### P2-2: 工具单元测试覆盖

**详细任务**:
1. 实现 read/write/edit/glob/grep 单元测试
2. 实现 bash/patch/move/delete 单元测试
3. 实现测试辅助工具（create_temp_project, mock_tool_context）

**验收标准**:
- [ ] 核心工具测试用例 ≥ 70% 覆盖率
- [ ] 关键路径 100% 覆盖

#### P2-3: Context Engine 压缩阈值

**详细任务**:
1. 实现 70% 预警阈值
2. 实现 85% 自动压缩阈值
3. 实现 92% 触发 summarize 阈值
4. 实现 95% 强制新 session 阈值

**验收标准**:
- [ ] 4 个阈值正确触发
- [ ] 压缩策略正确执行

---

## 3. 里程碑

| 里程碑 | 日期 | 完成条件 |
|--------|------|----------|
| M1: P0 完成 | T+7d | ✅ P0-1 + P0-2 + P0-3 验收通过 |
| M2: P1 完成 | T+17d | ✅ P1-1 ~ P1-5 验收通过 |
| M3: P2 完成 | T+25d | ✅ P2-1 ~ P2-3 验收通过 |
| M4: Alpha Release | T+30d | ✅ 全部 P0/P1/P2 验收通过 |

---

## 4. 资源分配

### 4.1 人力投入

| 角色 | 投入比例 |
|------|----------|
| 架构师 | 15% (P0 架构设计) |
| 后端工程师 | 65% (server/core/agent) |
| 前端工程师 | 15% (web ui) |
| 测试工程师 | 5% (关键路径) |

### 4.2 技术依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| rust | 1.75+ | 编译环境 |
| tokio | 1.x | 异步 runtime |
| wasmer | 2.x | WASM 执行引擎 |
| serde | 1.x | 序列化 |

---

## 5. 风险与缓解

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| P0-3 状态机复杂度 | 高 | 分阶段实现，每状态独立验证 |
| P1-4 Enterprise 模块全新 | 高 | 参考现有模块模式 |
| P2-2 测试覆盖工作量大 | 中 | 优先核心工具测试 |

---

## 6. 验收标准

### 6.1 P0 验收

- [ ] Provider 管理 API 完整（credentials/test/revoke）
- [ ] Permission 审批 API 完整（GET /permissions, POST /reply）
- [ ] Session State 12 状态完整实现

### 6.2 P1 验收

- [ ] Plan Agent 写限制正确执行
- [ ] Artifact API 完整（diff/snapshots/revert）
- [ ] Share 本地导出可用（JSON/Markdown）
- [ ] Enterprise 控制平面基础可用
- [ ] Plugin WASM 运行时可用

### 6.3 P2 验收

- [ ] Web UI 完整实现
- [ ] 工具测试覆盖 ≥ 70%
- [ ] Context Engine 压缩阈值 4 级正确

---

## 7. 追溯链

```
gap-analysis.md (iteration-8)
    │
    ├── P0: 3 项 → FR-089, FR-090, FR-091
    ├── P1: 5 项 → FR-092, FR-093, FR-094, FR-095, FR-072
    └── P2: 3 项 → FR-096, FR-097, FR-098
            │
            ▼
spec_v8.md
    │
    └── FR-089 ~ FR-098 (12 项)
            │
            ▼
本计划 (plan_v8.md)
    │
    └── Phase 1-3 实施任务
```

---

**文档状态**: 草稿  
**下一步**: 创建 tasks_v8.md 并开始 P0 任务实现
