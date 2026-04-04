# OpenCode-RS 规格文档 v2

**版本**: 2.0
**日期**: 2026-04-04
**基于**: PRD.md (2026-03) + Gap Analysis
**状态**: 草稿

---

## 1. 文档概述

### 1.1 背景

本规格文档基于 PRD.md 产品需求文档和差距分析报告生成，旨在识别当前实现与目标产品的差距，并为后续迭代提供明确的需求清单。

### 1.2 目标

- 为每个差距项分配唯一的需求编号 (FR-XXX)
- 按优先级组织需求
- 为每个需求定义验收标准
- 确保新功能有对应的规格定义

### 1.3 参考文档

- **PRD.md**: 产品需求文档 (1934 行)
- **gap-analysis.md**: 差距分析报告 (iteration-2)

---

## 2. 需求总览

| 优先级 | 数量 | 说明 |
|--------|------|------|
| P0 | 5 | 阻断性问题 |
| P1 | 5 | 核心功能缺失 |
| P2 | 5 | 完善性问题 |

---

## 3. P0 - 阻断性问题

### FR-001: Context Engine 实现

**模块**: core  
**严重程度**: P0  
**差距项**: Context Engine 未实现

#### 需求描述

实现 Token Budget 管理、Context Ranking 和 Compaction 机制。

#### 详细规格

1. **Token Budget 计算**
   - 实现 PRD Section 7.6 定义的上下文层次 (L0-L4)
   - 70% token 用于主要上下文
   - 20% 留给工具输出
   - 10% 留给响应空间
   - 支持 token 统计与预算预警

2. **Context Ranking**
   - 实现文件相关性评分算法
   - 支持最近文件优先
   - 支持基于 LSP 符号关系的排序

3. **Context Compaction**
   - 85% 预警阈值
   - 92% 触发 compact
   - 95% 强制新 session continuation
   - 支持摘要生成与替换

#### 验收标准

- [ ] Token 预算计算误差 < 5%
- [ ] 85% 阈值前触发预警
- [ ] 92% 阈值触发自动 compact
- [ ] Compact 后上下文可正常使用

---

### FR-002: Plugin System 实现

**模块**: plugin  
**严重程度**: P0  
**差距项**: Plugin System 未实现

#### 需求描述

实现 WASM 运行时和事件总线。

#### 详细规格

1. **WASM 插件运行时**
   - 集成 wasmtime 引擎
   - 插件隔离沙箱
   - 插件 API 能力授权机制

2. **事件总线**
   - 支持 PRD Section 7.10 定义的事件类型：
     - `session.created/updated/compacted`
     - `message.updated`
     - `tool.execute.before/after`
     - `permission.asked/replied`
     - `file.edited`
     - `lsp.updated`
     - `shell.env`
     - `tui.toast.show`

3. **插件能力**
   - 监听事件
   - 改写 prompt
   - 注入 shell 环境变量
   - 添加工具
   - 增加 context sources
   - 拦截敏感读取
   - 发送通知

#### 验收标准

- [ ] WASM 插件可加载并执行
- [ ] 事件总线支持所有定义的事件类型
- [ ] 插件崩溃不影响主进程
- [ ] 插件需通过 capability 授权

---

### FR-003: Skills 系统实现

**模块**: core  
**严重程度**: P0  
**差距项**: Skills 系统未实现

#### 需求描述

实现 Skill Loader 和 Semantic Matching。

#### 详细规格

1. **Skill 结构**
   - 路径: `.opencode/skills/<name>/SKILL.md`
   - 元信息 (frontmatter):
     - name
     - description
     - triggers (关键词数组)
     - priority

2. **延迟加载**
   - 启动时不预加载所有 skills
   - 按需发现并加载
   - 支持列出技能目录

3. **语义匹配**
   - 根据 prompt 语义匹配 skills
   - 支持手动指定
   - 支持全局与项目级别覆盖

4. **技能用途**
   - 仓库规范
   - 编码规范
   - 发布流程
   - 框架最佳实践
   - 团队约定

#### 验收标准

