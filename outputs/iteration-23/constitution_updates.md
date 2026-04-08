# Constitution 更新建议报告

**项目**: OpenCode-RS  
**分析日期**: 2026-04-08  
**源文件**: Constitution v1.0 (Master + C-024/C-055/C-056/C-057/C-058)  
**状态**: 草稿

---

## 1. 执行摘要

根据 v23 差距分析，现有 Constitution 覆盖了约 **65%** 的关键需求，**缺失以下 P0/P1 领域的正式规范**：

| 缺失领域 | 严重程度 | 当前状态 |
|----------|----------|----------|
| SDK 设计要求 | P0 | ❌ 完全缺失 |
| 敏感文件安全 (`.env` 默认 deny) | P0 | ❌ 完全缺失 |
| Plugin 事件总线 | P1 | ❌ 完全缺失 |
| WASM 沙箱隔离 | P1 | ❌ 完全缺失 |
| SSE/WebSocket 稳定性要求 | P1 | ❌ 完全缺失 |
| 错误代码完整体系 (1xxx-9xxx) | P1 | ❌ 完全缺失 |
| Context Compaction 阈值 | P1 | ❌ 完全缺失 |
| 凭据加密存储 | P1 | ❌ 完全缺失 |
| Session Fork Lineage | P1 | ⚠️ 部分覆盖 (C-024) |
| LSP Definition/References | P1 | ⚠️ C-058 有定义但实现缺失 |

---

## 2. 需要新增的 Constitution 文档

### 2.1 C-059: SDK Design Requirements (P0)

**建议位置**: `outputs/iteration-23/constitution/C-059_sdk_design.md`

**内容要点**:

```markdown
## Article 1: SDK Scope

| SDK | Language | Priority | Target Version |
|-----|----------|----------|----------------|
| `opencode-sdk` | Rust | P0 | v0.2 |
| `@opencode/sdk` | TypeScript | P0 | v0.2 |

## Article 2: Rust SDK Requirements

### Section 2.1: Core API Surface

```rust
// 必须暴露的 API
pub struct OpenCodeClient {
    pub async fn new(config: ClientConfig) -> Result<Self>;
    pub async fn session_create(&self) -> Result<SessionId>;
    pub async fn session_send(&self, id: SessionId, message: Message) -> Result<Message>;
    pub async fn session_load(&self, id: SessionId) -> Result<Session>;
    pub async fn session_save(&self, id: SessionId) -> Result<()>;
}
```

### Section 2.2: Error Handling

所有 SDK 方法必须返回 `Result<T, OpenCodeSDKError>`，错误码对应核心错误体系。

## Article 3: TypeScript SDK Requirements

### Section 3.1: API Surface

```typescript
interface OpenCodeClient {
  session.create(): Promise<string>;  // returns sessionId
  session.send(id: string, message: Message): Promise<Message>;
  session.load(id: string): Promise<Session>;
}
```

### Section 3.2: Browser Compatibility

- 必须支持 Node.js 18+ 和浏览器环境
- 浏览器环境使用 `fetch` API
- 类型定义必须完整 (`.d.ts`)
```

---

### 2.2 C-060: Sensitive File Security (P0)

**建议位置**: `outputs/iteration-23/constitution/C-060_sensitive_file_security.md`

**内容要点**:

```markdown
## Article 1: Default Security Posture

| File Type | Default Behavior | Override Required |
|-----------|------------------|-------------------|
| `.env` | **DENY** | Yes (explicit allow) |
| `*.pem`, `*.key` | **DENY** | Yes |
| `credentials.json` | **DENY** | Yes |
| `secrets.*` | **DENY** | Yes |

## Article 2: Permission Check Implementation

```rust
fn is_sensitive_path(path: &Path) -> bool {
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    // Exact matches
    if file_name == ".env" { return true; }
    
    // Extension matches
    if file_name.ends_with(".pem") || file_name.ends_with(".key") {
        return true;
    }
    
    // Pattern matches
    if file_name.contains("secret") || file_name.contains("credentials") {
        return true;
    }
    
    false
}
```

## Article 3: External Directory Protection

所有 `external_directory` 配置下的路径必须经过敏感文件检查，参考 Article 1。
```

---

### 2.3 C-061: Plugin Event Bus (P1)

