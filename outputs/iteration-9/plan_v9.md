# OpenCode-RS 实施计划 v9

**版本**: 9.0
**日期**: 2026-04-06
**基于**: spec_v9.md + iteration-8 完成后的剩余差距
**状态**: 已完成

---

## 1. 计划概述

### 1.1 背景

iteration-8 已完成 93/93 任务，但 gap-analysis.md 中仍有 9 项未完全解决的差距。本计划针对这些剩余差距。

### 1.2 目标

1. **P1 (4项)**: SSO/OIDC、credentialRef 解析、Tool Invocation 记录、技术债务修复
2. **P2 (5项)**: Permission 审计、Provider 动态管理、tracing 链路完善

---

## 2. 实施阶段

### Phase 1: P1 核心差距 (最高优先级)

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P1-1 | SSO/OIDC 认证流程 | auth | FR-099 | - | 3d |
| P1-2 | credentialRef 解析机制 | core/server | FR-100 | - | 2d |
| P1-3 | Tool Invocation 记录完善 | core | FR-101 | - | 1d |
| P1-4 | 技术债务修复 (错误码/脱敏/加密) | 多模块 | FR-104 | - | 3d |

### Phase 2: P2 增强功能

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P2-1 | Permission 审计模型完善 | permission | FR-102 | - | 1d |
| P2-2 | Provider 动态管理 | core/server | FR-103 | P1-2 | 2d |
| P2-3 | tracing 链路完善 | 多模块 | FR-104.3 | - | 1d |

---

## 3. 详细实施计划

### P1-1: SSO/OIDC 认证流程

**模块**: auth + control-plane
**预计工时**: 3d

1. 实现 SsoConfig 数据模型 (provider/entity_id/sso_url/certificate)
2. 实现 OIDC 授权请求端点 (POST /api/sso/oidc/authorize)
3. 实现 OIDC 回调处理 (POST /api/sso/oidc/callback)
4. 实现 SSO 配置 CRUD (GET/PUT /api/sso/config)
5. 集成 JWT token 验证
6. 单元测试

### P1-2: credentialRef 解析机制

**模块**: core + server
**预计工时**: 2d

1. 实现 CredentialStore 持久化层
2. 实现 CredentialRef::Ref 运行时解析
3. 实现 credential 轮换端点 (POST /providers/{id}/rotate)
4. 集成到 provider 认证流程
5. 单元测试

### P1-3: Tool Invocation 记录完善

**模块**: core
**预计工时**: 1d

1. ToolInvocation 结构体添加 args_hash 字段
2. ToolInvocation 结构体添加 latency_ms 字段
3. 工具执行时自动记录
4. 单元测试

### P1-4: 技术债务修复

**模块**: 多模块
**预计工时**: 3d

1. 统一错误码体系 (OpenCodeError 枚举)
2. 完善日志脱敏 (所有 credential/token)
3. Auth Store AES-256-GCM 加密
4. 集成测试

### P2-1: Permission 审计模型完善

**模块**: permission
**预计工时**: 1d

1. PermissionRequest 添加 scope 字段
2. PermissionRequest 添加 user_note 字段
3. 审计日志持久化
4. 单元测试

### P2-2: Provider 动态管理

**模块**: core + server
**预计工时**: 2d

1. 实现 enabled_providers 配置
2. 实现 disabled_providers 配置
3. 实现 provider 状态端点 (GET /providers/{id}/status)
4. 实现 provider 启用/禁用端点 (PUT /providers/{id}/enabled)
5. 热重载支持
6. 单元测试

### P2-3: tracing 链路完善

**模块**: 多模块
**预计工时**: 1d

1. 所有模块添加 tracing span
2. 关键路径添加 trace_id
3. 性能指标埋点
4. 集成测试

---

## 4. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| OIDC 集成复杂度 | 延期 | 使用成熟 crate (openidconnect) |
| credential 加密性能 | 延迟增加 | 异步加密 + 缓存 |
| 错误码重构影响面 | 破坏性变更 | 向后兼容过渡 |

---

## 5. 验收标准

- [ ] OIDC 认证流程完整 (7 步骤)
- [ ] credentialRef 引用可正常解析
- [ ] Tool Invocation 记录包含 args_hash 和 latency_ms
- [ ] PermissionDecision 包含 scope 和 user_note
- [ ] Provider 可动态启用/禁用
- [ ] 错误码体系统一
- [ ] 日志脱敏覆盖率 100%
- [ ] Auth Store 加密启用
- [ ] cargo test 全部通过

---

**文档状态**: 草稿
**下一步**: 创建 tasks_v9.md 任务清单
