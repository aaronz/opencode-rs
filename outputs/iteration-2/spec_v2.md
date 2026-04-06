# OpenCode-RS 规格文档 v2

**版本**: 2.0
**日期**: 2026-04-04
**基于**: PRD.md (2026-03) + Gap Analysis (iteration-2)
**状态**: 已完成

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
| P0 | 10 | 阻断性问题 |
| P1 | 10 | 核心功能缺失 |
| P2 | 12 | 完善性问题 |

---

## 3. P0 - 阻断性问题

### FR-001: Context Engine 实现

**模块**: core  
**严重程度**: P0  
**差距项**: Context Token Budget 压缩未实现 (P0-5)

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
**差距项**: Skills 延迟加载未实现 (P1-1)

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
**差距项**: Commands 系统未实现 (P0-1)

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

### FR-006: TUI 快捷输入解析器

**模块**: tui  
**严重程度**: P0  
**差距项**: `@file` / `!shell` / `/command` 快捷输入未完整实现 (P0-2)

#### 需求描述

实现 TUI 输入框的快捷输入解析，支持文件引用、Shell 执行、命令快捷方式。

#### 详细规格

1. **@file 引用解析**
   - 语法: `@<file-path>` 或 `@<filename>`
   - 支持相对路径和绝对路径
   - 支持 glob 模式 (如 `@*.ts`)
   - 解析结果作为 context 注入

2. **!shell 直接执行**
   - 语法: `!<shell-command>`
   - 解析后直接执行 shell 命令
   - 结果直接展示，不经过 LLM
   - 需权限确认

3. **/command 快捷命令**
   - 语法: `/<command> [args]`
   - 调用 FR-004 定义的 Commands 系统
   - 内置命令: /help, /init, /undo, /redo, /share, /agent, /model, /clear

4. **输入解析优先级**
   - `/` 开头 → 命令模式
   - `!` 开头 → Shell 模式
   - `@` 开头 → 文件引用模式
   - 其他 → 普通 prompt

5. **快捷输入 UI**
   - 输入框显示解析后的模式标签
   - 支持 Tab 自动补全 (命令/文件)
   - ESC 取消快捷模式

#### 验收标准

- [ ] `@file` 正确解析文件路径
- [ ] `!shell` 直接执行 shell 命令
- [ ] `/command` 正确调用命令系统
- [ ] Tab 补全正常工作

---

### FR-007: Session Fork 实现

**模块**: storage/server  
**严重程度**: P0  
**差距项**: Session Fork 未实现 (P0-3)

#### 需求描述

实现 Session Fork 功能和存储层支持。

#### 详细规格

1. **数据模型扩展**
   - 新增 `parent_session_id` 字段
   - 记录 fork lineage
   - 支持多层级分叉

2. **Fork 执行流程**
   - 当前 session 完整复制
   - 继承历史消息
   - 新 session 独立演进
   - 记录 parent 引用

3. **REST API**
   - `POST /sessions/{id}/fork` - 分叉会话
   - 返回新 session ID
   - 支持指定初始消息

4. **UI 支持**
   - Fork 操作入口
   - Lineage 展示
   - Fork 来源追溯

#### 验收标准

- [ ] Session 可成功 fork
- [ ] Fork 后消息独立
- [ ] Fork lineage 正确记录
- [ ] REST API 正常工作

---

### FR-008: 多层配置合并实现

**模块**: config  
**严重程度**: P0  
**差距项**: 多层配置合并未实现 (P0-4)

#### 需求描述

实现多层配置优先级合并逻辑。

#### 详细规格

1. **配置来源优先级 (高→低)**
   - CLI 显式传参
   - 环境变量
   - 项目配置 (.opencode/config.jsonc)
   - 全局配置 (~/.config/opencode-rs/config.jsonc)
   - 内建默认值

2. **合并原则**
   - 标量：后者覆盖
   - map：递归合并
   - list：按策略合并 (append/replace)
   - agent/command/mcp：按 key 合并

3. **JSONC 支持**
   - 解析 JSONC 格式
   - 注释支持
   - 多行字符串

