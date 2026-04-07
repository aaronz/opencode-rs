# Constitution 审计报告 (v1.8 → v1.9)

**日期**: 2026-04-06  
**审计范围**: Constitution v1.8 (C-001 ~ C-040, C-001 已废止) vs iteration-9~12 新发现的 P0/P1 问题  
**审计依据**: 
- spec_v9.md / plan_v9.md (iteration-9: SSO/OIDC, credentialRef, 技术债务)
- tasks_v10.md (iteration-10: TUI E2E, Agent 测试, OAuth PKCE)
- spec_v12.md / plan_v12.md (iteration-12: 集成测试, Provider 补全, 性能基准)
- iteration-11/13 gap-analysis (空/占位)

---

## 一、审计结论

### Constitution v1.8 状态: ❌ 需要更新

| 指标 | 值 |
|------|-----|
| Constitution 条款总数 | 40 (C-001 ~ C-040, C-001 已废止) |
| iteration-9~12 新发现 P0/P1 问题数 | **7 (P1: 5, P2: 2)** |
| **P1 被现有条款覆盖** | **1/5 (20%)** |
| **P2 被现有条款覆盖** | **1/2 (50%)** |
| 建议修改条款 | C-024, C-026, C-030 (各需扩展) |
| 建议新增条款 | C-041 ~ C-046 (6条) |

### 关键发现

1. **SSO/OIDC 完全无约束** — C-026 定义了 OAuth/Device Code，但**不包含**企业 SSO (SAML/OIDC) 流程
2. **CredentialStore 持久化层无定义** — C-026 §6 定义加密存储，但未定义 CredentialRef 解析机制
3. **Tool Invocation 审计不完整** — C-024 定义权限决策审计，但未定义 args_hash/latency_ms 等调用详情
4. **OpenCodeError 错误码体系无规范** — 现有条款未定义统一错误码体系
5. **集成测试框架无约束** — 无条款定义 TempProject/MockServer 测试基础设施
6. **Session Load/Save 工具无规范** — 无条款定义 session 导入导出格式和脱敏要求

---

## 二、iteration-9~12 P0/P1 问题详细分析

### 2.1 P1-1: SSO/OIDC 认证流程 (FR-099)

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| SsoConfig 数据模型 | provider/entity_id/sso_url/certificate | ❌ 无 |
| OIDC 授权请求端点 | POST /api/sso/oidc/authorize | ❌ 无 |
| OIDC 回调处理 | POST /api/sso/oidc/callback | ❌ 无 |
| SSO 配置 CRUD | GET/PUT /api/sso/config | ❌ 无 |
| JWT token 验证 | OIDC token 验证集成 | ❌ 无 |
| SAML provider 支持 | SAML 2.0 协议支持 | ❌ 无 |

**根本原因**: C-026 定义的是用户级认证 (API Key, OAuth Browser, Device Code)，**不包含**企业级 SSO (SAML/OIDC)。两者是不同层次的认证:
- **C-026**: 用户 → OpenCode 的认证
- **SSO/OIDC**: 企业 IdP → OpenCode 的联邦认证

### 2.2 P1-2: credentialRef 解析机制 (FR-100)

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| CredentialRef::Ref 运行时解析 | 引用解析为实际凭证 | ❌ 无 |
| CredentialStore 持久化层 | 凭证引用持久化存储 | ⚠️ C-026 §6 仅覆盖加密 |
| Credential 轮换机制 | POST /providers/{id}/rotate | ❌ 无 |

**根本原因**: C-026 §6 定义了凭证加密存储，但未定义 CredentialRef 间接引用机制和 CredentialStore 作为独立持久化层。

### 2.3 P1-3: Tool Invocation 记录完善 (FR-101)

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| args_hash 记录 | 参数哈希用于去重/审计 | ❌ 无 |
| latency_ms 记录 | 工具执行延迟 | ❌ 无 |
| 调用结果摘要 | 成功/失败/错误详情 | ⚠️ C-024 部分覆盖审计日志 |

