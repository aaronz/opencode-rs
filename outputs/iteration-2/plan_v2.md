# OpenCode-RS 实现计划 v2

**版本**: 2.0  
**日期**: 2026-04-04  
**基于**: spec_v2.md (差距分析)  
**状态**: 草稿

---

## 1. 计划概述

### 1.1 目标

基于 spec_v2.md 中定义的 15 个需求 (FR-001 至 FR-015)，制定分阶段实现计划，确保 P0 阻断性问题优先解决。

### 1.2 阶段划分

| Phase | 优先级 | 任务数 | 目标 |
|-------|--------|--------|------|
| Phase 1 | P0 | 5 | 阻断性问题 - 核心基础设施 |
| Phase 2 | P1 | 5 | 核心功能 - API 与协作 |
| Phase 3 | P2 | 5 | 完善性 - 体验与扩展 |

### 1.3 依赖关系图

```
FR-001 (Context Engine)
├── FR-003 (Skills) - 依赖 token budget 计算
├── FR-005 (MCP) - 依赖 token cost control
└── FR-012 (Session Summarize) - 依赖 compaction

FR-002 (Plugin System)
└── FR-009 (Event Bus) - 事件系统是插件基础

FR-004 (Commands)
└── FR-006 (Server API) - /command 端点需要 API 支持

FR-006 (Server API)
├── FR-007 (Share) - 需要 API 支持导出
└── FR-013 (Web UI) - 需要 API 支持前端
```

---

## 2. Phase 1: P0 阻断性问题

### 2.1 目标

实现核心基础设施，解决阻断性问题。

### 2.2 任务清单

#### Task 1.1: FR-001 Context Engine 实现

**目标**: 实现 Token Budget 管理、Context Ranking 和 Compaction 机制

**子任务**:
1. Token Budget 计算模块
   - 实现 PRD Section 7.6 定义的上下文层次 (L0-L4)
   - 70% token 用于主要上下文
   - 20% 留给工具输出
   - 10% 留给响应空间
   - 支持 token 统计与预算预警

2. Context Ranking 模块
   - 实现文件相关性评分算法
   - 支持最近文件优先
   - 支持基于 LSP 符号关系的排序

3. Context Compaction 模块
   - 85% 预警阈值
   - 92% 触发 compact
   - 95% 强制新 session continuation
   - 支持摘要生成与替换

**验收标准**:
- [ ] Token 预算计算误差 < 5%
- [ ] 85% 阈值前触发预警
- [ ] 92% 阈值触发自动 compact
- [ ] Compact 后上下文可正常使用

**预计工期**: 5 天

---

#### Task 1.2: FR-002 Plugin System 实现

**目标**: 实现 WASM 运行时和事件总线

**子任务**:
1. WASM 插件运行时
   - 集成 wasmtime 引擎
   - 插件隔离沙箱
   - 插件 API 能力授权机制

2. 事件总线
   - 实现 PRD Section 7.10 定义的事件类型
   - 支持事件订阅/取消订阅
   - 支持事件处理器添加/移除

3. 插件能力
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

**预计工期**: 5 天

---

#### Task 1.3: FR-003 Skills 系统实现

**目标**: 实现 Skill Loader 和 Semantic Matching

**子任务**:
1. Skill 结构定义
   - 路径: `.opencode/skills/<name>/SKILL.md`
   - 元信息 (frontmatter): name, description, triggers, priority

2. 延迟加载机制
   - 启动时不预加载所有 skills
   - 按需发现并加载
   - 支持列出技能目录

3. 语义匹配
   - 根据 prompt 语义匹配 skills
   - 支持手动指定
   - 支持全局与项目级别覆盖

**验收标准**:
- [ ] Skills 可从指定路径加载
- [ ] 支持触发词匹配
- [ ] 支持延迟加载
- [ ] 支持全局/项目级别覆盖

**预计工期**: 3 天

---

#### Task 1.4: FR-004 Commands 系统实现

