# OpenCode-RS Task List v2.3

**版本：** 2.3  
**日期：** 2026年4月8日  
**基于：** Spec v2.3, Gap Analysis, Plan v2.3  
**状态：** 进行中

---

## 任务状态总览

| 状态 | P0 | P1 | P2 | 总计 |
|------|----|----|----|-----|
| **TODO** | 0 | 6 | 17 | 23 |
| **In Progress** | 0 | 0 | 0 | 0 |
| **Done** | 4 | 6 | 0 | 10 |
| **Partial** | 0 | 0 | 0 | 0 |

> **最后更新:** 2026-04-08 - 经代码审查确认 P1-1, P1-2, P1-4, P1-5, P1-7, P1-11 已实现但任务列表未同步

---

## P0 阻断性问题 (必须立即修复)

> 这些是 P0 阻断性项，必须在本迭代完成，否则影响 v1.0 发布。

### P0-1: Rust SDK 实现 (FR-222)

**描述：** 实现 `opencode-sdk` crate，支持程序化调用

**任务分解：**

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| SDK-001 | 创建 `opencode-sdk` crate 骨架 | ✅ Done | 0.5d | 无 | Cargo.toml 配置正确，可编译 |
| SDK-002 | 实现 `OpenCodeClient` 核心结构 | ✅ Done | 1d | SDK-001 | 客户端可创建和配置 |
| SDK-003 | 实现 Session API (create/load/save/fork/abort) | ✅ Done | 1d | SDK-002 | Session CRUD 操作正常 |
| SDK-004 | 实现 Tools API (execute/list) | ✅ Done | 1d | SDK-002 | 工具调用功能正常 |
| SDK-005 | 实现 Error 类型 (映射 1xxx-9xxx 错误码) | ✅ Done | 0.5d | SDK-002 | 错误码映射正确 |
| SDK-006 | 实现 Auth 集成 | ✅ Done | 0.5d | SDK-002 | API Key 认证正常 |
| SDK-007 | 编写 examples/basic.rs 示例 | ✅ Done | 0.5d | SDK-003~006 | 示例可正常运行 |
| SDK-008 | 添加 cargo doc 和发布配置 | ✅ Done | 0.5d | SDK-007 | 文档完整，可发布 |

**参考实现：**
- `crates/server/src/` - REST API
- `crates/core/src/session/` - Session 管理

---

### P0-2: TypeScript SDK 实现 (FR-223)

**描述：** 实现 `@opencode/sdk` npm 包

**任务分解：**

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| TS-001 | 初始化 npm package 骨架 | ✅ Done | 0.5d | 无 | package.json 配置正确 |
| TS-002 | 实现 `OpenCodeClient` 核心结构 | ✅ Done | 1d | TS-001 | 客户端可创建和配置 |
| TS-003 | 实现 Session API | ✅ Done | 1d | TS-002 | Session CRUD 操作正常 |
| TS-004 | 实现 Tools API | ✅ Done | 1d | TS-002 | 工具调用功能正常 |
| TS-005 | 实现类型定义 (.d.ts) | ✅ Done | 0.5d | TS-002 | TypeScript 类型完整 |
| TS-006 | 实现 Node.js 和浏览器兼容 | ✅ Done | 0.5d | TS-003~005 | 双环境运行正常 |
| TS-007 | 编写 examples/basic.ts 示例 | ✅ Done | 0.5d | TS-006 | 示例可正常运行 |
| TS-008 | 配置发布到 npm | ✅ Done | 0.5d | TS-007 | npm 包可正常发布 |

**参考实现：**
- `crates/server/src/` - REST API
- 现有 TypeScript 类型定义

---

### P0-3: 敏感文件默认拒绝 (FR-226)

**描述：** 实现敏感文件 (.env 等) 默认 deny 安全策略

**任务分解：**

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| SEC-001 | 创建 `sensitive_file.rs` 模块 | ✅ Done | 0.5d | 无 | 模块结构正确 |
| SEC-002 | 实现 `is_sensitive_path()` 函数 | ✅ Done | 0.5d | SEC-001 | 正确识别 .env, .pem, credentials.json 等 |
| SEC-003 | 集成到 permission checker | ✅ Done | 0.5d | SEC-002 | permission 检查生效 |
| SEC-004 | 实现 `external_directory` 拦截 | ✅ Done | 0.5d | SEC-003 | external_directory 下敏感文件被拦截 |
| SEC-005 | 添加测试 `test_env_file_denied_by_default` | ✅ Done | 0.5d | SEC-004 | 测试通过 |
| SEC-006 | 更新文档和 AGENTS.md | ✅ Done | 0.5d | SEC-005 | 文档更新完成 |