**根本原因**: C-024 定义权限决策审计，但未定义 ToolInvocation 结构体的完整字段规范。

### 2.4 P1-4: OpenCodeError 统一错误码体系 (FR-104.1)

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| OpenCodeError 枚举定义 | 统一错误类型体系 | ❌ 无 |
| 错误码分类 | 认证/权限/工具/Provider 等分类 | ❌ 无 |
| 错误响应格式 | API 错误响应统一结构 | ⚠️ 部分条款隐含，无统一规范 |
| 错误码文档 | 错误码对照表 | ❌ 无 |

**根本原因**: 现有各条款各自定义错误处理，无统一错误码规范。

### 2.5 P1-5: 集成测试框架 (FR-110)

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| TempProject 测试基础设施 | 临时项目隔离 | ❌ 无 |
| MockServer 基础设施 | 模拟 LLM/外部服务 | ❌ 无 |
| Agent ↔ LLM 集成测试 | 跨 crate 验证 | ❌ 无 |
| Agent ↔ Tool 集成测试 | 工具链验证 | ❌ 无 |
| Server ↔ Session ↔ Storage | 端到端流程 | ❌ 无 |
| TUI ↔ Server WebSocket | UI 层集成 | ❌ 无 |

**根本原因**: Constitution 定义功能规范，但**从未定义测试基础设施规范**。

### 2.6 P2-1: PermissionDecision 审计完善 (FR-102)

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| scope 字段 | this/session/project 范围 | ❌ 无 |
| user_note 字段 | 用户备注 | ❌ 无 |
| 审计日志持久化 | 决策记录持久化 | ⚠️ C-024 部分覆盖 |

**根本原因**: C-024 定义权限评估和审计，但 PermissionDecision 模型缺 scope 和 user_note 字段规范。

### 2.7 P2-2: Session Load/Save 工具 (FR-112)

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| session_load 工具 | 从文件加载会话 | ❌ 无 |
| session_save 工具 | 保存会话到文件 | ❌ 无 |
| 导出格式 | JSON/Markdown 格式规范 | ❌ 无 |
| 敏感信息脱敏 | 导出时自动脱敏 | ⚠️ C-026 §6 覆盖加密，但导出脱敏无定义 |

**根本原因**: 无条款定义 Session 导入导出工具的格式和安全约束。

---

## 三、差距分析 P0/P1 问题映射

| iteration P0/P1 问题 | Constitution 覆盖 | 验证结论 |
|---------------------|-------------------|----------|
| **P1-1: SSO/OIDC 认证** | ❌ 无条款覆盖 | **需新增 C-041** |
| **P1-2: credentialRef 解析** | ⚠️ C-026 §6 部分覆盖 | **需扩展 C-026 或新增 C-042** |
| **P1-3: Tool Invocation 记录** | ⚠️ C-024 部分覆盖 | **需扩展 C-024 或新增 C-043** |
| **P1-4: OpenCodeError 错误码** | ❌ 无条款覆盖 | **需新增 C-044** |
| **P1-5: 集成测试框架** | ❌ 无条款覆盖 | **需新增 C-045** |
| **P2-1: PermissionDecision 审计** | ⚠️ C-024 部分覆盖 | **需扩展 C-024** |
| **P2-2: Session Load/Save** | ❌ 无条款覆盖 | **需新增 C-046** |

---

## 四、Constitution v1.9 修订建议

### 4.1 新增 C-041: SSO/OIDC 企业认证

