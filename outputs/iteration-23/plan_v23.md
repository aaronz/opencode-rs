# OpenCode-RS Implementation Plan v2.3

**版本:** 2.3 (Iteration-23)  
**日期:** 2026年4月8日  
**基于:** spec_v23.md, gap-analysis.md, constitution_updates.md  
**状态:** 进行中

---

## 1. 执行摘要

基于 v23 差距分析，本迭代聚焦于 **P0 阻断性问题** 和 **P1 高优先级问题**。

### 关键阻断项

| ID | 功能 | 状态 | 工作量 | 备注 |
|----|------|------|--------|------|
| FR-222 | Rust SDK | ❌ Missing | High | P0 阻断性 |
| FR-223 | TypeScript SDK | ❌ Missing | High | P0 阻断性 |
| FR-226 | 敏感文件默认拒绝 | ❌ Missing | Medium | P0 安全合规 |
| FR-227 | external_directory 拦截 | ❌ Missing | Medium | P0 安全合规 |

### 总体进度目标

```
Iteration-23 目标: 65% → 80%
P0 阻断项: 4 项 → 0 项
P1 高优先级: 14 项 → 完成 10+ 项
```

---

## 2. P0 阻断性问题解决方案

### 2.1 SDK 实现 (FR-222, FR-223)

#### 2.1.1 Rust SDK (`opencode-sdk`)

**Crate 结构:**

```
opencode-sdk/
├── Cargo.toml
├── src/
│   ├── lib.rs              # SDK 入口
│   ├── client.rs           # OpenCodeClient 主客户端
│   ├── session.rs          # Session 管理
│   ├── tools.rs            # 工具调用
│   ├── config.rs           # 配置管理
│   ├── auth.rs             # 认证
│   ├── error.rs            # SDK 错误类型
│   └── async_runtime.rs    # Tokio 集成
└── examples/
    └── basic.rs            # 基本使用示例
```

**核心 API:**

```rust
// 必须暴露的 API
pub struct OpenCodeClient {
    pub async fn new(config: ClientConfig) -> Result<Self>;
    pub async fn session_create(&self) -> Result<SessionId>;
    pub async fn session_send(&self, id: SessionId, message: Message) -> Result<Message>;
    pub async fn session_load(&self, id: SessionId) -> Result<Session>;
    pub async fn session_save(&self, id: SessionId) -> Result<()>;
    pub async fn session_fork(&self, id: SessionId) -> Result<SessionId>;
    pub async fn session_abort(&self, id: SessionId) -> Result<()>;
}

#[derive(Clone)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub timeout: Duration,
}
```

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| SDK-001 | 创建 `opencode-sdk` crate 骨架 | 无 | 0.5d |
| SDK-002 | 实现 `OpenCodeClient` 核心结构 | SDK-001 | 1d |
| SDK-003 | 实现 Session API (create/load/save/fork/abort) | SDK-002 | 1d |
| SDK-004 | 实现 Tools API (execute/list) | SDK-002 | 1d |
| SDK-005 | 实现 Error 类型 (映射 1xxx-9xxx 错误码) | SDK-002 | 0.5d |
| SDK-006 | 实现 Auth 集成 | SDK-002 | 0.5d |
| SDK-007 | 编写 examples/basic.rs 示例 | SDK-003~006 | 0.5d |
| SDK-008 | 添加 cargo doc 和发布配置 | SDK-007 | 0.5d |

**参考实现:**
- `opencode-server` REST API (`crates/server/src/`)
- `opencode-core` Session 管理 (`crates/core/src/session/`)

#### 2.1.2 TypeScript SDK (`@opencode/sdk`)

**Package 结构:**

```
@opencode/sdk/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts            # SDK 入口
│   ├── client.ts           # OpenCodeClient 主客户端
│   ├── session.ts          # Session 管理
│   ├── tools.ts            # 工具调用
│   ├── config.ts           # 配置管理
│   ├── auth.ts             # 认证
│   └── types.ts            # 类型定义
└── examples/
    └── basic.ts            # 基本使用示例
```

**核心 API:**