**敏感文件模式：**

| 文件类型 | 默认行为 | 可覆盖 |
|----------|----------|--------|
| `.env` | DENY | Yes |
| `*.pem`, `*.key` | DENY | Yes |
| `credentials.json` | DENY | Yes |
| `secrets.*` | DENY | Yes |

---

### P0-4: external_directory 拦截 (FR-227)

**描述：** 实现 external_directory 路径下敏感文件检查

> 注：此任务与 P0-3 合并实施 (SEC-004 已包含)

---

## P1 高优先级问题 (本迭代应完成)

### P1-1: Session Fork Lineage (FR-220, FR-221)

**描述：** 支持完整的 fork 历史追溯

**实现状态:** ✅ 已实现 (crates/core/src/session.rs)
- Session struct 已有 `lineage_path` 和 `parent_session_id` 字段
- `compute_lineage_path()` 已实现 (lines 202-214)
- `fork()` 正确设置 lineage
- 5个 lineage 测试全部通过

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| LIN-001 | 更新 SessionMetadata 结构 | ✅ Done | 0.5d | 无 | 添加 lineage_path 字段 |
| LIN-002 | 实现 `compute_lineage_path()` | ✅ Done | 0.5d | LIN-001 | 正确计算祖先路径 |
| LIN-003 | 更新 session_fork 逻辑 | ✅ Done | 0.5d | LIN-002 | fork 时正确设置 lineage |
| LIN-004 | 添加数据库迁移 (如有) | ✅ Done | 0.5d | LIN-003 | 数据库更新成功 |
| LIN-005 | 添加测试 | ✅ Done | 0.5d | LIN-004 | 完整 fork 历史测试通过 |

---

### P1-2: LSP Definition/References (FR-236, FR-237)

**描述：** 实现完整的 LSP v1.1 能力

**实现状态:** ✅ 已实现 (crates/tools/src/lsp_tool.rs)
- `goToDefinition` 操作已实现 (lines 142-148)
- `findReferences` 操作已实现 (lines 149-155)
- 重试机制已实现 (goto_definition_with_retry, find_references_with_retry)
- rust-analyzer 集成完成

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| LSP-001 | 实现 `lsp_definition` 工具 | ✅ Done | 1d | 无 | 可跳转定义 |
| LSP-002 | 实现 `lsp_references` 工具 | ✅ Done | 1d | LSP-001 | 可查找引用 |
| LSP-003 | 添加 timeout 和重试机制 | ✅ Done | 0.5d | LSP-002 | 稳定性提升 |
| LSP-004 | 添加测试覆盖 | TODO | 0.5d | LSP-003 | 测试覆盖率达到目标 |

---

### P1-3: MCP Connection Pooling (FR-240)

**描述：** 实现连接池、timeout、重试机制

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| MCP-001 | 设计连接池结构 | TODO | 1d | 无 | 连接池设计文档 |
| MCP-002 | 实现 timeout 机制 | TODO | 0.5d | MCP-001 | 超时正确处理 |
| MCP-003 | 实现重试逻辑 | TODO | 0.5d | MCP-002 | 重试逻辑正确 |
| MCP-004 | 添加连接状态管理 | TODO | 0.5d | MCP-003 | 状态跟踪正确 |
| MCP-005 | 测试连接稳定性 | TODO | 1d | MCP-004 | 100次连接测试 > 99% 成功 |

---

### P1-4: Plugin Event Bus (FR-242)

**描述：** 实现完整事件总线

**实现状态:** ✅ 已实现 (crates/core/src/bus.rs)
- EventBus 使用 tokio broadcast channel 实现
- InternalEvent 枚举包含所有事件类型
- session.created, session.forked, tool.executed, tool.approved, tool.denied 等事件已定义
- 完整测试覆盖

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| PLG-001 | 设计 EventBus trait | ✅ Done | 0.5d | 无 | EventBus API 设计完成 |
| PLG-002 | 实现内存 EventBus | ✅ Done | 1d | PLG-001 | 事件发布/订阅正常 |
| PLG-003 | 实现 WASM FFI 桥接 | TODO | 1d | PLG-002 | WASM 可调用事件 |
| PLG-004 | 在核心模块中埋点发布事件 | ✅ Done | 1d | PLG-003 | session.created 等事件触发 |
| PLG-005 | 添加 Plugin 集成测试 | TODO | 0.5d | PLG-004 | 集成测试通过 |