**目标**: 实现 Command Parser 和 Template Expansion

**子任务**:
1. 命令定义格式
   - 从 `.opencode/commands/*.md` 加载
   - 支持 YAML frontmatter + Markdown body
   - 可绑定 agent/model

2. 变量支持
   - `${file}` - 当前文件
   - `${selection}` - 选中文本
   - `${cwd}` - 工作目录
   - `${git_branch}` - Git 分支
   - `${input}` - 用户输入

3. 执行流程
   - `/test` → 解析模板 → 注入变量 → 创建用户消息 → 发送到 Session Engine

4. 内置命令
   - /help, /init, /undo, /redo, /share, /agent, /model, /clear

**验收标准**:
- [ ] Commands 可从指定路径加载
- [ ] 支持变量替换
- [ ] 可绑定 agent/model
- [ ] 内置命令可用

**预计工期**: 3 天

---

#### Task 1.5: FR-005 MCP 工具接入完善

**目标**: 实现 Schema Cache 和 Permission 集成

**子任务**:
1. 工具发现
   - 支持本地 MCP (stdio)
   - 支持远程 MCP (HTTP)
   - 工具列表获取

2. Schema 缓存
   - 工具调用前只注入元信息
   - 避免预灌满上下文
   - 缓存失效机制

3. Token 成本控制
   - 默认禁用重型 MCP
   - 只有模型明确请求时再执行
   - 执行结果摘要优先

4. Permission 集成
   - 远程 MCP 默认 ask
   - 权限请求与内置工具统一处理
   - 审计日志记录

**验收标准**:
- [ ] 本地/远程 MCP 可配置
- [ ] Schema 缓存正常工作
- [ ] Token 成本可控
- [ ] Permission 集成正常

**预计工期**: 3 天

---

### 2.3 Phase 1 并行策略

**可并行执行**:
- Task 1.3 (Skills) 和 Task 1.4 (Commands) - 两者独立，都依赖 FR-001
- Task 1.5 (MCP) - 可独立实现基础能力

**串行执行**:
- Task 1.1 (Context Engine) - 核心依赖，需优先
- Task 1.2 (Plugin System) - 依赖 Event Bus 完善

**依赖链**:
```
Context Engine (1.1) → Skills (1.3), Commands (1.4), MCP (1.5)
                                ↓
                    Plugin System (1.2)
```

---

## 3. Phase 2: P1 核心功能

### 3.1 目标

完善 Server API 和协作能力。

### 3.2 任务清单

#### Task 2.1: FR-006 Server API 完善

**目标**: 完善 Session/Messages/Tool/Artifact API

**子任务**:
1. Session API 完整实现
   - POST /sessions, GET /sessions, GET /sessions/{id}
   - POST /sessions/{id}/fork, /summarize, /abort
   - POST /sessions/{id}/prompt

2. Message API 完整实现
   - GET /sessions/{id}/messages
   - GET /sessions/{id}/messages/{msg_id}

3. Tool API 完整实现
   - POST /sessions/{id}/shell
   - POST /sessions/{id}/command
   - POST /sessions/{id}/permissions/{req_id}/reply

4. Artifact API 完整实现
   - GET /sessions/{id}/diff
   - GET /sessions/{id}/snapshots
   - POST /sessions/{id}/revert

5. Runtime API
   - GET /doc, GET /health
   - GET /providers, GET /models

**验收标准**:
- [ ] 所有 API 端点可访问
- [ ] REST API 返回正确状态码
- [ ] 流式响应正常工作 (SSE/WS)
- [ ] OpenAPI 文档生成

**预计工期**: 5 天

---

#### Task 2.2: FR-007 Share 功能实现

**目标**: 实现 Export JSON/Markdown 和 Share Server

**子任务**:
1. 本地导出
   - 导出 session JSON
   - 导出 Markdown transcript
   - 导出 patch bundle

2. 服务层 (可选)
   - self-hosted share server
   - 短链生成
   - 访问令牌
   - 过期时间

