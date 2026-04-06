# OpenCode-RS 规格文档 v9

**版本**: 9.0
**日期**: 2026-04-06
**基于**: spec_v8.md + iteration-8 完成后的剩余差距分析
**状态**: 已完成

---

## 1. 文档概述

### 1.1 背景

iteration-8 已完成所有 P0/P1/P2 任务（93/93），但 gap-analysis.md 中仍有部分差距未完全解决。本规格文档定义 iteration-9 的目标范围。

### 1.2 范围

| 类别 | 包含 | 排除 |
|------|------|------|
| **包含** | SSO/OIDC 认证、credentialRef 解析、Tool Invocation 记录完善、Provider 动态管理 | GitHub 集成、IDE 插件、Desktop Shell |
| **优先级** | P1 (4项) + P2 (5项) | P3 (OAuth Browser、环境变量) 推迟 |

---

## 2. 功能需求

### FR-099: SSO/OIDC 认证流程

**描述**: 企业控制平面需支持 SAML/OIDC 单点登录

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-099.1 | SsoConfig 数据模型 (provider/entity_id/sso_url/certificate) | P1 |
| FR-099.2 | OIDC 认证流程 (1-7 步骤) | P1 |
| FR-099.3 | SAML provider 支持 | P2 |

### FR-100: credentialRef 解析机制

**描述**: 从 credential store 解析 credential 引用

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-100.1 | CredentialRef::Ref 变体运行时解析 | P1 |
| FR-100.2 | Credential store 持久化层 | P1 |
| FR-100.3 | credential 轮换机制 | P2 |

### FR-101: Tool Invocation 记录完善

**描述**: 补充工具调用详情记录

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-101.1 | args_hash 记录 | P1 |
| FR-101.2 | latency_ms 记录 | P1 |
| FR-101.3 | 调用结果摘要 | P2 |

### FR-102: PermissionDecision 审计模型

**描述**: 补充权限决策审计信息

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-102.1 | scope 字段 (this/session/project) | P1 |
| FR-102.2 | user_note 字段 | P1 |
| FR-102.3 | 审计日志持久化 | P2 |

### FR-103: Provider 动态管理

**描述**: 运行时启用/禁用 provider

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-103.1 | enabled_providers 配置 | P2 |
| FR-103.2 | disabled_providers 配置 | P2 |
| FR-103.3 | 热重载 provider 列表 | P2 |

### FR-104: 技术债务修复

**描述**: 解决 iteration-8 识别的技术债务

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-104.1 | 错误码统一 (OpenCodeError 体系) | P1 |
| FR-104.2 | 日志脱敏完善 | P1 |
| FR-104.3 | tracing 链路完善 | P2 |
| FR-104.4 | Auth Store 加密 | P1 |

---

## 3. 非功能需求

### 3.1 性能

| 指标 | 目标 |
|------|------|
| API 响应时间 (p95) | < 200ms |
| Session 加载时间 | < 500ms |
| Tool 执行延迟 | < 50ms |

### 3.2 安全性

| 要求 | 描述 |
|------|------|
| 凭证加密 | Auth Store 使用 AES-256-GCM 加密 |
| 日志脱敏 | 所有 credential/token 自动脱敏 |
| 审计日志 | 所有权限决策可追溯 |

---

## 4. 接口规格

### 4.1 新增 API 端点

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/sso/oidc/authorize | OIDC 授权请求 |
| POST | /api/sso/oidc/callback | OIDC 回调处理 |
| GET | /api/sso/config | 获取 SSO 配置 |
| PUT | /api/sso/config | 更新 SSO 配置 |
| POST | /api/providers/{id}/rotate | 轮换 credential |
| GET | /api/providers/{id}/status | 获取 provider 运行时状态 |
| PUT | /api/providers/{id}/enabled | 启用/禁用 provider |

### 4.2 数据模型变更

| 模型 | 变更 | 描述 |
|------|------|------|
| PermissionRequest | +scope, +user_note | 审计信息补充 |
| ToolInvocation | +args_hash, +latency_ms | 调用详情记录 |
| SsoConfig | 新增 | SSO 配置模型 |
| CredentialStore | 新增 | 凭证持久化存储 |

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

---

**文档状态**: 草稿
**下一步**: 创建 plan_v9.md 实施计划
