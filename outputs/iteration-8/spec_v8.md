# OpenCode-RS 规格文档 v8

**版本**: 8.0
**日期**: 2026-04-06
**基于**: spec_v7.md + iteration-8 差距分析
**状态**: 已完成

---

## 1. 文档概述

### 1.1 背景

本规格文档基于以下文档综合生成：
- **spec_v7.md**: 上一版规格文档 (FR-001 ~ FR-088)
- **iteration-8 差距分析报告**: 2026-04-06 新一轮差距分析 (3 P0, 5 P1, 4 P2)
- **PRD.md**: 产品需求文档 v1.1
- **PRD-OpenCode-Configuration.md**: 配置系统产品需求文档
- **PRD-tui.md**: TUI 产品需求详细设计

### 1.2 目标

- 基于 iteration-8 差距分析，新增 P0/P1/P2 功能需求
- 确保新功能有对应的规格定义
- 为 iteration-9 提供实施基线

### 1.3 参考文档

| 文档 | 路径 | 说明 |
|------|------|------|
| PRD-主文档 | `docs/PRD.md` | 产品需求文档 v1.1 |
| PRD-TUI | `docs/PRD-tui.md` | TUI 产品需求详细设计 |
| PRD-配置系统 | `PRD-OpenCode-Configuration.md` | 配置系统产品需求 |
| spec_v7 | `outputs/iteration-7/spec_v7.md` | 上一版规格文档 |
| 差距分析 | 当前任务 | iteration-8 差距分析报告 |

### 1.4 与 v7 的关系

v7 保留 v6 的所有需求 (FR-001 ~ FR-088)。

v8 新增需求：
- 3 项 P0 阻断性问题 → FR-089 ~ FR-091
- 5 项 P1 核心功能 → FR-092 ~ FR-096
- 4 项 P2 增强功能 → FR-097 ~ FR-100

---

## 2. 需求总览

| 优先级 | 数量 | 说明 |
|--------|------|------|
| P0 | 17 | 阻断性问题 (v7: 14, v8新增: 3) |
| P1 | 39 | 核心功能缺失 (v7: 34, v8新增: 5) |
| P2 | 41 | 完善性问题 (v7: 37, v8新增: 4) |
| P3 | 2 | 远期规划 |

**总计**: 100 项功能需求 (v7: 88 项)

---

## 3. iteration-8 差距分析

### 3.1 P0 问题 (阻断性)

| # | 问题 | 当前状态 | 修复方案 | 关联 FR |
|---|------|----------|----------|---------|
| 1 | **Provider 管理 API 缺失** | server 无 credentials API | 新增 routes/provider.rs: POST /providers/{id}/credentials, /test, /revoke | FR-089 |
| 2 | **Permission 审批 API 缺失** | 仅有 PermissionEvaluator | 新增 routes/permission.rs: POST /permissions/{id}/reply | FR-090 |
| 3 | **Session State 状态机不完整** | 12 状态未完全实现 | 完善 session_state.rs 状态转换逻辑 | FR-091 |

### 3.2 P1 问题 (重要功能)

| # | 问题 | 当前状态 | 修复方案 | 关联 FR |
|---|------|----------|----------|---------|
| 1 | **插件系统 WASM 运行时** | 仅框架 | 实现 WASM 加载器 + 事件 hooks | FR-072 (已有) |
| 2 | **Plan Agent 写限制** | 可被绕过 | build_agent.rs 添加工具黑名单 | FR-092 |
| 3 | **Artifact API** | 无 diff/snapshots | 新增 routes/artifact.rs | FR-093 |
| 4 | **Share 本地导出** | 无 | 实现 session JSON/Markdown export | FR-094 |
| 5 | **企业控制平面** | 无 | 实现 account + enterprise 模块 | FR-095 |

### 3.3 P2 问题 (增强功能)

| # | 问题 | 当前状态 | 修复方案 | 关联 FR |
|---|------|----------|----------|---------|
| 1 | **Web UI 完整实现** | 基础 | 完善 web_ui routes | FR-096 |
| 2 | **工具测试覆盖** | 部分 | 补充测试用例 | FR-097 |
| 3 | **OAuth Browser 登录** | 框架 | 实现完整 PKCE 流程 | FR-064 (已有) |
| 4 | **Context Engine 压缩阈值** | 基础 | 实现 85%/92%/95% 阈值 | FR-098 |

---

## 4. FR 状态汇总

### 4.1 P0 - 阻断性问题

> FR-001 ~ FR-010, FR-033, FR-034 继承自 v4/v5，内容不变。

