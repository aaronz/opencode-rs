# OpenCode-RS 规格文档 v13

**版本**: 13.0
**日期**: 2026-04-06
**基于**: spec_v12.md + iteration-13 Constitution 审计发现的 P1/P2 差距
**状态**: 草稿

---

## 1. 文档概述

### 1.1 背景

spec_v12 定义了 FR-001 ~ FR-114 的完整需求集，覆盖集成测试框架、Provider 补全、Session 工具、代码质量与性能基准。

iteration-13 的 Constitution 审计 (v1.8 → v1.9) 发现 7 个 P1/P2 问题未被现有规格充分覆盖：
1. SSO/OIDC 企业认证流程完全无约束
2. CredentialRef 解析机制无定义
3. Tool Invocation 审计记录不完整
4. OpenCodeError 统一错误码体系缺失
5. 集成测试框架无 Constitution 约束 (已在 FR-110 定义)
6. PermissionDecision 审计字段缺失
7. Session Load/Save 工具无安全规范 (已在 FR-112 定义)

本规格文档在 v12 基础上新增/扩展需求，确保所有 P1/P2 差距有对应的 FR 定义。

### 1.2 范围

| 类别 | 包含 | 排除 |
|------|------|------|
| **包含** | SSO/OIDC 认证、CredentialRef 解析、Tool Invocation 审计、统一错误码、PermissionDecision 审计扩展 | GitHub 深度集成 (v1.5)、IDE 插件、Desktop Shell |
| **优先级** | P1 (SSO/OIDC + CredentialRef + 错误码 + 审计) | P2 (Provider 动态管理扩展) |

### 1.3 与 v12 的关系

v12 保留了 FR-001 ~ FR-114 的所有需求。

v13 新增需求：
- 1 项 P1 (SSO/OIDC 企业认证) → FR-115
- 1 项 P1 (CredentialRef 解析机制) → FR-116
- 1 项 P1 (Tool Invocation 审计完善) → FR-117
- 1 项 P1 (统一错误码体系) → FR-118
- 1 项 P2 (PermissionDecision 审计扩展) → FR-119
- 1 项 P2 (Provider 动态管理扩展) → FR-120

v13 扩展现有需求：
- FR-112 (Session Load/Save) 补充安全脱敏规范

---

## 2. 新增功能需求

### FR-115: SSO/OIDC 企业认证

**描述**: 实现企业级单点登录，支持 OIDC 和 SAML 2.0 协议

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-115.1 | SsoConfig 数据模型 (provider/entity_id/sso_url/certificate) | P1 |
| FR-115.2 | OIDC 授权端点 POST /api/sso/oidc/authorize | P1 |
| FR-115.3 | OIDC 回调处理 POST /api/sso/oidc/callback | P1 |
| FR-115.4 | ID Token 验证 (signature/audience/expiry via JWKS) | P1 |
| FR-115.5 | SAML AuthnRequest 生成与 Assertion 验证 | P1 |
| FR-115.6 | SAML 回调 POST /api/sso/saml/callback | P1 |
| FR-115.7 | SSO 配置 CRUD (GET/PUT /api/sso/config) | P1 |
| FR-115.8 | 用户映射 (email/sub → 本地用户) | P1 |
| FR-115.9 | 本地 JWT Session Token 签发 | P1 |
| FR-115.10 | CSRF 保护所有 SSO 端点 | P1 |
| FR-115.11 | SSO 登录失败审计日志 | P1 |
| FR-115.12 | 配置热重载无需重启 | P2 |

**对应 Constitution**: C-041

### FR-116: CredentialRef 解析机制

**描述**: 实现凭证引用的运行时解析和 CredentialStore 持久化层

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-116.1 | CredentialRef::Literal(value) 内联凭证 | P1 |
| FR-116.2 | CredentialRef::Ref(store_id) 引用解析 | P1 |
| FR-116.3 | CredentialRef::Env(var_name) 环境变量引用 | P1 |
| FR-116.4 | CredentialRef::File(path) 文件引用 | P1 |
| FR-116.5 | CredentialStore 持久化层 (id/name/type/encrypted_value/timestamps) | P1 |
| FR-116.6 | AES-256-GCM 加密存储 (复用 C-026 §6) | P1 |
| FR-116.7 | 解析失败返回 CredentialResolutionError | P1 |
| FR-116.8 | 凭证轮换端点 POST /providers/{id}/rotate | P1 |
| FR-116.9 | 轮换过渡期 (默认 5min) | P1 |
| FR-116.10 | 轮换失败回滚机制 | P1 |
| FR-116.11 | CredentialStore 访问审计日志 | P1 |

