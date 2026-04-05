# OpenCode-RS 规格文档 v5

**版本**: 5.0
**日期**: 2026-04-04
**基于**: spec_v4.md + Constitution v1.5 (C-023~C-032) + PRD §5.2/§5.7/§5.13/§5.14 + PRD.md 架构领域
**状态**: 草稿

---

## 1. 文档概述

### 1.1 背景

本规格文档基于以下文档综合生成：
- **spec_v4.md**: 上一版规格文档 (FR-001 ~ FR-052)
- **PRD-OpenCode-Configuration.md**: 配置系统产品需求文档 (v1.0)
- **docs/PRD.md**: 产品需求文档 v1.1 (完整系统级 PRD)
- **docs/PRD-providers.md**: Provider 详细规格
- **docs/PRD-tui.md**: TUI 产品需求
- **outputs/iteration-5/constitution_updates.md**: Constitution v1.5 更新 (C-023 ~ C-032)
- **outputs/iteration-3/gap-analysis.md**: 配置系统专项差距分析

### 1.2 目标

- 在 v4 基础上新增 Constitution v1.5 覆盖的 10 个架构领域需求
- 补充 PRD §5 中 v4 未覆盖的配置项 (Tools/Provider控制/Formatters/Instructions)
- 为每个新需求分配唯一的功能需求编号 (FR-XXX)
- 确保新功能有对应的规格定义和验收标准

### 1.3 参考文档

| 文档 | 路径 | 说明 |
|------|------|------|
| PRD-配置系统 | `PRD-OpenCode-Configuration.md` | 配置系统产品需求 |
| PRD-主文档 | `docs/PRD.md` | 产品需求文档 v1.1 |
| PRD-Providers | `docs/PRD-providers.md` | Provider 详细规格 |
| PRD-TUI | `docs/PRD-tui.md` | TUI 产品需求 |
| Constitution v1.5 | `outputs/iteration-5/constitution_updates.md` | 设计约束条款 (C-001 ~ C-032) |
| spec_v4 | `outputs/iteration-4/spec_v4.md` | 上一版规格文档 |

### 1.4 与 v4 的关系

v5 保留 v4 的所有需求 (FR-001 ~ FR-052)，并新增：
- **FR-053 ~ FR-056**: PRD 配置项遗漏 (Tools/Provider控制/Formatters/Instructions)
- **FR-057 ~ FR-062**: Constitution v1.5 新架构领域 (EventBus/EffectSystem/Streaming/ControlPlane/CLI/RemoteConfig)

---

## 2. 需求总览

| 优先级 | 数量 | 说明 |
|--------|------|------|
| P0 | 12 | 阻断性问题 (v4: 12, v5新增: 0) |
| P1 | 24 | 核心功能缺失 (v4: 20, v5新增: 4) |
| P2 | 26 | 完善性问题 (v4: 20, v5新增: 6) |

**总计**: 62 项功能需求 (v4: 52 项)

---

## 3. P0 - 阻断性问题

> FR-001 ~ FR-010, FR-033, FR-034 继承自 v4，内容不变。

*(内容同 v4，此处省略重复定义)*

---

## 4. P1 - 核心功能缺失

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

## 5. P2 - 完善性问题

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

## 6. 技术债务清单

| 债务项 | 位置 | 描述 | 关联 FR |
|--------|------|------|---------|
| **TOML vs JSON 格式分裂** | `config.rs:1012-1031` | `config_path()` 默认返回 `.toml`，但 PRD 要求 JSON/JSONC | FR-036 |
| **硬编码路径** | `config.rs:1031` | `"~/.config/opencode-rs/config.toml"` 硬编码 | FR-036 |
| **变量替换实现粗糙** | `config.rs:972-1009` | 字符串替换对嵌套/复杂情况可能出错 | FR-040 |
| **merge_configs 通过 JSON 中转** | `merge.rs:22-29` | 序列化→deep_merge→反序列化，丢失类型信息 | FR-021 |
| **fetch_remote_config 同步包装异步** | `config.rs:1107-1109` | 同步函数中创建 tokio runtime | FR-062 |
| **TimeoutConfig 枚举命名** | `config.rs:469-474` | `Disabled(bool)` 语义不清 | - |
| **PermissionConfig 大量重复字段** | `config.rs:628-697` | 应考虑宏生成或统一结构 | - |
| **Schema 验证空壳** | `schema.rs:5-40` | 只检查 2 个字段 | FR-043 |
| **DirectoryScanner 未使用 glob** | `directory_scanner.rs` | 手动 read_dir，不支持 glob 模式 | FR-035, FR-056 |
| **测试覆盖不足** | `core/tests/` | 仅 2 个测试文件，缺少集成测试 | - |
| **Event Bus 未实现** | - | PRD §7.10 定义的事件总线缺失 | FR-057 |
| **Effect System 未实现** | - | 副作用管理系统缺失 | FR-058 |
| **Streaming 消息格式不统一** | `server/src/routes/ws.rs`, `sse.rs` | WebSocket 和 SSE 消息格式需标准化 | FR-059 |
| **Control Plane / ACP 缺失** | - | Agent 通信协议和 session 编排缺失 | FR-060 |