- [ ] Skills 可从指定路径加载
- [ ] 支持触发词匹配
- [ ] 支持延迟加载
- [ ] 支持全局/项目级别覆盖

---

### FR-004: Commands 系统实现

**模块**: core  
**严重程度**: P0  
**差距项**: Commands 系统未实现

#### 需求描述

实现 Command Parser 和 Template Expansion。

#### 详细规格

1. **命令定义**
   - 从 `.opencode/commands/*.md` 加载
   - 支持 YAML frontmatter + Markdown body
   - 可绑定 agent/model

2. **命令结构**
   ```yaml
   ---
   description: Run tests with coverage
   agent: build
   model: default
   ---
   运行完整测试并总结失败原因，只给出最小修复方案。
   ```

3. **变量支持**
   - `${file}` - 当前文件
   - `${selection}` - 选中文本
   - `${cwd}` - 工作目录
   - `${git_branch}` - Git 分支
   - `${input}` - 用户输入

4. **执行流程**
   - `/test` → 解析模板 → 注入变量 → 创建用户消息 → 发送到 Session Engine

#### 验收标准

- [ ] Commands 可从指定路径加载
- [ ] 支持变量替换
- [ ] 可绑定 agent/model
- [ ] 内置命令可用 (/help, /init, /undo, /redo, /share, /agent, /model, /clear)

---

### FR-005: MCP 工具接入完善

**模块**: mcp  
**严重程度**: P0  
**差距项**: MCP 工具接入不完整

#### 需求描述

实现 Schema Cache 和 Permission 集成。

#### 详细规格

1. **工具发现**
   - 支持本地 MCP (stdio)
   - 支持远程 MCP (HTTP)
   - 工具列表获取

2. **Schema 缓存**
   - 工具调用前只注入元信息
   - 避免预灌满上下文
   - 缓存失效机制

3. **Token 成本控制**
   - 默认禁用重型 MCP
   - 只有模型明确请求时再执行
   - 执行结果摘要优先

4. **Permission 集成**
   - 远程 MCP 默认 ask
   - 权限请求与内置工具统一处理
   - 审计日志记录

#### 验收标准

- [ ] 本地/远程 MCP 可配置
- [ ] Schema 缓存正常工作
- [ ] Token 成本可控
- [ ] Permission 集成正常

---

## 4. P1 - 核心功能缺失

### FR-006: Server API 完善

**模块**: server  
**严重程度**: P1  
**差距项**: Server API 不完整

#### 需求描述

完善 Session/Messages/Tool/Artifact API。

#### 详细规格

1. **Session API**
   - `POST /sessions` - 创建会话
   - `GET /sessions` - 列出会话
   - `GET /sessions/{id}` - 获取会话详情
   - `POST /sessions/{id}/fork` - 分叉会话
   - `POST /sessions/{id}/summarize` - 摘要会话
   - `POST /sessions/{id}/abort` - 中止会话

2. **Message API**
   - `POST /sessions/{id}/prompt` - 发送 prompt
   - `GET /sessions/{id}/messages` - 获取消息列表
   - `GET /sessions/{id}/messages/{msg_id}` - 获取单条消息

3. **Tool API**
   - `POST /sessions/{id}/shell` - 执行 shell
   - `POST /sessions/{id}/command` - 执行命令
   - `POST /sessions/{id}/permissions/{req_id}/reply` - 权限响应

4. **Artifact API**
   - `GET /sessions/{id}/diff` - 获取 diff
   - `GET /sessions/{id}/snapshots` - 获取快照列表
   - `POST /sessions/{id}/revert` - 恢复快照

5. **Runtime API**
   - `GET /doc` - API 文档
   - `GET /health` - 健康检查
   - `GET /providers` - 列出 providers
   - `GET /models` - 列出 models

#### 验收标准

- [ ] 所有 API 端点可访问
- [ ] REST API 返回正确状态码
- [ ] 流式响应正常工作 (SSE/WS)
- [ ] OpenAPI 文档生成

---

### FR-007: Share 功能实现

**模块**: core  
**严重程度**: P1  
**差距项**: Share 能力未实现

#### 需求描述

实现 Export JSON/Markdown 和 Share Server。

