# OpenCode-RS 实现与 PRD 差距分析报告

**项目**: OpenCode-RS (Rust AI Coding Agent)  
**PRD 文档**: `/Users/aaronzh/Documents/GitHub/mycode/docs/PRD.md`  
**分析日期**: 2026-04-08  
**实现版本**: rust-opencode-port (当前代码库)

---

## 1. 执行摘要

当前实现处于 **v0.2-v0.3 阶段**，核心架构已基本搭建完成，但距离 PRD 中定义的 **v1.0 目标**仍有显著差距。实现进度约 **65-70%**。

### 已完成的核心模块

| 模块 | 状态 | 说明 |
|------|------|------|
| Workspace/Project 机制 | ✅ 完成 | 完整的项目绑定、文件树索引、Git 状态感知 |
| Session 会话系统 | ✅ 完成 | 支持创建/恢复/fork/abort/summarize，但 fork lineage 不完整 |
| Agent 系统 | ✅ 完成 | Build/Plan/General/Explore/Review/Refactor/Debug 7种 Agent |
| Tool Runtime | ✅ 完成 | 37个工具文件，基本覆盖 PRD 要求 |
| 权限系统 | ✅ 完成 | allow/ask/deny + 审计日志 + 队列管理 |
| Context Engine | ⚠️ 部分 | Token budget 和 compaction 已实现，上下文 ranking 不完整 |
| 配置系统 | ✅ 完成 | JSONC/TOML 多层配置合并，环境变量覆盖 |
| Model Gateway | ✅ 完成 | 17+ Provider 支持，认证抽象完整 |
| LSP 集成 | ⚠️ 部分 | diagnostics/symbols 已实现，definition/references 待完善 |
| MCP 系统 | ⚠️ 部分 | 基础协议完成，远程 MCP 不完整 |
| Plugin 系统 | ⚠️ 部分 | WASM runtime 已搭建，事件总线不完整 |
| TUI | ✅ 完成 | 三栏布局，消息流/权限确认/diff 预览完整 |
| Server API | ⚠️ 部分 | Session/Provider/Model/SSE/WS 已实现，SDK 缺失 |

---

## 2. 差距列表（表格格式）

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| Session fork lineage 不完整 - parent_session_id 追溯缺失 | P1 | Session | 在 session 表增加 lineage_path 字段，支持完整的 fork 历史追溯 |
| 缺少 lsp_definition/lsp_references 实现 | P1 | LSP | 在 lsp/src/ 中实现 definition 和 references 方法 |
| 缺少 lsp_hover/lsp_code_actions | P2 | LSP | v1.1 范围，但 PRD 已规划 |
| 远程 MCP 连接不稳定 | P1 | MCP | 补充 timeout 重试机制和连接状态管理 |
| MCP OAuth 认证未实现 | P2 | MCP | 仅有结构定义，需实现完整的 OAuth 流程 |
| Plugin 事件总线缺失 | P1 | Plugin | session.created 等事件未实现 |
| WASM 插件沙箱隔离不完整 | P1 | Plugin | crash 不影响主 Runtime 的隔离机制需加强 |
| SDK (Rust/TypeScript) 未实现 | P0 | Server | PRD 要求 v0.2 提供 SDK，当前仅 HTTP API |
| SSE 协议不完整 | P1 | Server | 心跳机制缺失，客户端重连处理不完整 |
| WebSocket 升级握手问题 | P1 | Server | 存在连接不稳定情况 |
| OAuth Browser 登录未实现 | P2 | Auth | 预留架构，API Key 模式可用 |
| Browser/OAuth callback state 校验 | P2 | Auth | PKCE 支持缺失 |
| 凭证明文存储风险 | P1 | Auth | credential ciphertext 加密存储未实现 |
| token refresh 与 session revoke | P2 | Auth | 仅有基础实现 |
| 缺少 /opencode 命令快捷方式 (@, !, /) 的完整解析 | P1 | TUI | 文件模糊匹配、多选功能不完整 |
| TUI diff 预览可展开功能 | P2 | TUI | 当前 diff_view 组件功能有限 |
| TUI token/cost 显示 | P2 | TUI | 状态栏有基础显示，准确性待验证 |
| Share 本地导出功能不完整 | P1 | Share | JSON/Markdown 导出缺少，patch bundle 未实现 |
| Share 服务端 (self-hosted share server) | P2 | Share | 短链/访问令牌/过期时间未实现 |
| GitHub Actions Runner 集成 | P2 | GitHub | PRD 规划 v2，非当前优先级 |
| GitHub issue/PR comment trigger | P2 | GitHub | PRD 规划 v2 |
| context compaction 阈值不准确 | P1 | Context | 85%/92%/95% 阈值逻辑未精确实现 |
| token budget 计算误差 | P1 | Context | tiktoken-rs 集成但计数不精确 |
| 压缩记忆 (session summary) 质量 | P1 | Context | SummaryGenerator 实现基础，准确性待提升 |
| 错误代码体系不完整 | P1 | Error | 1xxx-9xxx 错误代码部分缺失 |
| provider_header_invalid 等细分错误 | P2 | Error | 7xxx 验证错误缺失 |
| 崩溃恢复 (crash recovery) | P1 | Observability | 崩溃转储机制未实现 |
| Session traces / tool spans | P2 | Observability | tracing 集成但完整 traces 未实现 |
| provider latency/token/cost 统计 | P2 | Observability | CostCalculator 存在但统计不完整 |
| 敏感文件 (.env) 默认拒绝读取 | P1 | Security | 当前为 allow，PRD 要求默认 deny |
| 远程 MCP 默认 ask 未严格实施 | P1 | Security | 配置存在但执行层面检查缺失 |
| 导出 session/share 时 auth store 隔离 | P1 | Security | 明文 secret 可能被导出 |
| enterprise policy profile | P2 | Enterprise | 基础实现，高级策略缺失 |
| Windows 平台支持 | P2 | Platform | 仅验证 Linux/macOS |
| 自动化测试覆盖率低 | P1 | Testing | 仅有 session_tools_test, grep_tool_test, read_test, write_test, skill_test |