```typescript
interface OpenCodeClient {
  session.create(): Promise<string>;  // returns sessionId
  session.send(id: string, message: Message): Promise<Message>;
  session.load(id: string): Promise<Session>;
  session.save(id: string): Promise<void>;
  session.fork(id: string): Promise<string>;  // returns new sessionId
  session.abort(id: string): Promise<void>;
  tools.execute(name: string, args: Record<string, unknown>): Promise<ToolResult>;
  tools.list(): Promise<ToolDefinition[]>;
}

interface ClientConfig {
  apiKey?: string;
  baseUrl: string;
  timeout?: number;  // ms
}
```

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| TS-001 | 初始化 npm package 骨架 | 无 | 0.5d |
| TS-002 | 实现 `OpenCodeClient` 核心结构 | TS-001 | 1d |
| TS-003 | 实现 Session API | TS-002 | 1d |
| TS-004 | 实现 Tools API | TS-002 | 1d |
| TS-005 | 实现类型定义 (.d.ts) | TS-002 | 0.5d |
| TS-006 | 实现 Node.js 和浏览器兼容 | TS-003~005 | 0.5d |
| TS-007 | 编写 examples/basic.ts 示例 | TS-006 | 0.5d |
| TS-008 | 配置发布到 npm | TS-007 | 0.5d |

**参考实现:**
- Server REST API (`crates/server/src/`)
- 现有 TypeScript 类型定义

---

### 2.2 敏感文件安全 (FR-226, FR-227)

#### 2.2.1 实现方案

**敏感文件模式:**

| 文件类型 | 默认行为 | 覆盖要求 |
|----------|----------|----------|
| `.env` | **DENY** | Yes (explicit allow) |
| `*.pem`, `*.key` | **DENY** | Yes |
| `credentials.json` | **DENY** | Yes |
| `secrets.*` | **DENY** | Yes |

**实现位置:** `crates/permission/src/`

**核心实现:**

```rust
// crates/permission/src/sensitive_file.rs

const SENSITIVE_PATTERNS: &[&str] = &[
    ".env",
    ".env.local",
    ".env.production",
    "credentials.json",
    "secrets.toml",
    "id_rsa",
    "id_ed25519",
];

const SENSITIVE_EXTENSIONS: &[&str] = &[
    ".pem",
    ".key",
    ".crt",
    ".p12",
    ".pfx",
];

pub fn is_sensitive_path(path: &Path) -> bool {
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Exact matches
    if SENSITIVE_PATTERNS.iter().any(|p| file_name == *p) {
        return true;
    }

    // Extension matches
    if SENSITIVE_EXTENSIONS.iter().any(|ext| file_name.ends_with(ext)) {
        return true;
    }

    // Pattern matches (contains "secret" or "credentials")
    if file_name.contains("secret") || file_name.contains("credentials") {
        return true;
    }

    false
}
```

**Permission 检查集成:**

```rust
// crates/permission/src/checker.rs

pub fn check_file_permission(
    path: &Path,
    permission: &Permission,
    config: &PermissionConfig,
) -> Result<PermissionResult> {
    // Check sensitive file first
    if is_sensitive_path(path) && config.external_directory_enabled {
        return Ok(PermissionResult::Denied {
            reason: "Sensitive file access requires explicit allow".to_string(),
        });
    }

    // ... existing checks
}
```

#### 任务拆解

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| SEC-001 | 创建 `sensitive_file.rs` 模块 | 无 | 0.5d |
| SEC-002 | 实现 `is_sensitive_path()` 函数 | SEC-001 | 0.5d |
| SEC-003 | 集成到 permission checker | SEC-002 | 0.5d |
| SEC-004 | 实现 `external_directory` 拦截 | SEC-003 | 0.5d |
| SEC-005 | 添加测试 `test_env_file_denied_by_default` | SEC-004 | 0.5d |
| SEC-006 | 更新文档和 AGENTS.md | SEC-005 | 0.5d |

**参考实现:**
- `crates/permission/src/lib.rs`
- `crates/permission/src/checker.rs`

---

## 3. P1 高优先级问题解决方案

### 3.1 Session Fork Lineage (FR-220, FR-221)

**目标:** 支持完整的 fork 历史追溯