**建议位置**: `outputs/iteration-23/constitution/C-061_plugin_event_bus.md`

**内容要点**:

```markdown
## Article 1: Event Types

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

## Article 2: Event Bus API

```rust
pub trait EventBus: Send + Sync {
    fn subscribe(&self, event: EventType, handler: EventHandler) -> SubscriptionId;
    fn unsubscribe(&self, id: SubscriptionId) -> Result<()>;
    fn publish(&self, event: Event) -> Result<()>;
}
```

## Article 3: WASM Plugin Integration

Plugin 必须通过 `wasmer` 或 `wasmtime` 沙箱隔离，事件总线通过 FFI 传递。
```

---

### 2.4 C-062: Error Code System (P1)

**建议位置**: `outputs/iteration-23/constitution/C-062_error_code_system.md`

**内容要点**:

```markdown
## Article 1: Error Code Ranges

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

## Article 2: Error Response Format

```rust
#[derive(Serialize)]
struct ErrorResponse {
    code: u32,
    message: String,
    detail: Option<String>,
}
```

## Article 3: Implementation Requirements

所有 `OpenCodeError` 变体必须映射到对应错误码，不得使用 `0` 或未定义错误码。
```

---

### 2.5 C-063: Context Compaction Thresholds (P1)

**建议位置**: `outputs/iteration-23/constitution/C-063_context_compaction.md`

**内容要点**:

```markdown
## Article 1: Token Budget Thresholds

| Threshold | Trigger | Action |
|-----------|---------|--------|
| 85% | `context_length * 0.85` | Warning log, 继续对话 |
| 92% | `context_length * 0.92` | 开始 compact 低优先级消息 |
| 95% | `context_length * 0.95` | 强制 compact, 暂停新消息 |
| 100% | `context_length` | 拒绝新消息, 必须 compact |

## Article 2: Compaction Algorithm

1. 识别所有 `UserMessage` 和 `AssistantMessage`
2. 按时间排序，标记最旧的消息
3. 移除 `SystemMessage` 以外的非关键消息
4. 生成 summary 替换被移除的消息

## Article 3: Constants

```rust
const COMPACTION_WARN_THRESHOLD: f32 = 0.85;
const COMPACTION_START_THRESHOLD: f32 = 0.92;
const COMPACTION_FORCE_THRESHOLD: f32 = 0.95;
```
```

---

### 2.6 C-064: Credential Encryption (P1)

**建议位置**: `outputs/iteration-23/constitution/C-064_credential_encryption.md`

**内容要点**:

```markdown
## Article 1: Credential Storage

| Storage Location | Encryption Required |
|------------------|---------------------|
| `auth_store.json` | AES-256-GCM |
| Session credentials | Encrypted before persist |
| Environment variables | NOT persisted (in-memory only) |

## Article 2: Key Management

- Master key derived from user password via Argon2
- Salt stored separately from ciphertext
- Each credential has unique IV

## Article 3: Implementation

```rust
pub trait CredentialStore {
    fn save(&self, key: &str, value: &str) -> Result<()>;
    fn load(&self, key: &str) -> Result<String>;
    fn delete(&self, key: &str) -> Result<()>;
}
```
```

---

## 3. 需要修订的现有文档

### 3.1 C-024: Session Tools Permission (修订)

**当前状态**: 仅覆盖 `session_load` 和 `session_save`

**建议补充**:
- 添加 `session_fork` 权限要求
- 添加 `parent_session_id` lineage 追溯要求
- 明确 fork lineage 字段规范

```markdown
## Article 7: Session Fork

### Section 7.1: Fork Permission

`session_fork` 属于 Write Tool，需要 `Full` scope。

### Section 7.2: Lineage Tracking

所有 fork 操作必须在 session metadata 中记录:
```rust
struct SessionMetadata {
    parent_session_id: Option<SessionId>,  // Fork source
    lineage_path: Vec<SessionId>,          // Full ancestry path
    fork_timestamp: DateTime<Utc>,
}
```
```

---

### 3.2 C-055: Test Coverage Requirements (修订)

**当前状态**: 基础目标已定义，但覆盖不足

**建议补充**:
- 新增 P0/P1 功能测试要求
- 新增 SDK 测试要求
- 新增安全测试要求 (敏感文件 deny)

