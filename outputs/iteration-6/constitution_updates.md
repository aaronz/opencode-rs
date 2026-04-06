# Constitution 审计报告 (v1.5 → v1.6)

**日期**: 2026-04-05  
**审计范围**: Constitution v1.5 (C-001 ~ C-032) vs 差距分析新发现的 P0/P1 问题  
**审计依据**: PRD-providers.md, PRD-OpenCode-Configuration.md, 差距分析报告 (2026-04-05)

---

## 一、审计结论

### Constitution v1.5 状态: **P0 问题未覆盖，需紧急修订**

| 指标 | 值 |
|------|-----|
| Constitution 条款总数 | 32 (C-001 ~ C-032, C-001 已废止) |
| 新发现 P0 问题数 | 2 |
| **P0 被现有条款覆盖** | **0/2 (0%)** |
| 新发现 P1 问题数 | 9 |
| **P1 被现有条款覆盖** | **2/9 (22%)** |
| 建议修改条款 | C-026 (Auth), C-030 (Provider) — **重大重写** |
| 建议新增条款 | C-033 ~ C-037 (5 条) |

### 关键发现

1. **C-026 (Auth) 与 PRD-providers.md 严重脱节** — 当前 C-026 仅 4 条，覆盖 API Key + OAuth 基础管理，完全缺失 PRD-providers.md 定义的 **5 层认证架构** (Credential Source → Auth Mechanism → Provider Transport → Runtime Access Control)
2. **C-030 (Provider) 缺失核心优先级规则** — PRD 明确要求 `disabled_providers > enabled_providers`，C-030 虽提到但无强制约束力
3. **Constitution 仍不存在于项目根目录** — 连续 6 次迭代未被 AGENTS.md 或任何构建流程引用
4. **TUI 三栏布局/状态机无对应条款** — PRD-tui.md 定义 10 种状态 + 三栏布局 + Inspector 6 tab，当前无任何 Constitution 约束

---

## 二、P0/P1 问题覆盖度验证

### 2.1 P0 问题覆盖 (0/2 — 完全未覆盖)

| # | P0 问题 | 最接近条款 | 覆盖状态 | 差距 |
|---|---------|-----------|----------|------|
| 1 | **Provider 认证协议未分层抽象** | C-026 (Auth 系统) | ❌ 未覆盖 | C-026 仅定义 API Key/OAuth 基础管理，无 4 层架构 (Credential Source / Auth Mechanism / Provider Transport / Runtime Access Control) |
| 2 | **OAuth/Device Code 浏览器登录流程缺失** | C-026 §1b | ❌ 部分覆盖 | C-026 提到 "OAuth 认证 (通过 provider)" 但未定义 OAuth Browser Flow / Device Code Flow 的状态机、回调处理、token 刷新 |

### 2.2 P1 问题覆盖 (2/9 — 部分覆盖)

| # | P1 问题 | 最接近条款 | 覆盖状态 | 差距 |
|---|---------|-----------|----------|------|
| 3 | 云厂商原生认证 (AWS SigV4, Vertex AI, SAP AI Core) | C-030 §2b | ⚠️ 部分覆盖 | C-030 仅提到 Bedrock 额外字段 (region, profile, endpoint)，未定义 AWS Credential Chain 优先级 (Bearer Token > Credential Chain)、Vertex AI 的 GOOGLE_APPLICATION_CREDENTIALS、SAP AI Core 的 service key |
| 4 | Remote Config 自动发现 | C-011 (Config 优先级加载) | ⚠️ 间接覆盖 | C-011 定义配置加载优先级但未强制 Remote Config 的 `.well-known/opencode` 自动发现机制 |
| 5 | disabled_providers 优先级高于 enabled_providers | C-030 §1c | ✅ 已覆盖 | C-030 §1c 明确定义 "优先级: disabled_providers > enabled_providers (黑名单优先)" |
| 6 | MCP OAuth 独立 token store | C-025 (Plugin) | ❌ 未覆盖 | C-025 定义插件系统，MCP OAuth 独立存储 (`mcp-auth.json`) 无对应条款 |
| 7 | TUI 三栏布局/Inspector | 无 | ❌ 未覆盖 | 无 TUI 布局相关条款 |
| 8 | TUI 状态机 (10 种状态) | 无 | ❌ 未覆盖 | 无 TUI 状态机相关条款 |
| 9 | Context Engine 分层上下文 (L0-L4) | C-016 (Context Token Budget) | ⚠️ 部分覆盖 | C-016 定义 token budget + 压缩，但未定义 L0-L4 五层上下文结构 |
| 10 | Plugin WASM 运行时 | C-025 (Plugin 系统) | ⚠️ 部分覆盖 | C-025 定义插件发现/加载/能力，但未定义 WASM 运行时 (wasmtime) 和沙箱隔离 |
| 21 | 凭证加密存储 | C-026 §2b, C-028 §4b | ✅ 已覆盖 | C-026 §2b "禁止在配置文件中明文存储 API Key" + C-028 §4b "敏感字段必须加密存储" |

