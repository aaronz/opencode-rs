# OpenCode-RS 任务清单 v2

**版本**: 2.0  
**日期**: 2026-04-04  
**基于**: plan_v2.md (实现计划)  
**状态**: 草稿

---

## 1. 任务总览

| Phase | 优先级 | 任务数 | 状态 |
|-------|--------|--------|------|
| Phase 1 | P0 | 5 | 待开始 |
| Phase 2 | P1 | 5 | 待开始 |
| Phase 3 | P2 | 5 | 待开始 |

---

## 2. Phase 1: P0 阻断性问题

### Task 1.1: FR-001 Context Engine 实现

**ID**: TASK-1.1  
**优先级**: P0  
**模块**: core  
**状态**: pending  
**预计工期**: 5 天

**目标**: 实现 Token Budget 管理、Context Ranking 和 Compaction 机制

**子任务**:
- [ ] TASK-1.1.1: 实现 Token Budget 计算模块
  - 实现 PRD Section 7.6 定义的上下文层次 (L0-L4)
  - 70% token 用于主要上下文
  - 20% 留给工具输出
  - 10% 留给响应空间
  - 支持 token 统计与预算预警
- [ ] TASK-1.1.2: 实现 Context Ranking 模块
  - 实现文件相关性评分算法
  - 支持最近文件优先
  - 支持基于 LSP 符号关系的排序
- [ ] TASK-1.1.3: 实现 Context Compaction 模块
  - 85% 预警阈值
  - 92% 触发 compact
  - 95% 强制新 session continuation
  - 支持摘要生成与替换

**验收标准**:
- [ ] Token 预算计算误差 < 5%
- [ ] 85% 阈值前触发预警
- [ ] 92% 阈值触发自动 compact
- [ ] Compact 后上下文可正常使用

**依赖**: 无

---

### Task 1.2: FR-002 Plugin System 实现

**ID**: TASK-1.2  
**优先级**: P0  
**模块**: plugin  
**状态**: pending  
**预计工期**: 5 天

**目标**: 实现 WASM 运行时和事件总线

**子任务**:
- [ ] TASK-1.2.1: 实现 WASM 插件运行时
  - 集成 wasmtime 引擎
  - 插件隔离沙箱
  - 插件 API 能力授权机制
- [ ] TASK-1.2.2: 实现事件总线
  - 实现 PRD Section 7.10 定义的事件类型
  - 支持事件订阅/取消订阅
  - 支持事件处理器添加/移除
- [ ] TASK-1.2.3: 实现插件能力
  - 监听事件
  - 改写 prompt
  - 注入 shell 环境变量
  - 添加工具
  - 增加 context sources
  - 拦截敏感读取
  - 发送通知

**验收标准**:
- [ ] WASM 插件可加载并执行
- [ ] 事件总线支持所有定义的事件类型
- [ ] 插件崩溃不影响主进程
- [ ] 插件需通过 capability 授权

**依赖**: TASK-1.1 (Context Engine 可选)

---

### Task 1.3: FR-003 Skills 系统实现

**ID**: TASK-1.3  
**优先级**: P0  
**模块**: core  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 Skill Loader 和 Semantic Matching

**子任务**:
- [ ] TASK-1.3.1: 定义 Skill 结构
  - 路径: `.opencode/skills/<name>/SKILL.md`
  - 元信息 (frontmatter): name, description, triggers, priority
- [ ] TASK-1.3.2: 实现延迟加载机制
  - 启动时不预加载所有 skills
  - 按需发现并加载
  - 支持列出技能目录
- [ ] TASK-1.3.3: 实现语义匹配
  - 根据 prompt 语义匹配 skills
  - 支持手动指定
  - 支持全局与项目级别覆盖

**验收标准**:
- [ ] Skills 可从指定路径加载
- [ ] 支持触发词匹配
- [ ] 支持延迟加载
- [ ] 支持全局/项目级别覆盖

**依赖**: TASK-1.1 (Context Engine - token budget 计算)

---

### Task 1.4: FR-004 Commands 系统实现

**ID**: TASK-1.4  
**优先级**: P0  
**模块**: core  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 Command Parser 和 Template Expansion

**子任务**:
- [ ] TASK-1.4.1: 实现命令定义格式
  - 从 `.opencode/commands/*.md` 加载
  - 支持 YAML frontmatter + Markdown body
  - 可绑定 agent/model
- [ ] TASK-1.4.2: 实现变量支持
  - `${file}` - 当前文件
  - `${selection}` - 选中文本
  - `${cwd}` - 工作目录
  - `${git_branch}` - Git 分支
  - `${input}` - 用户输入