```markdown
## Article 8: Security Testing

### Section 8.1: Required Security Tests

| Test | Description |
|------|-------------|
| `test_env_file_denied_by_default` | Verify .env access is denied |
| `test_credential_encryption` | Verify credentials are encrypted at rest |
| `test_permission_escalation_blocked` | Verify scope escalation is prevented |
```

---

### 3.3 C-058: LSP Capabilities (修订)

**当前状态**: 定义了 LSP 架构但实现不完整

**建议补充**:
- 明确 `lsp_definition` 和 `lsp_references` 必须实现
- 添加 timeout 重试机制要求
- 添加连接池管理要求

```markdown
## Article 8: Implementation Requirements

### Section 8.1: Mandatory Methods

以下方法必须在当前版本实现:
- [x] `lsp_diagnostics` - 已实现
- [x] `lsp_symbols` - 已实现
- [ ] `lsp_definition` - **必须实现**
- [ ] `lsp_references` - **必须实现**

### Section 8.2: Connection Management

- 每个 LSP 连接必须有 5 秒 timeout
- 连接失败后自动重试最多 3 次
- 维护连接池 (最大 5 个并发连接)
```

---

## 4. Constitution 主体修订

### 4.1 Article 2: Foundational Principles (修订)

**建议在 Section 2.1 Code Quality 中新增**:

```markdown
### Section 2.1: Code Quality

1. **Type Safety First**: Never suppress type errors with `as any`, `@ts-ignore`, `@ts-expect-error`, or `unsafe`
2. **Error Handling**: Never use empty catch blocks `catch(e) {}`
3. **Testing**: Never delete failing tests to "pass" - fix the underlying issue
4. **Documentation**: Public APIs MUST have doc comments; internal code should be self-documenting
5. **Security**: Sensitive files (`.env`, `*.pem`, `*.key`, `secrets.*`) MUST be denied by default
6. **Error Codes**: All errors MUST use structured error codes from the 1xxx-9xxx range per C-062
```

**建议在 Section 2.2 Architectural Boundaries 中新增**:

```markdown
| Boundary | Principle |
|----------|-----------|
| Core ↔ Tools | Core is dependency-free; Tools depend on Core |
| Server ↔ Agent | Server handles HTTP; Agent handles execution |
| Permission | Separate crate (`opencode-permission`) with clear API |
| Storage | Abstracted behind `StorageService` trait |
| Plugin ↔ Runtime | Plugin runs in WASM sandbox; isolated from main runtime |
```

---

### 4.2 Article 3: Implementation Standards (修订)

**建议新增 Section 3.5: SDK Requirements**:

```markdown
### Section 3.5: SDK Requirements

**Reference**: C-059

| SDK | Crate | Language | Status |
|-----|-------|----------|--------|
| Rust SDK | `opencode-sdk` | Rust | Required v0.2 |
| TS SDK | `@opencode/sdk` | TypeScript | Required v0.2 |

All SDKs must:
- Have complete type definitions
- Return structured errors with error codes
- Support async/await patterns
- Include usage examples
```

**建议新增 Section 3.6: Security Requirements**:

```markdown
### Section 3.6: Security Requirements

**Reference**: C-060, C-064

1. **Sensitive File Access**:
   - `.env` files MUST be denied by default
   - `*.pem`, `*.key` files MUST be denied by default
   - Override requires explicit `allow` in config

2. **Credential Storage**:
   - All credentials MUST be encrypted at rest (AES-256-GCM)
   - Master key derived from user password (Argon2)
   - No plaintext credentials in logs or exports
```

**建议新增 Section 3.7: Protocol Stability**:

```markdown
### Section 3.7: Protocol Stability

**Reference**: C-057

1. **SSE Heartbeat**:
   - Server MUST send heartbeat every 30 seconds
   - Client reconnect on missing 2 heartbeats

2. **WebSocket**:
   - Proper handshake per RFC 6455
   - Clean close with status code
   - Automatic reconnection with exponential backoff
```

---

### 4.3 Article 4: Testing Requirements (修订)

**建议新增 Section 4.4: Security Tests**:

```markdown
### Section 4.4: Security Tests (Required)

| Crate | Required Security Tests |
|-------|-------------------------|
| `opencode-permission` | Sensitive file deny, escalation blocked |
| `opencode-storage` | Credential encryption verified |
| `opencode-auth` | Token encryption, refresh flow |
| `opencode-tools` | Dangerous tool isolation |
```

