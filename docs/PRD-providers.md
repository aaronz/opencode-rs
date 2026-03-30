下面这段可以直接补进你的产品设计文档，建议作为单独章节：

---

## 7.x Provider 与认证协议设计

### 7.x.1 OpenCode 当前的 Provider 能力边界

OpenCode 当前不是只绑定少数几家模型厂商，而是基于 **AI SDK + Models.dev** 做 Provider 抽象，官方文档写明可支持 **75+ LLM providers**，同时支持接入本地模型。Provider 的接入流程分两步：先通过 `/connect` 或 `opencode auth login` 写入凭证，再通过 `opencode.json` 配置 provider、model 和 small_model。凭证默认存放在 `~/.local/share/opencode/auth.json`；启动时还会读取环境变量以及项目目录下的 `.env`。([OpenCode][1])

官方文档当前明确列出的 Provider 目录包括：302.AI、Amazon Bedrock、Anthropic、Azure OpenAI、Azure Cognitive Services、Baseten、Cerebras、Cloudflare AI Gateway、Cloudflare Workers AI、Cortecs、DeepSeek、Deep Infra、Firmware、Fireworks AI、GitLab Duo、GitHub Copilot、Google Vertex AI、Groq、Hugging Face、Helicone、llama.cpp、IO.NET、LM Studio、Moonshot AI、MiniMax、Nebius Token Factory、Ollama、Ollama Cloud、OpenAI、OpenCode Zen、OpenRouter、SAP AI Core、STACKIT、OVHcloud AI Endpoints、Scaleway、Together AI、Venice AI、Vercel AI Gateway、xAI、Z.AI、ZenMux，以及自定义 Provider。这个目录本身已经说明，OpenCode 的产品定位是“Provider 平台层”，而不是“单一模型客户端”。([OpenCode][1])

### 7.x.2 模型标识与配置规则

OpenCode 的模型选择以 `provider/model` 形式组织；官方 CLI 的 `opencode models` 也明确按这个格式列出所有可用模型。配置层面，`model` 用于主模型，`small_model` 用于标题生成等轻量任务；若 Provider 有更便宜模型可用，会优先用更便宜模型，否则回退到主模型。对于自定义 Provider，`provider_id` 来自配置里的 `provider` key，`model_id` 来自 `provider.models` 的 key。([OpenCode][2])

### 7.x.3 Provider 配置抽象

OpenCode 对自定义 Provider 的抽象已经比较清晰，至少包含这些字段：

* `npm`：底层 AI SDK provider 包
* `name`：UI 展示名
* `models`：可选模型列表
* `options.baseURL`：API 入口
* `options.apiKey`：可直接在配置中写入或从环境变量/文件引用
* `options.headers`：自定义 Header
* `model.limit`：上下文和输出上限等能力约束

官方文档还特别说明：
对于大多数 **OpenAI-compatible** 接口，推荐用 `@ai-sdk/openai-compatible`；如果目标接口走的是 `/v1/responses` 而不是 `/v1/chat/completions`，则要用 `@ai-sdk/openai`。这意味着 Rust 版在设计时，不能把 Provider 仅仅抽象成“API Key + Base URL”，还需要显式抽象“协议族”。([OpenCode][1])

### 7.x.4 Provider 的启用、禁用与来源优先级

OpenCode 支持通过 `enabled_providers` 做白名单，通过 `disabled_providers` 做黑名单；并且官方文档明确说明 `disabled_providers` 优先级高于 `enabled_providers`。如果某个 Provider 被禁用，即使环境变量存在、即使 `/connect` 已配置过 API Key，它也不会被加载，模型列表里也不会出现。配置文件还支持 `{env:VAR}` 和 `{file:path}` 两种变量替换方式，因此密钥既可以来自环境变量，也可以来自单独的 secret 文件。([OpenCode][3])

---

## 7.x.5 认证协议分层

OpenCode 当前实际上有 **五类认证层**，Rust 版必须分开设计，不能混成一个 `AuthProvider`：

### A. Provider 凭证认证

最常见的是 API Key / Token / Service Key 模式，通过 `/connect` 或 `opencode auth login` 写入 `auth.json`，也可直接来自环境变量或配置文件。典型代表包括 OpenRouter、DeepSeek、Together AI、xAI、Z.AI、Baseten、Fireworks、Venice、ZenMux 等。([OpenCode][1])