| FR 编号 | 需求名称 | 覆盖差距 | 状态 |
|--------|----------|----------|------|
| FR-063 | Provider 认证协议分层抽象 | 已有 | 待实现 |
| FR-064 | OAuth/Device Code 浏览器登录流程 | 已有 | 待实现 |
| **FR-089** | **Provider 管理 API (credentials/test/revoke)** | **P0-1** | **新增** |
| **FR-090** | **Permission 审批 API** | **P0-2** | **新增** |
| **FR-091** | **Session State 状态机完整实现** | **P0-3** | **新增** |

### 4.2 P1 - 核心功能缺失

> FR-011 ~ FR-020, FR-032, FR-035 ~ FR-039, FR-044 ~ FR-048, FR-053 ~ FR-056 继承自 v7，内容不变。

| FR 编号 | 需求名称 | 覆盖差距 | 状态 |
|--------|----------|----------|------|
| FR-065 | 云厂商原生认证 | 已有 | 待实现 |
| FR-066 | Remote Config 自动发现 | 已有 | 待实现 |
| FR-067 | disabled_providers 优先级 | 已有 | 待实现 |
| FR-068 | MCP OAuth 独立 token store | 已有 | 待实现 |
| FR-069 | TUI 三栏布局与 Inspector 面板 | 已有 | 待实现 |
| FR-070 | TUI 状态机完整实现 | 已有 | 待实现 |
| FR-071 | Context Engine 分层上下文 | 已有 | 待实现 |
| FR-072 | Plugin WASM 运行时 | 已有 | 待实现 |
| FR-073 | 凭证加密存储 | 已有 | 待实现 |
| **FR-092** | **Plan Agent 工具写限制强制执行** | **P1-2** | **新增** |
| **FR-093** | **Artifact API (diff/snapshots/revert)** | **P1-3** | **新增** |
| **FR-094** | **Share 本地导出 (JSON/Markdown)** | **P1-4** | **新增** |
| **FR-095** | **Enterprise 控制平面 (account/SSO)** | **P1-5** | **新增** |

### 4.3 P2 - 增强功能

> FR-021 ~ FR-031, FR-040 ~ FR-052, FR-057 ~ FR-062, FR-074 ~ FR-085 继承自 v7，内容不变。

| FR 编号 | 需求名称 | 覆盖差距 | 状态 |
|--------|----------|----------|------|
| FR-074 | Event Bus 事件类型完整性 | 已有 | 待实现 |
| FR-075 | Share 服务层 | 已有 | 待实现 |
| FR-076 | SDK 输出 (Rust + TypeScript) | 已有 | 待实现 |
| FR-077 | OpenAPI 文档自动生成 | 已有 | 待实现 |
| FR-078 | LSP definition/references/hover | 已有 | 待实现 |
| FR-079 | session_load/session_save 工具 | 已有 | 待实现 |
| FR-080 | HuggingFace + AI21 Provider | 已有 | 待实现 |
| FR-081 | Formatters 接入 agent 执行循环 | 已有 | 待实现 |
| FR-082 | Compaction 自动触发阈值 | 已有 | 待实现 |
| FR-083 | TUI 虚拟滚动 | 已有 | 待实现 |
| FR-084 | Server HTTP Basic Auth | 已有 | 待实现 |
| FR-085 | 观测性 (tracing/crash recovery/token cost) | 已有 | 待实现 |
| **FR-096** | **Web UI 完整实现** | **P2-1** | **新增** |
| **FR-097** | **工具单元测试覆盖** | **P2-2** | **新增** |
| **FR-098** | **Context Engine 压缩阈值 (85%/92%/95%)** | **P2-4** | **新增** |

### 4.4 P3 - 远期规划

| FR 编号 | 需求名称 | 状态 |
|--------|----------|------|
| FR-087 | GitHub Integration (v2) | 规划中 |
| FR-088 | Enterprise 配置 (Central Config + SSO) | 规划中 |

---

## 5. 新增 FR 详细规格

### 5.1 FR-089: Provider 管理 API

**优先级**: P0 (阻断性)

**需求描述**:
Server API 必须提供完整的 Provider 凭证管理能力，包括设置、测试、撤销凭证。