3. 默认策略
   - 默认关闭自动分享
   - 手动触发
   - 明确提示"将上传对话内容"
   - 导出前脱敏检查

**验收标准**:
- [ ] 可导出 JSON 格式
- [ ] 可导出 Markdown 格式
- [ ] 可导出 patch bundle
- [ ] 敏感信息自动脱敏

**预计工期**: 3 天

---

#### Task 2.3: FR-008 LSP 功能增强

**目标**: 实现 Definition、References、Hover

**子任务**:
1. v1 已实现功能验证
   - diagnostics
   - workspace symbols
   - document symbols

2. v1.1 扩展功能
   - definition (跳转到定义)
   - references (查找引用)
   - hover (悬停信息)
   - code actions (只读建议)

3. LSP 集成原则
   - LSP 是"结构化上下文增强器"
   - 增量刷新
   - 自动重连

**验收标准**:
- [ ] diagnostics 正常工作
- [ ] workspace symbols 可搜索
- [ ] definition 跳转功能
- [ ] references 查找功能

**预计工期**: 3 天

---

#### Task 2.4: FR-009 插件事件总线完善

**目标**: 完善事件类型覆盖

**子任务**:
实现所有 PRD Section 7.10 定义的事件:
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

**验收标准**:
- [ ] 所有事件类型可触发
- [ ] 事件可订阅/取消订阅
- [ ] 事件处理器可添加/移除

**预计工期**: 2 天

---

#### Task 2.5: FR-010 凭证加密存储实现

**目标**: 实现 Auth Store 加密

**子任务**:
1. 加密存储
   - 明文 secret 不落 SQLite 主表
   - 使用系统密钥链或本地加密封装
   - 文件权限默认收紧到当前用户

2. Auth Store 结构扩展
   - credential id, provider id, auth strategy
   - secret ciphertext
   - expires_at / refreshed_at
   - scopes / account metadata
   - created_at / updated_at / revoked_at

3. OAuth/Session 扩展
   - access token, refresh token, expiry
   - account id / workspace id

**验收标准**:
- [ ] 凭证加密存储
- [ ] 支持系统密钥链集成
- [ ] 导出 session 时排除 auth store
- [ ] 凭证可撤销

**预计工期**: 3 天

---

### 3.3 Phase 2 并行策略

**可并行执行**:
- Task 2.2 (Share), Task 2.3 (LSP), Task 2.5 (Credential) - 独立功能

**依赖**:
- Task 2.1 (Server API) 完成后，Task 2.2 (Share) 可更好地实现 API 导出

---

## 4. Phase 3: P2 完善性

### 4.1 目标

完善配置、体验和扩展能力。

### 4.2 任务清单

#### Task 3.1: FR-011 配置系统完善

**目标**: 实现 JSONC Loader、Env Override、Schema Validation

**子任务**:
1. 配置来源优先级实现
   - CLI 显式传参 (最高)
   - 环境变量
   - 项目配置 (.opencode/config.jsonc)
   - 全局配置 (~/.config/opencode-rs/config.jsonc)
   - 内建默认值 (最低)

2. 合并原则
   - 标量：后者覆盖
   - map：递归合并
   - list：按策略合并 (append/replace)
   - agent/command/mcp：按 key 合并

3. JSONC 支持
   - 解析 JSONC 格式
   - 注释支持
   - 多行字符串

4. Schema Validation
   - 配置加载时验证
   - 错误提示

**验收标准**:
- [ ] JSONC 格式正确解析
- [ ] 环境变量覆盖配置
- [ ] 多层配置正确合并
- [ ] Schema 验证正常工作

**预计工期**: 3 天

---

#### Task 3.2: FR-012 Session Summarize 完善

**目标**: 实现自动 Compact 逻辑

**子任务**:
1. 自动 Compact 阈值
   - 85%：预警
   - 92%：触发 compact
   - 95%：强制转入新 session continuation

