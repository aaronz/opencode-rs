# OpenCode-RS 任务清单 v5

**版本**: 5.0
**日期**: 2026-04-04
**基于**: spec_v5.md 拆分 — 功能需求任务部分
**状态**: 草稿

---

## 1. 文档概述

本文档从 spec_v5.md 拆分而来，包含所有功能需求 (FR-001 ~ FR-062) 的详细规格定义和验收标准。
实施架构、阶段规划和状态追踪请参见 `plan_v5.md`。

### 1.1 与 v4 的关系

v5 保留 v4 的所有需求 (FR-001 ~ FR-052)，并新增：
- **FR-053 ~ FR-056**: PRD 配置项遗漏 (Tools/Provider控制/Formatters/Instructions)
- **FR-057 ~ FR-062**: Constitution v1.5 新架构领域 (EventBus/EffectSystem/Streaming/ControlPlane/CLI/RemoteConfig)

---

## 2. P0 - 阻断性问题

> FR-001 ~ FR-010, FR-033, FR-034 继承自 v4，内容不变。

*(内容同 v4，此处省略重复定义)*

---

## 3. P1 - 核心功能缺失

> FR-011 ~ FR-020, FR-032, FR-035 ~ FR-039, FR-044 ~ FR-048 继承自 v4，内容不变。以下为 v5 新增 P1 需求。

### FR-053: Tools 配置禁用机制

**模块**: core/config, core/tools
**严重程度**: P1
**来源**: v5 (PRD-Configuration §5.2, Constitution C-029)

#### 需求描述

实现 Tools 配置项，支持在配置层面禁用指定工具。

#### 详细规格

1. **配置格式**
   ```jsonc
   {
     "tools": {
       "write": false,
       "bash": false
     }
   }
   ```

2. **禁用优先级**
   - tools 配置 > agent.tools 配置 > 默认值
   - 被禁用的工具在工具列表中显示为 disabled

3. **安全约束**
   - bash 工具默认必须经过权限检查
   - write/edit 工具不得修改 .git/ 目录
   - 工具执行超时必须可配置 (默认 30s)

4. **内置工具清单**
   - read, write, edit, bash, glob, grep, lsp, web_search 等
   - 每个工具必须声明所需权限级别
   - 工具执行失败必须返回错误信息，不得静默失败

#### 验收标准

- [ ] tools 配置项中设为 false 的工具对当前会话全局禁用
- [ ] 禁用优先级正确 (tools > agent.tools > 默认值)
- [ ] 被禁用的工具在列表中显示为 disabled
- [ ] bash 工具默认经过权限检查
- [ ] write/edit 工具不得修改 .git/ 目录
- [ ] 工具执行超时可配置

---

### FR-054: Provider 控制 (disabled/enabled)

**模块**: core/config, core/llm
**严重程度**: P1
**来源**: v5 (PRD-Configuration §5.14, Constitution C-030)

#### 需求描述

实现 Provider 启用/禁用控制，支持黑名单和白名单模式。

#### 详细规格

1. **配置格式**
   ```jsonc
   {
     "disabled_providers": ["openai", "gemini"],
     "enabled_providers": ["anthropic", "openai"]
   }
   ```

2. **优先级规则**
   - disabled_providers > enabled_providers (黑名单优先)
   - 同时设置时，disabled 中的 provider 即使出现在 enabled 中也被禁用

3. **Provider 配置**
   - 每个 provider 可配置: timeout, chunkTimeout, setCacheKey
   - Amazon Bedrock 额外支持: region, profile, endpoint
   - Provider 配置支持 {env:VAR} 变量替换

4. **安全约束**
   - API Key 必须通过环境变量或 {file:path} 提供
   - 禁止在日志中记录 API Key 或完整请求体
   - Provider 连接失败时必须降级到可用 provider

#### 验收标准

- [ ] disabled_providers 黑名单生效
- [ ] enabled_providers 白名单生效
- [ ] 黑名单优先级高于白名单
- [ ] API Key 不落日志
- [ ] Provider 连接失败时正确降级

---

### FR-055: Formatters 自动格式化

**模块**: core/formatter
**严重程度**: P1
**来源**: v5 (PRD-Configuration §5.7, Constitution C-031)

#### 需求描述

实现文件自动格式化功能，支持内置和自定义 formatter。

#### 详细规格

1. **配置格式**
   ```jsonc
   {
     "formatter": {
       "prettier": {
         "disabled": true
       },
       "custom-prettier": {
         "command": ["npx", "prettier", "--write", "$FILE"],
         "environment": {
           "NODE_ENV": "development"
         },
         "extensions": [".js", ".ts", ".jsx", ".tsx"]
       }
     }
   }
   ```