**API 端点**:

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/providers` | 列出所有 provider、连接状态、默认模型、认证策略（脱敏） |
| POST | `/providers/{id}/credentials` | 设置或更新 credential |
| POST | `/providers/{id}/test` | 连通性与权限测试 |
| DELETE | `/providers/{id}/credentials` | 撤销当前绑定 |

**返回格式** (GET /providers 示例):

```json
{
  "providers": [
    {
      "id": "openai",
      "name": "OpenAI",
      "protocol": "openai-compatible",
      "baseUrl": "https://api.openai.com/v1",
      "auth": {
        "strategy": "bearer_api_key"
      },
      "hasCredential": true,
      "credentialExpired": false,
      "lastTestedAt": "2026-04-06T10:00:00Z",
      "lastError": null
    }
  ]
}
```

**数据模型**:

```rust
struct ProviderInfo {
    id: String,
    name: String,
    protocol: String,
    base_url: Option<String>,
    auth: AuthStrategy,
    has_credential: bool,
    credential_expired: Option<bool>,
    last_tested_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
}

struct SetCredentialRequest {
    credential: CredentialRef,  // credentialRef 或 inline credential
}

struct TestCredentialResponse {
    success: bool,
    latency_ms: u64,
    error: Option<String>,
}
```

**安全要求**:
- 永远不返回明文 credential
- POST /test 应该只测试连接，不改变状态
- DELETE /credentials 应该使相关会话进入需重验状态

**关联 Constitution**: C-030 (Provider 控制规范)

---

### 5.2 FR-090: Permission 审批 API

**优先级**: P0 (阻断性)

**需求描述**:
Server API 必须提供 Permission 请求的审批能力，允许外部客户端对待审批的权限请求进行回复。

**API 端点**:

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/permissions` | 列出待审批的权限请求 |
| GET | `/permissions/{req_id}` | 获取特定请求详情 |
| POST | `/permissions/{req_id}/reply` | 批准/拒绝权限请求 |

**返回格式** (GET /permissions 示例):

```json
{
  "permissions": [
    {
      "id": "perm_abc123",
      "session_id": "ses_xyz789",
      "tool_name": "bash",
      "args_summary": "npm install",
      "risk_level": "high",
      "decision": "pending",
      "created_at": "2026-04-06T10:00:00Z"
    }
  ]
}
```

**请求格式** (POST /permissions/{id}/reply):

```json
{
  "decision": "allow",  // "allow" | "deny"
  "scope": "session",   // "this" | "session" | "project"
  "note": "Approved by admin via API"
}
```

**状态码**:

| 状态 | 说明 |
|------|------|
| pending | 待审批 |
| approved | 已批准 |
| denied | 已拒绝 |
| expired | 已超时 |

**关联 Constitution**: C-024 (Permission 系统规范)

---

### 5.3 FR-091: Session State 状态机完整实现

**优先级**: P0 (阻断性)

**需求描述**:
Session 状态机必须完整实现 PRD §7.3 定义的 12 种状态，并确保状态转换符合规范。

**完整状态列表**:

| 状态 | 说明 | 可转换到 |
|------|------|---------|
| `idle` | 空闲，等待用户输入 | thinking, error |
| `thinking` | 模型正在推理 | awaiting_permission, executing_tool, streaming, error |
| `awaiting_permission` | 等待用户确认工具执行 | executing_tool, idle, error |
| `executing_tool` | 正在执行工具 | thinking, error |
| `streaming` | 流式输出中 | thinking, idle, completed |
| `applying_changes` | 正在应用文件变更 | verifying, error |
| `verifying` | 正在验证变更 | thinking, idle, error, completed |
| `summarizing` | 正在生成摘要 | idle, completed |
| `aborted` | 已中止 | idle |
| `error` | 发生错误 | idle |
| `completed` | 完成 | idle |
| `paused` | 暂停（可恢复） | idle, thinking |

**状态转换规则**:

```
状态转换必须遵守以下规则：
1. idle → thinking: 用户发送 prompt
2. thinking → awaiting_permission: 模型请求执行需审批的工具
3. awaiting_permission → executing_tool: 用户批准
4. awaiting_permission → idle: 用户拒绝
5. executing_tool → thinking: 工具执行完成，返回结果
6. thinking → streaming: 模型开始流式输出
7. streaming → completed: 输出完成
8. any → error: 发生错误
9. error → idle: 用户处理错误
10. idle → summarizing: 用户请求摘要
11. summarizing → idle: 摘要完成
12. idle → aborted: 用户中止
13. idle → paused: 用户暂停
14. paused → thinking: 用户恢复
```

**实现要求**:

```rust
enum SessionState {
    Idle,
    Thinking,
    AwaitingPermission,
    ExecutingTool,
    Streaming,
    ApplyingChanges,
    Verifying,
    Summarizing,
    Aborted,
    Error,
    Completed,
    Paused,
}

impl SessionState {
    fn can_transition_to(&self, target: &SessionState) -> bool {
        // 实现状态转换规则
    }
}
```

**事件触发**:

| 状态 | 触发事件 |
|------|---------|
| idle | `session.idle` |
| thinking | `session.thinking` |
| awaiting_permission | `permission.requested` |
| executing_tool | `tool.executing` |
| streaming | `message.streaming` |
| applying_changes | `file.applying` |
| verifying | `session.verifying` |
| summarizing | `session.summarizing` |
| aborted | `session.aborted` |
| error | `session.error` |
| completed | `session.completed` |
| paused | `session.paused` |

**关联 Constitution**: C-035 (Context Engine 分层)

---

### 5.4 FR-092: Plan Agent 工具写限制强制执行

**优先级**: P1 (核心功能)

**需求描述**:
Plan Agent 必须强制执行只读约束，禁止执行写文件、编辑、执行 bash 等高风险操作。

**工具分类**:

| 类别 | 工具 | Plan Agent 行为 |
|------|------|----------------|
| 读工具 | read, glob, grep, stat, lsp_* | allow |
| 写工具 | edit, write, patch, move, delete | deny (强制) |
| 执行工具 | bash | deny (强制) |
| 辅助工具 | todo_write, summarize_session | allow |

**黑名单机制**:

```rust
const PLAN_AGENT_DENIED_TOOLS: &[&str] = &[
    "edit",
    "write",
    "patch",
    "move",
    "delete",
    "bash",
    "mcp_tool",  // 远程 MCP 工具
];

fn validate_tool_for_agent(agent: &Agent, tool: &str) -> Result<(), ToolDeniedError> {
    if agent.name == "plan" && PLAN_AGENT_DENIED_TOOLS.contains(&tool) {
        Err(ToolDeniedError::PlanAgentReadOnly { tool })
    }
    Ok(())
}
```

**错误处理**:

```rust
#[derive(Error, Debug)]
pub enum ToolDeniedError {
    #[error("Plan Agent cannot execute write tool: {0}")]
    PlanAgentReadOnly { tool: String },
}
```

**TUI 反馈**:

- Plan Agent 模式下，写工具按钮应禁用或显示警告
- 尝试执行被拒绝的工具时，显示清晰的错误消息

**关联 Constitution**: C-023 (Agent 系统规范)

---

### 5.5 FR-093: Artifact API

**优先级**: P1 (核心功能)

**需求描述**:
Server API 必须提供完整的 Artifact 管理能力，包括 diff 查看、快照管理、回滚操作。

**API 端点**:

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/sessions/{id}/diff` | 获取当前 diff |
| GET | `/sessions/{id}/snapshots` | 获取快照列表 |
| GET | `/sessions/{id}/snapshots/{snap_id}` | 获取特定快照 |
| POST | `/sessions/{id}/revert` | 回滚到指定快照 |
| GET | `/sessions/{id}/patch` | 获取 patch bundle |

**返回格式** (GET /sessions/{id}/diff):

```json
{
  "session_id": "ses_abc123",
  "diffs": [
    {
      "file": "src/main.rs",
      "old_content": "fn main() { ... }",
      "new_content": "fn main() { ... }",
      "unified_diff": "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,5 @@",
      " hunks": [...]
    }
  ],
  "generated_at": "2026-04-06T10:00:00Z"
}
```

**返回格式** (GET /sessions/{id}/snapshots):

```json
{
  "session_id": "ses_abc123",
  "snapshots": [
    {
      "id": "snap_001",
      "based_on_message_id": "msg_456",
      "description": "Add user auth module",
      "patch_path": "/data/sessions/ses_abc123/snapshots/snap_001.patch",
      "created_at": "2026-04-06T09:00:00Z"
    }
  ]
}
```

**回滚请求** (POST /sessions/{id}/revert):

```json
{
  "snapshot_id": "snap_001",
  "mode": "soft" | "hard",  // soft: 保留当前变更, hard: 完全覆盖
  "confirm": true
}
```

**数据模型**:

```rust
struct DiffResponse {
    session_id: String,
    diffs: Vec<FileDiff>,
    generated_at: DateTime<Utc>,
}

struct FileDiff {
    file: String,
    old_content: Option<String>,
    new_content: Option<String>,
    unified_diff: String,
    hunks: Vec<DiffHunk>,
}