**实现方案:**

```rust
// Session metadata 更新
struct SessionMetadata {
    parent_session_id: Option<SessionId>,  // Fork source
    lineage_path: Vec<SessionId>,           // Full ancestry path
    fork_timestamp: DateTime<Utc>,
}

// lineage_path 计算逻辑
fn compute_lineage_path(parent: &Option<Session>, new_id: SessionId) -> Vec<SessionId> {
    match parent {
        Some(p) => {
            let mut path = p.metadata.lineage_path.clone();
            path.push(p.id);
            path
        }
        None => vec![],
    }
}
```

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| LIN-001 | 更新 SessionMetadata 结构 | 无 | 0.5d |
| LIN-002 | 实现 `compute_lineage_path()` | LIN-001 | 0.5d |
| LIN-003 | 更新 session_fork 逻辑 | LIN-002 | 0.5d |
| LIN-004 | 添加数据库迁移 (如有) | LIN-003 | 0.5d |
| LIN-005 | 添加测试 | LIN-004 | 0.5d |

---

### 3.2 LSP Definition/References (FR-236, FR-237)

**目标:** 实现完整的 LSP v1.1 能力

**实现位置:** `crates/lsp/src/`

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| LSP-001 | 实现 `lsp_definition` 工具 | 无 | 1d |
| LSP-002 | 实现 `lsp_references` 工具 | LSP-001 | 1d |
| LSP-003 | 添加 timeout 和重试机制 | LSP-002 | 0.5d |
| LSP-004 | 添加测试覆盖 | LSP-003 | 0.5d |

**参考实现:**
- `crates/lsp/src/`
- `crates/tools/src/lsp_tool.rs` (现有 diagnostics/symbols)

---

### 3.3 MCP Connection Pooling (FR-240)

**目标:** 实现连接池、timeout、重试机制

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| MCP-001 | 设计连接池结构 | 无 | 1d |
| MCP-002 | 实现 timeout 机制 | MCP-001 | 0.5d |
| MCP-003 | 实现重试逻辑 | MCP-002 | 0.5d |
| MCP-004 | 添加连接状态管理 | MCP-003 | 0.5d |
| MCP-005 | 测试连接稳定性 | MCP-004 | 1d |

---

### 3.4 Plugin Event Bus (FR-242)

**目标:** 实现完整事件总线

**Event Types:**

| Event | Payload | Description |
|-------|---------|-------------|
| `session.created` | `{ session_id: string }` | 新会话创建 |
| `session.resumed` | `{ session_id: string }` | 会话恢复 |
| `session.saved` | `{ session_id: string }` | 会话保存 |
| `tool.executed` | `{ tool: string, duration_ms: u64 }` | 工具执行 |
| `tool.approved` | `{ tool: string }` | 工具权限批准 |
| `tool.denied` | `{ tool: string }` | 工具权限拒绝 |
| `llm.request` | `{ provider: string, model: string }` | LLM 请求 |
| `llm.response` | `{ tokens_used: u64 }` | LLM 响应 |

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| PLG-001 | 设计 EventBus trait | 无 | 0.5d |
| PLG-002 | 实现内存 EventBus | PLG-001 | 1d |
| PLG-003 | 实现 WASM FFI 桥接 | PLG-002 | 1d |
| PLG-004 | 在核心模块中埋点发布事件 | PLG-003 | 1d |
| PLG-005 | 添加 Plugin 集成测试 | PLG-004 | 0.5d |

---

### 3.5 SSE/WebSocket 稳定性 (FR-234, FR-235)

**目标:** 修复 SSE 和 WebSocket 连接不稳定问题

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| SSE-001 | 实现 SSE heartbeat (30s interval) | 无 | 0.5d |
| SSE-002 | 实现客户端重连逻辑 | SSE-001 | 0.5d |
| SSE-003 | 修复 WebSocket handshake | SSE-002 | 1d |
| SSE-004 | 添加连接状态监控 | SSE-003 | 0.5d |
| SSE-005 | 压力测试验证 | SSE-004 | 1d |

---

### 3.6 Credential Encryption (FR-229, FR-271)