**对应 Constitution**: C-026 §7

### FR-117: Tool Invocation 审计完善

**描述**: 完善工具调用审计记录，支持去重和性能监控

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-117.1 | ToolInvocation 结构体 (id/session_id/tool_name/args) | P1 |
| FR-117.2 | args_hash (SHA-256) 用于去重/审计 | P1 |
| FR-117.3 | latency_ms 执行延迟记录 | P1 |
| FR-117.4 | result (success/error/timeout) 执行结果 | P1 |
| FR-117.5 | result_summary 截断至 1KB, 不含敏感信息 | P1 |
| FR-117.6 | permission_request_id 关联权限请求 | P1 |
| FR-117.7 | ToolInvocation 持久化 (同 Session 生命周期) | P1 |
| FR-117.8 | 敏感信息自动脱敏 (credential/token) | P1 |

**对应 Constitution**: C-024 §7

### FR-118: 统一错误码体系

**描述**: 建立 OpenCodeError 统一错误类型和错误码编号规范

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-118.1 | OpenCodeError 枚举定义 (8 种错误类型) | P1 |
| FR-118.2 | AuthenticationError (1xxx): token 过期/无效 | P1 |
| FR-118.3 | AuthorizationError (2xxx): 权限不足/被拒 | P1 |
| FR-118.4 | ProviderError (3xxx): provider 未找到/认证失败 | P1 |
| FR-118.5 | ToolError (4xxx): 工具未找到/执行超时 | P1 |
| FR-118.6 | SessionError (5xxx): 会话未找到/已过期 | P1 |
| FR-118.7 | ConfigError (6xxx): 配置缺失/无效 | P1 |
| FR-118.8 | InternalError (9xxx): 内部错误/服务不可用 | P1 |
| FR-118.9 | ValidationError: 参数校验错误 | P1 |
| FR-118.10 | API 错误响应统一格式 {error: {code, message, detail}} | P1 |
| FR-118.11 | HTTP 状态码与错误码映射 (401→1xxx, 403→2xxx, ...) | P1 |
| FR-118.12 | 生产环境 detail 字段隐藏 (除非 debug 模式) | P1 |
| FR-118.13 | 错误日志包含 trace_id | P1 |
| FR-118.14 | 错误码向后兼容约束 | P1 |

**对应 Constitution**: C-044

### FR-119: PermissionDecision 审计扩展

**描述**: 扩展权限决策审计字段，支持作用域和用户备注

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-119.1 | scope 字段 (this/session/project) | P2 |
| FR-119.2 | user_note 字段 (用户审批备注) | P2 |
| FR-119.3 | decision_timestamp 决策时间戳 | P2 |
| FR-119.4 | 审计日志持久化 (同 ToolInvocation) | P2 |

**对应 Constitution**: C-024 §7

### FR-120: Provider 动态管理扩展

**描述**: 支持 Provider 运行时启用/禁用和热重载

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-120.1 | enabled_providers 白名单配置 | P2 |
| FR-120.2 | disabled_providers 黑名单配置 (优先) | P2 |
| FR-120.3 | GET /providers/{id}/status 运行时状态 | P2 |
| FR-120.4 | PUT /providers/{id}/enabled 启用/禁用 | P2 |
| FR-120.5 | 配置热重载无需重启 | P2 |
| FR-120.6 | 配置变更 SSE 事件广播 | P2 |

**对应 Constitution**: C-030 §3

---

## 3. 现有需求扩展

### FR-112 扩展: Session Load/Save 安全规范

在 spec_v12 FR-112 基础上新增：

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-112.5 | JSON 导出格式规范 {version, session, messages, tools} | P2 |
| FR-112.6 | Markdown 导出格式规范 (标题/元数据/消息/工具) | P2 |
| FR-112.7 | 导出自动脱敏 (credential/token/api_key → "***REDACTED***") | P2 |
| FR-112.8 | 导入文件格式校验 | P2 |
| FR-112.9 | 版本不兼容 VersionMismatchError | P2 |
| FR-112.10 | 文件损坏 SessionCorruptionError | P2 |

**对应 Constitution**: C-046

---

## 4. 非功能需求

### 4.1 性能

| 指标 | 目标 |
|------|------|
| API 响应时间 (p95) | < 200ms |
| Session 加载时间 | < 500ms |
| Tool 执行延迟 | < 50ms |
| TUI 渲染帧率 | ≥ 30fps |
| SSO 认证延迟 | < 1s (含 IdP 往返) |
| CredentialRef 解析延迟 | < 10ms |