4. **环境变量绑定**
   - `OPENAI_API_KEY` → provider.openai.apiKey
   - `ANTHROPIC_API_KEY` → provider.anthropic.apiKey
   - `OPENCODE_TUI_CONFIG` → tui.configPath

5. **Schema Validation**
   - 配置加载时验证
   - 错误提示

#### 验收标准

- [ ] 配置优先级正确
- [ ] JSONC 格式正确解析
- [ ] 环境变量覆盖配置
- [ ] Schema 验证正常工作

---

### FR-009: .opencode 目录加载实现

**模块**: config  
**严重程度**: P0  
**差距项**: .opencode 目录加载未实现

#### 需求描述

实现 `.opencode/` 目录下的 agents、commands、modes、plugins、skills、tools、themes 加载。

#### 详细规格

1. **目录结构**
   ```
   .opencode/
   ├── config.json          # 主配置文件
   ├── agents/              # 自定义 agents
   │   └── *.md             # agent 定义
   ├── commands/            # 自定义命令
   │   └── *.md             # 命令定义
   ├── modes/               # 自定义模式
   │   └── *.md             # 模式定义
   ├── plugins/             # 插件
   │   └── *.wasm           # WASM 插件
   ├── skills/              # 技能
   │   └── */SKILL.md       # 技能定义
   ├── tools/               # 自定义工具
   │   └── *.md             # 工具定义
   └── themes/              # 主题
       └── *.json           # 主题文件
   ```

2. **加载规则**
   - 启动时扫描目录结构
   - 延迟加载非必要模块
   - 支持目录级别覆盖

3. **配置合并**
   - `.opencode/config.json` 与主配置合并
   - agents/commands 等独立注册
   - 同名配置项按优先级覆盖

#### 验收标准

- [ ] .opencode 目录结构正确识别
- [ ] agents/commands/modes 正确加载
- [ ] plugins/skills/tools/themes 目录可用
- [ ] 目录级覆盖生效

---

### FR-010: Provider 环境变量约定

**模块**: config/llm  
**严重程度**: P0  
**差距项**: 环境变量约定 - Provider-specific

#### 需求描述

实现 Provider-specific 环境变量自动绑定机制。

#### 详细规格

1. **环境变量映射**
   | 环境变量 | 映射到 |
   |----------|--------|
   | `OPENAI_API_KEY` | provider.openai.apiKey |
   | `ANTHROPIC_API_KEY` | provider.anthropic.apiKey |
   | `GOOGLE_GENERATIVEAI_API_KEY` | provider.google.apiKey |
   | `AZURE_OPENAI_API_KEY` | provider.azure.openai.apiKey |
   | `MISTRAL_API_KEY` | provider.mistral.apiKey |

2. **加载时机**
   - 启动时扫描环境变量
   - 自动注入到 provider 配置
   - 优先级低于显式配置

3. **Credential 引用**
   - 支持 `credentialRef` 引用机制
   - 不使用明文密钥
   - 通过 Auth Store 解析

4. **覆盖规则**
   - 显式配置 > 环境变量 > 默认值

#### 验收标准

- [ ] 环境变量自动绑定到 provider
- [ ] credentialRef 正确解析
- [ ] 覆盖规则正确执行

---

## 4. P1 - 核心功能缺失

### FR-011: Server API 完善

**模块**: server  
**严重程度**: P1  
**差距项**: Server API 不完整, Provider API 凭证管理不完整

#### 需求描述

完善 Session/Messages/Tool/Artifact/Provider API。

#### 详细规格

1. **Session API**
   - `POST /sessions` - 创建会话
   - `GET /sessions` - 列出会话
   - `GET /sessions/{id}` - 获取会话详情
   - `POST /sessions/{id}/fork` - 分叉会话 (FR-007)
   - `POST /sessions/{id}/summarize` - 摘要会话
   - `POST /sessions/{id}/abort` - 中止会话

2. **Message API**
   - `POST /sessions/{id}/prompt` - 发送 prompt
   - `GET /sessions/{id}/messages` - 获取消息列表
   - `GET /sessions/{id}/messages/{msg_id}` - 获取单条消息