**目标:** 凭据加密存储

**方案:** AES-256-GCM + Argon2 key derivation

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| CRYPT-001 | 实现 `CredentialStore` trait | 无 | 1d |
| CRYPT-002 | 实现 AES-256-GCM 加密 | CRYPT-001 | 1d |
| CRYPT-003 | 实现 Argon2 key derivation | CRYPT-002 | 0.5d |
| CRYPT-004 | 集成到 auth 模块 | CRYPT-003 | 0.5d |
| CRYPT-005 | 添加 security tests | CRYPT-004 | 0.5d |

---

### 3.7 Error Code System (FR-259-266)

**目标:** 完整的 1xxx-9xxx 错误代码体系

**错误码映射:**

| Range | Category | Examples |
|-------|----------|----------|
| 1xxx | Authentication | `1001` API key invalid, `1002` token expired |
| 2xxx | Authorization | `2001` permission denied, `2002` scope insufficient |
| 3xxx | Provider | `3001` model unavailable, `3002` rate limited |
| 4xxx | Tool | `4001` tool not found, `4002` tool execution failed |
| 5xxx | Session | `5001` session not found, `5002` session corrupt |
| 6xxx | Config | `6001` config parse error, `6002` config missing field |
| 7xxx | Validation | `7001` invalid arguments, `7002` field too long |
| 8xxx | MCP | `8001` MCP connection failed, `8002` MCP timeout |
| 9xxx | Internal | `9001` panic caught, `9002` assertion failed |

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| ERR-001 | 更新 `crates/core/src/error.rs` | 无 | 1d |
| ERR-002 | 添加 1xxx Authentication 错误 | ERR-001 | 0.5d |
| ERR-003 | 添加 2xxx Authorization 错误 | ERR-001 | 0.5d |
| ERR-004 | 添加 3xxx Provider 错误 | ERR-001 | 0.5d |
| ERR-005 | 添加 4xxx Tool 错误 | ERR-001 | 0.5d |
| ERR-006 | 添加 5xxx Session 错误 | ERR-001 | 0.5d |
| ERR-007 | 添加 6xxx Config 错误 | ERR-001 | 0.5d |
| ERR-008 | 添加 7xxx Validation 错误 | ERR-001 | 0.5d |
| ERR-009 | 添加 8xxx MCP 错误 | ERR-001 | 0.5d |
| ERR-010 | 添加 9xxx Internal 错误 | ERR-001 | 0.5d |
| ERR-011 | 更新 SDK Error 类型映射 | ERR-010 | 0.5d |
| ERR-012 | 验证所有错误码不冲突 | ERR-011 | 0.5d |

---

### 3.8 Context Compaction (FR-256, FR-257)

**目标:** 精确的 token budget 和 compaction 阈值

**阈值定义:**

```rust
const COMPACTION_WARN_THRESHOLD: f32 = 0.85;
const COMPACTION_START_THRESHOLD: f32 = 0.92;
const COMPACTION_FORCE_THRESHOLD: f32 = 0.95;
```

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| CTX-001 | 校准 tiktoken-rs token 计数 | 无 | 1d |
| CTX-002 | 实现精确阈值常量 | CTX-001 | 0.5d |
| CTX-003 | 更新 compaction 触发逻辑 | CTX-002 | 1d |
| CTX-004 | 添加性能测试 | CTX-003 | 0.5d |

---

### 3.9 Crash Recovery (FR-267)

**目标:** 崩溃转储机制

**任务拆解:**

| Task | 描述 | 依赖 | 工作量 |
|------|------|------|--------|
| CR-001 | 设计崩溃转储格式 | 无 | 0.5d |
| CR-002 | 实现 panic handler | CR-001 | 1d |
| CR-003 | 实现 session 状态保存 | CR-002 | 1d |
| CR-004 | 实现恢复逻辑 | CR-003 | 1d |
| CR-005 | 测试崩溃恢复流程 | CR-004 | 1d |

---

## 4. P2 中优先级问题 (计划纳入后续迭代)

