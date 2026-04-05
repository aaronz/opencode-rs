# Constitution 审计报告 (v1.4 → v1.5)

**日期**: 2026-04-04  
**审计范围**: Constitution v1.4 (C-001 ~ C-022) vs 当前完整实现状态  
**审计依据**: PRD-OpenCode-Configuration.md, rust-opencode-port/ 代码库, openspec 变更档案

---

## 一、审计结论

### Constitution v1.4 状态: **严重滞后，需大规模更新**

| 指标 | 值 |
|------|-----|
| Constitution 条款总数 | 22 (C-001 ~ C-022, C-001 已废止) |
| 已覆盖 PRD 配置领域 | 6/6 (100% — C-011~C-022 覆盖 Config/TUI/Server/Compaction/Watcher/MCP) |
| **未覆盖的新架构领域** | **7 个** (Agent/Permission/Plugin/Auth/Share/Storage/Git) |
| **未覆盖的 PRD 配置项** | **5 个** (Tools/Provider控制/Formatters/Instructions/Provider配置) |
| 建议新增条款 | C-023 ~ C-032 (10 条) |

### 关键发现

1. **Constitution 仍不存在于项目根目录** — 仅存在于 `outputs/iteration-*/constitution_updates.md`，未被 AGENTS.md 或任何构建/测试流程引用
2. **Config 领域已完全合规** — 所有 C-011~C-022 条款在 rust-opencode-port 中均被正确实现，无违规
3. **7 个新架构领域完全未受约束** — Agent 系统、Permission 系统、Plugin 系统、Auth 系统、Share 系统、Storage 系统、Git 扩展均无 Constitution 条款
4. **5 个 PRD 配置项无对应条款** — Tools 禁用、Provider 控制、Formatters、Instructions、Provider 专属配置

---

## 二、现有条款覆盖度验证 (v1.4)

### 2.1 C-011 ~ C-013: Config 系统

| 条款 | 实现文件 | 覆盖状态 |
|------|----------|----------|
| C-011 (配置优先级加载) | `crates/core/src/config.rs` — `load_multi()` | ✅ 完整覆盖 |
| C-012 (环境变量替换) | `crates/core/src/config.rs` — `{env:VAR}` 替换 | ✅ 完整覆盖 |
| C-013 (目录结构) | `crates/core/src/config/directory_scanner.rs` — 7 子目录扫描 | ✅ 完整覆盖 |

### 2.2 C-014 ~ C-016: TUI/Session/Context

| 条款 | 实现文件 | 覆盖状态 |
|------|----------|----------|
| C-014 (TUI Input Parser) | `crates/tui/src/input_parser.rs` — `@file/!shell//command` | ✅ 完整覆盖 |
| C-015 (Session Fork) | `crates/core/src/session.rs` + `crates/server/src/routes/session.rs` | ✅ 完整覆盖 |
| C-016 (Context Token Budget) | `crates/core/src/compaction.rs` — token budgeting + 压缩 | ✅ 完整覆盖 |

### 2.3 C-017 ~ C-019: TUI 配置分离/路径命名/文件引用

| 条款 | 实现文件 | 覆盖状态 |
|------|----------|----------|
| C-017 (TUI 配置分离) | `crates/core/src/config.rs` — tui.json + `OPENCODE_TUI_CONFIG` | ✅ 完整覆盖 |
| C-018 (路径命名) | `crates/core/src/config.rs` — `directories` crate, "opencode" 命名 | ✅ 完整覆盖 |
| C-019 (文件引用变量) | `crates/core/src/config.rs` — `{env:VAR}` + `{file:path}` | ✅ 完整覆盖 |

### 2.4 C-020 ~ C-022: Server/Compaction/Watcher