- [ ] TASK-1.4.3: 实现执行流程
  - `/test` → 解析模板 → 注入变量 → 创建用户消息 → 发送到 Session Engine
- [ ] TASK-1.4.4: 实现内置命令
  - /help, /init, /undo, /redo, /share, /agent, /model, /clear

**验收标准**:
- [ ] Commands 可从指定路径加载
- [ ] 支持变量替换
- [ ] 可绑定 agent/model
- [ ] 内置命令可用

**依赖**: 无

---

### Task 1.5: FR-005 MCP 工具接入完善

**ID**: TASK-1.5  
**优先级**: P0  
**模块**: mcp  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 Schema Cache 和 Permission 集成

**子任务**:
- [ ] TASK-1.5.1: 实现工具发现
  - 支持本地 MCP (stdio)
  - 支持远程 MCP (HTTP)
  - 工具列表获取
- [ ] TASK-1.5.2: 实现 Schema 缓存
  - 工具调用前只注入元信息
  - 避免预灌满上下文
  - 缓存失效机制
- [ ] TASK-1.5.3: 实现 Token 成本控制
  - 默认禁用重型 MCP
  - 只有模型明确请求时再执行
  - 执行结果摘要优先
- [ ] TASK-1.5.4: 实现 Permission 集成
  - 远程 MCP 默认 ask
  - 权限请求与内置工具统一处理
  - 审计日志记录

**验收标准**:
- [ ] 本地/远程 MCP 可配置
- [ ] Schema 缓存正常工作
- [ ] Token 成本可控
- [ ] Permission 集成正常

**依赖**: 无

---

## 3. Phase 2: P1 核心功能

### Task 2.1: FR-006 Server API 完善

**ID**: TASK-2.1  
**优先级**: P1  
**模块**: server  
**状态**: pending  
**预计工期**: 5 天

**目标**: 完善 Session/Messages/Tool/Artifact API

**子任务**:
- [ ] TASK-2.1.1: 实现 Session API 完整功能
  - POST /sessions, GET /sessions, GET /sessions/{id}
  - POST /sessions/{id}/fork, /summarize, /abort
  - POST /sessions/{id}/prompt
- [ ] TASK-2.1.2: 实现 Message API 完整功能
  - GET /sessions/{id}/messages
  - GET /sessions/{id}/messages/{msg_id}
- [ ] TASK-2.1.3: 实现 Tool API 完整功能
  - POST /sessions/{id}/shell
  - POST /sessions/{id}/command
  - POST /sessions/{id}/permissions/{req_id}/reply
- [ ] TASK-2.1.4: 实现 Artifact API 完整功能
  - GET /sessions/{id}/diff
  - GET /sessions/{id}/snapshots
  - POST /sessions/{id}/revert
- [ ] TASK-2.1.5: 实现 Runtime API
  - GET /doc, GET /health
  - GET /providers, GET /models

**验收标准**:
- [ ] 所有 API 端点可访问
- [ ] REST API 返回正确状态码
- [ ] 流式响应正常工作 (SSE/WS)
- [ ] OpenAPI 文档生成

**依赖**: 无

---

### Task 2.2: FR-007 Share 功能实现

**ID**: TASK-2.2  
**优先级**: P1  
**模块**: core  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 Export JSON/Markdown 和 Share Server

**子任务**:
- [ ] TASK-2.2.1: 实现本地导出
  - 导出 session JSON
  - 导出 Markdown transcript
  - 导出 patch bundle
- [ ] TASK-2.2.2: 实现服务层 (可选)
  - self-hosted share server
  - 短链生成
  - 访问令牌
  - 过期时间
- [ ] TASK-2.2.3: 实现默认策略
  - 默认关闭自动分享
  - 手动触发
  - 明确提示"将上传对话内容"
  - 导出前脱敏检查

**验收标准**:
- [ ] 可导出 JSON 格式
- [ ] 可导出 Markdown 格式
- [ ] 可导出 patch bundle
- [ ] 敏感信息自动脱敏

**依赖**: TASK-2.1 (Server API)

---

### Task 2.3: FR-008 LSP 功能增强

**ID**: TASK-2.3  
**优先级**: P1  
**模块**: lsp  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 Definition、References、Hover

**子任务**:
- [ ] TASK-2.3.1: 验证 v1 已实现功能
  - diagnostics
  - workspace symbols
  - document symbols
- [ ] TASK-2.3.2: 实现 v1.1 扩展功能
  - definition (跳转到定义)
  - references (查找引用)
  - hover (悬停信息)
  - code actions (只读建议)
- [ ] TASK-2.3.3: 实现 LSP 集成原则
  - 增量刷新
  - 自动重连