---

## 3. P0/P1/P2 问题分类

### P0 阻断性问题 (必须立即修复)

| 问题 | 影响 | 修复方案 |
|------|------|----------|
| **SDK 未实现** | 无法实现程序化调用，违背 PRD 核心场景 E | 实现 Rust SDK (opencode-sdk crate) 和 TypeScript SDK |
| **敏感文件默认读取** | 安全风险，违背 PRD 安全要求 | 实现 permission external_directory 拦截，默认 deny .env |

### P1 高优先级问题 (本迭代应完成)

| 问题 | 影响 | 修复方案 |
|------|------|----------|
| Session fork lineage 不完整 | 无法追溯完整的 fork 历史 | 补充 parent_session_id 追溯逻辑 |
| LSP definition/references 缺失 | 无法跳转定义/查找引用 | 在 lsp/src/ 实现完整 LSP v1.1 能力 |
| 远程 MCP 不稳定 | 外部工具集成受限 | 实现连接池、timeout、重试机制 |
| Plugin 事件总线缺失 | 插件无法监听 session 生命周期 | 实现完整事件总线 (session.created 等) |
| WASM 插件沙箱隔离不完整 | 插件崩溃可能影响主进程 | 完善 wasmtime 隔离配置 |
| SSE 心跳机制缺失 | 长连接不稳定 | 实现 heartbeat 模块 |
| WebSocket 连接不稳定 | TUI attach 场景受限 | 修复握手协议 |
| token budget 计算不精确 | 上下文管理不准确 | 校准 tiktoken-rs token 计数 |
| context compaction 阈值不准确 | 内存管理不可靠 | 实现精确的 85%/92%/95% 阈值触发 |
| 错误代码体系不完整 | 问题诊断困难 | 补充完整的 1xxx-9xxx 错误代码 |
| 凭证明文存储风险 | 安全漏洞 | 实现 credential ciphertext 加密 |
| 自动化测试覆盖率低 | 代码质量无保障 | 增加核心模块测试 |

### P2 中优先级问题 (下个迭代完成)

| 问题 | 影响 | 修复方案 |
|------|------|----------|
| LSP hover/code_actions | 代码辅助功能不完整 | 实现 LSP v1.1 扩展能力 |
| MCP OAuth | 远程 MCP 认证不完整 | 实现 OAuth 完整流程 |
| Browser/OAuth callback | 企业登录场景受限 | 实现 PKCE + state 校验 |
| token refresh/revoke | 凭证管理不完整 | 实现完整的 refresh 流程 |
| TUI @ 文件引用多选 | 用户体验不完整 | 完善模糊匹配 + 多选 |
| TUI diff 可展开 | 代码审查体验受限 | 增强 diff_view 组件 |
| Share 导出不完整 | session 分享受限 | 实现 JSON/Markdown 导出 |
| Share 服务端 | 公开发布场景受限 | 实现 self-hosted share server |
| Crash recovery | 异常恢复不完善 | 实现崩溃转储机制 |
| Session traces | 可观测性不足 | 完善 tracing 集成 |
| Windows 支持 | 平台覆盖不足 | 添加 Windows CI 测试 |