---

## 7. 验收标准对照 (PRD §10)

| 验收项 | PRD § | 状态 | 关联 FR | 备注 |
|--------|-------|------|---------|------|
| JSON/JSONC 格式支持 | 10.1 | ✅ | - | `jsonc.rs` 完整实现 |
| 配置合并逻辑正确 | 10.1 | ✅ | FR-021 | `merge.rs` deep_merge 实现 |
| 6 个配置位置按优先级加载 | 10.1 | ⚠️ | FR-033, FR-039 | 缺少 OPENCODE_TUI_CONFIG，.opencode 目录扫描未集成 |
| `{env:VARIABLE_NAME}` 正确替换 | 10.2 | ✅ | - | 实现正确 |
| `{file:path}` 正确读取文件 | 10.2 | ⚠️ | FR-037, FR-038 | 不支持 `~` 和相对路径 |
| 未设置变量替换为空字符串 | 10.2 | ❌ | FR-040 | 当前保留原字符串 |
| TUI 配置与 runtime 分离 | 10.3 | ❌ | FR-034 | 未实现独立 tui.json |
| `OPENCODE_TUI_CONFIG` 自定义路径 | 10.3 | ❌ | FR-033 | 完全缺失 |
| Provider timeout/chunkTimeout/setCacheKey | 10.4 | ✅ | - | `ProviderOptions` 完整 |
| Amazon Bedrock 配置 | 10.4 | ✅ | - | awsRegion/awsProfile/awsEndpoint |
| disabled_providers 优先级 | 10.4 | ⚠️ | FR-054 | 逻辑存在但需完善 |
| 自定义 agent 配置 | 10.5 | ✅ | FR-042 | AgentConfig 完整，但 AgentMapConfig 需改为动态 |
| default_agent 设置 | 10.5 | ✅ | - | 字段存在且被 env 覆盖 |
| 命令模板变量替换 | 10.5 | ⚠️ | FR-004 | 命令模板变量替换未明确实现 |
| permission 配置 | 10.6 | ✅ | - | `PermissionConfig` 完整 |
| API Key 文件引用 | 10.6 | ⚠️ | FR-037, FR-038 | 依赖 `{file:path}`，但该功能不完整 |
| **Tools 配置禁用** | **§5.2** | **❌** | **FR-053** | **完全缺失** |
| **Formatters 配置** | **§5.7** | **❌** | **FR-055** | **完全缺失** |
| **Instructions 配置** | **§5.13** | **❌** | **FR-056** | **完全缺失** |
| **Provider 控制** | **§5.14** | **⚠️** | **FR-054** | **基础存在，需完善** |

---

## 8. 功能需求清单汇总

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

## 9. 实施建议

### Phase 1: P0 阻断性问题 (当前优先级)

*(同 v4，内容不变)*

1. **FR-033 OPENCODE_TUI_CONFIG 环境变量** - 配置系统基础
2. **FR-034 TUI 配置分离** - 核心架构要求
3. **FR-001 Context Engine** - 核心依赖
4. **FR-005 MCP 工具接入** - 工具系统扩展
5. **FR-004 Commands 系统** - TUI 输入增强
6. **FR-006 TUI 快捷输入解析器** - 核心交互
7. **FR-003 Skills 系统** - 上下文增强
8. **FR-002 Plugin System** - 扩展性基础
9. **FR-007 Session Fork** - 会话分叉
10. **FR-008 多层配置合并** - 配置管理
11. **FR-009 .opencode 目录加载** - 模块化配置支持
12. **FR-010 Provider 环境变量约定** - 环境变量绑定

### Phase 2: P1 核心功能

