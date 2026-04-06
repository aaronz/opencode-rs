# OpenCode-RS 任务清单 v8

**版本**: 8.0  
**日期**: 2026-04-06  
**基于**: plan_v8.md  
**状态**: 已完成

---

## 任务总览

| Phase | 优先级 | 任务数 | 预计工时 |
|-------|--------|--------|----------|
| Phase 1 | P0 | 3 | 7d |
| Phase 2 | P1 | 5 | 12d |
| Phase 3 | P2 | 3 | 8d |
| **总计** | - | **11** | **27d** |

---

## Phase 1: P0 阻断性问题

### P0-1: Provider 管理 API

**FR**: FR-089  
**模块**: server  
**依赖**: -  
**预计工时**: 2d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P0-1.1 | 实现 GET /providers 端点 | 返回所有 provider 脱敏信息 | 已完成 |
| P0-1.2 | 实现 POST /providers/{id}/credentials 端点 | 设置或更新 credential | 已完成 |
| P0-1.3 | 实现 POST /providers/{id}/test 端点 | 连通性与权限测试 | 已完成 |
| P0-1.4 | 实现 DELETE /providers/{id}/credentials 端点 | 撤销当前绑定 | 已完成 |
| P0-1.5 | 实现 ProviderInfo 数据模型 | id/name/protocol/baseUrl/auth/hasCredential | 已完成 |
| P0-1.6 | 实现 SetCredentialRequest 请求模型 | 支持 credentialRef | 已完成 |
| P0-1.7 | 实现 TestCredentialResponse 响应模型 | success/latency_ms/error | 已完成 |
| P0-1.8 | 安全验证：明文 credential 不返回 | 脱敏验证通过 | 已完成 |
| P0-1.9 | API 集成测试 | 各端点功能正常 | 已完成 |

### P0-2: Permission 审批 API

**FR**: FR-090  
**模块**: server  
**依赖**: -  
**预计工时**: 2d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P0-2.1 | 实现 GET /permissions 端点 | 列出待审批的权限请求 | 已完成 |
| P0-2.2 | 实现 GET /permissions/{req_id} 端点 | 获取特定请求详情 | 已完成 |
| P0-2.3 | 实现 POST /permissions/{req_id}/reply 端点 | 批准/拒绝权限请求 | 已完成 |
| P0-2.4 | 实现 PermissionRequest 数据模型 | id/session_id/tool_name/args_summary/risk_level/decision | 已完成 |
| P0-2.5 | 实现 ReplyRequest 请求模型 | decision/scope/note | 已完成 |
| P0-2.6 | 实现状态转换逻辑 (pending→approved/denied) | 状态正确转换 | 已完成 |
| P0-2.7 | API 集成测试 | 各端点功能正常 | 已完成 |

### P0-3: Session State 状态机完整实现

**FR**: FR-091  
**模块**: core  
**依赖**: -  
**预计工时**: 3d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P0-3.1 | 实现 12 状态枚举 | Idle/Thinking/AwaitingPermission/ExecutingTool/Streaming/ApplyingChanges/Verifying/Summarizing/Aborted/Error/Completed/Paused | 已完成 |
| P0-3.2 | 实现 SessionState::can_transition_to 方法 | 14 条转换规则正确 | 已完成 |
| P0-3.3 | 实现状态转换规则验证 | 非法转换被拒绝 | 已完成 |
| P0-3.4 | 实现事件触发机制 (12 个事件) | 事件正确绑定状态 | 已完成 |
| P0-3.5 | 实现状态与事件映射 | 状态-事件对应正确 | 已完成 |
| P0-3.6 | 实现错误状态处理 | error → idle 转换正确 | 已完成 |
| P0-3.7 | 状态机单元测试 | 12 状态 + 转换规则测试通过 | 已完成 |
| P0-3.8 | 状态机集成测试 | 完整流程测试通过 | 已完成 |

---

## Phase 2: P1 核心功能

### P1-1: Plan Agent 工具写限制强制执行

**FR**: FR-092  
**模块**: agent  
**依赖**: -  
**预计工时**: 1d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P1-1.1 | 定义 PLAN_AGENT_DENIED_TOOLS 黑名单 | edit/write/patch/move/delete/bash/mcp_tool | 已完成 |
| P1-1.2 | 实现 validate_tool_for_agent 函数 | 写工具被拒绝 | 已完成 |
| P1-1.3 | 实现 ToolDeniedError 错误类型 | 错误消息清晰 | 已完成 |
| P1-1.4 | TUI 反馈：禁用写工具按钮 | 按钮禁用/警告显示 | 已完成 |
| P1-1.5 | 集成测试 | Plan Agent 模式下写工具被正确拒绝 | 已完成 |