---

## 三、现有条款修订 (v1.6)

### 3.1 C-026 重大重写: Auth 系统规范

**当前 C-026 问题**: 仅 4 条，覆盖 API Key + OAuth 基础管理，完全缺失 PRD-providers.md 定义的 5 层认证架构。

**修订后 C-026**:

```
条款 C-026: 认证系统规范 (v1.6 重写)

1. 认证分层架构 (PRD-providers.md §7.x.5):
   a) 认证必须分为 4 层，不得混为单一 AuthProvider:
      - Layer 1: Credential Source (凭证来源)
      - Layer 2: Auth Mechanism (认证机制)
      - Layer 3: Provider Transport (传输层)
      - Layer 4: Runtime Access Control (运行时访问控制)
   b) Provider 认证 与 Runtime 访问认证 必须分开接口
   c) MCP OAuth 必须是独立 token store (见 C-033)

2. Credential Source (Layer 1):
   a) 支持的凭证来源:
      - auth.json (~/.local/share/opencode/auth.json)
      - 环境变量 (OPENCODE_* 或 provider 特定变量)
      - .env 文件 (项目根目录)
      - 配置文件内联变量
      - 文件引用 ({file:path})
      - OAuth Store (OAuth 流程结果)
      - System Keychain (系统密钥链，Rust 版可扩展)
   b) 凭证来源优先级: 环境变量 > auth.json > .env > 配置文件
   c) 凭证加载失败不阻断启动，记录 warning

3. Auth Mechanism (Layer 2):
   a) 支持的认证机制:
      - API Key (简单 token 传递)
      - Bearer Token (OAuth/OIDC token)
      - Basic Auth (username:password base64)
      - OAuth Browser Flow (浏览器授权回调)
      - Device Code Flow (用户输入 code 完成授权)
      - AWS Credential Chain (AWS 凭证链)
      - Service Account JSON (GCP/SAP 服务账户)
      - SSO Gateway Token (企业内部网关 token)
   b) 每种机制必须声明其 Credential Source 兼容性
   c) OAuth Browser Flow 必须实现:
      - 本地 HTTP 回调服务器 (接收 OAuth callback)
      - 浏览器自动打开 (xdg-open / open)
      - Token 持久化到 auth.json 或 OAuth Store
      - Token 自动刷新 (refresh_token 机制)
   d) Device Code Flow 必须实现:
      - Device code 获取与展示
      - 轮询授权状态 (polling interval 遵循 provider 规范)
      - 超时与取消处理
   e) AWS Credential Chain 优先级:
      - Bearer Token (AWS_BEARER_TOKEN_BEDROCK) > AWS Credential Chain
      - Credential Chain 顺序: 环境变量 > 配置文件 > IAM Role (OIDC/IRSA)

4. Provider Transport (Layer 3):
   a) 支持的传输方式:
      - Header Authorization: Bearer {token}
      - Header Authorization: Basic {base64}
      - 自定义 Header (通过 options.headers 配置)
      - Query / Body 字段注入
      - AWS SigV4 签名
      - OpenAI-compatible transport (/v1/chat/completions)
      - Responses API transport (/v1/responses)
   b) 每个 provider 必须声明其 transport 类型
   c) OpenAI-compatible provider 必须允许自定义 baseURL + headers + model map

5. Runtime Access Control (Layer 4):
   a) 负责本地 server / web / MCP 工具的访问控制
   b) Server Basic Auth:
      - 通过 OPENCODE_SERVER_PASSWORD 启用
      - 用户名默认 "opencode"，可通过 OPENCODE_SERVER_USERNAME 覆盖
   c) Enterprise Central Policy:
      - 支持强制只走内部 AI Gateway
      - 支持禁用所有外部 Provider
   d) Provider allow/deny list 由 C-030 定义

6. 安全约束:
   a) 认证失败不暴露具体错误原因 (防止枚举攻击)
   b) API Key 文件权限必须为 600 (仅所有者可读写)
   c) 认证状态缓存不得超过 24 小时
   d) 禁止在日志中记录 API Key、token 或完整请求体
   e) 订阅式登录 (ChatGPT Plus/Pro, GitHub Copilot, GitLab Duo) 不得硬编码为通用协议
   f) Claude Pro/Max 订阅直连不得作为稳定能力支持 (Anthropic 明确禁止)
```