| 条款 | 实现文件 | 覆盖状态 |
|------|----------|----------|
| C-020 (Server 配置) | `crates/core/src/server.rs` + `crates/server/src/routes/` | ✅ 完整覆盖 |
| C-021 (Compaction 配置) | `crates/core/src/compaction.rs` — auto/prune/reserved | ✅ 完整覆盖 |
| C-022 (Watcher 配置) | `crates/core/src/watcher.rs` — ignore patterns + 默认排除 | ✅ 完整覆盖 |

### 2.5 合规总结

**C-011 ~ C-022 全部合规**。实现严格遵循 Constitution 约束，无违规发现。

---

## 三、未覆盖领域分析

### 3.1 架构级未覆盖 (7 个领域)

以下领域已在 rust-opencode-port 中实现，但无任何 Constitution 条款约束其设计决策：

| # | 领域 | 实现文件 | 需要约束的原因 |
|---|------|----------|----------------|
| 1 | **Agent 系统** | `crates/agent/src/*.rs` (8 个 agent) | Agent 架构、tool 权限隔离、model 选择策略 |
| 2 | **Permission 系统** | `crates/permission/src/*.rs` (evaluator/audit) | 安全边界、权限级别、审计日志 |
| 3 | **Plugin 系统** | `crates/plugin/src/*.rs` (discovery/loader/registry) | 插件生命周期、沙箱隔离、加载顺序 |
| 4 | **Auth 系统** | `crates/auth/src/manager.rs` + `crates/core/src/account.rs` | 认证流程、token 管理、多 provider 认证 |
| 5 | **Share 系统** | `crates/core/src/share.rs` | 分享模式 (manual/auto/disabled)、数据隐私 |
| 6 | **Storage 系统** | `crates/storage/src/*.rs` (migration/service) | 数据库 schema 管理、迁移策略、数据持久化 |
| 7 | **Git 扩展** | `crates/git/src/github.rs` + tools | GitHub 集成、PR 操作、认证 |

### 3.2 PRD 配置项未覆盖 (5 个)

以下 PRD §5 配置项在 Constitution 中无对应条款：

| # | PRD 配置项 | PRD 章节 | 需要约束的原因 |
|---|-----------|----------|----------------|
| 1 | **Tools 配置** | PRD §5.2 | 工具禁用语义、默认启用策略 |
| 2 | **Provider 控制** | PRD §5.14 | `disabled_providers` vs `enabled_providers` 优先级 |
| 3 | **Formatters 配置** | PRD §5.7 | 自定义 formatter 命令格式、扩展名匹配 |
| 4 | **Instructions 配置** | PRD §5.13 | 指令文件加载顺序、glob 模式、路径解析 |
| 5 | **Provider 专属配置** | PRD §5.3 | timeout/chunkTimeout/Bedrock 专属选项 |

### 3.3 基础设施未覆盖 (3 个)

| # | 领域 | 实现文件 | 需要约束的原因 |
|---|------|----------|----------------|
| 1 | **Streaming 架构** | `crates/server/src/routes/ws.rs`, `sse.rs` | WebSocket vs SSE 使用场景、消息格式、错误处理 |
| 2 | **MCP 协议** | `crates/mcp/src/*.rs` (server/client) | MCP server 注册、tool 桥接、transport 选择 |
| 3 | **Control Plane** | `crates/control-plane/src/acp_stream.rs` | ACP 协议流、session 编排 |

---

## 四、新增 Constitution 条款 (v1.5)

### 4.1 Agent 系统规范 (C-023)

**条款 C-023: Agent 系统架构规范**