| ID | 功能 | 预计工作量 | 建议迭代 |
|----|------|-----------|----------|
| FR-238 | LSP hover | Low | v24 |
| FR-239 | LSP code_actions | Medium | v24 |
| FR-241 | MCP OAuth | High | v24 |
| FR-272 | PKCE Support | Medium | v24 |
| FR-273 | Token Refresh/Revoke | Medium | v24 |
| FR-230 | TUI @ 多选 | Medium | v24 |
| FR-232 | TUI Diff 可展开 | Medium | v24 |
| FR-252/253 | Share JSON/Markdown Export | Low | v24 |
| FR-254 | Patch Bundle | Medium | v24 |
| FR-255 | Self-hosted Share Server | High | v25 |
| FR-258 | Summary Quality | Medium | v24 |
| FR-268 | Tool Spans | Medium | v25 |
| FR-269 | Cost Calculator | Medium | v24 |
| FR-274 | Export Auth Isolation | Medium | v24 |
| FR-275 | Enterprise Policy Profile | High | v25 |
| FR-276 | Windows Support | High | v25 |

---

## 5. 技术债务处理

| ID | 债务项 | 优先级 | 建议 |
|----|--------|--------|------|
| T1 | opencode-core 单一职责膨胀 | 高 | v24 开始拆分 domain 模块 |
| T2 | thiserror vs anyhow 混用 | 低 | 统一为 thiserror |
| T6 | 日志脱敏不完整 | 中 | 结合 SEC-001~006 处理 |
| T7 | 配置字段别名处理 | 低 | 重构 config 模块 |
| T8 | 错误处理不一致 | 中 | 结合 ERR-001~012 处理 |
| T10 | 依赖版本未锁定 | 中 | 使用 `version = "=1.0.0"` |
| T11 | Dead code 清理 | 低 | 持续清理 |
| T12 | Binary size 优化 | 中 | v24 进行专项优化 |

---

## 6. 迭代任务总览

### 6.1 Iteration-23 任务清单

#### P0 阻断性问题 (4 项)

| # | Task ID | 任务 | 工作量 | 负责人 |
|---|---------|------|--------|--------|
| 1 | SDK-001~008 | Rust SDK 实现 | 5.5d | - |
| 2 | TS-001~008 | TypeScript SDK 实现 | 5.5d | - |
| 3 | SEC-001~006 | 敏感文件默认拒绝 | 3d | - |
| 4 | SEC-001~006 | external_directory 拦截 | 0.5d (与上面合并) | - |

#### P1 高优先级问题 (14 项)

| # | Task ID | 任务 | 工作量 | 备注 |
|---|---------|------|--------|------|
| 5 | LIN-001~005 | Session Fork Lineage | 2.5d | |
| 6 | LSP-001~004 | LSP Definition/References | 3.5d | |
| 7 | MCP-001~005 | MCP Connection Pooling | 3.5d | |
| 8 | PLG-001~005 | Plugin Event Bus | 4d | |
| 9 | SSE-001~005 | SSE/WebSocket 稳定性 | 3.5d | |
| 10 | CRYPT-001~005 | Credential Encryption | 3.5d | |
| 11 | ERR-001~012 | Error Code System | 5.5d | |
| 12 | CTX-001~004 | Context Compaction | 3d | |
| 13 | CR-001~005 | Crash Recovery | 4.5d | |

#### 测试覆盖率提升

| # | Task ID | 任务 | 工作量 | 备注 |
|---|---------|------|--------|------|
| 14 | TEST-001 | opencode-permission 测试 | 1d | 安全测试 |
| 15 | TEST-002 | opencode-tools 测试 | 1d | |
| 16 | TEST-003 | opencode-session 测试 | 1d | |

---

### 6.2 估算工作量汇总

| 类别 | 任务数 | 估算工期 |
|------|--------|----------|
| P0 阻断性问题 | 4 | 14.5d |
| P1 高优先级问题 | 9 | 33.5d |
| 测试覆盖率 | 3 | 3d |
| **合计** | **16** | **~50d (10人日/人)** |

**建议:** 分配 2-3 人并行处理

---

### 6.3 依赖关系图