### 3.2 C-030 修订: Provider 控制规范

**当前 C-030 问题**: 已定义 `disabled_providers > enabled_providers` 优先级，但缺失云厂商认证细节、transport 协议族声明、gateway provider 特殊处理。

**修订后 C-030**:

```
条款 C-030: Provider 控制规范 (v1.6 修订)

1. Provider 启用/禁用:
   a) disabled_providers: 黑名单，禁用指定 provider
   b) enabled_providers: 白名单，仅允许指定 provider
   c) 优先级: disabled_providers > enabled_providers (黑名单优先，强制)
   d) 被禁用的 provider 不得出现在模型列表中

2. Provider 配置:
   a) 每个 provider 可配置: timeout, chunkTimeout, setCacheKey
   b) Amazon Bedrock 额外支持: region, profile, endpoint, awsCredentialChain
   c) Google Vertex AI 额外支持: project, credentials (GOOGLE_APPLICATION_CREDENTIALS)
   d) SAP AI Core 额外支持: serviceKey (AICORE_SERVICE_KEY)
   e) Cloudflare 额外支持: accountId, gatewayId
   f) Provider 配置支持 {env:VAR} 和 {file:path} 变量替换
   g) 每个 provider 必须声明其 transport 协议族:
      - openai-compatible (@ai-sdk/openai-compatible)
      - responses-api (@ai-sdk/openai)
      - anthropic
      - custom (通过 npm 包指定)

3. Model 选择:
   a) 全局 model 通过 "provider/model" 格式指定
   b) small_model 用于轻量级任务 (标题生成等)，若 provider 有更便宜模型则优先使用
   c) Agent 可覆盖全局 model 设置

4. Gateway Provider 特殊处理:
   a) Cloudflare AI Gateway, Vercel AI Gateway, Helicone, OpenRouter 为网关型 provider
   b) 网关型 provider 必须支持 provider routing (order, only 配置)
   c) 网关型 provider 不得限制下游 provider 的认证方式

5. 本地 Provider:
   a) llama.cpp, LM Studio, Ollama 统一走 openai-compatible transport
   b) 本地 provider 必须允许自定义 baseURL + model list

6. 安全约束:
   a) API Key 必须通过环境变量或 {file:path} 提供，禁止配置文件中明文
   b) 禁止在日志中记录 API Key 或完整请求体
   c) Provider 连接失败时必须降级到可用 provider (如有)
   d) Enterprise 模式下必须支持强制只走内部 AI Gateway
```

---

## 四、新增 Constitution 条款 (v1.6)

### 4.1 MCP OAuth 独立存储规范 (C-033)

**条款 C-033: MCP OAuth 独立存储规范**