### P1-2: Artifact API

**FR**: FR-093  
**模块**: server  
**依赖**: -  
**预计工时**: 2d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P1-2.1 | 实现 GET /sessions/{id}/diff 端点 | 返回 unified diff | 已完成 |
| P1-2.2 | 实现 GET /sessions/{id}/snapshots 端点 | 返回快照列表 | 已完成 |
| P1-2.3 | 实现 GET /sessions/{id}/snapshots/{snap_id} 端点 | 返回特定快照 | 已完成 |
| P1-2.4 | 实现 POST /sessions/{id}/revert 端点 | 回滚到指定快照 | 已完成 |
| P1-2.5 | 实现 DiffResponse 数据模型 | session_id/diffs/generated_at | 已完成 |
| P1-2.6 | 实现 FileDiff 数据模型 | file/old_content/new_content/unified_diff/hunks | 已完成 |
| P1-2.7 | 实现 Snapshot 数据模型 | id/based_on_message_id/description/patch_path/created_at | 已完成 |
| P1-2.8 | 实现 RevertRequest 请求模型 | snapshot_id/mode/confirm | 已完成 |
| P1-2.9 | API 集成测试 | 各端点功能正常 | 已完成 |

### P1-3: Share 本地导出

**FR**: FR-094  
**模块**: core  
**依赖**: -  
**预计工时**: 2d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P1-3.1 | 实现 GET /export/sessions/{id} 端点 | JSON 导出完整 | 已完成 |
| P1-3.2 | 实现 GET /export/sessions/{id}/transcript 端点 | Markdown 导出正确 | 已完成 |
| P1-3.3 | 实现 GET /export/sessions/{id}/patch 端点 | Patch Bundle 导出 | 已完成 |
| P1-3.4 | 实现 CLI 命令 opencode export | --format json/markdown/patch | 已完成 |
| P1-3.5 | 实现敏感信息脱敏 | credential/环境变量被移除 | 已完成 |
| P1-3.6 | 实现导出数据模型 | Session/Messages/Artifacts 结构 | 已完成 |
| P1-3.7 | 集成测试 | 导出功能正常 | 已完成 |

### P1-4: Enterprise 控制平面

**FR**: FR-095  
**模块**: control-plane  
**依赖**: -  
**预计工时**: 5d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P1-4.1 | 创建 opencode-control-plane crate | Cargo.toml 正确配置 | 已完成 |
| P1-4.2 | 实现 account 模块 (Account/User/Team) | 数据模型完整 | 已完成 |
| P1-4.3 | 实现 enterprise 模块 (Enterprise/Policy) | 数据模型完整 | 已完成 |
| P1-4.4 | 实现 sso 模块 (SAML/OIDC provider) | SSO 提供商支持 | 已完成 |
| P1-4.5 | 实现 central_config 模块 (Remote Config fetcher) | 远程配置获取 | 已完成 |
| P1-4.6 | 实现 SsoConfig 数据模型 | provider/entity_id/sso_url/certificate | 已完成 |
| P1-4.7 | 实现 Policy 数据模型 | permission_profile/allowed_providers/mcp_restrictions | 已完成 |
| P1-4.8 | 实现 OIDC 认证流程 | 1-7 步骤正确 | 已完成 |
| P1-4.9 | 实现 Remote Config fetcher | HTTP GET + JSON Schema 验证 | 已完成 |
| P1-4.10 | 单元测试 | 各模块测试通过 | 已完成 |

### P1-5: Plugin WASM 运行时

**FR**: FR-072  
**模块**: plugin  
**依赖**: -  
**预计工时**: 2d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P1-5.1 | 集成 WASM 执行引擎 (wasmer/wasmtime) | 引擎可用 | 已完成 |
| P1-5.2 | 实现 Plugin 加载功能 | 插件加载正常 | 已完成 |
| P1-5.3 | 实现 Plugin 卸载功能 | 插件卸载正常 | 已完成 |
| P1-5.4 | 实现插件隔离机制 | 隔离正确 | 已完成 |
| P1-5.5 | 实现事件 hooks | 事件正确触发 | 已完成 |
| P1-5.6 | WASM 插件加载测试 | 插件执行正常 | 已完成 |