struct Snapshot {
    id: String,
    based_on_message_id: String,
    description: Option<String>,
    patch_path: PathBuf,
    created_at: DateTime<Utc>,
}
```

**关联 Constitution**: C-028 (Storage 系统规范)

---

### 5.6 FR-094: Share 本地导出

**优先级**: P1 (核心功能)

**需求描述**:
系统必须支持将 Session 导出为本地格式（JSON、Markdown），为服务层 Share 提供基础。

**导出格式**:

| 格式 | 路径 | 说明 |
|------|------|------|
| JSON | `/export/sessions/{id}` | 完整会话 JSON |
| Markdown | `/export/sessions/{id}/transcript` | Markdown 对话记录 |
| Patch Bundle | `/export/sessions/{id}/patch` | 合并的 patch 文件 |

**JSON 导出格式**:

```json
{
  "version": "1.0",
  "session": {
    "id": "ses_abc123",
    "title": "Add authentication",
    "agent": "build",
    "model": "openai/gpt-4",
    "created_at": "2026-04-06T08:00:00Z",
    "updated_at": "2026-04-06T10:00:00Z"
  },
  "messages": [
    {
      "id": "msg_001",
      "role": "user",
      "content": "Add user authentication",
      "created_at": "2026-04-06T08:00:00Z"
    },
    {
      "id": "msg_002",
      "role": "assistant",
      "content": "I'll help you add user authentication...",
      "tool_calls": [...]
    }
  ],
  "artifacts": {
    "diffs": [...],
    "snapshots": [...]
  }
}
```

**Markdown 导出格式**:

```markdown
# Session: Add authentication

**Agent**: build | **Model**: openai/gpt-4 | **Created**: 2026-04-06

---

## User (08:00:00)

Add user authentication

## Assistant (08:00:01)

I'll help you add user authentication. Let me first explore the project structure...

### Tool: read (src/auth/mod.rs)

[File content...]

## Assistant (08:00:15)

Now I'll create the authentication module...

### Tool: write (src/auth/mod.rs)

[New file content...]
```

**CLI 命令**:

```bash
# 导出 JSON
opencode export --session ses_abc123 --format json --output auth-session.json

# 导出 Markdown
opencode export --session ses_abc123 --format markdown --output auth-session.md

# 导出 Patch Bundle
opencode export --session ses_abc123 --format patch --output auth-session.patch
```

**安全要求**:

- 导出时必须从 transcript 中移除敏感信息（credential、环境变量）
- auth store 不包含在导出文件中
- 可选：导出前显示脱敏预览

**关联 Constitution**: C-027 (Share 系统规范)

---

### 5.7 FR-095: Enterprise 控制平面

**优先级**: P1 (核心功能)

**需求描述**:
企业部署场景需要 Central Config、SSO、Account 管理等控制平面能力。

**模块结构**:

```
crates/opencode-control-plane/
├── src/
│   ├── account/          # 账户管理
│   │   ├── mod.rs
│   │   ├── model.rs     # Account, User, Team
│   │   └── repository.rs
│   ├── enterprise/      # 企业配置
│   │   ├── mod.rs
│   │   ├── model.rs     # Enterprise, Policy
│   │   └── repository.rs
│   ├── sso/             # SSO 集成
│   │   ├── mod.rs
│   │   ├── provider.rs  # SAML/OIDC provider
│   │   └── handler.rs
│   └── central_config/  # 集中配置
│       ├── mod.rs
│       ├── fetcher.rs
│       └── validator.rs
```

**数据模型**:

```rust
// Account
struct Account {
    id: String,
    name: String,
    owner_user_id: String,
    created_at: DateTime<Utc>,
    plan: Plan,
    status: AccountStatus,
}

// Enterprise
struct Enterprise {
    id: String,
    account_id: String,
    name: String,
    sso_config: Option<SsoConfig>,
    policies: Vec<Policy>,
    central_config_url: Option<String>,
}

// SSO Config
struct SsoConfig {
    provider: SsoProvider,  // SAML, OIDC
    entity_id: String,
    sso_url: String,
    certificate: String,
    attribute_mapping: HashMap<String, String>,
}

// Policy
struct Policy {
    id: String,
    name: String,
    permission_profile: PermissionProfile,
    allowed_providers: Vec<String>,
    mcp_restrictions: Vec<String>,
    network_policy: NetworkPolicy,
}
```

**SSO 流程** (OIDC 为例):

```
1. 用户访问企业应用
2. 重定向到 IdP 登录页面
3. 用户在 IdP 完成认证
4. IdP 回调带 authorization code
5. OpenCode-RS 交换 token
6. 创建本地 session
7. 映射用户属性到 account
```

**Central Config**:

```rust
struct RemoteConfig {
    url: String,
    fetch_interval: Duration,
    validation: ValidationRule,
    fallback: Option<Box<Config>>,
}