---

## 4. 技术债务清单

| 债务项 | 模块 | 复杂度 | 说明 |
|--------|------|--------|------|
| `opencode-core` 单一职责膨胀 | core | 高 | 62个文件，职责过多，建议拆分 domain 模块 |
| thiserror vs anyhow 混用 | error | 低 | 部分用 thiserror，部分用 anyhow，应统一 |
| 异步运行时混用风险 | 全局 | 中 | tokio 为主，部分模块未明确标注 |
| 硬编码超时值 | MCP/Server | 低 | 应抽取为配置项 |
| 魔法数字 (85%/92%/95%) | compaction | 低 | 应定义为常量 |
| 日志脱敏不完整 | 全局 | 中 | 部分 credential 可能泄露到日志 |
| 配置字段别名处理 | config | 低 | `defaultAgent` vs `default_agent` 等 alias 处理重复 |
| 错误处理不一致 | tools | 中 | 部分工具返回 Result，部分直接 panic |
| 文档注释缺失 | 部分模块 | 低 | 公共 API 文档不完整 |
| 依赖版本未锁定 | Cargo.toml | 中 | 使用 `version = "1.0"` 而非 `version = "=1.0.0"` |

---

## 5. 实现进度总结

### 按 PRD v1.0 目标达成率

| 功能域 | PRD 要求 | 已实现 | 达成率 | 关键差距 |
|--------|----------|--------|--------|----------|
| **核心能力** |
| 项目感知 AI 会话 | ✅ | ✅ | 100% | - |
| TUI 交互 | ✅ | ✅ | 95% | diff 可展开、token/cost 显示待完善 |
| Tool Calling | ✅ | ✅ | 90% | external_directory 权限控制缺失 |
| 权限系统 | ✅ | ✅ | 85% | 敏感文件默认 deny 未实现 |
| 文件操作 (read/edit/patch/bash) | ✅ | ✅ | 95% | move/delete 工具缺失 |
| Session 持久化 | ✅ | ✅ | 90% | fork lineage 不完整 |
| 模型抽象 | ✅ | ✅ | 100% | - |
| 配置系统 | ✅ | ✅ | 95% | schema validation 可加强 |
| **平台能力** |
| Server API | ✅ | ⚠️ | 70% | SDK 缺失，SSE 心跳缺失 |
| LSP 接入 | ✅ | ⚠️ | 75% | definition/references 缺失 |
| MCP 接入 | ✅ | ⚠️ | 65% | OAuth 缺失，远程连接不稳定 |
| 自定义 Commands | ✅ | ⚠️ | 80% | 命令模板变量扩展不完整 |
| Skills 系统 | ✅ | ✅ | 85% | 语义匹配能力有限 |
| 基础插件系统 | ✅ | ⚠️ | 60% | 事件总线缺失 |
| **扩展能力** |
| Share 能力 | ✅ | ⚠️ | 50% | 服务端未实现 |
| GitHub 集成 | ✅ | ❌ | 0% | 未开始 (PRD 规划 v2) |
| Web 前端 | ✅ | ❌ | 0% | 未开始 (PRD 规划 v1.5) |
| IDE 插件 | ✅ | ❌ | 0% | 未开始 (PRD 规划 v2) |
| **非功能需求** |
| 性能 (<500ms 启动) | ✅ | ⚠️ | 80% | 未做专项 benchmark |
| 可靠性 (崩溃恢复) | ✅ | ❌ | 0% | 崩溃转储未实现 |
| 安全 (凭证加密) | ✅ | ⚠️ | 60% | 明文存储风险 |
| 可观测性 | ✅ | ⚠️ | 40% | tracing 不完整，统计缺失 |
| **测试覆盖** | - | ⚠️ | 30% | 核心模块测试不足 |

### 总体进度

```
[===========================================--------------------------------] 65%
                           已完成                              未完成
```

### 建议优先级

**当前迭代 (P0-P1)**:
1. 实现 TypeScript/Rust SDK
2. 修复敏感文件默认 deny 权限
3. 实现 LSP definition/references
4. 完善 Plugin 事件总线
5. 实现 SSE heartbeat
6. 提高测试覆盖率

**下个迭代 (P1-P2)**:
1. 完善 MCP OAuth
2. 实现 Share 服务端
3. 实现崩溃恢复机制
4. 完善可观测性 (traces/cost 统计)
5. 完善 context compaction