2. **Formatter 注册**
   - 内置 formatter: prettier (可通过 disabled 禁用)
   - 自定义 formatter: 通过 command + extensions 定义
   - Formatter 按文件扩展名匹配

3. **执行流程**
   - 文件写入后自动触发匹配的 formatter
   - Formatter 执行失败不阻断写入，记录 warning
   - 多个 formatter 匹配同一扩展名时，按配置顺序执行

4. **安全约束**
   - Formatter 命令执行超时必须可配置 (默认 10s)
   - Formatter 不得修改非目标文件
   - Formatter 执行目录为项目根目录

#### 验收标准

- [ ] 内置 prettier formatter 可禁用
- [ ] 自定义 formatter 可定义和注册
- [ ] 文件扩展名匹配正确
- [ ] Formatter 失败不阻断写入
- [ ] 多个 formatter 按配置顺序执行
- [ ] 命令超时可配置

---

### FR-056: Instructions 指令文件加载

**模块**: core/config, core/context
**严重程度**: P1
**来源**: v5 (PRD-Configuration §5.13, Constitution C-032)

#### 需求描述

实现 Instructions 指令文件的加载，作为 system prompt 的一部分注入上下文。

#### 详细规格

1. **配置格式**
   ```jsonc
   {
     "instructions": [
       "CONTRIBUTING.md",
       "docs/guidelines.md",
       ".cursor/rules/*.md"
     ]
   }
   ```

2. **路径解析**
   - 以 / 开头: 绝对路径
   - 以 ./ 或 ../ 开头: 相对于项目根目录
   - 包含 * 或 **: glob 模式，匹配多个文件
   - 文件不存在: 记录 warning，不阻断启动

3. **加载顺序**
   - 按配置列表顺序加载，先加载的优先级低
   - 多个指令文件内容按加载顺序拼接
   - 指令文件内容变化必须触发 context 更新

4. **安全约束**
   - 指令文件不得包含可执行代码
   - 指令文件大小限制: 单文件不超过 100KB
   - 指令文件总大小不超过模型上下文窗口的 10%

#### 验收标准

- [ ] 指令文件路径支持绝对路径、相对路径、glob 模式
- [ ] 文件不存在时记录 warning 不阻断
- [ ] 指令文件内容作为 system prompt 注入
- [ ] 多个指令文件按顺序拼接
- [ ] 指令文件内容变化触发 context 更新
- [ ] 单文件大小限制 100KB
- [ ] 总大小不超过上下文窗口 10%

---

## 4. P2 - 完善性问题

> FR-021 ~ FR-031, FR-040 ~ FR-052 继承自 v4，内容不变。以下为 v5 新增 P2 需求。

### FR-057: Event Bus 事件总线

**模块**: core/event
**严重程度**: P2
**来源**: v5 (PRD.md §7.10, openspec archive: event-bus)

#### 需求描述

实现系统事件总线，支持插件和系统组件间的事件通信。

#### 详细规格

1. **事件类型** (PRD §7.10)
   - session.created / session.updated / session.compacted
   - message.updated
   - tool.execute.before / tool.execute.after
   - permission.asked / permission.replied
   - file.edited
   - lsp.updated
   - shell.env
   - tui.toast.show

2. **事件总线架构**
   - 发布/订阅模式
   - 异步事件分发
   - 事件过滤和路由

3. **插件集成**
   - 插件可订阅指定事件类型
   - 插件可发布自定义事件
   - 事件处理失败不影响主流程

#### 验收标准

- [ ] 所有 PRD 定义的事件类型可发布和订阅
- [ ] 事件异步分发不阻塞主流程
- [ ] 插件可订阅和发布事件
- [ ] 事件处理失败有容错机制

---

### FR-058: Effect System 效果系统

**模块**: core/effect
**严重程度**: P2
**来源**: v5 (PRD.md 架构设计, openspec archive: effect-system)

#### 需求描述

实现效果系统 (Effect System)，用于管理副作用和状态变更的声明式描述。

#### 详细规格

1. **Effect 类型**
   - 文件修改 effect
   - 状态变更 effect
   - 通知 effect
   - 外部调用 effect

2. **Effect 执行**
   - 声明式定义 effect
   - 执行顺序可编排
   - 支持回滚/撤销

3. **与 Session 集成**
   - Effect 记录到 session timeline
   - Effect 失败触发错误处理流程

#### 验收标准

- [ ] Effect 可声明式定义
- [ ] Effect 执行顺序可编排
- [ ] Effect 支持回滚
- [ ] Effect 记录到 session timeline