```
1. Agent 架构:
   a) 所有 Agent 必须实现 Agent trait
   b) Agent 通过 system prompt 定义行为，而非硬编码逻辑
   c) Agent 可配置 tool 子集 (tools 字段为 false 表示禁用)
   d) Agent 可独立指定 model (覆盖全局 model 设置)

2. 内置 Agent 清单:
   a) build: 代码构建和实现
   b) plan: 规划和任务分解
   c) debug: 调试和错误分析
   d) review: 代码审查 (支持 general/security/performance focus)
   e) refactor: 代码重构 (支持 preview mode)
   f) explore: 代码探索和理解
   g) general: 通用任务处理

3. Agent 安全约束:
   a) Agent 的 tool 禁用必须在创建时锁定，运行时不可修改
   b) Agent 的 model 选择必须在 provider 白名单内
   c) 自定义 agent 不得覆盖内置 agent 的名称

4. Agent 加载优先级:
   a) 内置 agent (代码定义)
   b) 配置文件中的 agent 字段 (覆盖内置)
   c) .opencode/agents/ 目录扫描 (追加)
```

### 4.2 Permission 系统规范 (C-024)

**条款 C-024: 权限系统规范**

```
1. 权限级别:
   a) "allow" — 自动执行，无需用户确认
   b) "ask"   — 执行前需用户确认
   c) "deny"  — 禁止执行

2. 权限作用域:
   a) 工具级权限: 针对特定 tool (edit, bash, write 等)
   b) 路径级权限: 针对特定文件系统路径 (未来扩展)
   c) 权限配置来源: permission 配置字段

3. 权限评估:
   a) PermissionEvaluator 负责评估访问请求
   b) 评估结果必须记录到 AuditLog
   c) deny 优先级最高，不可被 allow 覆盖

4. AuditLog 规范:
   a) 记录每次权限评估: timestamp, tool, action, result, session_id
   b) 审计日志持久化到 storage
   c) 用户可通过 API 查询审计历史

5. 安全约束:
   a) 权限配置不得通过远程配置 (Remote Config) 覆盖
   b) 生产环境禁止所有工具设为 "allow"
   c) bash 工具默认必须为 "ask" 或 "deny"
```

### 4.3 Plugin 系统规范 (C-025)

**条款 C-025: 插件系统规范**

```
1. 插件发现:
   a) PluginDiscovery 扫描 .opencode/plugins/ 和全局插件目录
   b) 插件标识格式: 裸名 (opencode-xxx) 或 作用域名 (@org/xxx)
   c) 插件配置文件: plugin.json

2. 插件加载:
   a) PluginLoader 负责加载和初始化插件
   b) PluginRegistry 管理已注册插件的生命周期
   c) 加载失败不阻断启动，记录 warning

3. 插件能力:
   a) 插件可注册自定义 tools
   b) 插件可注册自定义 commands
   c) 插件可注册 event listeners

4. 安全约束:
   b) 插件不得访问文件系统除自身目录外的路径
   b) 插件执行权限继承自主进程 (无沙箱隔离)
   c) 远程插件 (@org/xxx) 必须通过 npm 或 git 安装，禁止自动下载
```

### 4.4 Auth 系统规范 (C-026)

**条款 C-026: 认证系统规范**

```
1. 认证方式:
   a) API Key 认证 (通过环境变量或配置文件)
   b) OAuth 认证 (通过 provider)
   c) AuthManager 统一管理认证生命周期

2. Token 管理:
   a) API Key 存储在配置文件中时，必须使用 {file:path} 引用
   b) 禁止在配置文件中明文存储 API Key
   c) Token 刷新由 provider 自行处理

3. Account 管理:
   a) Account 结构包含: id, email, provider, status
   b) 多 provider 账户关联支持
   c) 账户信息持久化到 storage

4. 安全约束:
   a) 认证失败不暴露具体错误原因 (防止枚举攻击)
   b) API Key 文件权限必须为 600 (仅所有者可读写)
   c) 认证状态不得缓存超过 24 小时
```

### 4.5 Share 系统规范 (C-027)

**条款 C-027: 会话分享规范**