**后续迭代 (P2)**:
1. GitHub Actions 集成
2. Web 前端
3. IDE 插件
4. Windows 支持
5. enterprise policy profile

---

## 6. 附录

### A. 项目结构

```
rust-opencode-port/
├── Cargo.toml (workspace)
├── crates/
│   ├── core/        (62 files) - 核心领域模型
│   ├── agent/       (11 files) - 7种 Agent 实现
│   ├── tools/       (37 files) - 工具注册与实现
│   ├── llm/        (37 files) - 17+ Provider 适配
│   ├── tui/         (多文件) - ratatui TUI
│   ├── server/      (20 files) - actix-web HTTP API
│   ├── storage/     (5 files) - SQLite 持久化
│   ├── permission/  (4 files) - 权限引擎
│   ├── auth/        (7 files) - 认证管理
│   ├── lsp/         (8 files) - LSP 桥接
│   ├── mcp/         (8 files) - MCP 桥接
│   ├── plugin/      (5 files) - WASM 插件
│   ├── git/         (2 files) - Git 操作
│   ├── control-plane/ (多文件) - 企业控制平面
│   └── cli/         (多文件) - CLI 命令
└── tests/           - 集成测试
```

### B. 已实现的工具列表

| 工具 | 文件 | 状态 |
|------|------|------|
| read | tools/src/read.rs | ✅ |
| write | tools/src/write.rs | ✅ |
| edit | tools/src/edit.rs | ✅ |
| patch/apply_patch | tools/src/apply_patch.rs | ✅ |
| glob | tools/src/glob.rs | ✅ |
| grep | tools/src/grep_tool.rs | ✅ |
| ls | tools/src/ls.rs | ✅ |
| bash | tools/src/bash.rs | ✅ |
| git_status | tools/src/git_tools.rs | ✅ |
| git_diff | tools/src/git_tools.rs | ✅ |
| git_log | tools/src/git_tools.rs | ✅ |
| git_show | tools/src/git_tools.rs | ✅ |
| todo_write | tools/src/todowrite.rs | ✅ |
| webfetch | tools/src/webfetch.rs | ✅ |
| websearch | tools/src/web_search.rs | ✅ |
| lsp_diagnostics | tools/src/lsp_tool.rs | ✅ |
| lsp_symbols | tools/src/lsp_tool.rs | ✅ |
| summarize_session | tools/src/session_tools.rs | ✅ |
| move | - | ❌ 缺失 |
| delete | - | ❌ 缺失 |

### C. 已实现的 Agent 列表

| Agent | 文件 | 状态 |
|-------|------|------|
| build | agent/src/build_agent.rs | ✅ |
| plan | agent/src/plan_agent.rs | ✅ |
| general | agent/src/general_agent.rs | ✅ |
| explore | agent/src/explore_agent.rs | ✅ |
| review | agent/src/review_agent.rs | ✅ |
| refactor | agent/src/refactor_agent.rs | ✅ |
| debug | agent/src/debug_agent.rs | ✅ |

### D. 已实现的 Provider 列表

| Provider | 文件 | 状态 |
|----------|------|------|
| OpenAI compatible | llm/src/openai.rs | ✅ |
| Anthropic | llm/src/anthropic.rs | ✅ |
| Gemini | llm/src/google.rs | ✅ |
| OpenRouter | llm/src/openrouter.rs | ✅ |
| Ollama (local) | llm/src/ollama.rs | ✅ |
| Azure | llm/src/azure.rs | ✅ |
| AWS Bedrock | llm/src/bedrock.rs | ✅ |
| Vertex | llm/src/vertex.rs | ✅ |
| Cohere | llm/src/cohere.rs | ✅ |
| Mistral | llm/src/mistral.rs | ✅ |
| HuggingFace | llm/src/huggingface.rs | ✅ |
| Perplexity | llm/src/perplexity.rs | ✅ |
| Groq | llm/src/groq.rs | ✅ |
| DeepInfra | llm/src/deepinfra.rs | ✅ |
| TogetherAI | llm/src/togetherai.rs | ✅ |
| Cerebras | llm/src/cerebras.rs | ✅ |
| xAI | llm/src/xai.rs | ✅ |
| Vercel | llm/src/vercel.rs | ✅ |
| SAP AI Core | llm/src/sap_aicore.rs | ✅ |
| GitHub Copilot | llm/src/copilot.rs | ✅ |

---

**报告生成时间**: 2026-04-08  
**分析工具**: 直接代码审查 + PRD 文档对比
