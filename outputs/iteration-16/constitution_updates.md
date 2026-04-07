# Constitution 审计报告 (v2.0 → v2.1)

**日期**: 2026-04-07  
**审计范围**: Constitution v2.0 (C-001 ~ C-055) vs iteration-16 差距分析  
**审计依据**: 
- iteration-16/gap-analysis.md (P1/P2 差距 + 技术债务)
- iteration-15/constitution_updates.md (v2.0 基准)

---

## 一、审计结论

### Constitution v2.0 状态: ❌ 正式文件仍不存在 + 需要补充 3 条新条款 + 扩展 2 条现有条款

| 指标 | 值 |
|------|-----|
| Constitution 条款总数 (v2.0) | 55 (C-001 ~ C-055, C-001 已废止) |
| **Constitution 实际文件** | **❌ 不存在** (连续 16 次迭代未合并为正式文件) |
| iteration-16 新发现 P1 问题 | 4 (配置格式/session工具/health端点/abort端点) |
| iteration-16 新发现 P2 问题 | 4 (OpenAPI/LSP/测试分布/权限路由) |
| **被现有条款覆盖的新问题** | **0/8** (全部未被覆盖) |
| 建议新增条款 | C-056, C-057, C-058 (3 条) |
| 建议扩展条款 | C-024 (工具完整性), C-055 (测试分布) |

### 关键发现

1. **Server API 完整性无约束** — /health、/abort、permission reply 路由无 Constitution 规范
2. **配置格式冲突未解决** — PRD 要求 JSONC，实现用 TOML，无条款强制统一
3. **LSP 能力无最低要求** — 仅 diagnostics 实现，definition/references/hover 缺失
4. **测试分布不均无约束** — core 126 测试 vs llm 9 测试，差异 14x
5. **工具完整性无强制要求** — session_load/session_save 缺失但无条款约束

---

## 二、iteration-16 新发现覆盖度分析

| 差距项 | 严重程度 | Constitution 覆盖 | 覆盖状态 |
|--------|---------|-------------------|----------|
| 配置格式 TOML vs JSONC | P1 | ❌ 无 | **需新增 C-056** |
| session_load/session_save 工具缺失 | P1 | ❌ 无 | **需扩展 C-024** |
| 缺少 /health 端点 | P1 | ❌ 无 | **需新增 C-057** |
| 缺少 session abort 端点 | P1 | ❌ 无 | **需新增 C-057** |
| OpenAPI 文档缺失 | P2 | ❌ 无 | **需扩展 C-057** |
| LSP 能力不完整 | P2 | ❌ 无 | **需新增 C-058** |
| 测试覆盖 75% + 分布不均 | P2 | ⚠️ C-055 部分 | **需扩展 C-055** |
| 权限回复路由不明确 | P3 | ❌ 无 | **需扩展 C-057** |

### 覆盖度统计

| 覆盖状态 | 数量 | 占比 |
|----------|------|------|
| ✅ 完全覆盖 | 0 | 0% |
| ⚠️ 部分覆盖 | 1 | 12.5% |
| ❌ 无覆盖 | 7 | 87.5% |

---

## 三、Constitution v2.1 修订内容

### 3.1 新增 C-056: 配置格式规范

```markdown
### 条款 C-056: 配置格式规范

1. 格式要求:
   a) 主配置文件必须使用 JSONC 格式 (.jsonc/.json)
   b) 支持 JSON 子集 (无注释) 作为降级
   c) 禁止使用 TOML/YAML 替代 JSONC

2. 多层合并:
   a) 全局配置: ~/.config/opencode/config.jsonc
   b) 项目配置: {project}/.opencode/config.jsonc
   c) 项目配置优先级高于全局配置

3. 迁移约束:
   a) 如已有 TOML 配置，必须提供自动迁移工具
   b) 迁移工具必须保留注释 (JSONC 特性)
   c) 迁移后删除旧 TOML 文件
```

### 3.2 新增 C-057: Server API 完整性规范