1. **FR-039 .opencode/ 目录扫描集成** - 配置加载完整性
2. **FR-037 {file:path} ~ 路径展开** - 变量替换完整性
3. **FR-038 {file:path} 相对路径支持** - 变量替换完整性
4. **FR-035 modes/ 目录扫描** - 目录结构完整性
5. **FR-036 配置路径命名统一** - 生态兼容性
6. **FR-044 session_load/session_save** - 会话持久化
7. **FR-045 剩余内建 Skills 补全** - 能力扩展
8. **FR-046 剩余 Commands 补全** - 命令完整性
9. **FR-011 Server API** - API 完整性
10. **FR-013 LSP 功能增强** - 开发体验
11. **FR-012 Share 功能** - 协作能力
12. **FR-015 凭证加密存储** - 安全合规
13. **FR-014 插件事件总线** - 事件系统
14. **FR-016 Permission 审计记录** - 权限追踪
15. **FR-017 TUI Token/Cost 显示** - 成本感知
16. **FR-018 TUI Schema 验证** - 配置验证增强
17. **FR-019 scroll_acceleration 结构修复** - 类型修正
18. **FR-020 keybinds 自定义绑定** - 绑定扩展
19. **FR-032 Snapshot 元数据完善** - 数据完整性
20. **FR-047 OAuth 登录支持** - 用户认证 (v1.5+)
21. **FR-048 GitHub 集成** - DevOps 集成 (v1.5+)
22. **FR-053 Tools 配置禁用机制** - 工具控制 (v5 新增)
23. **FR-054 Provider 控制** - Provider 管理 (v5 新增)
24. **FR-055 Formatters 自动格式化** - 代码格式化 (v5 新增)
25. **FR-056 Instructions 指令文件加载** - 上下文注入 (v5 新增)

### Phase 3: P2 完善性

1. **FR-040 变量替换覆盖完整性** - 配置系统完善
2. **FR-041 theme/keybinds 迁移** - 废弃声明一致性
3. **FR-042 AgentMapConfig 动态 HashMap** - 灵活性
4. **FR-043 JSON Schema 远程验证** - 配置校验
5. **FR-049 HuggingFace/AI21 Provider** - LLM 覆盖完整性
6. **FR-050 Server mDNS 服务发现** - 局域网发现
7. **FR-051 Compaction 会话压缩** - 上下文管理
8. **FR-052 文件 Watcher 配置** - 文件监视
9. **FR-057 Event Bus 事件总线** - 事件通信 (v5 新增)
10. **FR-058 Effect System 效果系统** - 副作用管理 (v5 新增)
11. **FR-059 Streaming 消息架构** - 流式消息标准化 (v5 新增)
12. **FR-060 Control Plane / ACP 协议** - Agent 通信 (v5 新增)
13. **FR-061 CLI 命令架构完善** - CLI 架构 (v5 新增)
14. **FR-062 Remote Config 安全验证** - 远程配置安全 (v5 新增)
15. **FR-021 配置系统** - 配置灵活性
16. **FR-022 Session Summarize** - 会话管理
17. **FR-023 TUI 布局切换** - UI 增强
18. **FR-024 TUI 右栏功能完善** - 面板功能
19. **FR-025 TUI Patch 预览展开** - Diff 交互
20. **FR-026 Web UI** - 多端支持
21. **FR-027 IDE 扩展预留** - 生态扩展
22. **FR-028 GitHub 集成预留** - DevOps 集成
23. **FR-029 OAuth 登录预留** - 认证扩展
24. **FR-030 废弃字段清理** - 代码清理
25. **FR-031 theme 路径解析增强** - 主题功能增强

---

## 10. 附录

### A. 数据模型状态

*(同 v4，内容不变)*

### B. API 状态

*(同 v4，内容不变)*

### C. 配置系统状态

| 配置项 | 实现状态 | 关联 FR | 备注 |
|--------|----------|---------|------|
| JSON/JSONC 格式 | ✅ 完整 | - | jsonc.rs |
| 配置合并 | ✅ 完整 | FR-021 | merge.rs |
| Remote Config | ⚠️ 部分 | FR-008, FR-062 | fetch_remote_config 同步包装异步，安全验证缺失 |
| Global Config | ⚠️ 部分 | FR-036 | 路径使用 opencode-rs |
| OPENCODE_CONFIG | ✅ 完整 | - | 环境变量支持 |
| OPENCODE_TUI_CONFIG | ❌ 未实现 | FR-033 | 完全缺失 |
| OPENCODE_CONFIG_CONTENT | ✅ 完整 | - | 内联配置 |
| Project Config | ✅ 完整 | - | .opencode/config.json |
| .opencode/ 目录扫描 | ⚠️ 部分 | FR-035, FR-039 | 缺少 modes/，未集成到 load_multi |
| {env:VAR} 变量替换 | ✅ 完整 | - | |
| {file:path} 变量替换 | ⚠️ 部分 | FR-037, FR-038 | 不支持 ~ 和相对路径 |
| TUI 配置分离 | ❌ 未实现 | FR-034 | 内嵌在主配置中 |
| Schema 验证 | ⚠️ 空壳 | FR-043 | 只检查 2 个字段 |
| Agent 配置 | ✅ 完整 | FR-042 | AgentMapConfig 需改为动态 |
| Command 配置 | ✅ 完整 | FR-004 | |
| Permission 配置 | ✅ 完整 | - | |
| Provider 配置 | ✅ 完整 | - | |
| MCP 配置 | ⚠️ 部分 | FR-005 | |
| theme 配置 | ⚠️ 部分 | FR-031, FR-041 | 未迁移到 tui.json |
| keybinds 配置 | ⚠️ 部分 | FR-020, FR-041 | 未迁移到 tui.json |
| Server 配置 (mDNS/CORS) | ⚠️ 部分 | FR-050 | 基础实现存在，mDNS 待完善 |
| Compaction 配置 | ⚠️ 部分 | FR-051 | 基础结构存在，自动压缩待实现 |
| Watcher 配置 | ⚠️ 部分 | FR-052 | 基础监视存在，ignore 配置待完善 |
| **Tools 配置** | **❌ 未实现** | **FR-053** | **完全缺失** |
| **Formatters 配置** | **❌ 未实现** | **FR-055** | **完全缺失** |
| **Instructions 配置** | **❌ 未实现** | **FR-056** | **完全缺失** |
| **disabled_providers** | **⚠️ 部分** | **FR-054** | **基础逻辑存在，需完善** |