```
1. 存储隔离:
   a) MCP OAuth 凭证必须存储在独立文件: ~/.local/share/opencode/mcp-auth.json
   b) MCP OAuth 不得与普通 provider auth (auth.json) 混用
   c) MCP OAuth 凭证结构独立于 Provider Auth 结构

2. OAuth 流程:
   a) 通过 opencode mcp auth 命令触发浏览器 OAuth 流程
   b) OAuth 回调由本地 HTTP 服务器接收
   c) Token 持久化到 mcp-auth.json

3. 非 OAuth MCP Server:
   a) 不走 OAuth 的 MCP Server 可在配置中显式设置 oauth: false
   b) 通过 headers.Authorization 手动注入 Bearer Token
   c) API Key / Header 方式认证的 MCP Server 不使用 mcp-auth.json

4. 安全约束:
   a) mcp-auth.json 文件权限必须为 600
   b) MCP token 过期时必须自动触发刷新或重新认证
   c) MCP token 不得传递给非 MCP 的 provider
```

### 4.2 TUI 布局与状态机规范 (C-034)

**条款 C-034: TUI 布局与状态机规范**

```
1. 三栏布局 (PRD-tui.md):
   a) 左栏: Sidebar (会话列表、Agent 选择、设置入口)
   b) 中栏: Timeline (对话主区域，消息流、工具执行结果)
   c) 右栏: Inspector (详细信息面板)

2. Inspector 面板 (6 个 tab):
   a) Todo: 当前任务的 todo 列表
   b) Diff: 文件变更 diff 视图
   c) Diagnostics: LSP 诊断信息
   d) Context: 上下文文件/变量列表
   e) Permissions: 权限请求历史
   f) Files: 会话涉及的文件列表

3. TUI 状态机 (10 种状态):
   a) idle: 空闲，等待用户输入
   b) composing: 用户正在输入
   c) submitting: 消息提交中
   d) streaming: LLM 响应流式输出中
   e) executing_tool: 工具执行中
   f) awaiting_permission: 等待用户权限确认
   g) showing_diff: 展示文件 diff
   h) showing_error: 展示错误信息
   i) aborting: 中止操作中
   j) reconnecting: 重连中

4. 状态流转约束:
   a) 每次状态变更必须触发 UI 刷新
   b) aborting 状态必须可中断 streaming / executing_tool
   c) reconnecting 状态必须有超时机制

5. 性能约束:
   a) 10k+ 消息的 session 必须可打开 (虚拟滚动)
   b) 状态切换延迟不得超过 16ms (60fps)
   c) Inspector tab 切换不得重新渲染整个面板
```

### 4.3 Context Engine 分层上下文规范 (C-035)

**条款 C-035: Context Engine 分层上下文规范**

```
1. 五层上下文结构 (PRD):
   a) L0 - 显式输入: 用户直接提供的输入 (prompt, @file 引用)
   b) L1 - 会话上下文: 当前会话的对话历史、工具执行结果
   c) L2 - 项目上下文: 项目结构、AGENTS.md、instructions 文件
   d) L3 - 结构化上下文: LSP 符号、诊断、git diff
   e) L4 - 压缩记忆: 历史会话的压缩摘要

2. Token Budget 计算:
   a) 每层上下文必须声明其 token 预算上限
   b) 总 token 使用不得超过模型上下文窗口的 85%
   c) 超过 85% 必须触发预警
   d) 超过 92% 必须触发自动 compact
   e) 超过 95% 必须强制转入新 session

3. Relevance Ranking:
   a) 上下文条目必须按相关性排序
   b) L0 > L1 > L2 > L3 > L4 (优先级递减)
   c) 同层内按时间倒序或用户显式指定顺序

4. 安全约束:
   a) 上下文不得包含 API Key 或敏感环境变量
   b) 文件引用必须检查读取权限
   c) 压缩后的摘要必须保持语义连贯性
```

### 4.4 Plugin WASM 运行时规范 (C-036)

**条款 C-036: Plugin WASM 运行时规范**