```
1. 分享模式:
   a) "manual"   — 用户手动触发分享
   b) "auto"     — 会话结束后自动分享
   c) "disabled" — 禁止分享

2. 分享内容:
   a) 分享包含: 对话历史、工具调用记录、文件修改
   b) 分享不包含: API Key、环境变量值、敏感文件内容
   c) 分享前必须执行敏感信息过滤

3. 分享存储:
   a) 分享数据上传到远程服务器
   b) 分享链接有效期: 30 天 (可配置)
   c) 分享数据加密存储

4. 安全约束:
   a) disabled 模式下，分享 API 必须返回 403
   b) 分享链接必须包含随机 token (不可预测)
   c) 用户必须明确知晓分享行为 (auto 模式需提前提示)
```

### 4.6 Storage 系统规范 (C-028)

**条款 C-028: 数据存储规范**

```
1. 数据库:
   a) 使用 SQLite 作为本地存储引擎
   b) 数据库文件位置: ~/.local/share/opencode/opencode.db
   c) 数据库通过 StorageService 访问

2. Schema 管理:
   a) 数据库 schema 通过 migration 系统管理
   b) 迁移文件位于 crates/storage/migrations/
   c) 迁移按版本号顺序执行，支持回滚

3. 数据模型:
   a) sessions 表: 会话元数据 (id, title, created_at, parent_session_id)
   b) messages 表: 对话消息 (id, session_id, role, content, timestamp)
   c) tool_results 表: 工具执行结果
   d) audit_log 表: 权限审计日志

4. 安全约束:
   a) 数据库文件权限必须为 600
   b) 敏感字段 (API Key 等) 必须加密存储
   c) 数据库损坏时不得自动删除，应重命名备份
```

### 4.7 Tools 配置规范 (C-029)

**条款 C-029: 工具配置规范**

```
1. 工具禁用:
   a) tools 配置项中设为 false 的工具对当前会话全局禁用
   b) 禁用优先级: tools 配置 > agent.tools 配置 > 默认值
   c) 被禁用的工具在工具列表中显示为 disabled

2. 内置工具清单:
   a) read, write, edit, bash, glob, grep, lsp, web_search 等
   b) 每个工具必须声明所需权限级别
   c) 工具执行失败必须返回错误信息，不得静默失败

3. 自定义工具:
   a) 通过 MCP server 注册外部工具
   b) 通过 plugin 注册自定义工具
   c) 自定义工具不得覆盖内置工具名称

4. 安全约束:
   a) bash 工具默认必须经过权限检查
   b) write/edit 工具不得修改 .git/ 目录
   c) 工具执行超时必须可配置 (默认 30s)
```

### 4.8 Provider 控制规范 (C-030)

**条款 C-030: Provider 控制规范**

```
1. Provider 启用/禁用:
   a) disabled_providers: 黑名单，禁用指定 provider
   b) enabled_providers: 白名单，仅允许指定 provider
   c) 优先级: disabled_providers > enabled_providers (黑名单优先)

2. Provider 配置:
   a) 每个 provider 可配置: timeout, chunkTimeout, setCacheKey
   b) Amazon Bedrock 额外支持: region, profile, endpoint
   c) Provider 配置支持 {env:VAR} 变量替换 (用于 API Key)

3. Model 选择:
   a) 全局 model 通过 "provider/model" 格式指定
   b) small_model 用于轻量级任务 (标题生成等)
   c) Agent 可覆盖全局 model 设置

4. 安全约束:
   a) API Key 必须通过环境变量或 {file:path} 提供
   b) 禁止在日志中记录 API Key 或完整请求体
   c) Provider 连接失败时必须降级到可用 provider (如有)
```

### 4.9 Formatters 规范 (C-031)

**条款 C-031: 格式化器规范**