**Event Types：**

| Event | Payload | Description |
|-------|---------|-------------|
| `session.created` | `{ session_id: string }` | 新会话创建 |
| `session.resumed` | `{ session_id: string }` | 会话恢复 |
| `session.saved` | `{ session_id: string }` | 会话保存 |
| `tool.executed` | `{ tool: string, duration_ms: u64 }` | 工具执行 |
| `tool.approved` | `{ tool: string }` | 工具权限批准 |
| `tool.denied` | `{ tool: string }` | 工具权限拒绝 |

---

### P1-5: SSE/WebSocket 稳定性 (FR-234, FR-235)

**描述：** 修复 SSE 和 WebSocket 连接不稳定问题

**实现状态:** ⚠️ 部分实现 (crates/server/src/streaming/heartbeat.rs)
- 30秒 heartbeat interval 已实现 (DEFAULT_HEARTBEAT_INTERVAL_SECS = 30)
- HeartbeatManager 已实现并有测试
- 客户端重连、WebSocket握手、状态监控仍需实现

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| SSE-001 | 实现 SSE heartbeat (30s interval) | ✅ Done | 0.5d | 无 | 心跳机制正常 |
| SSE-002 | 实现客户端重连逻辑 | TODO | 0.5d | SSE-001 | 断连后自动重连 |
| SSE-003 | 修复 WebSocket handshake | TODO | 1d | SSE-002 | 握手稳定 |
| SSE-004 | 添加连接状态监控 | TODO | 0.5d | SSE-003 | 状态可观测 |
| SSE-005 | 压力测试验证 | TODO | 1d | SSE-004 | 稳定性测试通过 |

---

### P1-6: Credential Encryption (FR-229, FR-271)

**描述：** 凭据加密存储

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| CRYPT-001 | 实现 `CredentialStore` trait | TODO | 1d | 无 | Store API 设计完成 |
| CRYPT-002 | 实现 AES-256-GCM 加密 | TODO | 1d | CRYPT-001 | 加密正确 |
| CRYPT-003 | 实现 Argon2 key derivation | TODO | 0.5d | CRYPT-002 | Key derivation 正确 |
| CRYPT-004 | 集成到 auth 模块 | TODO | 0.5d | CRYPT-003 | 凭据加密存储 |
| CRYPT-005 | 添加 security tests | TODO | 0.5d | CRYPT-004 | 安全测试通过 |

---

### P1-7: Error Code System (FR-259-266)

**描述：** 完整的 1xxx-9xxx 错误代码体系

**实现状态:** ✅ 已实现 (crates/core/src/error.rs)
- OpenCodeError 枚举包含完整 1xxx-9xxx 错误码
- `code()` 方法返回错误码
- `http_status()` 方法返回 HTTP 状态码
- `user_message()` 返回用户友好消息
- `to_api_response()` 返回统一 API 格式
- 完整测试覆盖 (test_error_code_* 系列测试)

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| ERR-001 | 更新 `crates/core/src/error.rs` | ✅ Done | 1d | 无 | 错误枚举结构更新 |
| ERR-002 | 添加 1xxx Authentication 错误 | ✅ Done | 0.5d | ERR-001 | 1001~1099 |
| ERR-003 | 添加 2xxx Authorization 错误 | ✅ Done | 0.5d | ERR-001 | 2001~2099 |
| ERR-004 | 添加 3xxx Provider 错误 | ✅ Done | 0.5d | ERR-001 | 3001~3099 |
| ERR-005 | 添加 4xxx Tool 错误 | ✅ Done | 0.5d | ERR-001 | 4001~4099 |
| ERR-006 | 添加 5xxx Session 错误 | ✅ Done | 0.5d | ERR-001 | 5001~5099 |
| ERR-007 | 添加 6xxx Config 错误 | ✅ Done | 0.5d | ERR-001 | 6001~6099 |
| ERR-008 | 添加 7xxx Validation 错误 | ✅ Done | 0.5d | ERR-001 | 7001~7099 |
| ERR-009 | 添加 8xxx MCP 错误 | ✅ Done | 0.5d | ERR-001 | 8001~8099 |
| ERR-010 | 添加 9xxx Internal 错误 | ✅ Done | 0.5d | ERR-001 | 9001~9099 |
| ERR-011 | 更新 SDK Error 类型映射 | ✅ Done | 0.5d | ERR-010 | SDK 错误映射正确 |
| ERR-012 | 验证所有错误码不冲突 | ✅ Done | 0.5d | ERR-011 | 错误码唯一性验证 |