```markdown
### 条款 C-041: SSO/OIDC 企业单点登录规范

1. SsoConfig 数据模型:
   a) provider: "oidc" | "saml" — 协议类型
   b) entity_id: string — IdP 实体标识
   c) sso_url: Url — IdP SSO 端点
   d) certificate: string — IdP 签名证书 (PEM)
   e) client_id / client_secret: CredentialRef — OIDC 客户端凭证

2. OIDC 认证流程 (7 步骤):
   a) POST /api/sso/oidc/authorize — 发起授权
      - body: { "provider": "oidc", "redirect_uri": "..." }
      - response: { "auth_url": "..." }
   b) 用户重定向至 IdP 登录
   c) IdP 回调 POST /api/sso/oidc/callback
      - body: { "code": "...", "state": "..." }
   d) 后端交换 code → token (后端直连 IdP)
   e) 验证 ID Token (signature, audience, expiry)
   f) 创建/映射本地用户 (基于 email/sub)
   g) 签发本地 JWT Session Token

3. SAML 认证流程:
   a) SSO 发起 → 生成 SAML AuthnRequest
   b) 用户重定向至 IdP
   c) IdP 返回 SAML Response (POST /api/sso/saml/callback)
   d) 验证 SAML Assertion (signature, conditions, audience)
   e) 映射属性 (email, name, groups) → 本地用户
   f) 签发本地 JWT Session Token

4. SSO 配置管理:
   a) GET /api/sso/config — 获取 SSO 配置 (掩码敏感字段)
   b) PUT /api/sso/config — 更新 SSO 配置 (需 admin)
   c) 配置变更需热重载

5. 安全约束:
   a) OIDC: 必须验证 ID Token signature (JWKS)
   b) SAML: 必须验证 Assertion signature
   c) 所有 SSO 端点需 CSRF 保护
   d) SSO 登录失败记录审计日志
   e) Token 映射需处理属性冲突 (email 重复)
```

### 4.2 扩展 C-026: CredentialRef 解析机制 (C-026 §7 新增)

```markdown
### §7. CredentialRef 解析机制

1. CredentialRef 类型:
   a) CredentialRef::Literal(value) — 内联凭证值
   b) CredentialRef::Ref(store_id) — 引用 CredentialStore 中的凭证
   c) CredentialRef::Env(var_name) — 环境变量引用
   d) CredentialRef::File(path) — 文件路径引用

2. 解析流程:
   a) 运行时检测 CredentialRef 变体
   b) Ref 变体 → 查询 CredentialStore 获取实际值
   c) Env 变体 → 读取环境变量
   d) File 变体 → 读取文件内容
   e) 解析失败 → 返回 CredentialResolutionError

3. CredentialStore 持久化层:
   a) 存储结构: { id, name, type, encrypted_value, created_at, updated_at }
   b) 加密: AES-256-GCM (同 C-026 §6)
   c) 访问控制: 仅授权服务可读取
   d) 审计: 所有读取操作记录审计日志

4. Credential 轮换:
   a) POST /providers/{id}/rotate — 轮换凭证
   b) 旧凭证保留过渡期 (默认 5min)
   c) 轮换操作记录审计日志
   d) 轮换失败回滚机制
```

### 4.3 扩展 C-024: Tool Invocation 审计 (C-024 §7 新增)

```markdown
### §7. Tool Invocation 审计记录

1. ToolInvocation 结构体:
   a) id: Uuid — 唯一标识
   b) session_id: Uuid — 所属会话
   c) tool_name: String — 工具名称
   d) args: serde_json::Value — 调用参数
   e) args_hash: String — 参数 SHA-256 哈希 (用于去重/审计)
   f) latency_ms: u64 — 执行延迟 (毫秒)
   g) result: ToolResult — 执行结果 (success/error/timeout)
   h) result_summary: Option<String> — 结果摘要 (截断至 1KB)
   i) timestamp: DateTime<Utc> — 调用时间
   j) permission_request_id: Option<Uuid> — 关联权限请求

2. 审计要求:
   a) 每次工具调用必须记录 ToolInvocation
   b) args_hash 用于检测重复调用
   c) latency_ms 用于性能监控和告警
   d) result_summary 不包含敏感信息 (credential/token 脱敏)
   e) ToolInvocation 持久化到 storage (同 Session 生命周期)

3. PermissionDecision 审计扩展:
   a) scope: "this" | "session" | "project" — 决策作用域
   b) user_note: Option<String> — 用户备注 (审批时填写)
   c) decision_timestamp: DateTime<Utc> — 决策时间
   d) 审计日志持久化 (同 ToolInvocation)
```

### 4.4 新增 C-044: 统一错误码体系