### D. Agent/Tool/Provider 实现状态

*(同 v4，内容不变)*

### E. Constitution 条款映射

| Constitution 条款 | 覆盖领域 | 关联 FR |
|-------------------|----------|---------|
| C-001 | 已废止 (被 C-016 替代) | - |
| C-002 ~ C-010 | 基础架构 | FR-001, FR-002 |
| C-011 | Config Schema 设计 | FR-008, FR-021 |
| C-012 | 变量替换规范 | FR-037, FR-038, FR-040 |
| C-013 | 目录扫描规范 (含 modes/) | FR-009, FR-035, FR-039 |
| C-014 | TUI Input Parser | FR-006 |
| C-015 | Session Fork | FR-007 |
| C-016 | Context Token Budget | FR-001 |
| C-017 | TUI 配置分离 | FR-033, FR-034 |
| C-018 | 路径命名统一 | FR-036 |
| C-019 | 文件引用变量 | FR-037, FR-038 |
| C-020 | Server 配置规范 | FR-050 |
| C-021 | Compaction 配置规范 | FR-051 |
| C-022 | Watcher 配置规范 | FR-052 |
| **C-023** | **Agent 系统规范** | **FR-045, FR-046** |
| **C-024** | **Permission 系统规范** | **FR-016** |
| **C-025** | **Plugin 系统规范** | **FR-002, FR-014** |
| **C-026** | **Auth 系统规范** | **FR-015, FR-047** |
| **C-027** | **Share 系统规范** | **FR-012** |
| **C-028** | **Storage 系统规范** | **FR-032** |
| **C-029** | **Tools 配置规范** | **FR-053** |
| **C-030** | **Provider 控制规范** | **FR-054** |
| **C-031** | **Formatters 规范** | **FR-055** |
| **C-032** | **Instructions 规范** | **FR-056** |
| *(未映射)* | **Event Bus** | **FR-057** |
| *(未映射)* | **Effect System** | **FR-058** |
| *(未映射)* | **Streaming 架构** | **FR-059** |
| *(未映射)* | **Control Plane/ACP** | **FR-060** |
| *(未映射)* | **CLI 架构** | **FR-061** |
| *(未映射)* | **Remote Config 安全** | **FR-062** |

### F. v4 → v5 变更摘要

| 变更项 | 说明 |
|--------|------|
| 新增 FR-053 | Tools 配置禁用机制 (P1) |
| 新增 FR-054 | Provider 控制 disabled/enabled (P1) |
| 新增 FR-055 | Formatters 自动格式化 (P1) |
| 新增 FR-056 | Instructions 指令文件加载 (P1) |
| 新增 FR-057 | Event Bus 事件总线 (P2) |
| 新增 FR-058 | Effect System 效果系统 (P2) |
| 新增 FR-059 | Streaming 消息架构 (P2) |
| 新增 FR-060 | Control Plane / ACP 协议 (P2) |
| 新增 FR-061 | CLI 命令架构完善 (P2) |
| 新增 FR-062 | Remote Config 安全验证 (P2) |
| 更新 §2 | 需求总览 (P1: 20→24, P2: 20→26) |
| 更新 §6 | 技术债务清单 (新增 4 项) |
| 更新 §7 | 验收标准对照 (新增 4 项 PRD 配置项) |
| 更新 §8 | 功能需求清单汇总 (新增 10 项) |
| 更新 §9 | 实施建议 (Phase 2/3 新增项) |
| 更新 §10.C | 配置系统状态 (新增 Tools/Formatters/Instructions) |
| 更新 §10.E | Constitution 条款映射 (C-023~C-032 + 未映射项) |
| 更新 §10.F | v4 → v5 变更摘要 |

---

**文档状态**: 草稿
**下一步**: 基于本规格文档创建迭代 5 实施计划