#### 详细规格

1. **本地导出**
   - 导出 session JSON
   - 导出 Markdown transcript
   - 导出 patch bundle

2. **服务层 (可选)**
   - self-hosted share server
   - 短链生成
   - 访问令牌
   - 过期时间

3. **默认策略**
   - 默认关闭自动分享
   - 手动触发
   - 明确提示"将上传对话内容"
   - 导出前脱敏检查

#### 验收标准

- [ ] 可导出 JSON 格式
- [ ] 可导出 Markdown 格式
- [ ] 可导出 patch bundle
- [ ] 敏感信息自动脱敏

---

### FR-008: LSP 功能增强

**模块**: lsp  
**严重程度**: P1  
**差距项**: LSP 功能有限

#### 需求描述

实现 Definition、References、Hover。

#### 详细规格

1. **v1 已实现**
   - diagnostics
   - workspace symbols
   - document symbols

2. **v1.1 扩展**
   - definition (跳转到定义)
   - references (查找引用)
   - hover (悬停信息)
   - code actions (只读建议)

3. **LSP 集成原则**
   - LSP 是"结构化上下文增强器"
   - 不是替代全文检索
   - 增量刷新
   - 自动重连

#### 验收标准

- [ ] diagnostics 正常工作
- [ ] workspace symbols 可搜索
- [ ] definition 跳转功能
- [ ] references 查找功能

---

### FR-009: 插件事件总线完善

**模块**: core  
**严重程度**: P1  
**差距项**: 插件事件总线不完整

#### 需求描述

完善事件类型覆盖。

#### 详细规格

实现所有 PRD Section 7.10 定义的事件：

- session.created
- session.updated
- session.compacted
- message.updated
- tool.execute.before
- tool.execute.after
- permission.asked
- permission.replied
- file.edited
- lsp.updated
- shell.env
- tui.toast.show

#### 验收标准

- [ ] 所有事件类型可触发
- [ ] 事件可订阅/取消订阅
- [ ] 事件处理器可添加/移除

---

### FR-010: 凭证加密存储实现

**模块**: auth  
**严重程度**: P1  
**差距项**: 凭证加密存储缺失

#### 需求描述

实现 Auth Store 加密。

#### 详细规格

1. **加密存储**
   - 明文 secret 不落 SQLite 主表
   - 使用系统密钥链或本地加密封装
   - 文件权限默认收紧到当前用户

2. **Auth Store 结构**
   - credential id
   - provider id
   - auth strategy
   - secret ciphertext
   - expires_at / refreshed_at
   - scopes / account metadata
   - created_at / updated_at
   - revoked_at

3. **OAuth/Session 扩展**
   - access token
   - refresh token
   - expiry
   - account id / workspace id

#### 验收标准

- [ ] 凭证加密存储
- [ ] 支持系统密钥链集成
- [ ] 导出 session 时排除 auth store
- [ ] 凭证可撤销

---

## 5. P2 - 完善性问题

### FR-011: 配置系统完善

**模块**: core  
**严重程度**: P2  
**差距项**: 配置系统不完整

#### 需求描述

实现 JSONC Loader、Env Override、Schema Validation。

#### 详细规格

1. **配置来源优先级**
   - CLI 显式传参 (最高)
   - 环境变量
   - 项目配置 (.opencode/config.jsonc)
   - 全局配置 (~/.config/opencode-rs/config.jsonc)
   - 内建默认值 (最低)

2. **合并原则**
   - 标量：后者覆盖
   - map：递归合并
   - list：按策略合并 (append/replace)
   - agent/command/mcp：按 key 合并

3. **JSONC 支持**
   - 解析 JSONC 格式
   - 注释支持
   - 多行字符串

4. **Schema Validation**
   - 配置加载时验证
   - 错误提示

#### 验收标准

- [ ] JSONC 格式正确解析
- [ ] 环境变量覆盖配置
- [ ] 多层配置正确合并
- [ ] Schema 验证正常工作

---

### FR-012: Session Summarize 完善

**模块**: core  
**严重程度**: P2  
**差距项**: Session summarize 不完整