```markdown
### 条款 C-044: OpenCodeError 错误码规范

1. OpenCodeError 枚举定义:
   a) AuthenticationError { code, message, detail } — 认证失败
   b) AuthorizationError { code, message, required_role } — 权限不足
   c) ProviderError { code, provider, message } — Provider 相关错误
   d) ToolError { code, tool_name, message } — 工具执行错误
   e) SessionError { code, session_id, message } — 会话相关错误
   f) ConfigError { code, key, message } — 配置错误
   g) InternalError { code, message, trace_id } — 内部服务错误
   h) ValidationError { code, field, message } — 参数校验错误

2. 错误码编号规则:
   a) 1xxx — 认证类 (1001: token 过期, 1002: token 无效, ...)
   b) 2xxx — 权限类 (2001: 权限不足, 2002: 权限被拒, ...)
   c) 3xxx — Provider 类 (3001: provider 未找到, 3002: 认证失败, ...)
   d) 4xxx — 工具类 (4001: 工具未找到, 4002: 执行超时, ...)
   e) 5xxx — 会话类 (5001: 会话未找到, 5002: 会话已过期, ...)
   f) 6xxx — 配置类 (6001: 配置缺失, 6002: 配置无效, ...)
   g) 9xxx — 内部类 (9001: 内部错误, 9002: 服务不可用, ...)

3. API 错误响应格式:
   a) { "error": { "code": 1001, "message": "Token expired", "detail": "..." } }
   b) HTTP 状态码与错误码映射 (401 → 1xxx, 403 → 2xxx, 404 → 3xxx/5xxx, 500 → 9xxx)
   c) 错误消息用户友好 (不包含堆栈/内部路径)
   d) 生产环境隐藏 detail 字段 (除非 debug 模式)

4. 错误处理约束:
   a) 所有错误必须映射到 OpenCodeError 变体
   b) 禁止在 API 层返回裸 error (必须转换)
   c) 错误日志包含 trace_id (用于追踪)
   d) 错误码变更需向后兼容
```

### 4.5 新增 C-045: 集成测试框架规范

```markdown
### 条款 C-045: 集成测试基础设施规范

1. 测试基础设施:
   a) TempProject — 临时项目隔离
      - 每个测试创建独立临时目录
      - 测试结束自动清理
      - 预置标准项目结构 (.opencode/, config.json)
   b) MockServer — 模拟外部服务
      - Mock LLM Provider (模拟 OpenAI/Anthropic 响应)
      - Mock OAuth/OIDC IdP
      - Mock MCP Server
      - 支持自定义响应/延迟/错误注入

2. 集成测试分层:
   a) Agent ↔ LLM Provider — 验证 Agent 与 LLM 交互
      - 测试: Build/Plan/General Agent 正常对话流程
      - 测试: 工具调用 → LLM 解析 → 工具执行
   b) Agent ↔ Tool — 验证工具链
      - 测试: 35 内置工具端到端执行
      - 测试: MCP 外部工具调用
   c) Server ↔ Session ↔ Storage — 验证会话管理
      - 测试: Session 创建/加载/保存/删除
      - 测试: Session 状态机转换
   d) TUI ↔ Server WebSocket — 验证 UI 层
      - 测试: WebSocket 消息收发
      - 测试: SSE 流式响应
   e) 权限系统集成 — 验证权限评估
      - 测试: allow/ask/deny 决策链
      - 测试: 审批工作流

3. 测试约束:
   a) 集成测试标记: #[tokio::test] + 异步 runtime
   b) 超时: 单个集成测试 ≤ 30s
   c) 并行: 集成测试可并行执行 (TempProject 隔离)
   d) 覆盖率: 核心路径 100% 覆盖
   e) Mock 数据版本化 (与真实 API 响应同步)

4. Agent 只读约束测试:
   a) Plan Agent 写工具被拒绝
   b) Review Agent 只读验证
   c) Agent 切换后权限隔离
```

### 4.6 新增 C-046: Session Load/Save 工具规范