---

## Phase 3: P2 完善性

### P2-1: Web UI 完整实现

**FR**: FR-096  
**模块**: server  
**依赖**: -  
**预计工时**: 3d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P2-1.1 | 实现页面路由 (/) | 引导页正确 | 已完成 |
| P2-1.2 | 实现页面路由 (/sessions) | 会话列表正确 | 已完成 |
| P2-1.3 | 实现页面路由 (/session/:id) | 会话详情正确 | 已完成 |
| P2-1.4 | 实现页面路由 (/settings) | 设置页正确 | 已完成 |
| P2-1.5 | 实现页面路由 (/admin) | 管理面板正确 | 已完成 |
| P2-1.6 | 实现 MessageList 组件 | 消息列表正常 | 已完成 |
| P2-1.7 | 实现 InputArea 组件 | 消息输入框正常 | 已完成 |
| P2-1.8 | 实现 DiffViewer 组件 | 文件变更展示正常 | 已完成 |
| P2-1.9 | 实现 FileTree 组件 | 项目文件树正常 | 已完成 |
| P2-1.10 | 实现 ProviderManager 组件 | Provider 配置正常 | 已完成 |
| P2-1.11 | 实现 PermissionQueue 组件 | 权限审批队列正常 | 已完成 |
| P2-1.12 | 实现 SSE/WebSocket 实时通信 | 实时更新正常 | 已完成 |
| P2-1.13 | UI 集成测试 | Web UI 功能正常 | 已完成 |

### P2-2: 工具单元测试覆盖

**FR**: FR-097  
**模块**: tools  
**依赖**: -  
**预计工时**: 3d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P2-2.1 | 实现 read 工具测试 (5+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.2 | 实现 write 工具测试 (5+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.3 | 实现 edit 工具测试 (8+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.4 | 实现 glob 工具测试 (4+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.5 | 实现 grep 工具测试 (5+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.6 | 实现 bash 工具测试 (4+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.7 | 实现 patch 工具测试 (5+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.8 | 实现 move 工具测试 (3+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.9 | 实现 delete 工具测试 (4+ 用例) | 覆盖率 ≥ 70% | 已完成 |
| P2-2.10 | 实现测试辅助工具 | create_temp_project/mock_tool_context | 已完成 |
| P2-2.11 | 覆盖率验证 | 语句 ≥ 70%, 分支 ≥ 60% | 已完成 |

### P2-3: Context Engine 压缩阈值

**FR**: FR-098  
**模块**: core  
**依赖**: -  
**预计工时**: 2d

| # | 子任务 | 验收标准 | 状态 |
|---|--------|----------|------|
| P2-3.1 | 实现 CompactionConfig 结构体 | warnAt/compactAt/summarizeAt/forceAt | 已完成 |
| P2-3.2 | 实现 70% 预警阈值 | 提醒用户 token 消耗 | 已完成 |
| P2-3.3 | 实现 85% 自动压缩阈值 | 自动压缩非关键上下文 | 已完成 |
| P2-3.4 | 实现 92% 触发 summarize 阈值 | 执行 session summarize | 已完成 |
| P2-3.5 | 实现 95% 强制新 session 阈值 | 暂停并提示用户 | 已完成 |
| P2-3.6 | 实现压缩策略 (RemoveToolCallFullContent/TrimOldMessages等) | 策略正确执行 | 已完成 |
| P2-3.7 | 实现配置选项 | JSON 配置正确加载 | 已完成 |
| P2-3.8 | 用户交互提示 | 70%/85%/92%/95% UI 提示正确 | 已完成 |
| P2-3.9 | 集成测试 | 阈值触发测试通过 | 已完成 |

---

## 任务状态汇总

| 状态 | 数量 |
|------|------|
| 待开始 | 0 |
| 进行中 | 0 |
| 已完成 | 93 |

---

## 追溯链

```
plan_v8.md (本计划)
    │
    ├── Phase 1: P0 (3 任务, 24 子任务)
    ├── Phase 2: P1 (5 任务, 33 子任务)
    └── Phase 3: P2 (3 任务, 36 子任务)
            │
            ▼
tasks_v8.md (本清单)
    │
    └── 93 个子任务 (已完成)
```

---

**文档状态**: 已完成  
**下一步**: 无 - 所有任务已完成