**验收标准**:
- [ ] diagnostics 正常工作
- [ ] workspace symbols 可搜索
- [ ] definition 跳转功能
- [ ] references 查找功能

**依赖**: 无

---

### Task 2.4: FR-009 插件事件总线完善

**ID**: TASK-2.4  
**优先级**: P1  
**模块**: core  
**状态**: pending  
**预计工期**: 2 天

**目标**: 完善事件类型覆盖

**子任务**:
- [ ] TASK-2.4.1: 实现核心事件
  - session.created
  - session.updated
  - session.compacted
  - message.updated
- [ ] TASK-2.4.2: 实现工具事件
  - tool.execute.before
  - tool.execute.after
- [ ] TASK-2.4.3: 实现权限事件
  - permission.asked
  - permission.replied
- [ ] TASK-2.4.4: 实现系统事件
  - file.edited
  - lsp.updated
  - shell.env
  - tui.toast.show

**验收标准**:
- [ ] 所有事件类型可触发
- [ ] 事件可订阅/取消订阅
- [ ] 事件处理器可添加/移除

**依赖**: TASK-1.2 (Plugin System - 事件总线基础)

---

### Task 2.5: FR-010 凭证加密存储实现

**ID**: TASK-2.5  
**优先级**: P1  
**模块**: auth  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 Auth Store 加密

**子任务**:
- [ ] TASK-2.5.1: 实现加密存储
  - 明文 secret 不落 SQLite 主表
  - 使用系统密钥链或本地加密封装
  - 文件权限默认收紧到当前用户
- [ ] TASK-2.5.2: 扩展 Auth Store 结构
  - credential id, provider id, auth strategy
  - secret ciphertext
  - expires_at / refreshed_at
  - scopes / account metadata
  - created_at / updated_at / revoked_at
- [ ] TASK-2.5.3: 实现 OAuth/Session 扩展
  - access token, refresh token, expiry
  - account id / workspace id

**验收标准**:
- [ ] 凭证加密存储
- [ ] 支持系统密钥链集成
- [ ] 导出 session 时排除 auth store
- [ ] 凭证可撤销

**依赖**: 无

---

## 4. Phase 3: P2 完善性

### Task 3.1: FR-011 配置系统完善

**ID**: TASK-3.1  
**优先级**: P2  
**模块**: core  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 JSONC Loader、Env Override、Schema Validation

**子任务**:
- [ ] TASK-3.1.1: 实现配置来源优先级
  - CLI 显式传参 (最高)
  - 环境变量
  - 项目配置 (.opencode/config.jsonc)
  - 全局配置 (~/.config/opencode-rs/config.jsonc)
  - 内建默认值 (最低)
- [ ] TASK-3.1.2: 实现合并原则
  - 标量：后者覆盖
  - map：递归合并
  - list：按策略合并 (append/replace)
  - agent/command/mcp：按 key 合并
- [ ] TASK-3.1.3: 实现 JSONC 支持
  - 解析 JSONC 格式
  - 注释支持
  - 多行字符串
- [ ] TASK-3.1.4: 实现 Schema Validation
  - 配置加载时验证
  - 错误提示

**验收标准**:
- [ ] JSONC 格式正确解析
- [ ] 环境变量覆盖配置
- [ ] 多层配置正确合并
- [ ] Schema 验证正常工作

**依赖**: 无

---

### Task 3.2: FR-012 Session Summarize 完善

**ID**: TASK-3.2  
**优先级**: P2  
**模块**: core  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现自动 Compact 逻辑

**子任务**:
- [ ] TASK-3.2.1: 实现自动 Compact 阈值
  - 85%：预警
  - 92%：触发 compact
  - 95%：强制转入新 session continuation
- [ ] TASK-3.2.2: 实现摘要生成
  - 保留关键信息
  - 压缩冗长对话
  - 保持上下文连贯性
- [ ] TASK-3.2.3: 实现 Checkpoint 粒度
  - 每次消息后持久化
  - 支持回滚到任意点

**验收标准**:
- [ ] 85% 阈值前触发预警
- [ ] 92% 阈值触发自动 compact
- [ ] 每次消息后 checkpoint 持久化
- [ ] 可恢复到历史 checkpoint

**依赖**: TASK-1.1 (Context Engine - compaction 逻辑)

---

### Task 3.3: FR-013 Web UI 实现

**ID**: TASK-3.3  
**优先级**: P2  
**模块**: tui  
**状态**: pending  
**预计工期**: 5 天

**目标**: 实现 PRD 中规划的 Web Shell v1.5

**子任务**:
- [ ] TASK-3.3.1: 实现 Web Shell 功能
  - 浏览器访问
  - 会话管理
  - 消息输入
  - 工具结果展示