```
1. WASM 运行时:
   a) 使用 wasmtime 作为 WASM 执行引擎
   b) WASM 插件必须声明其所需 capabilities (文件系统/网络/环境变量)
   c) WASM 插件通过 Sidecar 模式与主进程通信

2. 插件能力:
   a) WASM 插件可注册自定义 tools
   b) WASM 插件可注册自定义 commands
   c) WASM 插件可注册 event listeners
   d) WASM 插件不得直接访问主进程内存

3. 沙箱隔离:
   a) WASM 插件默认只能访问自身目录
   b) 文件系统访问必须通过 WASI 接口，且受 capabilities 限制
   c) 网络访问必须显式声明允许的域名
   d) 环境变量访问必须显式声明允许的变量名

4. 生命周期:
   a) WASM 插件加载失败不阻断启动，记录 warning
   b) WASM 插件执行超时必须可配置 (默认 30s)
   c) WASM 插件崩溃不得导致主进程退出

5. 安全约束:
   a) WASM 插件不得执行系统命令 (除非显式声明 bash capability)
   b) WASM 插件不得读取 .git/ 目录
   c) WASM 插件不得修改非自身目录的文件 (除非显式声明 write capability)
```

### 4.5 Remote Config 自动发现规范 (C-037)

**条款 C-037: Remote Config 自动发现规范**

```
1. 自动发现机制:
   a) 启动时检查 .well-known/opencode 端点 (如果配置了 remote URL)
   b) Remote Config 优先级高于本地配置 (见 C-011)
   c) Remote Config 获取失败时降级到本地配置，记录 warning

2. 配置内容:
   a) Remote Config 可覆盖: providers, agents, tools, formatters, instructions
   b) Remote Config 不得覆盖: permission (安全约束，见 C-024 §5a)
   c) Remote Config 必须支持 JSON 格式

3. 安全约束:
   a) Remote Config 必须支持 HTTPS (禁止 HTTP)
   b) Remote Config 响应必须进行 JSON Schema 验证
   c) Remote Config 不得包含 API Key 明文 (必须使用变量引用)
   d) Remote Config 缓存不得超过 1 小时
```

---

## 五、条款更新映射 (v1.6)

### 修订条款

| 条款 | 变更类型 | 变更内容 |
|------|----------|----------|
| C-026 | **重大重写** | 从 4 条扩展至 6 大节，新增 4 层认证架构、OAuth Browser/Device Code、AWS Credential Chain 优先级、5 类认证机制、3 类传输协议 |
| C-030 | **修订** | 新增 transport 协议族声明、Gateway Provider 特殊处理、本地 Provider 规范、Enterprise Gateway 强制模式 |

### 新增条款

| 条款 | 模块 | 覆盖 PRD 章节 |
|------|------|---------------|
| C-033 | MCP OAuth 独立存储 | PRD-providers.md §7.x.5.D |
| C-034 | TUI 布局与状态机 | PRD-tui.md |
| C-035 | Context Engine 分层 | PRD §6 (Context Engine) |
| C-036 | Plugin WASM 运行时 | PRD §8 (Plugin System) |
| C-037 | Remote Config 自动发现 | PRD-OpenCode-Configuration.md §1 |

### 保持不变

| 条款 | 说明 |
|------|------|
| C-001 | 已废止 (被 C-016 替代) |
| C-002 ~ C-025 | 不受本次更新影响，保持有效 |
| C-027 ~ C-032 | 不受本次更新影响，保持有效 |

---

## 六、P0/P1 覆盖度更新

### 修订后覆盖度