### B. 浏览器登录 / 订阅账号认证

OpenCode 对一部分“订阅式服务”提供了浏览器授权流程，而不是只接受裸 API Key。官方文档明确写到：

* OpenAI 可在 `/connect` 里选择 **ChatGPT Plus/Pro**，浏览器完成认证；也可手动输入 API Key。([OpenCode][1])
* GitHub Copilot 使用 **device code** 流程，用户访问 `github.com/login/device` 输入代码完成授权。部分模型需要 Pro+ 订阅。([OpenCode][1])
* GitLab Duo 支持 **OAuth（推荐）** 或 **Personal Access Token** 两种方式。PAT 需要 `api` scope；Self-Managed 场景还可用 `GITLAB_INSTANCE_URL`、`GITLAB_TOKEN`、`GITLAB_AI_GATEWAY_URL` 等环境变量。([OpenCode][1])

### C. 云厂商原生认证

这类 Provider 不是简单的 API Key，而是复用云平台认证体系：

* **Amazon Bedrock** 支持 `AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY`、`AWS_PROFILE`、`AWS_BEARER_TOKEN_BEDROCK`，以及 `AWS_WEB_IDENTITY_TOKEN_FILE + AWS_ROLE_ARN` 这类 OIDC / IRSA 场景。官方还给出了认证优先级：Bearer Token 优先于 AWS Credential Chain。([OpenCode][1])
* **Google Vertex AI** 需要 `GOOGLE_CLOUD_PROJECT`，认证可以选 `GOOGLE_APPLICATION_CREDENTIALS` 指向 service account JSON，或者走 `gcloud auth application-default login`。([OpenCode][1])
* **SAP AI Core** 接受 service key JSON，也可通过 `AICORE_SERVICE_KEY` 环境变量注入。([OpenCode][1])
* **Cloudflare AI Gateway / Workers AI** 除 API token 外，还要求 Account ID，AI Gateway 还需要 Gateway ID。([OpenCode][1])

### D. MCP 认证

MCP 是独立于 Provider 的认证层。OpenCode 提供 `opencode mcp auth`，会拉起浏览器完成 OAuth；认证结果存储在 `~/.local/share/opencode/mcp-auth.json`。如果某个 MCP Server 不走 OAuth，而是 API Key/Header 方式，可以在 `mcp` 配置里显式把 `oauth: false`，并通过 `headers.Authorization` 之类字段手动注入 Bearer Token。([OpenCode][4])

### E. Server / Enterprise 认证

这不是“模型 Provider 认证”，而是 OpenCode Runtime 本身的访问控制：

* 本地/远程 `opencode serve` 和 `opencode web` 可通过 `OPENCODE_SERVER_PASSWORD` 开启 **HTTP Basic Auth**，用户名默认 `opencode`，也可用 `OPENCODE_SERVER_USERNAME` 覆盖。([OpenCode][5])
* Enterprise 版本支持 **Central Config + SSO integration**，并可强制所有请求只走企业内部 AI gateway，同时禁用其他外部 Provider。([OpenCode][6])

---

## 7.x.6 特殊 Provider 说明

### OpenCode Zen / OpenCode Go

这两个并不是通用第三方 Provider，而是 OpenCode 团队提供的“经过验证的模型清单/套餐”。它们都通过 `opencode.ai/auth` 获取 API Key，再由 `/connect` 写入本地认证存储。Zen 偏向“官方验证模型列表”，Go 偏向“低成本订阅方案”。([OpenCode][1])

### OpenAI / Anthropic 的订阅接入边界

官方文档当前明确支持：

* OpenAI：ChatGPT Plus/Pro 浏览器认证，或手动 API Key。([OpenCode][1])
* Anthropic：文档提到可在 `/connect` 中选择 Claude Pro/Max 浏览器认证，但同页也明确写了：**允许把 Claude Pro/Max 用在 OpenCode 的那些插件已不再随 OpenCode 一起分发，并注明 Anthropic 明确禁止这种做法**。换句话说，Rust 版文档里不应把“Claude Pro/Max 订阅直连”当作稳定、推荐、可默认支持的正式能力。([OpenCode][1])

### 网关型 Provider

Cloudflare AI Gateway、Vercel AI Gateway、Helicone、OpenRouter 这类 Provider 的核心价值不是“自有模型”，而是“统一入口 / 路由 / 账单 / 观测”。其中：