```markdown
### 条款 C-046: Session 导入导出工具规范

1. session_load 工具:
   a) 从 JSON/Markdown 文件加载会话
   b) 验证文件格式和完整性
   c) 恢复会话上下文 (消息历史/工具状态)
   d) 冲突处理: 与现有会话 ID 冲突时生成新 ID

2. session_save 工具:
   a) 导出当前会话到文件
   b) 支持格式: JSON (完整数据) / Markdown (可读)
   c) 导出内容: 消息历史/工具调用/权限决策/元数据

3. 导出格式规范:
   a) JSON 格式:
      { "version": "1.0", "session": { ... }, "messages": [...], "tools": [...] }
   b) Markdown 格式:
      # Session: {title}
      ## Metadata
      ## Messages (按时间顺序)
      ## Tool Invocations

4. 安全约束:
   a) 导出时自动脱敏敏感信息 (credential/token/api_key)
   b) 脱敏规则: 替换为 "***REDACTED***"
   c) 导出文件不包含明文凭证
   d) 导入时验证文件签名/完整性 (可选)

5. 错误处理:
   a) 文件格式不匹配 → ValidationError
   b) 文件损坏/不完整 → SessionCorruptionError
   c) 版本不兼容 → VersionMismatchError (提示升级)
```

### 4.7 扩展 C-030: Provider 动态管理 (C-030 §3 新增)

```markdown
### §3. Provider 运行时管理

1. 配置项:
   a) enabled_providers: Option<Vec<String>> — 白名单 (仅列出可用)
   b) disabled_providers: Vec<String> — 黑名单 (列出的禁用)
   c) 优先级: disabled_providers > enabled_providers (黑名单优先)

2. 管理端点:
   a) GET /providers/{id}/status — 获取运行时状态
      - response: { "id": "...", "enabled": true, "active_sessions": 5 }
   b) PUT /providers/{id}/enabled — 启用/禁用
      - body: { "enabled": false }
      - 响应: { "status": "updated" }

3. 热重载:
   a) 配置变更后无需重启
   b) 已活跃 Session 不受影响
   c) 新请求使用最新配置
   d) 配置变更事件广播 (SSE)
```

---

## 五、修订计划

| 条款 | 操作 | 说明 | 优先级 |
|------|------|------|--------|
| C-041 | 新增 | SSO/OIDC 企业认证 | P1 |
| C-026 §7 | 扩展 | CredentialRef 解析机制 | P1 |
| C-024 §7 | 扩展 | Tool Invocation 审计 + PermissionDecision 完善 | P1 |
| C-044 | 新增 | 统一错误码体系 | P1 |
| C-045 | 新增 | 集成测试框架规范 | P1 |
| C-030 §3 | 扩展 | Provider 动态管理 | P2 |
| C-046 | 新增 | Session Load/Save 工具 | P2 |

---

## 六、与 iteration-8 对比

| 指标 | iteration-8 (v1.8) | iteration-13 (v1.9) | 变化 |
|------|-------------------|---------------------|------|
| Constitution 版本 | v1.8 | v1.9 | +1 |
| 条款总数 | 40 (C-001~C-040) | 46 (C-001~C-046) | +6 |
| 需新增条款 | 3 (C-038~C-040) | 6 (C-041~C-046) | +3 |
| 需扩展条款 | 2 (C-024, C-034) | 3 (C-024, C-026, C-030) | +1 |
| P1 覆盖率 | N/A | 20% → 100% (修订后) | ✅ 修复 |
| P2 覆盖率 | N/A | 50% → 100% (修订后) | ✅ 修复 |

---

## 七、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.7 | 2026-04-05 | 审计确认: OAuth/Device Code P0 100% 覆盖 |
| 1.8 | 2026-04-06 | 新增 P0: Provider API, Permission API, Session State |
| **1.9** | **2026-04-06** | **新增 P1/P2: SSO/OIDC, CredentialRef, Error Code, 集成测试, Session 工具** |

---

*本文档识别 iteration-9~12 差距分析中的 7 个 P1/P2 问题需要 Constitution v1.9 新增/扩展条款覆盖。*