### 4.2 安全

| 要求 | 描述 |
|------|------|
| 凭证加密 | AES-256-GCM |
| SSO CSRF 保护 | 所有 SSO 端点 |
| 导出脱敏 | 自动替换敏感字段 |
| 审计日志 | 所有认证/权限/工具调用 |
| 错误信息 | 生产环境不暴露内部细节 |

### 4.3 质量

| 要求 | 描述 |
|------|------|
| clippy 警告 | 0 |
| 文档覆盖率 | ≥ 80% |
| 集成测试覆盖 | 核心路径 100% |
| 单元测试覆盖率 | ≥ 70% |

---

## 5. 需求追溯矩阵

| 需求编号 | 描述 | 优先级 | Constitution | spec_v12 状态 |
|----------|------|--------|-------------|---------------|
| FR-001 ~ FR-109 | 已有功能 (Core/CLI/LLM/TUI/Agent/Server/...) | P1-P3 | C-001 ~ C-040 | ✅ 保留 |
| FR-110 | 集成测试框架 | P1 | C-045 | ✅ 保留 |
| FR-111 | HuggingFace + AI21 Provider | P2 | - | ✅ 保留 |
| FR-112 | Session Load/Save 工具 | P2 | C-046 | ✅ 保留 + 扩展 |
| FR-113 | 代码质量完善 | P2 | - | ✅ 保留 |
| FR-114 | 性能基准测试 | P2 | - | ✅ 保留 |
| **FR-115** | **SSO/OIDC 企业认证** | **P1** | **C-041** | **🆕 新增** |
| **FR-116** | **CredentialRef 解析机制** | **P1** | **C-026 §7** | **🆕 新增** |
| **FR-117** | **Tool Invocation 审计完善** | **P1** | **C-024 §7** | **🆕 新增** |
| **FR-118** | **统一错误码体系** | **P1** | **C-044** | **🆕 新增** |
| **FR-119** | **PermissionDecision 审计扩展** | **P2** | **C-024 §7** | **🆕 新增** |
| **FR-120** | **Provider 动态管理扩展** | **P2** | **C-030 §3** | **🆕 新增** |

---

## 6. 验收标准

### 6.1 spec_v12 继承标准

- [ ] 集成测试框架可用，核心路径覆盖 100%
- [ ] 18/18 LLM Provider 完整实现
- [ ] 35/35 内置工具完整实现
- [ ] session_load/save 工具可用
- [ ] clippy 零警告
- [ ] 文档注释覆盖率 ≥ 80%
- [ ] 性能基准测试通过
- [ ] cargo test 全部通过
- [ ] cargo build --release 成功

### 6.2 v13 新增标准

- [ ] SSO/OIDC 认证流程端到端可用 (FR-115)
- [ ] CredentialRef 4 种变体均正确解析 (FR-116)
- [ ] 所有工具调用记录 ToolInvocation (FR-117)
- [ ] 所有错误映射到 OpenCodeError 变体 (FR-118)
- [ ] API 错误响应统一格式 {error: {code, message, detail}} (FR-118)
- [ ] PermissionDecision 包含 scope/user_note (FR-119)
- [ ] Provider 运行时启用/禁用可用 (FR-120)
- [ ] Session 导出自动脱敏敏感信息 (FR-112 扩展)

---

## 7. 追溯链

```
spec_v12.md (FR-001 ~ FR-114)
    │
    ├── iteration-13 Constitution 审计 (v1.8 → v1.9)
    │       ├── P1-1: SSO/OIDC 认证 → C-041 → FR-115
    │       ├── P1-2: CredentialRef 解析 → C-026 §7 → FR-116
    │       ├── P1-3: Tool Invocation 审计 → C-024 §7 → FR-117
    │       ├── P1-4: OpenCodeError 错误码 → C-044 → FR-118
    │       ├── P1-5: 集成测试框架 → C-045 → FR-110 (已有)
    │       ├── P2-1: PermissionDecision 审计 → C-024 §7 → FR-119
    │       ├── P2-2: Session Load/Save → C-046 → FR-112 (已有 + 扩展)
    │       └── P2-3: Provider 动态管理 → C-030 §3 → FR-120
    │
    ▼
spec_v13.md (本文档) — FR-001 ~ FR-120
```

---

**文档状态**: 草稿
**需求总数**: 120 (FR-001 ~ FR-120)
**新增需求**: 6 (FR-115 ~ FR-120)
**扩展需求**: 1 (FR-112)
**下一步**: 创建 plan_v13.md 实施计划