- [ ] TASK-3.3.2: 选择技术栈
  - 前端框架 (React/Vue)
  - WebSocket 通信
  - 响应式布局

**验收标准**:
- [ ] 可通过浏览器访问
- [ ] 基本会话功能可用
- [ ] 消息流式展示

**依赖**: TASK-2.1 (Server API)

---

### Task 3.4: FR-014 IDE 扩展预留

**ID**: TASK-3.4  
**优先级**: P2  
**模块**: -  
**状态**: pending  
**预计工期**: 2 天

**目标**: 为 IDE 扩展预留接口 (v2 目标)

**子任务**:
- [ ] TASK-3.4.1: 预留接口
  - LSP 协议兼容
  - MCP 协议兼容
  - Server-first 架构
- [ ] TASK-3.4.2: 输出 SDK
  - Rust SDK
  - TypeScript SDK

**验收标准**:
- [ ] SDK 可用
- [ ] 协议接口稳定

**依赖**: TASK-2.1 (Server API), TASK-2.3 (LSP)

---

### Task 3.5: FR-015 GitHub 集成预留

**ID**: TASK-3.5  
**优先级**: P2  
**模块**: git  
**状态**: pending  
**预计工期**: 2 天

**目标**: 为 GitHub 集成预留接口 (v2 目标)

**子任务**:
- [ ] TASK-3.5.1: 预留 GitHub 功能 (v2)
  - GitHub Action Runner 集成
  - issue/PR comment trigger
  - 自动新分支
  - 自动提交 patch
  - 自动创建 PR
  - 安全沙箱与密钥隔离
- [ ] TASK-3.5.2: 准备当前能力
  - Git 工具基础能力
  - API 抽象预留

**验收标准**:
- [ ] Git 基础工具可用
- [ ] 架构支持后续扩展

**依赖**: 无

---

## 5. 任务依赖图

```
Phase 1:
TASK-1.1 (Context Engine) ──────┬──────> TASK-1.3 (Skills)
         │                       │
         └───────────────────────┼──────> TASK-1.4 (Commands)
                                 │
TASK-1.2 (Plugin System) <──────┘
         │
         └──────> TASK-2.4 (Event Bus)

Phase 2:
TASK-2.1 (Server API) ──────────> TASK-2.2 (Share)
         │                             
         └────────────────> TASK-3.3 (Web UI)

TASK-2.3 (LSP) ─────────────────> TASK-3.4 (IDE)
TASK-2.5 (Credential) ─────────> (独立)

Phase 3:
TASK-1.1 (Context Engine) ─────> TASK-3.2 (Session Summarize)
TASK-2.1 (Server API) ─────────> TASK-3.3 (Web UI)
TASK-2.3 (LSP) ─────────────────> TASK-3.4 (IDE)
```

---

## 6. 任务状态追踪

| Phase | Task ID | 任务名称 | 状态 | 优先级 | 预计工期 |
|-------|---------|----------|------|--------|----------|
| 1 | TASK-1.1 | Context Engine | pending | P0 | 5d |
| 1 | TASK-1.2 | Plugin System | pending | P0 | 5d |
| 1 | TASK-1.3 | Skills 系统 | pending | P0 | 3d |
| 1 | TASK-1.4 | Commands 系统 | pending | P0 | 3d |
| 1 | TASK-1.5 | MCP 工具接入 | pending | P0 | 3d |
| 2 | TASK-2.1 | Server API | pending | P1 | 5d |
| 2 | TASK-2.2 | Share 功能 | pending | P1 | 3d |
| 2 | TASK-2.3 | LSP 功能增强 | pending | P1 | 3d |
| 2 | TASK-2.4 | 插件事件总线 | pending | P1 | 2d |
| 2 | TASK-2.5 | 凭证加密存储 | pending | P1 | 3d |
| 3 | TASK-3.1 | 配置系统 | pending | P2 | 3d |
| 3 | TASK-3.2 | Session Summarize | pending | P2 | 3d |
| 3 | TASK-3.3 | Web UI | pending | P2 | 5d |
| 3 | TASK-3.4 | IDE 扩展预留 | pending | P2 | 2d |
| 3 | TASK-3.5 | GitHub 集成预留 | pending | P2 | 2d |

**总计**: 15 tasks, 50 人天

---

## 7. 验收检查清单

每个任务完成后需满足:

- [ ] 功能正常运行
- [ ] 错误处理正确
- [ ] 性能满足要求
- [ ] 文档完整
- [ ] 测试覆盖

---

**文档状态**: 草稿  
**下一步**: 开始 Phase 1 任务