#### 需求描述

实现自动 Compact 逻辑。

#### 详细规格

1. **自动 Compact 阈值**
   - 85%：预警
   - 92%：触发 compact
   - 95%：强制转入新 session continuation

2. **摘要生成**
   - 保留关键信息
   - 压缩冗长对话
   - 保持上下文连贯性

3. **Checkpoint 粒度**
   - 每次消息后持久化
   - 支持回滚到任意点

#### 验收标准

- [ ] 85% 阈值前触发预警
- [ ] 92% 阈值触发自动 compact
- [ ] 每次消息后 checkpoint 持久化
- [ ] 可恢复到历史 checkpoint

---

### FR-013: Web UI 实现

**模块**: tui  
**严重程度**: P2  
**差距项**: Web UI 未实现

#### 需求描述

实现 PRD 中规划的 Web Shell v1.5。

#### 详细规格

1. **Web Shell 功能**
   - 浏览器访问
   - 会话管理
   - 消息输入
   - 工具结果展示

2. **技术选型**
   - 前端框架 (React/Vue)
   - WebSocket 通信
   - 响应式布局

#### 验收标准

- [ ] 可通过浏览器访问
- [ ] 基本会话功能可用
- [ ] 消息流式展示

---

### FR-014: IDE 扩展预留

**模块**: -  
**严重程度**: P2  
**差距项**: IDE 扩展未实现

#### 需求描述

为 IDE 扩展预留接口 (v2 目标)。

#### 详细规格

1. **预留接口**
   - LSP 协议兼容
   - MCP 协议兼容
   - Server-first 架构

2. **SDK 输出**
   - Rust SDK
   - TypeScript SDK

#### 验收标准

- [ ] SDK 可用
- [ ] 协议接口稳定

---

### FR-015: GitHub 集成预留

**模块**: git  
**严重程度**: P2  
**差距项**: GitHub 集成未实现

#### 需求描述

为 GitHub 集成预留接口 (v2 目标)。

#### 详细规格

1. **GitHub 功能 (v2)**
   - GitHub Action Runner 集成
   - issue/PR comment trigger
   - 自动新分支
   - 自动提交 patch
   - 自动创建 PR
   - 安全沙箱与密钥隔离

2. **当前准备**
   - Git 工具基础能力
   - API 抽象预留

#### 验收标准

- [ ] Git 基础工具可用
- [ ] 架构支持后续扩展

---

## 6. 功能需求清单汇总

### 按模块分组

| 模块 | FR 编号 | 需求名称 | 优先级 |
|------|---------|----------|--------|
| core | FR-001 | Context Engine | P0 |
| core | FR-003 | Skills 系统 | P0 |
| core | FR-004 | Commands 系统 | P0 |
| core | FR-007 | Share 功能 | P1 |
| core | FR-009 | 插件事件总线 | P1 |
| core | FR-011 | 配置系统 | P2 |
| core | FR-012 | Session Summarize | P2 |
| plugin | FR-002 | Plugin System | P0 |
| mcp | FR-005 | MCP 工具接入 | P0 |
| server | FR-006 | Server API | P1 |
| lsp | FR-008 | LSP 功能增强 | P1 |
| auth | FR-010 | 凭证加密存储 | P1 |
| tui | FR-013 | Web UI | P2 |
| - | FR-014 | IDE 扩展预留 | P2 |
| git | FR-015 | GitHub 集成预留 | P2 |

### 按优先级分组

| 优先级 | FR 编号 |
|--------|---------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005 |
| P1 | FR-006, FR-007, FR-008, FR-009, FR-010 |
| P2 | FR-011, FR-012, FR-013, FR-014, FR-015 |

---

## 7. 验收标准模板

每个需求应满足以下验收标准格式：

```markdown
#### 验收标准

- [ ] 功能正常运行
- [ ] 错误处理正确
- [ ] 性能满足要求
- [ ] 文档完整
- [ ] 测试覆盖
```

---

## 8. 实施建议

### Phase 1: P0 阻断性问题 (当前优先级)