---

### FR-059: Streaming 消息架构 (WebSocket/SSE)

**模块**: server/streaming
**严重程度**: P2
**来源**: v5 (PRD.md §7.16, Constitution 审计发现)

#### 需求描述

完善 WebSocket 和 SSE 流式消息架构，确保消息格式标准化。

#### 详细规格

1. **协议支持**
   - SSE (Server-Sent Events)
   - WebSocket
   - 两种协议消息格式统一

2. **消息格式**
   - 统一的 JSON-RPC 或自定义消息格式
   - 支持心跳/保活机制
   - 错误消息标准化

3. **错误处理**
   - 连接断开自动重连
   - 消息丢失检测
   - 超时处理

#### 验收标准

- [ ] SSE 和 WebSocket 均可工作
- [ ] 消息格式统一
- [ ] 心跳/保活机制正常
- [ ] 连接断开可自动重连
- [ ] 错误消息标准化

---

### FR-060: Control Plane / ACP 协议

**模块**: control-plane
**严重程度**: P2
**来源**: v5 (PRD.md 架构设计, Constitution 审计发现)

#### 需求描述

实现 Control Plane 和 ACP (Agent Communication Protocol) 协议，支持 session 编排。

#### 详细规格

1. **ACP 协议**
   - Agent 间通信协议
   - Session 编排控制
   - 任务分发和结果聚合

2. **Control Plane**
   - Session 生命周期管理
   - Agent 调度和资源分配
   - 多 session 并发控制

#### 验收标准

- [ ] ACP 协议可支持 Agent 间通信
- [ ] Session 编排正常工作
- [ ] 多 session 并发控制正确
- [ ] 资源分配合理

---

### FR-061: CLI 命令架构完善

**模块**: cli
**严重程度**: P2
**来源**: v5 (PRD.md §7.15, Constitution 审计发现)

#### 需求描述

完善 CLI 命令注册、参数解析、帮助文本等架构。

#### 详细规格

1. **命令注册**
   - 基于 clap 的命令派生
   - 子命令嵌套支持
   - 动态命令发现

2. **参数解析**
   - 位置参数和选项参数
   - 参数验证
   - 默认值和帮助文本

3. **帮助系统**
   - 自动生成帮助文本
   - 命令使用说明
   - 示例输出

#### 验收标准

- [ ] CLI 命令注册机制完善
- [ ] 参数解析正确
- [ ] 帮助文本自动生成
- [ ] 子命令嵌套支持

---

### FR-062: Remote Config 安全验证

**模块**: config/remote
**严重程度**: P2
**来源**: v5 (PRD-Configuration §8, Constitution 审计发现)

#### 需求描述

实现远程配置的安全验证机制，确保远程配置的认证和完整性。

#### 详细规格

1. **认证机制**
   - 远程配置拉取时的身份验证
   - Token 或 API Key 认证

2. **完整性验证**
   - 配置内容签名验证
   - 配置哈希校验
   - 防篡改检测

3. **缓存机制**
   - 远程配置本地缓存
   - 缓存过期策略
   - 离线降级策略

#### 验收标准

- [ ] 远程配置拉取有认证机制
- [ ] 配置内容完整性可验证
- [ ] 本地缓存机制正常
- [ ] 离线降级策略合理

---

## 5. 功能需求清单汇总

### 按模块分组