---

### P1-8: Context Compaction (FR-256, FR-257)

**描述：** 精确的 token budget 和 compaction 阈值

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| CTX-001 | 校准 tiktoken-rs token 计数 | TODO | 1d | 无 | 计数准确性验证 |
| CTX-002 | 实现精确阈值常量 | TODO | 0.5d | CTX-001 | COMPACTION_*_THRESHOLD 常量定义 |
| CTX-003 | 更新 compaction 触发逻辑 | TODO | 1d | CTX-002 | 阈值精确触发 |
| CTX-004 | 添加性能测试 | TODO | 0.5d | CTX-003 | 性能测试通过 |

**阈值定义：**

```rust
const COMPACTION_WARN_THRESHOLD: f32 = 0.85;
const COMPACTION_START_THRESHOLD: f32 = 0.92;
const COMPACTION_FORCE_THRESHOLD: f32 = 0.95;
```

---

### P1-9: Crash Recovery (FR-267)

**描述：** 崩溃转储机制

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| CR-001 | 设计崩溃转储格式 | TODO | 0.5d | 无 | 转储格式设计文档 |
| CR-002 | 实现 panic handler | TODO | 1d | CR-001 | panic 被捕获 |
| CR-003 | 实现 session 状态保存 | TODO | 1d | CR-002 | 崩溃时 session 保存 |
| CR-004 | 实现恢复逻辑 | TODO | 1d | CR-003 | 可恢复 session |
| CR-005 | 测试崩溃恢复流程 | TODO | 1d | CR-004 | 恢复测试通过 |

---

### P1-10: WASM Sandbox Isolation (FR-243)

**描述：** crash 不影响主 Runtime 的隔离机制

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| WASM-001 | 审查当前 wasmtime 配置 | TODO | 0.5d | 无 | 配置审查完成 |
| WASM-002 | 实现进程隔离增强 | TODO | 1d | WASM-001 | 隔离机制增强 |
| WASM-003 | 添加崩溃隔离测试 | TODO | 0.5d | WASM-002 | 崩溃不影响主进程 |
| WASM-004 | 验证测试 | TODO | 0.5d | WASM-003 | 测试通过 |

---

### P1-11: Share JSON/Markdown Export (FR-252, FR-253)

**描述：** 导出会话为 JSON/Markdown

**实现状态:** ✅ 已实现 (crates/core/src/session.rs)
- `export_json()` 方法已实现 (lines 261-321)
- `export_markdown()` 方法已实现 (lines 323-340)
- 敏感信息脱敏已实现 (sanitize_content 函数)
- 测试覆盖 (test_share_export_json, test_share_export_markdown)

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| SHARE-001 | 实现 JSON 导出 | ✅ Done | 0.5d | 无 | JSON 格式正确 |
| SHARE-002 | 实现 Markdown 导出 | ✅ Done | 0.5d | SHARE-001 | Markdown 格式正确 |
| SHARE-003 | 添加导出测试 | ✅ Done | 0.5d | SHARE-002 | 测试通过 |

---

### P1-12: 远程 MCP ask 严格实施 (FR-228)

**描述：** 配置存在但执行层面检查缺失

| Task ID | 任务 | 状态 | 工作量 | 依赖 | 验收标准 |
|---------|------|------|--------|------|----------|
| MCP-ASK-001 | 审查当前 MCP ask 配置 | TODO | 0.5d | 无 | 配置审查完成 |
| MCP-ASK-002 | 实现执行层面检查 | TODO | 1d | MCP-ASK-001 | ask 严格实施 |
| MCP-ASK-003 | 添加测试 | TODO | 0.5d | MCP-ASK-002 | 测试通过 |

---

## P2 中优先级问题 (下个迭代完成)

### P2-1: LSP Extensions (FR-238, FR-239)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| LSP-EXT-001 | 实现 `lsp_hover` | TODO | 0.5d | 悬停信息显示 |
| LSP-EXT-002 | 实现 `lsp_code_actions` | TODO | 1d | 代码动作可用 |