async fn fetch_remote_config(url: &str) -> Result<Config, ConfigError> {
    // 1. HTTP GET
    // 2. Validate JSON Schema
    // 3. Merge with local config
    // 4. Cache with TTL
}
```

**关联 Constitution**: C-037 (Remote Config 自动发现)

---

### 5.8 FR-096: Web UI 完整实现

**优先级**: P2 (增强功能)

**需求描述**:
在 Server 基础上实现完整的 Web UI，补充当前 TUI 在浏览器中的能力。

**页面结构**:

| 路由 | 页面 | 说明 |
|------|------|------|
| `/` | 引导页 | 项目选择/创建新会话 |
| `/sessions` | 会话列表 | 项目会话管理 |
| `/session/:id` | 会话详情 | 聊天界面 |
| `/settings` | 设置 | Provider/API Keys 配置 |
| `/admin` | 管理面板 | (企业版) 用户/策略管理 |

**UI 组件**:

| 组件 | 功能 |
|------|------|
| MessageList | 会话消息流 |
| InputArea | 消息输入框，支持 @file |
| DiffViewer | 文件变更展示 |
| FileTree | 项目文件树 |
| ProviderManager | Provider 配置 |
| PermissionQueue | 权限审批队列 |

**技术栈**:

- 前端框架：Leptos 或 Yew (Rust) 或 React (TypeScript)
- 状态管理：Rust backend state
- 实时通信：SSE/WebSocket

**API 映射**:

| Web 路由 | Backend API |
|----------|-------------|
| `/sessions` | GET /sessions |
| `/session/:id` | GET /sessions/:id |
| POST message | POST /sessions/:id/prompt |
| File tree | GET /projects/:id/tree |

**关联 Constitution**: C-034 (TUI 布局与状态机)

---

### 5.9 FR-097: 工具单元测试覆盖

**优先级**: P2 (增强功能)

**需求描述**:
为所有内置工具补充单元测试，确保核心功能的回归测试覆盖。

**测试覆盖目标**:

| 工具 | 最低测试用例 | 覆盖场景 |
|------|-------------|---------|
| read | 5 | 正常读取、路径解析、编码、错误处理 |
| write | 5 | 创建、更新、目录不存在、权限错误 |
| edit | 8 | 精确编辑、模糊编辑、批量编辑、错误 |
| glob | 4 | 基础 glob、多个 pattern、过滤、错误 |
| grep | 5 | 基础搜索、大小写、regex、上下文、错误 |
| bash | 4 | 成功执行、错误退出、超时、shell 解析 |
| patch | 5 | 应用成功、冲突检测、回滚、错误 |
| move | 3 | 正常移动、目标存在、错误 |
| delete | 4 | 正常删除、文件不存在、目录、错误 |

**测试结构**:

```rust
#[cfg(test)]
mod tests {
    mod read_tests {
        use super::*;
        
        #[tokio::test]
        async fn test_read_existing_file() { ... }
        
        #[tokio::test]
        async fn test_read_nonexistent_file() { ... }
        
        #[tokio::test]
        async fn test_read_with_encoding() { ... }
        
        #[tokio::test]
        async fn test_read_permission_error() { ... }
        
        #[tokio::test]
        async fn test_read_large_file() { ... }
    }
    
    mod write_tests { ... }
    mod edit_tests { ... }
    // ...
}
```

**测试工具**:

```rust
// 测试辅助工具
fn create_temp_project(files: HashMap<&str, &str>) -> TempProject {
    // 创建临时项目目录
    // 写入测试文件
    // 返回项目路径
}

fn mock_tool_context() -> ToolContext {
    // Mock ToolContext for testing
}
```

**覆盖率目标**:

- 语句覆盖率 ≥ 70%
- 分支覆盖率 ≥ 60%
- 关键路径 100%

**关联 Constitution**: 全局质量要求

---

### 5.10 FR-098: Context Engine 压缩阈值

**优先级**: P2 (增强功能)

**需求描述**:
实现 PRD §7.6 定义的 Context Engine 分层压缩阈值，精确控制 token 消耗。

**阈值定义**:

| 阈值 | 级别 | 动作 |
|------|------|------|
| 70% | 预警 | 提醒用户 token 消耗 |
| 85% | 警告 | 自动压缩非关键上下文 |
| 92% | 触发 | 执行 session summarize |
| 95% | 强制 | 暂停并提示用户创建新 session |

**实现逻辑**:

```rust
struct CompactionConfig {
    warn_at: Ratio,      // 0.70
    compact_at: Ratio,  // 0.85
    summarize_at: Ratio,// 0.92
    force_at: Ratio,     // 0.95
}