3. **Tool API**
   - `POST /sessions/{id}/shell` - 执行 shell
   - `POST /sessions/{id}/command` - 执行命令 (FR-004)
   - `POST /sessions/{id}/permissions/{req_id}/reply` - 权限响应

4. **Artifact API**
   - `GET /sessions/{id}/diff` - 获取 diff
   - `GET /sessions/{id}/snapshots` - 获取快照列表
   - `POST /sessions/{id}/revert` - 恢复快照

5. **Provider API**
   - `GET /providers` - 列出 providers
   - `GET /models` - 列出 models
   - `POST /providers/{id}/credentials` - 设置凭证
   - `POST /providers/{id}/credentials/test` - 测试连通性
   - `DELETE /providers/{id}/credentials` - 撤销凭证
   - `GET /providers/{id}/status` - 凭证过期状态

6. **Runtime API**
   - `GET /doc` - API 文档
   - `GET /health` - 健康检查

#### 验收标准

- [ ] 所有 API 端点可访问
- [ ] REST API 返回正确状态码
- [ ] 流式响应正常工作 (SSE/WS)
- [ ] OpenAPI 文档生成
- [ ] Provider 凭证管理完整

---

### FR-012: Share 功能实现

**模块**: core  
**严重程度**: P1  
**差距项**: Share 导出功能缺失 (P1-3)

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

4. **存储扩展**
   - 新增 `shared_id` 字段
   - 分享状态跟踪

#### 验收标准

- [ ] 可导出 JSON 格式
- [ ] 可导出 Markdown 格式
- [ ] 可导出 patch bundle
- [ ] 敏感信息自动脱敏

---

### FR-013: LSP 功能增强

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

### FR-014: 插件事件总线完善

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

### FR-015: 凭证加密存储实现

**模块**: auth  
**严重程度**: P1  
**差距项**: Auth Store 加密未实现 (P2-5)

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

### FR-016: Permission 审计记录

**模块**: storage/permission  
**严重程度**: P1  
**差距项**: Permission 审计记录不完整 (P1-2)

#### 需求描述

完善权限决策审计表。

#### 详细规格

1. **审计表结构**
   - permission_id
   - session_id
   - tool_name
   - arguments (脱敏)
   - decision (allow/deny)
   - reason
   - timestamp
   - scope_granted

2. **Scope 记忆**
   - 记住用户同意的 scope
   - 同类操作自动允许
   - 支持范围扩大时的重新确认

3. **审计查询**
   - 按 session 过滤
   - 按时间范围过滤
   - 统计报表

#### 验收标准

- [ ] 权限决策完整记录
- [ ] Scope 记忆功能正常
- [ ] 审计日志可查询

---

### FR-017: TUI Token/Cost 显示

**模块**: tui  
**严重程度**: P1  
**差距项**: Token/Cost 统计未显示 (P1-5)

#### 需求描述

实现 TUI 中的 token 统计与成本显示。

#### 详细规格

1. **状态栏显示**
   - 当前 session token 计数
   - 累计 token 消耗
   - 预估成本 (基于模型定价)

2. **成本计算**
   - 支持多模型定价
   - 按 input/output 分别统计
   - 显示货币单位

3. **详细面板**
   - 点击展开详细统计
   - 按消息/按工具拆分
   - 历史 session 对比

4. **警告机制**
   - 超过阈值时颜色提示
   - 可配置阈值

#### 验收标准

- [ ] 状态栏显示 token 计数
- [ ] 成本计算正确
- [ ] 警告机制正常工作

---

### FR-018: TUI Schema 验证实现

**模块**: schema  
**严重程度**: P1  
**差距项**: TUI Schema 验证缺失

#### 需求描述

实现 `https://opencode.ai/tui.json` 的 Schema 验证。

#### 详细规格

1. **Schema 来源**
   - 远程获取: `https://opencode.ai/tui.json`
   - 本地缓存: `~/.config/opencode/schemas/tui.json`
   - 内建 fallback: 内置默认 schema