| 模块 | FR 编号 | 需求名称 | 优先级 | 来源 |
|------|---------|----------|--------|------|
| core | FR-001 | Context Engine | P0 | v2 |
| core | FR-003 | Skills 系统 | P0 | v2 |
| core | FR-004 | Commands 系统 | P0 | v2 |
| core | FR-012 | Share 功能 | P1 | v2 |
| core | FR-014 | 插件事件总线 | P1 | v2 |
| core | FR-022 | Session Summarize | P2 | v2 |
| core/tools | FR-044 | session_load/session_save | P1 | v4 |
| core/tools | FR-053 | Tools 配置禁用机制 | P1 | v5 |
| core/skills | FR-045 | 剩余内建 Skills 补全 | P1 | v4 |
| core/commands | FR-046 | 剩余 Commands 补全 | P1 | v4 |
| core/session | FR-051 | Compaction 会话压缩 | P2 | v4 |
| core/watcher | FR-052 | 文件 Watcher 配置 | P2 | v4 |
| core/event | FR-057 | Event Bus 事件总线 | P2 | v5 |
| core/effect | FR-058 | Effect System 效果系统 | P2 | v5 |
| core/context | FR-056 | Instructions 指令文件加载 | P1 | v5 |
| config | FR-008 | 多层配置合并 | P0 | v2 |
| config | FR-009 | .opencode 目录加载 | P0 | v2 |
| config | FR-010 | Provider 环境变量约定 | P0 | v2 |
| config | FR-021 | 配置系统完善 | P2 | v2 |
| config | FR-030 | 废弃字段清理 | P2 | v2 |
| config | FR-033 | OPENCODE_TUI_CONFIG 环境变量 | P0 | v3 |
| config | FR-034 | TUI 配置分离为独立文件 | P0 | v3 |
| config | FR-035 | modes/ 目录扫描 | P1 | v3 |
| config | FR-036 | 配置路径命名统一 | P1 | v3 |
| config | FR-037 | {file:path} ~ 路径展开 | P1 | v3 |
| config | FR-038 | {file:path} 相对路径支持 | P1 | v3 |
| config | FR-039 | .opencode/ 目录扫描集成 | P1 | v3 |
| config | FR-040 | 变量替换覆盖完整性 | P2 | v3 |
| config | FR-041 | theme/keybinds 迁移到 TUI | P2 | v3 |
| config | FR-042 | AgentMapConfig 动态 HashMap | P2 | v3 |
| config | FR-043 | JSON Schema 远程验证 | P2 | v3 |
| config | FR-054 | Provider 控制 (disabled/enabled) | P1 | v5 |
| config | FR-055 | Formatters 自动格式化 | P1 | v5 |
| config | FR-056 | Instructions 指令文件加载 | P1 | v5 |
| config/tui | FR-019 | scroll_acceleration 结构修复 | P1 | v2 |
| config/tui | FR-020 | keybinds 自定义绑定 | P1 | v2 |
| config/tui | FR-031 | theme 路径解析增强 | P2 | v2 |
| config/remote | FR-062 | Remote Config 安全验证 | P2 | v5 |
| schema | FR-018 | TUI Schema 验证 | P1 | v2 |
| plugin | FR-002 | Plugin System | P0 | v2 |
| mcp | FR-005 | MCP 工具接入 | P0 | v2 |
| server | FR-006 | TUI 快捷输入解析器 | P0 | v2 |
| server | FR-007 | Session Fork | P0 | v2 |
| server | FR-011 | Server API 完善 | P1 | v2 |
| server | FR-050 | Server mDNS 服务发现 | P2 | v4 |
| server/streaming | FR-059 | Streaming 消息架构 | P2 | v5 |
| storage | FR-032 | Snapshot 元数据完善 | P1 | v2 |
| storage/permission | FR-016 | Permission 审计记录 | P1 | v2 |
| lsp | FR-013 | LSP 功能增强 | P1 | v2 |
| auth | FR-015 | 凭证加密存储 | P1 | v2 |
| auth | FR-029 | OAuth 登录预留 | P2 | v2 |
| auth | FR-047 | OAuth 登录支持 | P1 | v4 |
| tui | FR-017 | TUI Token/Cost 显示 | P1 | v2 |
| tui | FR-023 | TUI 布局切换 | P2 | v2 |
| tui | FR-024 | TUI 右栏功能完善 | P2 | v2 |
| tui | FR-025 | TUI Patch 预览展开 | P2 | v2 |
| tui | FR-026 | Web UI | P2 | v2 |
| git | FR-028 | GitHub 集成预留 | P2 | v2 |
| git | FR-048 | GitHub 集成 | P1 | v4 |
| llm | FR-049 | HuggingFace/AI21 Provider | P2 | v4 |
| control-plane | FR-060 | Control Plane / ACP 协议 | P2 | v5 |
| cli | FR-061 | CLI 命令架构完善 | P2 | v5 |
| formatter | FR-055 | Formatters 自动格式化 | P1 | v5 |
| - | FR-027 | IDE 扩展预留 | P2 | v2 |

### 按优先级分组

| 优先级 | FR 编号 |
|--------|---------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010, FR-033, FR-034 |
| P1 | FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020, FR-032, FR-035, FR-036, FR-037, FR-038, FR-039, FR-044, FR-045, FR-046, FR-047, FR-048, FR-053, FR-054, FR-055, FR-056 |
| P2 | FR-021, FR-022, FR-023, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029, FR-030, FR-031, FR-040, FR-041, FR-042, FR-043, FR-049, FR-050, FR-051, FR-052, FR-057, FR-058, FR-059, FR-060, FR-061, FR-062 |

---

**文档状态**: 草稿
**下一步**: 基于本任务清单 + plan_v5.md 创建迭代 5 实施计划