```
1. Formatter 注册:
   a) 内置 formatter: prettier (可通过 disabled 禁用)
   b) 自定义 formatter: 通过 command + extensions 定义
   c) Formatter 按文件扩展名匹配

2. 自定义 Formatter:
   a) command: 执行格式化的命令数组 (支持 $FILE 变量)
   b) environment: 可选的环境变量映射
   c) extensions: 支持的文件扩展名列表

3. 执行流程:
   a) 文件写入后自动触发匹配的 formatter
   b) Formatter 执行失败不阻断写入，记录 warning
   c) 多个 formatter 匹配同一扩展名时，按配置顺序执行

4. 安全约束:
   a) Formatter 命令执行超时时必须可配置 (默认 10s)
   b) Formatter 不得修改非目标文件
   c) Formatter 执行目录为项目根目录
```

### 4.10 Instructions 规范 (C-032)

**条款 C-032: 指令文件规范**

```
1. 指令文件加载:
   a) instructions 配置项接受文件路径列表
   b) 路径支持: 绝对路径、相对路径 (相对于项目根)、glob 模式
   c) 加载顺序: 按配置列表顺序，先加载的优先级低

2. 路径解析:
   a) 以 / 开头: 绝对路径
   b) 以 ./ 或 ../ 开头: 相对于项目根目录
   c) 包含 * 或 **: glob 模式，匹配多个文件
   d) 文件不存在: 记录 warning，不阻断启动

3. 指令内容:
   a) 指令文件内容作为 system prompt 的一部分
   b) 多个指令文件内容按加载顺序拼接
   c) 指令文件内容变化必须触发 context 更新

4. 安全约束:
   a) 指令文件不得包含可执行代码
   b) 指令文件大小限制: 单文件不超过 100KB
   c) 指令文件总大小不超过模型上下文窗口的 10%
```

---

## 五、条款更新映射 (v1.5)

### 新增条款

| 条款 | 模块 | 覆盖 PRD 章节 |
|------|------|---------------|
| C-023 | Agent 系统 | PRD §5.4 |
| C-024 | Permission 系统 | PRD §5.8 |
| C-025 | Plugin 系统 | PRD §5.12 |
| C-026 | Auth 系统 | PRD §4 |
| C-027 | Share 系统 | PRD §5.6 |
| C-028 | Storage 系统 | PRD §10 |
| C-029 | Tools 配置 | PRD §5.2 |
| C-030 | Provider 控制 | PRD §5.3, §5.14 |
| C-031 | Formatters | PRD §5.7 |
| C-032 | Instructions | PRD §5.13 |

### 保持不变

| 条款 | 说明 |
|------|------|
| C-001 | 已废止 (被 C-016 替代) |
| C-002 ~ C-022 | 不受本次更新影响，保持有效 |

---

## 六、验证清单 (v1.5 新增)

### C-023: Agent 系统
- [ ] 所有 Agent 是否实现 Agent trait
- [ ] Agent 的 tool 禁用是否在创建时锁定
- [ ] 自定义 agent 是否未覆盖内置 agent 名称
- [ ] Agent 加载优先级是否正确

### C-024: Permission 系统
- [ ] 权限评估是否记录到 AuditLog
- [ ] deny 是否优先级最高
- [ ] bash 工具默认是否为 "ask" 或 "deny"
- [ ] 权限配置是否不可被 Remote Config 覆盖

### C-025: Plugin 系统
- [ ] 插件加载失败是否不阻断启动
- [ ] 插件是否不得访问自身目录外的文件
- [ ] 远程插件是否禁止自动下载

### C-026: Auth 系统
- [ ] API Key 是否通过 {file:path} 引用而非明文
- [ ] 认证失败是否不暴露具体错误原因
- [ ] 认证状态是否不缓存超过 24 小时

### C-027: Share 系统
- [ ] disabled 模式是否返回 403
- [ ] 分享前是否执行敏感信息过滤
- [ ] 分享链接是否包含随机 token

### C-028: Storage 系统
- [ ] 数据库文件权限是否为 600
- [ ] 敏感字段是否加密存储
- [ ] 迁移是否按版本号顺序执行

### C-029: Tools 配置
- [ ] bash 工具是否默认经过权限检查
- [ ] write/edit 是否不得修改 .git/ 目录
- [ ] 工具执行超时是否可配置