| 问题 | 原覆盖状态 | 修订后覆盖状态 | 对应条款 |
|------|-----------|---------------|----------|
| P0-1: Provider 认证协议未分层 | ❌ 未覆盖 | ✅ C-026 §1 (4 层架构) |
| P0-2: OAuth/Device Code 缺失 | ❌ 未覆盖 | ✅ C-026 §3c-d (Browser/Device Code 状态机) |
| P1-3: 云厂商原生认证 | ⚠️ 部分 | ✅ C-026 §3e + C-030 §2b-e |
| P1-4: Remote Config 自动发现 | ⚠️ 间接 | ✅ C-037 (自动发现机制) |
| P1-5: disabled_providers 优先级 | ✅ 已覆盖 | ✅ C-030 §1c |
| P1-6: MCP OAuth 独立存储 | ❌ 未覆盖 | ✅ C-033 |
| P1-7: TUI 三栏布局/Inspector | ❌ 未覆盖 | ✅ C-034 §1-2 |
| P1-8: TUI 状态机 | ❌ 未覆盖 | ✅ C-034 §3-4 |
| P1-9: Context Engine 分层 | ⚠️ 部分 | ✅ C-035 (L0-L4 五层 + token budget) |
| P1-10: Plugin WASM 运行时 | ⚠️ 部分 | ✅ C-036 (wasmtime + 沙箱) |
| P1-21: 凭证加密存储 | ✅ 已覆盖 | ✅ C-026 §6 + C-028 §4b |

**修订后 P0 覆盖率: 2/2 (100%)**  
**修订后 P1 覆盖率: 9/9 (100%)**

---

## 七、验证清单 (v1.6 新增)

### C-026 (重写): Auth 系统
- [ ] 认证是否分为 4 层 (Credential Source / Auth Mechanism / Provider Transport / Runtime Access Control)
- [ ] Provider 认证与 Runtime 访问认证是否分开接口
- [ ] OAuth Browser Flow 是否实现本地回调服务器 + 浏览器自动打开 + token 持久化 + 自动刷新
- [ ] Device Code Flow 是否实现 code 获取 + 轮询授权 + 超时处理
- [ ] AWS Credential Chain 优先级是否为 Bearer Token > Credential Chain
- [ ] 订阅式登录 (ChatGPT Plus/Pro, GitHub Copilot, GitLab Duo) 是否未硬编码为通用协议
- [ ] 认证失败是否不暴露具体错误原因
- [ ] 日志是否不记录 API Key 或完整请求体

### C-030 (修订): Provider 控制
- [ ] disabled_providers 是否严格优先于 enabled_providers
- [ ] 每个 provider 是否声明其 transport 协议族
- [ ] Gateway Provider 是否支持 provider routing
- [ ] 本地 provider 是否允许自定义 baseURL + model list
- [ ] Enterprise 模式是否支持强制只走内部 AI Gateway

### C-033: MCP OAuth 独立存储
- [ ] MCP OAuth 是否存储在独立 mcp-auth.json
- [ ] MCP OAuth 是否未与普通 provider auth 混用
- [ ] mcp-auth.json 文件权限是否为 600

### C-034: TUI 布局与状态机
- [ ] TUI 是否为三栏布局 (Sidebar/Timeline/Inspector)
- [ ] Inspector 是否包含 6 个 tab (Todo/Diff/Diagnostics/Context/Permissions/Files)
- [ ] 10 种状态是否全部实现
- [ ] aborting 状态是否可中断 streaming / executing_tool
- [ ] 10k+ 消息 session 是否可打开 (虚拟滚动)

### C-035: Context Engine 分层
- [ ] 是否实现 L0-L4 五层上下文
- [ ] 85% 预警 / 92% compact / 95% 强制转新 session 是否实现
- [ ] 上下文是否按相关性排序 (L0 > L1 > L2 > L3 > L4)

### C-036: Plugin WASM 运行时
- [ ] 是否使用 wasmtime 作为 WASM 执行引擎
- [ ] WASM 插件是否通过 WASI 接口访问文件系统
- [ ] WASM 插件崩溃是否不导致主进程退出
- [ ] WASM 插件是否不得执行系统命令 (除非显式声明)

### C-037: Remote Config 自动发现
- [ ] 启动时是否检查 .well-known/opencode 端点
- [ ] Remote Config 是否不得覆盖 permission 配置
- [ ] Remote Config 是否仅支持 HTTPS
- [ ] Remote Config 缓存是否不超过 1 小时

---