2. **验证时机**
   - TUI 配置加载时验证
   - 用户配置变更时验证
   - 启动时全局检查

3. **验证行为**
   - 验证失败时显示具体错误
   - 支持警告模式 (非阻断)
   - 提供修复建议

#### 验收标准

- [ ] TUI Schema 正确加载
- [ ] 验证错误提示清晰
- [ ] 离线模式正常工作

---

### FR-019: scroll_acceleration 结构修复

**模块**: config/tui  
**严重程度**: P1  
**差距项**: scroll_acceleration 配置结构不匹配

#### 需求描述

修复 scroll_acceleration 配置类型，使其与 PRD 定义匹配。

#### 详细规格

1. **当前实现**
   - 类型: `f32` (数字)
   - 示例: `scroll_acceleration: 0.5`

2. **PRD 要求**
   - 类型: `object` (对象)
   - 结构:
     ```jsonc
     {
       "scroll_acceleration": {
         "enabled": true
       }
     }
     ```

3. **兼容处理**
   - 支持旧格式 (数字) 作为 fallback
   - 新格式优先
   - 迁移时自动转换

#### 验收标准

- [ ] 支持新对象格式 `{"enabled": true}`
- [ ] 向后兼容旧数字格式
- [ ] 配置迁移平滑

---

### FR-020: keybinds 自定义绑定实现

**模块**: config/tui  
**严重程度**: P1  
**差距项**: keybinds 配置结构不完整

#### 需求描述

实现 keybinds 自定义绑定支持，允许用户定义自己的快捷键。

#### 详细规格

1. **当前实现**
   - 只有固定字段 (如 `scroll_speed`, `scroll_acceleration`)
   - 不支持自定义绑定

2. **PRD 要求**
   - 支持空对象 `{}`
   - 支持自定义键值对
   - 格式:
     ```jsonc
     {
       "keybinds": {
         "Ctrl+c": "copy",
         "Ctrl+v": "paste"
       }
     }
     ```

3. **扩展性**
   - 预留扩展接口
   - 支持插件注入绑定
   - 支持 keybinds 文件分离

#### 验收标准

- [ ] 支持空 keybinds `{}`
- [ ] 支持自定义键值绑定
- [ ] 绑定冲突处理正确

---

## 5. P2 - 完善性问题

### FR-021: 配置系统完善

**模块**: config  
**严重程度**: P2  
**差距项**: 配置系统不完整, JSONC 解析器缺失

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

### FR-022: Session Summarize 完善

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

### FR-023: TUI 布局切换

**模块**: tui  
**严重程度**: P2  
**差距项**: TUI 三栏/双栏切换 (P2-1)

#### 需求描述

实现可切换的 TUI 布局。

#### 详细规格

1. **布局模式**
   - 双栏模式: 左侧消息列表 + 右侧详情
   - 三栏模式: 左侧导航 + 中间消息 + 右侧面板

2. **切换方式**
   - 快捷键切换
   - 配置文件指定
   - 运行时动态切换

3. **右栏面板内容**
   - diagnostics
   - todo 列表
   - 权限队列
   - 文件预览

#### 验收标准

- [ ] 双栏/三栏切换正常
- [ ] 右栏面板功能完整
- [ ] 布局状态可持久化

---

### FR-024: TUI 右栏功能完善

**模块**: tui  
**严重程度**: P2  
**差距项**: TUI 右栏功能 - diagnostics/todo/权限队列

#### 需求描述

完善 TUI 右栏面板功能。

#### 详细规格

1. **Diagnostics 面板**
   - 当前文件错误/警告列表
   - 点击跳转
   - 过滤选项

2. **Todo 列表**
   - 当前 session todo 项
   - 状态标记
   - 优先级排序

3. **权限队列**
   - 待处理权限请求
   - 快速允许/拒绝
   - 历史决策

4. **文件预览**
   - 选中文件内容预览
   - 语法高亮
   - 搜索功能

#### 验收标准

- [ ] Diagnostics 显示正确
- [ ] Todo 列表功能完整
- [ ] 权限队列可操作

---

### FR-025: TUI Patch 预览展开