```
SDK-001 ─┬─ SDK-002 ─┬─ SDK-003 ─┬─ SDK-007 ─┬─ SDK-008
         │           │           │           │
         │           │           └─ SDK-004 ─┴─ (并行)
         │           │
         │           └─ SDK-005 ─┴─ SDK-006

TS-001 ─┬─ TS-002 ─┬─ TS-003 ─┬─ TS-007 ─┬─ TS-008
         │           │           │           │
         │           │           └─ TS-004 ─┴─ (并行)
         │           │
         │           └─ TS-005 ─┬─ TS-006

SEC-001 ─┬─ SEC-002 ─┬─ SEC-003 ─┬─ SEC-004 ─┬─ SEC-005 ─┬─ SEC-006
         │           │           │           │           │
         └───────────┴───────────┴───────────┴───────────┘ (串行)

LIN-001 ─┬─ LIN-002 ─┬─ LIN-003 ─┬─ LIN-004 ─┬─ LIN-005

LSP-001 ─┬─ LSP-002 ─┬─ LSP-003 ─┬─ LSP-004

MCP-001 ─┬─ MCP-002 ─┬─ MCP-003 ─┬─ MCP-004 ─┬─ MCP-005

PLG-001 ─┬─ PLG-002 ─┬─ PLG-003 ─┬─ PLG-004 ─┬─ PLG-005

SSE-001 ─┬─ SSE-002 ─┬─ SSE-003 ─┬─ SSE-004 ─┬─ SSE-005

CRYPT-001 ─┬─ CRYPT-002 ─┬─ CRYPT-003 ─┬─ CRYPT-004 ─┬─ CRYPT-005

ERR-001 ─┬─ ERR-002 ~ ERR-010 (并行) ─┬─ ERR-011 ─┬─ ERR-012
         └─────────────────────────────┘

CTX-001 ─┬─ CTX-002 ─┬─ CTX-003 ─┬─ CTX-004

CR-001 ─┬─ CR-002 ─┬─ CR-003 ─┬─ CR-004 ─┬─ CR-005
```

---

## 7. 验收标准

### 7.1 P0 验收

- [ ] `cargo test -p opencode-sdk` 全部通过
- [ ] `@opencode/sdk` npm 包可正常安装使用
- [ ] `.env` 文件读取被默认拒绝
- [ ] `external_directory` 路径下敏感文件被拦截

### 7.2 P1 验收

- [ ] `session_fork` 返回正确的 `lineage_path`
- [ ] `lsp_definition` 和 `lsp_references` 工具正常工作
- [ ] MCP 连接稳定性 > 99% (100次连接测试)
- [ ] Plugin 可订阅 `session.created` 等事件
- [ ] SSE 心跳正常，客户端可自动重连
- [ ] WebSocket 握手稳定
- [ ] 凭据以加密形式存储 (验证: 解密后可正常读取)
- [ ] 所有错误码按 1xxx-9xxx 分类
- [ ] Context compaction 阈值精确触发
- [ ] 崩溃后 session 可恢复

### 7.3 测试覆盖率验收

| Crate | 目标覆盖率 | 当前覆盖率 |
|-------|-----------|-----------|
| opencode-core | 70% | ~40% |
| opencode-permission | 80% | ~40% |
| opencode-tools | 70% | ~30% |
| opencode-sdk | 80% | N/A |

---

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| SDK 实现复杂度高 | 时间延误 | 优先实现核心 API，扩展功能延后 |
| 安全测试覆盖不足 | 遗漏漏洞 | 安排专项安全 review |
| 依赖重构影响稳定性 | 破坏现有功能 | 充分 CI 测试，灰度发布 |
| 并行任务协调 | 冲突 | 明确模块边界，定期 sync |

---

## 9. 下一步行动

1. **立即开始 (Day 1):**
   - 创建 `opencode-sdk` crate 骨架
   - 创建 `sensitive_file.rs` 模块

2. **本周完成:**
   - SDK 核心 API 实现
   - 敏感文件安全实现

3. **两周内完成:**
   - 所有 P0 阻断项
   - 所有 P1 高优先级项 (14 项中的 10+ 项)

---

**文档状态:** 进行中  
**下次审查:** Iteration-24 开始时  
**变更要求:** 通过 PR 进行代码审查