* Cloudflare AI Gateway 能统一接 OpenAI、Anthropic、Workers AI 等，并支持 Unified Billing。([OpenCode][1])
* Vercel AI Gateway 能统一接 OpenAI、Anthropic、Google、xAI 等，并支持按 `order`、`only`、`zeroDataRetention` 做 provider routing。([OpenCode][1])
* Helicone 通过自定义 headers 支持缓存、用户追踪、session tracking。([OpenCode][1])

### 本地 Provider

官方文档明确给出了 `llama.cpp`、LM Studio、Ollama 三类本地模型接入示例，它们统一走 `@ai-sdk/openai-compatible`，核心是配置 `baseURL` 和可用模型列表。([OpenCode][1])

---

## 7.x.7 Rust 版建议的认证架构

Rust 版不要把所有认证揉成一个 Provider trait，建议拆成四层：

### 1. Credential Source

负责“凭证从哪里来”：

* auth.json
* env
* .env
* config variables
* file secrets
* browser callback result
* device code result

### 2. Auth Mechanism

负责“如何拿到有效凭证”：

* API Key
* Bearer Token
* Basic Auth
* OAuth Browser Flow
* Device Code Flow
* Cloud Credential Chain
* Service Account JSON
* SSO-issued internal gateway token

### 3. Provider Transport

负责“如何把凭证放进请求”：

* Header `Authorization: Bearer ...`
* 自定义 Header
* query / body 字段
* AWS SigV4 / credential chain
* OpenAI-compatible transport
* Responses API transport

### 4. Runtime Access Control

负责“谁可以访问本地 server / web / MCP 工具”：

* server basic auth
* MCP token store
* enterprise central policy
* provider allow/deny list

---

## 7.x.8 Rust 版建议的数据结构

```rust id="go5w1l"
pub enum AuthMechanism {
    ApiKey,
    BearerToken,
    BasicAuth,
    OAuthBrowser,
    DeviceCode,
    AwsCredentialChain,
    ServiceAccountJson,
    SsoGatewayToken,
}

pub enum CredentialSource {
    AuthFile,
    EnvVar,
    DotEnv,
    ConfigInline,
    FileRef,
    OAuthStore,
    SystemKeychain, // Rust版可扩展
}

pub struct ProviderAuthSpec {
    pub provider_id: String,
    pub mechanism: AuthMechanism,
    pub source: CredentialSource,
    pub supports_interactive_login: bool,
    pub supports_env_override: bool,
    pub supports_header_injection: bool,
}
```

---

## 7.x.9 产品约束建议

1. **Provider 认证** 和 **Runtime 访问认证** 必须分开。
   不要把 server basic auth 和模型 API key 放进同一套接口。([OpenCode][5])

2. **MCP OAuth** 必须是独立 token store。
   官方就是把它单独放在 `mcp-auth.json`，不和普通 provider auth 混用。([OpenCode][4])

3. **本地/网关/OpenAI-compatible Provider** 必须允许自定义 `baseURL + headers + model map`。
   这是 OpenCode 现有生态可扩展的关键。([OpenCode][1])

4. **订阅式登录能力不要硬编码为通用协议**。
   ChatGPT Plus/Pro、GitHub Copilot、GitLab Duo 这些更像“特定厂商登录适配器”，不应污染通用 Provider 接口。([OpenCode][1])

5. **企业版必须支持“只允许内部 AI Gateway”**。
   这在官方 Enterprise 设计里是明确能力边界。([OpenCode][6])

---

如果你要，我下一步可以继续把这一节再扩成 **“Rust 版 Provider/认证模块详细设计”**，包括：
**crate 划分、trait 设计、auth.json 与 mcp-auth.json 数据结构、OAuth/device code 状态机、错误码与回退策略**。

[1]: https://opencode.ai/docs/providers/ "Providers | OpenCode"
[2]: https://opencode.ai/docs/cli/ "CLI | OpenCode"
[3]: https://opencode.ai/docs/config/ "Config | OpenCode"
[4]: https://opencode.ai/docs/mcp-servers/ "MCP servers | OpenCode"
[5]: https://opencode.ai/docs/server/ "Server | OpenCode"
[6]: https://opencode.ai/docs/enterprise/ "Enterprise | OpenCode"