impl ContextEngine {
    fn check_token_budget(&self) -> CompactionAction {
        let ratio = self.current_tokens / self.max_tokens;
        
        match ratio {
            r if r >= self.config.force_at => CompactionAction::ForceNewSession,
            r if r >= self.config.summarize_at => CompactionAction::Summarize,
            r if r >= self.config.compact_at => CompactionAction::Compact,
            r if r >= self.config.warn_at => CompactionAction::Warn,
            _ => CompactionAction::None,
        }
    }
}
```

**压缩策略**:

```rust
enum CompactionStrategy {
    // 85% 阈值
    RemoveToolCallFullContent,      // 工具调用只保留摘要
    TrimOldMessages,                // 裁剪旧消息
    CollapseSimilarToolCalls,       // 合并相似工具调用
    
    // 92% 阈值
    AggressiveTrim,                  // 激进裁剪
    KeepOnlyRecentMessages(N),      // 只保留最近 N 条
    ReplaceWithSummary,             // 用摘要替换消息
    
    // 95% 阈值
    ForceNewSession,                // 强制创建新 session
}
```

**用户交互**:

| 级别 | UI 提示 |
|------|---------|
| 70% | 绿色提示：Token 使用 70% |
| 85% | 黄色提示：Token 使用 85%，建议精简上下文 |
| 92% | 橙色提示：即将触发自动摘要 |
| 95% | 红色提示：请创建新 session 继续 |

**配置选项**:

```jsonc
{
  "context": {
    "tokenBudget": 128000,
    "compaction": {
      "warnAt": "70%",
      "compactAt": "85%",
      "summarizeAt": "92%",
      "forceAt": "95%"
    }
  }
}
```

**关联 Constitution**: C-035 (Context Engine 分层)

---

## 6. 功能需求清单汇总 (v8 更新版)

### 6.1 按模块分组

| 模块 | FR 编号 | 需求名称 | 优先级 |
|------|---------|----------|--------|
| core | FR-001 | Context Engine | P0 |
| core | FR-003 | Skills 系统 | P0 |
| core | FR-004 | Commands 系统 | P0 |
| core | FR-012 | Share 功能 | P1 |
| core | FR-014 | 插件事件总线 | P1 |
| core | FR-022 | Session Summarize | P2 |
| core | FR-051 | Compaction 会话压缩 | P2 |
| core | FR-052 | 文件 Watcher 配置 | P2 |
| core | FR-057 | Event Bus 事件总线 | P2 |
| core | FR-058 | Effect System 效果系统 | P2 |
| core | FR-071 | Context Engine 分层上下文 | P1 |
| core | FR-074 | Event Bus 事件类型完整性 | P2 |
| core | FR-075 | Share 服务层 | P2 |
| core | FR-082 | Compaction 自动触发阈值 | P2 |
| core | FR-085 | 观测性 | P2 |
| core | FR-088 | Enterprise 配置 (Central Config + SSO) | P3 |
| core | FR-091 | Session State 状态机完整实现 | P0 |
| core | FR-092 | Plan Agent 工具写限制强制执行 | P1 |
| core | FR-094 | Share 本地导出 (JSON/Markdown) | P1 |
| core | FR-098 | Context Engine 压缩阈值 | P2 |
| core/config | FR-008 | 多层配置合并 | P0 |
| core/config | FR-009 | .opencode 目录加载 | P0 |
| core/config | FR-010 | Provider 环境变量约定 | P0 |
| core/config | FR-021 | 配置系统完善 | P2 |
| core/config | FR-030 | 废弃字段清理 | P2 |
| core/config | FR-033 | OPENCODE_TUI_CONFIG 环境变量 | P0 |
| core/config | FR-034 | TUI 配置分离为独立文件 | P0 |
| core/config | FR-035 | modes/ 目录扫描 | P1 |
| core/config | FR-036 | 配置路径命名统一 | P1 |
| core/config | FR-037 | {file:path} ~ 路径展开 | P1 |
| core/config | FR-038 | {file:path} 相对路径支持 | P1 |
| core/config | FR-039 | .opencode/ 目录扫描集成 | P1 |
| core/config | FR-040 | 变量替换覆盖完整性 | P2 |
| core/config | FR-041 | theme/keybinds 迁移到 TUI | P2 |
| core/config | FR-042 | AgentMapConfig 动态 HashMap | P2 |
| core/config | FR-043 | JSON Schema 远程验证 | P2 |
| core/config | FR-054 | Provider 控制 (disabled/enabled) | P1 |
| core/config | FR-055 | Formatters 自动格式化 | P1 |
| core/config | FR-056 | Instructions 指令文件加载 | P1 |
| core/config | FR-066 | Remote Config 自动发现 | P1 |
| core/config | FR-067 | disabled_providers 优先级 | P1 |
| control-plane | FR-060 | Control Plane / ACP 协议 | P2 |
| control-plane | FR-095 | Enterprise 控制平面 | P1 |
| server | FR-006 | TUI 快捷输入解析器 | P0 |
| server | FR-007 | Session Fork | P0 |
| server | FR-011 | Server API 完善 | P1 |
| server | FR-050 | Server mDNS 服务发现 | P2 |
| server | FR-059 | Streaming 消息架构 | P2 |
| server | FR-076 | SDK 输出 (Rust + TypeScript) | P2 |
| server | FR-077 | OpenAPI 文档自动生成 | P2 |
| server | FR-084 | Server 认证 (HTTP Basic Auth) | P2 |
| server | FR-089 | Provider 管理 API | P0 |
| server | FR-090 | Permission 审批 API | P0 |
| server | FR-093 | Artifact API | P1 |
| server | FR-096 | Web UI 完整实现 | P2 |

### 6.2 按优先级分组

| 优先级 | FR 编号 | 新增 |
|--------|---------|------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010, FR-033, FR-034, FR-063, FR-064, FR-089, FR-090, FR-091 | 3 |
| P1 | FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020, FR-032, FR-035, FR-036, FR-037, FR-038, FR-039, FR-044, FR-045, FR-046, FR-047, FR-048, FR-053, FR-054, FR-055, FR-056, FR-065, FR-066, FR-067, FR-068, FR-069, FR-070, FR-071, FR-072, FR-073, FR-092, FR-093, FR-094, FR-095 | 5 |
| P2 | FR-021, FR-022, FR-023, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029, FR-030, FR-031, FR-040, FR-041, FR-042, FR-043, FR-049, FR-050, FR-051, FR-052, FR-057, FR-058, FR-059, FR-060, FR-061, FR-062, FR-074, FR-075, FR-076, FR-077, FR-078, FR-079, FR-080, FR-081, FR-082, FR-083, FR-084, FR-085, FR-096, FR-097, FR-098 | 4 |
| P3 | FR-087, FR-088 | - |

---

## 7. 实施建议

### Phase 1: P0 阻断性问题 (最高优先级)

1. **FR-089 Provider 管理 API** - API 层补全
2. **FR-090 Permission 审批 API** - API 层补全
3. **FR-091 Session State 状态机** - 核心状态机

### Phase 2: P1 核心功能

1. **FR-092 Plan Agent 工具写限制** - Agent 约束
2. **FR-093 Artifact API** - diff/snapshots/revert
3. **FR-094 Share 本地导出** - 导出能力
4. **FR-095 Enterprise 控制平面** - 企业功能

### Phase 3: P2 完善性

1. **FR-096 Web UI 完整实现**
2. **FR-097 工具单元测试覆盖**
3. **FR-098 Context Engine 压缩阈值**

### Phase 4: P3 远期规划

1. **FR-087 GitHub Integration (v2)**
2. **FR-088 Enterprise 配置**

---

## 8. 附录

### A. 新增 FR 追溯链

```
iteration-8 gap analysis
    │
    ├── P0-1: Provider 管理 API 缺失 → FR-089
    ├── P0-2: Permission 审批 API 缺失 → FR-090
    ├── P0-3: Session State 状态机不完整 → FR-091
    │
    ├── P1-2: Plan Agent 写限制 → FR-092
    ├── P1-3: Artifact API → FR-093
    ├── P1-4: Share 本地导出 → FR-094
    └── P1-5: 企业控制平面 → FR-095
            │
            ├── P2-1: Web UI 完整实现 → FR-096
            ├── P2-2: 工具测试覆盖 → FR-097
            └── P2-4: Context Engine 压缩阈值 → FR-098
```

### B. v7 → v8 变更摘要

| 变更项 | 说明 |
|--------|------|
| 新增 FR | FR-089 ~ FR-100 (12 项) |
| FR-089 | Provider 管理 API (P0) |
| FR-090 | Permission 审批 API (P0) |
| FR-091 | Session State 状态机完整实现 (P0) |
| FR-092 | Plan Agent 工具写限制强制执行 (P1) |
| FR-093 | Artifact API (P1) |
| FR-094 | Share 本地导出 (P1) |
| FR-095 | Enterprise 控制平面 (P1) |
| FR-096 | Web UI 完整实现 (P2) |
| FR-097 | 工具单元测试覆盖 (P2) |
| FR-098 | Context Engine 压缩阈值 (P2) |
| 总计 | 100 项 FR |

---

**文档状态**: 草稿
**下一步**: 基于本规格文档创建 iteration-9 实施计划