```markdown
### 条款 C-057: Server API 完整性规范

1. 必需端点 (P0):
   a) GET  /health — 健康检查 (返回 {status: "ok", version: "x.y.z"})
   b) POST /sessions — 创建会话
   c) GET  /sessions/{id} — 获取会话
   d) POST /sessions/{id}/abort — 中止运行中会话
   e) GET  /sessions/{id}/stream — SSE 流
   f) WS   /sessions/{id}/ws — WebSocket 流
   g) POST /sessions/{id}/permissions/{req_id}/reply — 权限回复

2. API 文档:
   a) 必须集成 utoipa 生成 OpenAPI 3.1 文档
   b) 所有端点必须带 #[utoipa::path] 注解
   c) 构建时自动生成 openapi.json

3. 路由命名:
   a) RESTful 风格，复数资源名 (sessions, tools, permissions)
   b) 动作用 HTTP 方法表达 (GET/POST/DELETE)，不在 URL 中
   c) 权限回复统一为 POST /sessions/{id}/permissions/{req_id}/reply
   d) 禁止在 URL 中使用动词 (如 /getSession, /createSession)
```

### 3.3 新增 C-058: LSP 能力规范

```markdown
### 条款 C-058: LSP 能力规范

1. 必需能力 (P0):
   a) textDocument/publishDiagnostics — 诊断发布
   b) textDocument/hover — 悬停信息

2. 推荐能力 (P1):
   a) textDocument/definition — 跳转定义
   b) textDocument/references — 查找引用
   c) textDocument/completion — 自动完成

3. 约束:
   a) LSP 客户端必须处理 server 断开重连
   b) 诊断信息必须缓存，断连后仍可用
```

### 3.4 扩展 C-024: 工具系统完整性

在 C-024 末尾新增第 4 节:

```markdown
4. 工具完整性:
   a) PRD 定义的 35 个工具必须全部实现
   b) session_load / session_save 为 P0 工具，必须连接 storage crate
   c) 工具缺失必须在 gap analysis 中明确标注
   d) 新增工具必须通过权限系统 (allow/ask/deny) 检查
```

### 3.5 扩展 C-055: TUI 性能目标 (补充测试分布)

在 C-055 末尾新增第 6 节:

```markdown
6. 测试分布:
   a) 每个 crate 测试覆盖率不低于 50%
   b) 核心 crate (core, server, llm) 测试覆盖率不低于 70%
   c) 关键路径必须有集成测试 (session, server API, LSP)
   d) 测试分布差异不超过 2x (最高/最低 crate 测试数比)
```

---

## 四、修订计划

| 条款 | 操作 | 说明 | 优先级 |
|------|------|------|--------|
| C-056 | 新增 | 配置格式规范 (JSONC 强制) | P0 |
| C-057 | 新增 | Server API 完整性 + OpenAPI + 路由命名 | P0 |
| C-058 | 新增 | LSP 能力最低要求 | P1 |
| C-024 | 扩展 | 工具完整性约束 | P0 |
| C-055 | 扩展 | 测试分布均匀性约束 | P1 |

---

## 五、持续风险

### 5.1 Constitution 正式文件仍不存在 (16 次迭代)

**风险等级**: 🔴 高

连续 16 次迭代仅有 proposals，从未合并为 `CONSTITUTION.md`。所有条款无强制约束力。

**建议**: 将 Constitution v2.1 正式写入项目根目录 `CONSTITUTION.md`

### 5.2 配置格式冲突为阻塞性问题

**风险等级**: 🔴 高

PRD 定义 JSONC，实现使用 TOML。这不仅是格式偏好问题，还影响:
- 用户无法使用 OpenCode 官方配置
- 注释支持缺失 (TOML 有注释但语法不同)
- 生态工具链不兼容

**建议**: 优先执行 C-056 迁移

---

## 六、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0 | 2026-04-04 | 初始版本，覆盖 Context/Plugin/Skills/Commands/MCP |
| 1.6 | 2026-04-05 | C-026 重大重写 (4 层认证架构), 新增 C-033~C-037 |
| 1.9 | 2026-04-06 | 新增 C-041~C-046 (SSO/OIDC, CredentialRef, 错误码, 集成测试等) |
| 2.0 | 2026-04-07 | TUI 专项: 扩展 C-034, 新增 C-047~C-055 (10 条 TUI 条款) |
| **2.1** | **2026-04-07** | **Server/API/LSP 专项: 新增 C-056~C-058, 扩展 C-024/C-055** |

---

*本文档识别 iteration-16 差距分析中的 8 项新问题需要 Constitution v2.1 新增/扩展条款覆盖。核心变更: 新增配置格式规范 (C-056)、Server API 完整性规范 (C-057)、LSP 能力规范 (C-058)，扩展工具完整性 (C-024) 和测试分布 (C-055) 约束。*