**模块**: tui  
**严重程度**: P2  
**差距项**: TUI Patch 预览展开

#### 需求描述

实现 diff 展开/收起交互。

#### 详细规格

1. **折叠模式**
   - 默认折叠显示
   - 显示文件变更统计
   - 展开/收起动画

2. **展开功能**
   - 点击展开单个文件
   - 全部展开/收起
   - 跳转到具体行

3. **Diff 展示**
   - 统一 diff 格式
   - 语法高亮
   - 上下文行数可配置

#### 验收标准

- [ ] 折叠/展开正常工作
- [ ] 动画流畅
- [ ] 跳转功能正常

---

### FR-026: Web UI 实现

**模块**: tui  
**严重程度**: P2  
**差距项**: Web UI 未实现 (P2-3)

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

### FR-027: IDE 扩展预留

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

### FR-028: GitHub 集成预留

**模块**: git  
**严重程度**: P2  
**差距项**: GitHub Integration 未实现 (P2-2)

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

### FR-029: OAuth 登录预留

**模块**: auth  
**严重程度**: P2  
**差距项**: OAuth 登录预留 (P2-4)

#### 需求描述

为 OAuth 登录预留扩展字段。

#### 详细规格

1. **扩展字段**
   - OAuth 状态存储
   - token 刷新机制
   - 多账户支持

2. **Provider 支持**
   - GitHub OAuth
   - Google OAuth
   - 自定义 OAuth

3. **会话管理**
   - OAuth session 存储
   - 自动刷新 token
   - 登出处理

#### 验收标准

- [ ] 扩展字段可用
- [ ] 支持主流 OAuth
- [ ] token 刷新正常

---

### FR-030: 废弃字段清理

**模块**: config  
**严重程度**: P2  
**差距项**: 废弃字段仍在代码中

#### 需求描述

清理已废弃的配置字段，避免混淆和潜在问题。

#### 详细规格

1. **已废弃字段**
   - `mode`: 使用 `agent` 替代
   - `layout`: 已废弃，无替代
   - `theme` (在 opencode.json): 移至 tui.json
   - `keybinds` (在 opencode.json): 移至 tui.json
   - `tui` (在 opencode.json): 移至 tui.json

2. **处理策略**
   - 标记废弃字段 (deprecation warning)
   - 迁移期后自动移除
   - 提供迁移指南

3. **迁移支持**
   - 自动检测旧格式
   - 提示用户迁移
   - 保持向后兼容

#### 验收标准

- [ ] 废弃字段标记清晰
- [ ] 使用时显示警告
- [ ] 文档说明迁移方式

---

### FR-031: theme 路径解析增强

**模块**: config/tui  
**严重程度**: P2  
**差距项**: theme 配置缺少 themes/ 目录扫描

#### 需求描述

增强 theme 配置，支持从 themes/ 目录自动扫描主题文件。

#### 详细规格

1. **当前实现**
   - `theme.name`: 使用内置主题
   - `theme.path`: 指定主题文件路径

2. **扩展功能**
   - 自动扫描 `.opencode/themes/` 目录
   - 自动扫描 `~/.config/opencode/themes/` 目录
   - 主题发现列表 API

3. **主题格式**
   - JSON 格式主题文件
   - 支持主题元信息 (name, author, description)
   - 支持主题继承

#### 验收标准

- [ ] themes/ 目录自动扫描
- [ ] 扫描结果可用于主题选择
- [ ] 自定义主题正确加载

---

### FR-032: Snapshot 元数据完善

**模块**: storage  
**严重程度**: P1  
**差距项**: Snapshot 元数据

#### 需求描述

完善 snapshots 表关联。

#### 详细规格

1. **元数据字段**
   - session_id
   - checkpoint_id
   - created_at
   - message_count
   - token_count
   - description

2. **关联查询**
   - 按 session 获取 snapshot 列表
   - 按时间范围查询
   - 差异对比

#### 验收标准

- [ ] Snapshot 元数据完整
- [ ] 关联查询正常
- [ ] 差异对比可用

---

## 6. 功能需求清单汇总

### 按模块分组