1. **FR-001 Context Engine** - 核心依赖，其他功能需要
2. **FR-005 MCP 工具接入** - 工具系统扩展
3. **FR-004 Commands 系统** - TUI 输入增强
4. **FR-003 Skills 系统** - 上下文增强
5. **FR-002 Plugin System** - 扩展性基础

### Phase 2: P1 核心功能

1. **FR-006 Server API** - API 完整性
2. **FR-008 LSP 功能增强** - 开发体验
3. **FR-007 Share 功能** - 协作能力
4. **FR-010 凭证加密存储** - 安全合规
5. **FR-009 插件事件总线** - 事件系统

### Phase 3: P2 完善性

1. **FR-011 配置系统** - 配置灵活性
2. **FR-012 Session Summarize** - 会话管理
3. **FR-013 Web UI** - 多端支持
4. **FR-014 IDE 扩展预留** - 生态扩展
5. **FR-015 GitHub 集成预留** - DevOps 集成

---

## 9. 附录

### A. 数据模型状态

| PRD 数据模型 | 实现状态 | 备注 |
|--------------|----------|------|
| Session | ✅ 完整 | |
| Message | ✅ 完整 | |
| ToolCall | ✅ 完整 | |
| Snapshot | ✅ 完整 | |
| PermissionDecision | ✅ 完整 | |
| Provider/Credential | ✅ 完整 | |
| Project | ✅ 完整 | |
| Checkpoint | ✅ 完整 | |

### B. API 状态

| PRD API 路径 | 实现状态 | FR 编号 |
|--------------|----------|---------|
| POST /sessions | ⚠️ 部分 | FR-006 |
| GET /sessions | ⚠️ 部分 | FR-006 |
| GET /sessions/{id} | ⚠️ 部分 | FR-006 |
| POST /sessions/{id}/fork | ❌ 未实现 | FR-006 |
| POST /sessions/{id}/summarize | ⚠️ 部分 | FR-006, FR-012 |
| POST /sessions/{id}/abort | ✅ 已实现 | |
| POST /sessions/{id}/prompt | ⚠️ 部分 | FR-006 |
| GET /sessions/{id}/messages | ⚠️ 部分 | FR-006 |
| POST /sessions/{id}/shell | ✅ 已实现 | |
| POST /sessions/{id}/command | ❌ 未实现 | FR-004, FR-006 |
| POST /sessions/{id}/permissions/{req_id}/reply | ✅ 已实现 | |
| GET /sessions/{id}/diff | ⚠️ 部分 | FR-006 |
| GET /sessions/{id}/snapshots | ⚠️ 部分 | FR-006 |
| POST /sessions/{id}/revert | ⚠️ 部分 | FR-006 |
| GET /providers | ⚠️ 部分 | FR-006 |
| GET /models | ⚠️ 部分 | FR-006 |

### C. 功能状态

| PRD 功能 | 实现状态 | FR 编号 |
|----------|----------|---------|
| Workspace/Project 机制 | ✅ 已实现 | |
| Session 会话系统 | ✅ 已实现 | |
| Agent 系统 | ✅ 已实现 | |
| 文件工具 | ✅ 已实现 | |
| Shell 工具 | ✅ 已实现 | |
| Git 工具 | ⚠️ 基础 | FR-015 |
| 权限系统 | ✅ 已实现 | |
| @file 引用 | ⚠️ 部分 | |
| !shell 直接执行 | ⚠️ 部分 | |
| /command 快捷命令 | ❌ 未实现 | FR-004 |
| Skills 延迟加载 | ❌ 未实现 | FR-003 |
| Commands 自定义 | ❌ 未实现 | FR-004 |
| MCP 本地/远程 | ⚠️ 基础 | FR-005 |
| LSP diagnostics | ⚠️ 基础 | FR-008 |
| Server API | ⚠️ 基础 | FR-006 |
| Share 分享 | ❌ 未实现 | FR-007 |
| Plugin WASM 宿主 | ❌ 未实现 | FR-002 |
| 凭证加密存储 | ❌ 未实现 | FR-010 |

---

**文档状态**: 草稿  
**下一步**: 基于本规格文档创建迭代计划