---

### 4.4 Article 6: Technical Debt Management (修订)

**建议更新 T1-T5 状态并新增**:

```markdown
### Section 6.1: Known Technical Debt

| ID | Description | Risk | Status |
|----|-------------|------|--------|
| T1 | Config format inconsistency | High | Mitigated by C-056 |
| T2 | README outdated | Medium | Pending |
| T3 | Permission routing unclear | Medium | Mitigated by C-024 |
| T4 | TUI permission confirmation | Low | Accepted |
| T5 | auth_layered not integrated | Low | Pending |
| T6 | SDK not implemented | **High** | **NEW - P0** |
| T7 | Sensitive file .env not protected | **High** | **NEW - P0** |
| T8 | LSP definition/references missing | **High** | **NEW - P1** |
| T9 | Plugin event bus not implemented | **High** | **NEW - P1** |
| T10 | Credential encryption missing | **High** | **NEW - P1** |
| T11 | SSE heartbeat not implemented | Medium | **NEW - P1** |
| T12 | Error code system incomplete | Medium | **NEW - P1** |
```

---

## 5. Constitution Hierarchy 更新建议

**建议在 Article 1: Source of Authority 中新增**:

```markdown
## Article 1: Source of Authority

This constitution is derived from and supersedes all previous iteration-specific design documents. Individual constitution documents provide detailed specifications for their respective domains.

### Section 1.1: Constitution Hierarchy

| Level | Document | Authority |
|-------|----------|-----------|
| Master | `outputs/.specify/memory/constitution.md` | This file |
| Domain | `outputs/iteration-23/constitution/C-0*.md` | Detailed specs |

### Section 1.2: Incorporation by Reference

The following constitution documents are fully incorporated herein:

| ID | Title | Description | Status |
|----|-------|-------------|--------|
| C-024 | Session Tools Permission | `session_load`/`session_save` permission model | Current |
| C-055 | Test Coverage Requirements | ≥70% coverage targets, TEST_MAPPING.md | Current |
| C-056 | Config JSONC Migration | TOML→JSONC migration, deprecation timeline | Current |
| C-057 | Server API Endpoints | Health, abort, permission reply endpoints | Current |
| C-058 | LSP Capabilities | JSON-RPC protocol, language server detection | **Needs Update** |
| C-059 | SDK Design Requirements | Rust/TypeScript SDK API surface | **NEW** |
| C-060 | Sensitive File Security | .env default deny, credential encryption | **NEW** |
| C-061 | Plugin Event Bus | Event types, WASM integration | **NEW** |
| C-062 | Error Code System | 1xxx-9xxx error code ranges | **NEW** |
| C-063 | Context Compaction | Token budget thresholds (85%/92%/95%) | **NEW** |
| C-064 | Credential Encryption | AES-256-GCM, key management | **NEW** |
```

---

## 6. 修订优先级

| 优先级 | 文档 | 预计工作量 | 关联 Issue |
|--------|------|------------|------------|
| P0 | C-059 SDK Design | Medium | SDK 未实现 |
| P0 | C-060 Sensitive File Security | Low | 安全漏洞 |
| P1 | C-062 Error Code System | Medium | 错误诊断困难 |
| P1 | C-063 Context Compaction | Low | 内存管理不可靠 |
| P1 | C-064 Credential Encryption | Medium | 安全漏洞 |
| P1 | C-061 Plugin Event Bus | Medium | 插件系统不完整 |
| P1 | C-058 LSP 修订 | Low | LSP 实现缺失 |
| P1 | C-024 修订 (Lineage) | Low | Session fork 不完整 |
| P2 | C-055 修订 (Security Tests) | Low | 测试覆盖不足 |
| P2 | Constitution Master 修订 | Low | 文档更新 |

---

## 7. 修订流程建议

按照 Article 7: Amendments 流程:

1. **RFC 阶段**: 本文档作为 RFC 发布到 `outputs/rfcs/`
2. **评审阶段**: 需要 2+ 维护者批准
3. **生效阶段**: 更新 master constitution 和对应 C-xxx 文档
4. **通知阶段**: 在项目沟通渠道公告变更

---

**文档状态**: 等待评审  
**建议生效版本**: v0.3  
**下次审查**: v24 迭代开始时
