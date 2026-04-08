# OpenCode-RS Task List v2.2

**版本：** 2.2  
**日期：** 2026-04-08  
**基于：** Spec v2.2 Gap Analysis  
**状态：** 已发布

---

## P0 任务 (v1.0 - 当前迭代)

| ID | 任务 | 描述 | 状态 | 优先级 | 工作量 | 指派 | 验收标准 |
|----|------|------|------|--------|--------|------|----------|
| T-001 | session_load 工具 | 实现 session_load 工具，从存储加载会话 | **Done** | P0 | Low | - | 工具可加载会话并返回正确数据 |
| T-002 | session_save 工具 | 实现 session_save 工具，保存会话到存储 | **Done** | P0 | Low | - | 工具可保存会话并持久化 |
| T-003 | Context Panel | 实现 Token 预算显示面板 | **Done** | P0 | Medium | - | 显示当前/最大 token，百分比 |
| T-004 | HuggingFace 提供商 | 添加 HuggingFace LLM 提供商 | **Done** | P0 | Low | - | 可配置并使用 HuggingFace 模型 |
| T-005 | AI21 提供商 | 添加 AI21 LLM 提供商 | **Done** | P0 | Low | - | 可配置并使用 AI21 模型 |
| T-006 | Dead Code 清理 | 清理 5 个 dead code 警告 | **Done** | P0 | Low | - | `cargo build` 无 dead_code 警告 |
| T-007 | Binary Size 优化 | 优化二进制大小 (15-20MB → <12MB) | **Done** | P0 | Medium | - | Release 二进制 < 12MB |

---

## P1 任务 (v1.1)

| ID | 任务 | 描述 | 状态 | 优先级 | 工作量 | 指派 | 验收标准 |
|----|------|------|------|--------|--------|------|----------|
| T-008 | Todo Panel 增强 | 增强 Todo Panel 功能 | **Done** | P1 | Medium | - | 更丰富的待办项显示和交互 |
| T-009 | Diff Panel 增强 | 增强 Diff Panel 功能 | **Partial** | P1 | Medium | - | 更完整的差异比较功能 |
| T-010 | Diagnostics Panel 增强 | 增强 Diagnostics Panel 功能 | **Partial** | P1 | Medium | - | LSP 诊断信息更完整 |
| T-011 | Files Panel 增强 | 增强 Files Panel 功能 | **Partial** | P1 | Medium | - | 文件树导航更流畅 |
| T-012 | Permissions Panel 增强 | 增强 Permissions Panel 功能 | **Partial** | P1 | Medium | - | 权限状态显示更清晰 |
| T-013 | Built-in Skills 完善 | 完成剩余 5 个 Built-in Skills | **Not Started** | P1 | Medium | - | 10/10 Built-in Skills 完成 |
| T-014 | OAuth Login | 实现浏览器认证流程 | **Done** | P1 | High | - | 支持 OAuth 认证流程 |
| T-015 | Plugin ABI 稳定 | 稳定化 WASM 插件接口 | **Not Started** | P1 | High | - | Plugin ABI 版本控制 |

---

## P2 任务 (v1.5+)

| ID | 任务 | 描述 | 状态 | 优先级 | 工作量 | 指派 | 验收标准 |
|----|------|------|------|--------|--------|------|----------|
| T-016 | GitHub 集成 | 实现 GitHub Issue/PR 触发 | **Partial** | P2 | High | - | 可通过 GitHub 事件触发操作 |
| T-017 | Desktop Shell | 完整桌面应用程序 | **Not Started** | P2 | High | - | 独立桌面应用发布 |
| T-018 | IDE 扩展 | VS Code / JetBrains 插件 | **Not Started** | P2 | High | - | IDE 内使用 OpenCode |
| T-019 | Public Share Server | 公共会话分享服务 | **Done** | P2 | Medium | - | 云端会话分享 |

---

## 任务状态汇总

| 状态 | P0 | P1 | P2 | 总计 |
|------|----|----|----|------|
| **TODO** | 0 | 2 | 2 | 4 |
| **In Progress** | 0 | 0 | 0 | 0 |
| **Done** | 7 | 2 | 1 | 10 |
| **Partial** | 0 | 4 | 1 | 5 |
| **Not Started** | 0 | 2 | 2 | 4 |
| **Blocked** | 0 | 0 | 0 | 0 |

---

## 实现改进

### T-008 (Todo Panel) ✅
- 添加 `TodoEntry` 结构体解析 `- [ ]` 格式
- 从消息历史解析 todos
- Right Panel 现在显示待办事项

### T-009~T-012 (Panels) 🔄 改进
- Messages tab: 现在显示最近15条消息
- Sessions tab: 现在显示已保存的会话
- Config tab: 现在显示当前配置(provider/model/agent)
- Debug tab: 现在显示tokens/cost/messages计数
- Diff/Permissions/Files/Diagnostics tabs: 仍需进一步集成

---

## 技术参考

### 权限检查模板 (C-024)

```rust
use opencode_permission::{check_tool_permission_default, ApprovalResult};

async fn my_tool_execute(...) -> Result<ToolResult, OpenCodeError> {
    if check_tool_permission_default("my_tool") != ApprovalResult::AutoApprove {
        return Ok(ToolResult::err("Permission denied"));
    }
    // ... implementation
}
```

### 健康检查实现 (C-057)

```rust
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

### API 错误响应 (C-057)

```rust
json_error(StatusCode::NOT_FOUND, "session_not_found", "Session does not exist")
```

---

## 依赖关系

```
T-001 (session_load) ─┬─> Storage crate
T-002 (session_save) ─┘
T-003 (Context Panel) ─> TUI crate
T-004 (HuggingFace) ──> LLM crate
T-005 (AI21) ─────────> LLM crate
T-006 (Dead Code) ────> All crates
T-007 (Binary Size) ──> Build process

T-008~T-012 (Panels) ─> T-003
T-013 (Skills) ──────> Skills crate
T-014 (OAuth) ───────> Auth crate
T-015 (Plugin ABI) ──> Plugin crate
```

---

**最后更新：** 2026-04-08  
**下次审查：** Iteration-23