## 八、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0 | 2026-04-04 | 初始版本，覆盖 Context/Plugin/Skills/Commands/MCP |
| 1.1 | 2026-04-04 | 新增 Config System 条款 (C-011, C-012, C-013) |
| 1.2 | 2026-04-04 | 新增 TUI Input Parser (C-014), Session Fork (C-015), Context Token Budget (C-016) |
| 1.3 | 2026-04-04 | 新增 TUI 配置分离 (C-017), 路径命名 (C-018), 文件引用变量 (C-019), 细化 C-013 |
| 1.4 | 2026-04-04 | 新增 Server 配置 (C-020), Compaction (C-021), Watcher (C-022) |
| 1.5 | 2026-04-04 | 新增 Agent (C-023), Permission (C-024), Plugin (C-025), Auth (C-026), Share (C-027), Storage (C-028), Tools (C-029), Provider (C-030), Formatters (C-031), Instructions (C-032) |
| **1.6** | **2026-04-05** | **C-026 重大重写 (4 层认证架构), C-030 修订 (transport 协议族/Gateway/本地 Provider), 新增 C-033 (MCP OAuth 独立存储), C-034 (TUI 布局与状态机), C-035 (Context Engine 分层), C-036 (Plugin WASM 运行时), C-037 (Remote Config 自动发现)** |

---

## 九、设计决策约束 (v1.6 更新版)

| 设计决策 | 必须遵循条款 |
|----------|-------------|
| Config 系统实现 | C-011, C-012, C-013, C-017, C-018, C-019, C-037 |
| TUI 实现 | C-014, C-017, **C-034** |
| 变量替换实现 | C-012, C-019 |
| 目录扫描实现 | C-013 |
| 路径获取实现 | C-018 |
| Server 实现 | C-020 |
| Compaction 实现 | C-021, **C-035** |
| Watcher 实现 | C-022 |
| Agent 实现 | C-023 |
| Permission 实现 | C-024 |
| Plugin 实现 | C-025, **C-036** |
| **Auth 实现** | **C-026 (v1.6 重写)** |
| Share 实现 | C-027 |
| Storage 实现 | C-028 |
| Tools 实现 | C-029 |
| **Provider 实现** | **C-030 (v1.6 修订)** |
| Formatters 实现 | C-031 |
| Instructions 实现 | C-032 |
| **MCP 实现** | **C-033** |
| **Context Engine 实现** | **C-035** |

---

## 十、待解决 (v1.7 候选)

以下差距分析中发现的问题尚未被 Constitution 覆盖，建议纳入 v1.7:

| # | 问题 | 优先级 | 原因 |
|---|------|--------|------|
| 11 | Event Bus 事件类型不完整 | P2 | 12+ 事件类型需标准化消息格式 |
| 12 | Share 服务层未实现 | P2 | self-hosted share server + 短链 + 访问令牌 |
| 13 | SDK 输出 (Rust + TypeScript) | P2 | PRD 要求提供 SDK |
| 14 | OpenAPI 文档自动生成 | P2 | PRD v1 验收标准 |
| 15 | LSP definition/references/hover/code actions | P2 | v1.1 扩展功能 |
| 16 | session_load/session_save 工具 | P2 | 上次分析已识别仍未实现 |
| 19 | Formatters 接入 agent 执行循环 | P2 | 需验证是否接入 |
| 20 | Compaction 自动触发阈值 | P2 | 85%/92%/95% 阈值 (部分被 C-035 覆盖) |
| 22 | TUI 虚拟滚动 | P2 | 10k+ 消息性能要求 (已被 C-034 §5 覆盖) |
| 23 | Server HTTP Basic Auth | P2 | 已被 C-026 §5b 覆盖 |
| 25 | 观测性 (tracing/crash recovery/token cost) | P2 | 结构化日志、session traces、provider 统计 |

---

*本文档作为 OpenCode-RS 项目的 Constitution v1.6 更新建议。核心变更: C-026 从 4 条重写为 6 大节覆盖 4 层认证架构，C-030 修订增加 transport 协议族声明，新增 5 条条款覆盖 MCP OAuth 存储/TUI 布局状态机/Context Engine 分层/WASM 运行时/Remote Config 自动发现。*