| 模块 | FR 编号 | 需求名称 | 优先级 |
|------|---------|----------|--------|
| core | FR-001 | Context Engine | P0 |
| core | FR-003 | Skills 系统 | P0 |
| core | FR-004 | Commands 系统 | P0 |
| core | FR-012 | Share 功能 | P1 |
| core | FR-014 | 插件事件总线 | P1 |
| core | FR-021 | 配置系统 | P2 |
| core | FR-022 | Session Summarize | P2 |
| config | FR-008 | 多层配置合并 | P0 |
| config | FR-009 | .opencode 目录加载 | P0 |
| config | FR-010 | Provider 环境变量约定 | P0 |
| config | FR-030 | 废弃字段清理 | P2 |
| config/tui | FR-019 | scroll_acceleration 结构修复 | P1 |
| config/tui | FR-020 | keybinds 自定义绑定 | P1 |
| config/tui | FR-031 | theme 路径解析增强 | P2 |
| schema | FR-018 | TUI Schema 验证 | P1 |
| plugin | FR-002 | Plugin System | P0 |
| mcp | FR-005 | MCP 工具接入 | P0 |
| server | FR-006 | TUI 快捷输入解析器 | P0 |
| server | FR-007 | Session Fork | P0 |
| server | FR-011 | Server API 完善 | P1 |
| storage | FR-032 | Snapshot 元数据完善 | P1 |
| storage/permission | FR-016 | Permission 审计记录 | P1 |
| lsp | FR-013 | LSP 功能增强 | P1 |
| auth | FR-015 | 凭证加密存储 | P1 |
| auth | FR-029 | OAuth 登录预留 | P2 |
| tui | FR-017 | TUI Token/Cost 显示 | P1 |
| tui | FR-023 | TUI 布局切换 | P2 |
| tui | FR-024 | TUI 右栏功能完善 | P2 |
| tui | FR-025 | TUI Patch 预览展开 | P2 |
| tui | FR-026 | Web UI | P2 |
| llm | FR-010 | Provider 环境变量约定 | P0 |
| - | FR-027 | IDE 扩展预留 | P2 |
| git | FR-028 | GitHub 集成预留 | P2 |

### 按优先级分组

| 优先级 | FR 编号 |
|--------|---------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010 |
| P1 | FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020, FR-032 |
| P2 | FR-021, FR-022, FR-023, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029, FR-030, FR-031 |

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
4. **FR-006 TUI 快捷输入解析器** - 核心交互
5. **FR-003 Skills 系统** - 上下文增强
6. **FR-002 Plugin System** - 扩展性基础
7. **FR-007 Session Fork** - 会话分叉
8. **FR-008 多层配置合并** - 配置管理
9. **FR-009 .opencode 目录加载** - 模块化配置支持
10. **FR-010 Provider 环境变量约定** - 环境变量绑定

### Phase 2: P1 核心功能

1. **FR-011 Server API** - API 完整性
2. **FR-013 LSP 功能增强** - 开发体验
3. **FR-012 Share 功能** - 协作能力
4. **FR-015 凭证加密存储** - 安全合规
5. **FR-014 插件事件总线** - 事件系统
6. **FR-016 Permission 审计记录** - 权限追踪
7. **FR-017 TUI Token/Cost 显示** - 成本感知
8. **FR-018 TUI Schema 验证** - 配置验证增强
9. **FR-019 scroll_acceleration 结构修复** - 类型修正
10. **FR-020 keybinds 自定义绑定** - 绑定扩展
11. **FR-032 Snapshot 元数据完善** - 数据完整性

### Phase 3: P2 完善性

1. **FR-021 配置系统** - 配置灵活性
2. **FR-022 Session Summarize** - 会话管理
3. **FR-023 TUI 布局切换** - UI 增强
4. **FR-024 TUI 右栏功能完善** - 面板功能
5. **FR-025 TUI Patch 预览展开** - Diff 交互
6. **FR-026 Web UI** - 多端支持
7. **FR-027 IDE 扩展预留** - 生态扩展
8. **FR-028 GitHub 集成预留** - DevOps 集成
9. **FR-029 OAuth 登录预留** - 认证扩展
10. **FR-030 废弃字段清理** - 代码清理
11. **FR-031 theme 路径解析增强** - 主题功能增强