### C-030: Provider 控制
- [ ] disabled_providers 是否优先级高于 enabled_providers
- [ ] API Key 是否通过环境变量或 {file:path} 提供
- [ ] 日志是否不记录 API Key

### C-031: Formatters
- [ ] Formatter 执行失败是否不阻断写入
- [ ] 多个 formatter 是否按配置顺序执行
- [ ] Formatter 命令超时是否可配置

### C-032: Instructions
- [ ] 指令文件路径是否支持 glob 模式
- [ ] 文件不存在是否记录 warning 不阻断
- [ ] 指令文件总大小是否不超过上下文窗口 10%

---

## 七、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0 | 2026-04-04 | 初始版本，覆盖 Context/Plugin/Skills/Commands/MCP |
| 1.1 | 2026-04-04 | 新增 Config System 条款 (C-011, C-012, C-013) |
| 1.2 | 2026-04-04 | 新增 TUI Input Parser (C-014), Session Fork (C-015), Context Token Budget (C-016) |
| 1.3 | 2026-04-04 | 新增 TUI 配置分离 (C-017), 路径命名 (C-018), 文件引用变量 (C-019), 细化 C-013 |
| 1.4 | 2026-04-04 | 新增 Server 配置 (C-020), Compaction (C-021), Watcher (C-022) |
| **1.5** | **2026-04-04** | **新增 Agent (C-023), Permission (C-024), Plugin (C-025), Auth (C-026), Share (C-027), Storage (C-028), Tools (C-029), Provider (C-030), Formatters (C-031), Instructions (C-032)** |

---

## 八、设计决策约束 (v1.5 更新版)

| 设计决策 | 必须遵循条款 |
|----------|-------------|
| Config 系统实现 | C-011, C-012, C-013, C-017, C-018, C-019 |
| TUI 配置实现 | C-014, C-017 |
| 变量替换实现 | C-012, C-019 |
| 目录扫描实现 | C-013 |
| 路径获取实现 | C-018 |
| Server 实现 | C-020 |
| Compaction 实现 | C-021 |
| Watcher 实现 | C-022 |
| **Agent 实现** | **C-023** |
| **Permission 实现** | **C-024** |
| **Plugin 实现** | **C-025** |
| **Auth 实现** | **C-026** |
| **Share 实现** | **C-027** |
| **Storage 实现** | **C-028** |
| **Tools 实现** | **C-029** |
| **Provider 实现** | **C-030** |
| **Formatters 实现** | **C-031** |
| **Instructions 实现** | **C-032** |

---

## 九、建议 (非 Constitution 约束)

### 9.1 Constitution 位置建议

当前 Constitution 仅存在于 `outputs/iteration-*/constitution_updates.md`，建议：

1. **合并到单一文件**: 将 v1.5 所有条款合并为 `docs/constitution.md`
2. **在 AGENTS.md 中引用**: 确保 AI agent 启动时加载 Constitution
3. **建立版本控制**: 每次迭代更新时递增版本号
4. **建立检查流程**: 每次 PR 提交时验证新代码是否遵循 Constitution

### 9.2 仍待覆盖的领域 (v1.6 候选)

| 领域 | 优先级 | 原因 |
|------|--------|------|
| Streaming 消息格式 | P1 | WebSocket/SSE 消息结构需标准化 |
| Control Plane / ACP | P1 | session 编排协议需约束 |
| CLI 命令架构 | P2 | 命令注册、参数解析、帮助文本 |
| 远程配置安全 | P2 | Remote Config 的认证和完整性验证 |
| 事件总线 | P2 | 插件/系统间事件通信机制 |

---

*本文档作为 OpenCode-RS 项目的 Constitution v1.5 更新建议，新增 10 条条款覆盖 Agent/Permission/Plugin/Auth/Share/Storage/Tools/Provider/Formatters/Instructions 领域。*