2. 摘要生成
   - 保留关键信息
   - 压缩冗长对话
   - 保持上下文连贯性

3. Checkpoint 粒度
   - 每次消息后持久化
   - 支持回滚到任意点

**验收标准**:
- [ ] 85% 阈值前触发预警
- [ ] 92% 阈值触发自动 compact
- [ ] 每次消息后 checkpoint 持久化
- [ ] 可恢复到历史 checkpoint

**预计工期**: 3 天

---

#### Task 3.3: FR-013 Web UI 实现

**目标**: 实现 PRD 中规划的 Web Shell v1.5

**子任务**:
1. Web Shell 功能
   - 浏览器访问
   - 会话管理
   - 消息输入
   - 工具结果展示

2. 技术选型
   - 前端框架 (React/Vue)
   - WebSocket 通信
   - 响应式布局

**验收标准**:
- [ ] 可通过浏览器访问
- [ ] 基本会话功能可用
- [ ] 消息流式展示

**预计工期**: 5 天

---

#### Task 3.4: FR-014 IDE 扩展预留

**目标**: 为 IDE 扩展预留接口 (v2 目标)

**子任务**:
1. 预留接口
   - LSP 协议兼容
   - MCP 协议兼容
   - Server-first 架构

2. SDK 输出
   - Rust SDK
   - TypeScript SDK

**验收标准**:
- [ ] SDK 可用
- [ ] 协议接口稳定

**预计工期**: 2 天

---

#### Task 3.5: FR-015 GitHub 集成预留

**目标**: 为 GitHub 集成预留接口 (v2 目标)

**子任务**:
1. GitHub 功能 (v2)
   - GitHub Action Runner 集成
   - issue/PR comment trigger
   - 自动新分支
   - 自动提交 patch
   - 自动创建 PR
   - 安全沙箱与密钥隔离

2. 当前准备
   - Git 工具基础能力
   - API 抽象预留

**验收标准**:
- [ ] Git 基础工具可用
- [ ] 架构支持后续扩展

**预计工期**: 2 天

---

## 5. 资源分配

### 5.1 人力估算

| Phase | 任务数 | 预计工期 | 总人天 |
|-------|--------|----------|--------|
| Phase 1 | 5 | 19 天 | 19 人天 |
| Phase 2 | 5 | 16 天 | 16 人天 |
| Phase 3 | 5 | 15 天 | 15 人天 |
| **总计** | **15** | **50 天** | **50 人天** |

### 5.2 并行机会

- Phase 1: Skills (1.3) + Commands (1.4) 可并行
- Phase 2: LSP (2.3) + Credential (2.5) 可并行
- Phase 3: IDE (3.4) + GitHub (3.5) 可并行

---

## 6. 风险与缓解

### 6.1 技术风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| WASM 运行时集成复杂 | 中 | 高 | 提前进行 wasmtime 原型验证 |
| Token 计算误差 | 中 | 中 | 使用现有 tiktoken 库 |
| LSP 协议兼容性 | 高 | 中 | 复用现有 lsp-types 库 |

### 6.2 进度风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Phase 1 延期 | 中 | 高 | Buffer 3 天 |
| 依赖链阻塞 | 高 | 中 | 明确并行策略 |

---

## 7. 里程碑

| Milestone | Phase | 任务 | 预计完成 |
|-----------|-------|------|----------|
| M1 | Phase 1 | P0 阻断性问题 | Week 4 |
| M2 | Phase 2 | P1 核心功能 | Week 8 |
| M3 | Phase 3 | P2 完善性 | Week 11 |

---

## 8. 验收流程

### 8.1 每个任务的验收标准

1. 功能正常运行
2. 错误处理正确
3. 性能满足要求
4. 文档完整
5. 测试覆盖

### 8.2 阶段验收

- Phase 1 完成后进行代码评审
- Phase 2 完成后进行 API 评审
- Phase 3 完成后进行完整测试

---

**文档状态**: 草稿  
**下一步**: 创建 tasks_v2.md 任务清单