---

## 9. 附录

### A. 数据模型状态

| PRD 数据模型 | 实现状态 | 备注 |
|--------------|----------|------|
| Session | ✅ 完整 | |
| Session.parent_session_id | ❌ 未实现 | FR-007 |
| Session.shared_id | ❌ 未实现 | FR-012 |
| Message | ✅ 完整 | |
| ToolCall | ✅ 完整 | |
| Snapshot | ⚠️ 部分 | FR-032 需完善 |
| PermissionDecision | ⚠️ 部分 | FR-016 需完善 |
| Provider/Credential | ✅ 完整 | |
| Project | ✅ 完整 | |
| Checkpoint | ✅ 完整 | |

### B. API 状态

| PRD API 路径 | 实现状态 | FR 编号 |
|--------------|----------|---------|
| POST /sessions | ⚠️ 部分 | FR-011 |
| GET /sessions | ⚠️ 部分 | FR-011 |
| GET /sessions/{id} | ⚠️ 部分 | FR-011 |
| POST /sessions/{id}/fork | ❌ 未实现 | FR-007, FR-011 |
| POST /sessions/{id}/summarize | ⚠️ 部分 | FR-011, FR-022 |
| POST /sessions/{id}/abort | ✅ 已实现 | |
| POST /sessions/{id}/prompt | ⚠️ 部分 | FR-011 |
| GET /sessions/{id}/messages | ⚠️ 部分 | FR-011 |
| POST /sessions/{id}/shell | ✅ 已实现 | |
| POST /sessions/{id}/command | ❌ 未实现 | FR-004, FR-011 |
| POST /sessions/{id}/permissions/{req_id}/reply | ✅ 已实现 | |
| GET /sessions/{id}/diff | ⚠️ 部分 | FR-011 |
| GET /sessions/{id}/snapshots | ⚠️ 部分 | FR-011, FR-032 |
| POST /sessions/{id}/revert | ⚠️ 部分 | FR-011 |
| GET /providers | ⚠️ 部分 | FR-011 |
| GET /models | ⚠️ 部分 | FR-011 |
| POST /providers/{id}/credentials | ❌ 未实现 | FR-011 |
| POST /providers/{id}/credentials/test | ❌ 未实现 | FR-011 |
| DELETE /providers/{id}/credentials | ❌ 未实现 | FR-011 |

### C. 功能状态

| PRD 功能 | 实现状态 | FR 编号 |
|----------|----------|---------|
| Workspace/Project 机制 | ✅ 已实现 | |
| Session 会话系统 | ✅ 已实现 | |
| Session Fork | ❌ 未实现 | FR-007 |
| Agent 系统 | ✅ 已实现 | |
| 文件工具 | ✅ 已实现 | |
| Shell 工具 | ✅ 已实现 | |
| Git 工具 | ⚠️ 基础 | FR-028 |
| 权限系统 | ✅ 已实现 | |
| @file 引用 | ❌ 未实现 | FR-006 |
| !shell 直接执行 | ❌ 未实现 | FR-006 |
| /command 快捷命令 | ❌ 未实现 | FR-004, FR-006 |
| Skills 延迟加载 | ❌ 未实现 | FR-003 |
| Commands 自定义 | ❌ 未实现 | FR-004 |
| MCP 本地/远程 | ⚠️ 基础 | FR-005 |
| LSP diagnostics | ⚠️ 基础 | FR-013 |
| Server API | ⚠️ 基础 | FR-011 |
| Share 分享 | ❌ 未实现 | FR-012 |
| Plugin WASM 宿主 | ❌ 未实现 | FR-002 |
| 凭证加密存储 | ❌ 未实现 | FR-015 |
| Token/Cost 显示 | ❌ 未实现 | FR-017 |
| TUI 布局切换 | ❌ 未实现 | FR-023 |

---

**文档状态**: 草稿  
**下一步**: 基于本规格文档创建迭代计划