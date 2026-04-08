# OpenCode-RS Implementation Plan v2.2

**版本：** 2.2  
**日期：** 2026-04-08  
**基于：** Spec v2.2 + Gap Analysis  
**状态：** 已发布

---

## 1. 执行摘要

根据 v2.2 规格文档的差距分析，当前实现完成度约 **88-92%**。核心运行时、Agent 系统、工具系统、权限系统和服务器 API 已全部功能化并符合规范。

### 优先级分布

| 优先级 | 任务数 | 关键里程碑 |
|--------|--------|------------|
| **P0 (v1.0)** | 7 | session_load/session_save, Context Panel, HuggingFace/AI21, Dead code 清理 |
| **P1 (v1.1)** | 8 | Inspector Panels 增强, Built-in Skills, OAuth, Plugin ABI |
| **P2 (v1.5+)** | 4 | GitHub 集成, Desktop shell, IDE 扩展, Public share server |

---

## 2. P0 任务清单 (v1.0 - 当前迭代)

### 2.1 缺失工具实现

| 任务 | 描述 | 影响 | 工作量 | 依赖 |
|------|------|------|--------|------|
| **T-001** | 实现 `session_load` 工具 | 功能完整性 | Low | Storage crate |
| **T-002** | 实现 `session_save` 工具 | 功能完整性 | Low | Storage crate |

**技术参考：** Constitution C-024 Session Tools Permission

```rust
// 实现模板
async fn execute(&self, args: Value, ctx: Option<ToolContext>) -> Result<ToolResult, OpenCodeError> {
    let permission_check = check_tool_permission_default(self.name());
    if permission_check != ApprovalResult::AutoApprove {
        return Ok(ToolResult::err("Permission denied"));
    }
    // ... implementation
}
```

### 2.2 TUI 增强

| 任务 | 描述 | 影响 | 工作量 | 依赖 |
|------|------|------|--------|------|
| **T-003** | 实现 Context Panel (Token 预算显示) | UX 完整性 | Medium | TUI crate |

**验收标准：**
- [ ] 显示当前会话 token 使用量
- [ ] 显示 token 预算百分比
- [ ] 警告当接近预算限制

### 2.3 LLM 提供商扩展

| 任务 | 描述 | 影响 | 工作量 | 依赖 |
|------|------|------|--------|------|
| **T-004** | 实现 HuggingFace 提供商 | 提供商覆盖 | Low | LLM crate |
| **T-005** | 实现 AI21 提供商 | 提供商覆盖 | Low | LLM crate |

### 2.4 代码质量

| 任务 | 描述 | 影响 | 工作量 | 依赖 |
|------|------|------|--------|------|
| **T-006** | 清理 dead code 警告 | 代码质量 | Low | None |
| **T-007** | 二进制大小优化 | 分发效率 | Medium | Build process |

**Dead code 位置：**
- `crates/tui/src/app.rs`: `tool_registry`, `agent_executor`, `mcp_manager`, `MAX_HISTORY_SIZE`, `TOKEN_ESTIMATE_DIVISOR`
- `crates/cli/src/output/ndjson.rs`: `write_chunk`, `write_done`, `write_error` 等

---

## 3. P1 任务清单 (v1.1)

### 3.1 Inspector Panels 增强

| 任务 | 描述 | 当前状态 | 工作量 | 依赖 |
|------|------|----------|--------|------|
| **T-008** | Todo Panel 增强 | Partial | Medium | T-003 |
| **T-009** | Diff Panel 增强 | Partial | Medium | T-003 |
| **T-010** | Diagnostics Panel 增强 | Partial | Medium | T-003 |
| **T-011** | Files Panel 增强 | Partial | Medium | T-003 |
| **T-012** | Permissions Panel 增强 | Partial | Medium | Permission crate |

### 3.2 Skills 系统完善

| 任务 | 描述 | 当前状态 | 工作量 | 依赖 |
|------|------|----------|--------|------|
| **T-013** | Built-in Skills (5/10 → 10/10) | 5/10 | Medium | Skills crate |

### 3.3 企业功能

| 任务 | 描述 | 当前状态 | 工作量 | 依赖 |
|------|------|----------|--------|------|
| **T-014** | OAuth login (浏览器认证) | Pending | High | Auth crate |
| **T-015** | Plugin ABI 稳定性 | Pending | High | Plugin crate |

---

## 4. P2 任务清单 (v1.5+)

| 任务 | 描述 | 优先级 | 工作量 | 依赖 |
|------|------|--------|--------|------|
| **T-016** | GitHub 集成 (Issue/PR 触发) | Low | High | Git crate |
| **T-017** | Desktop shell | Low | High | TUI crate |
| **T-018** | IDE 扩展 (VS Code, JetBrains) | Low | High | LSP crate |
| **T-019** | Public share server | Low | Medium | Server crate |

---

## 5. 技术参考

### 5.1 权限模型 (C-024)

所有工具分类：

| 类别 | 自动批准 | 示例 |
|------|----------|------|
| Read | `ReadOnly` | `read`, `grep`, `session_load` |
| Safe | `Restricted` | `glob`, `ls` |
| Write | `Full` | `write`, `bash`, `session_save` |

### 5.2 配置优先级 (C-056)

| 优先级 | 格式 | 状态 |
|--------|------|------|
| 1 | `.opencode/config.jsonc` | 首选 |
| 2 | `.opencode/config.json` | 支持 |
| 3 | `.opencode/config.toml` | **已废弃** |

### 5.3 健康检查端点 (C-057)

```rust
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

---

## 6. 里程碑规划

| 里程碑 | 目标日期 | 包含任务 |
|--------|----------|----------|
| **v1.0 RC1** | 2026-04-08 | T-001, T-002, T-003, T-004, T-005, T-006 |
| **v1.0 GA** | 2026-04-15 | T-007 (Binary size) |
| **v1.1** | TBD | T-008 ~ T-015 |
| **v1.5** | TBD | T-016 ~ T-019 |

---

## 7. 风险与缓解

| ID | 风险 | 影响 | 概率 | 缓解措施 |
|----|------|------|------|----------|
| R1 | Binary size 优化失败 | Medium | Low | 渐进式优化，保留功能 |
| R2 | OAuth 集成复杂 | High | Medium | 使用成熟库 (oauth2-rs) |
| R3 | Plugin ABI 不稳定 | Medium | Medium | 明确接口版本控制 |

---

## 8. 验收标准

### 8.1 P0 验收

- [ ] `session_load` 工具可正常加载会话
- [ ] `session_save` 工具可正常保存会话
- [ ] Context Panel 显示 token 预算
- [ ] HuggingFace 提供商可用
- [ ] AI21 提供商可用
- [ ] `cargo build --release` 无警告
- [ ] 二进制大小 < 12MB

### 8.2 构建验证

```bash
cargo build --release
cargo clippy --all -- -D warnings
cargo test
```

---

**下次审查：** Iteration-23  
**变更要求：** RFC 流程 (Article 7)