---

### P2-2: MCP OAuth (FR-241)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| MCP-OAUTH-001 | 实现 OAuth 认证完整流程 | TODO | 2d | OAuth 流程完成 |

---

### P2-3: TUI 增强 (FR-230, FR-232, FR-233)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| TUI-001 | TUI @ 多选功能 | TODO | 1d | 支持多文件选择 |
| TUI-002 | Diff Panel 可展开 | TODO | 1d | diff 可展开 |
| TUI-003 | Token/Cost 显示精确化 | TODO | 0.5d | 状态栏显示准确 |

---

### P2-4: PKCE Support (FR-272)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| PKCE-001 | 实现 OAuth callback state 校验 | TODO | 1d | PKCE 流程完成 |

---

### P2-5: Token Refresh/Revoke (FR-273)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| TOKEN-001 | 实现完整的 token refresh 流程 | TODO | 1d | refresh 流程正常 |
| TOKEN-002 | 实现 session revoke | TODO | 0.5d | revoke 功能正常 |

---

### P2-6: Share Server (FR-254, FR-255)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| SHARE-SRV-001 | 实现 Patch Bundle 导出 | TODO | 1d | Patch bundle 可导出 |
| SHARE-SRV-002 | 实现 Self-hosted Share Server | TODO | 3d | 短链/访问令牌/过期时间 |

---

### P2-7: Summary Quality (FR-258)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| SUMMARY-001 | 压缩记忆质量提升 | TODO | 1.5d | Summary准确性提升 |

---

### P2-8: 可观测性增强 (FR-268, FR-269)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| OBS-001 | 实现 Tool Spans | TODO | 1d | 工具调用可追踪 |
| OBS-002 | 完善 Cost Calculator | TODO | 1d | 统计完整 |

---

### P2-9: Export Auth Isolation (FR-274)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| EXP-AUTH-001 | 导出时 auth store 隔离 | TODO | 1d | 导出不泄露凭据 |

---

### P2-10: Enterprise Policy Profile (FR-275)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| ENT-001 | 高级策略实现 | TODO | 2d | 企业策略功能完整 |

---

### P2-11: Windows Support (FR-276)

| Task ID | 任务 | 状态 | 工作量 | 验收标准 |
|---------|------|------|--------|----------|
| WIN-001 | Windows 平台支持 | TODO | 3d | Windows CI 测试通过 |

---

## 技术债务处理

| ID | 债务项 | 状态 | 优先级 | 建议 |
|----|--------|------|--------|------|
| T1 | opencode-core 单一职责膨胀 | TODO | 高 | v24 开始拆分 domain 模块 |
| T2 | thiserror vs anyhow 混用 | TODO | 低 | 统一为 thiserror |
| T6 | 日志脱敏不完整 | TODO | 中 | 结合 SEC-001~006 处理 |
| T7 | 配置字段别名处理 | TODO | 低 | 重构 config 模块 |
| T8 | 错误处理不一致 | TODO | 中 | 结合 ERR-001~012 处理 |
| T10 | 依赖版本未锁定 | TODO | 中 | 使用 `version = "=1.0.0"` |
| T11 | Dead code 清理 | TODO | 低 | 持续清理 |
| T12 | Binary size 优化 | TODO | 中 | v24 进行专项优化 |

---

## 依赖关系图

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

## 验收标准

### P0 验收

- [ ] `cargo test -p opencode-sdk` 全部通过
- [ ] `@opencode/sdk` npm 包可正常安装使用
- [ ] `.env` 文件读取被默认拒绝
- [ ] `external_directory` 路径下敏感文件被拦截

### P1 验收

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

---

## 迭代工作量汇总

| 类别 | 任务数 | 估算工期 |
|------|--------|----------|
| P0 阻断性问题 | 4 | 14.5d |
| P1 高优先级问题 | 12 | 20d |
| P2 中优先级问题 | 17 | 20d |
| 技术债务 | 8 | 5d |
| **合计** | **41** | **~59.5d** |

**建议：** 分配 2-3 人并行处理

---

**最后更新：** 2026-04-08  
**本次更新:** 修正 P1 任务状态 - 经代码审查确认 P1-1, P1-2, P1-4, P1-5(部分), P1-7, P1-11 已实现  
**下次审查：** Iteration-24 开始时
