我将分析当前实现与PRD的差距。让我先收集上下文信息。
Let me check what actually exists in the current working directory to understand the real implementation state:
基于收集的上下文，现在我可以生成差距分析报告。

---

# 差距分析报告

**分析日期**: 2026-04-05  
**PRD基线**: `docs/PRD.md`, `docs/PRD-tui.md`, `docs/PRD-providers.md`, `PRD-OpenCode-Configuration.md`  
**实现基线**: `rust-opencode-port/` (15 crates) + `.opencode/` (Node.js plugin)  
**上次分析**: iteration-6 (2026-04-04)

---

## 一、当前实现结构

### 1.1 Rust 实现 (rust-opencode-port/)
```
crates/
├── server/     # HTTP/WS/SSE API, mDNS
├── core/       # 配置、权限、session管理
├── llm/        # 18+ providers, 认证分层
├── tools/      # 工具集 (read/write/edit/bash/grep等)
├── tui/        # 终端UI (dialogs, components)
├── agent/      # agent逻辑
├── lsp/        # LSP诊断/symbols
├── storage/    # SQLite数据库
├── permission/ # 权限队列/审计
├── auth/       # 认证层
├── control-plane/ # ACP流
├── plugin/     # 插件发现/加载/WASM
├── git/        # Git集成
├── mcp/        # MCP协议
└── cli/        # 命令行入口
```

### 1.2 .opencode/ 结构
```
.opencode/
├── package.json    # @opencode-ai/plugin 依赖
└── commands/      # 空目录 (待实现)
```

---

## 二、差距列表

| # | 差距项 | 严重程度 | 模块 | 修复建议 |
|---|--------|----------|------|----------|
| 1 | **OAuth/Device Code 浏览器登录未实现** | P0 | llm/auth | PRD要求支持GitHub Copilot/GitLab Duo的OAuth Browser Flow和Device Code Flow，需实现完整的OAuth流程 |
| 2 | **Provider认证分层架构未完成** | P0 | llm/auth | `auth_layered/` 模块存在但需验证各层(layer1-4)是否完整实现，PRD要求Credential Source→Auth Mechanism→Provider Transport→Runtime Access Control |
| 3 | **Remote Config自动发现未实现** | P1 | core/config | PRD-Configuration优先级1为Remote Config (.well-known/opencode)，当前需环境变量触发，非自动发现 |
| 4 | **disabled_providers优先级未实现** | P1 | core/config | PRD明确要求disabled_providers优先级高于enabled_providers |
| 5 | **TUI三栏布局Inspector面板不完整** | P1 | tui | PRD-tui要求Sidebar/Timeline/Inspector三栏，Inspector需6个tab，当前仅有right_panel.rs |
| 6 | **MCP OAuth独立token存储缺失** | P1 | mcp/auth | PRD要求MCP OAuth存储在独立mcp-auth.json |
| 7 | **云厂商原生认证不完整** | P1 | llm | Bedrock/AWS credential chain优先级未完全实现，Vertex AI的GOOGLE_APPLICATION_CREDENTIALS支持不完整 |
| 8 | **Plugin WASM运行时缺失** | P1 | plugin | 已有discovery/loader/registry，但WASM执行引擎需验证 |
| 9 | **凭证加密存储未实现** | P1 | auth | PRD要求凭证本地加密存储，需接入系统密钥链 |
| 10 | **Share服务层未实现** | P2 | core/share | 仅本地导出，PRD要求self-hosted share server + 短链 + 访问令牌 |
| 11 | **OpenAPI文档自动生成缺失** | P2 | server | PRD v1验收标准要求完整OpenAPI文档 |
| 12 | **LSP definition/references/hover未实现** | P2 | lsp | 仅实现diagnostics+symbols |
| 13 | **session_load/session_save工具缺失** | P2 | tools | 工具需实现 |
| 14 | **Formatters配置未完全接入** | P2 | core/formatter | 配置存在但需验证是否接入agent执行循环 |
| 15 | **Compaction自动触发阈值未实现** | P2 | core/compaction | PRD要求85%预警/92%触发/95%强制新session |
| 16 | **Server HTTP Basic Auth未实现** | P2 | server | PRD要求OPENCODE_SERVER_PASSWORD/USERNAME支持 |
| 17 | **观测性不完整** | P2 | core | 需完善tracing/crash recovery/token cost统计 |
| 18 | **自定义Commands目录空置** | P2 | .opencode | commands/目录存在但为空 |
| 19 | **HuggingFace/AI21 Provider状态存疑** | P2 | llm | 文件存在但可能为stub |

---

## 三、P0/P1/P2问题分类

### 🔴 P0 - 阻塞发布 (2项)
| # | 问题 | 原因 |
|---|------|------|
| 1 | OAuth/Device Code缺失 | 无法支持GitHub Copilot等关键provider登录 |
| 2 | Provider认证分层未完成 | 架构缺陷，影响75+ providers支持 |

### 🟡 P1 - 重要功能缺失 (7项)
| # | 问题 | 影响 |
|---|------|------|
| 3 | Remote Config自动发现 | 企业部署无法统一管理 |
| 4 | disabled_providers优先级 | 配置冲突行为不可预期 |
| 5 | TUI Inspector面板 | UX不完整 |
| 6 | MCP OAuth独立存储 | 安全架构不符 |
| 7 | 云厂商原生认证 | AWS/Vertex用户无法使用 |
| 8 | Plugin WASM运行时 | 插件能力受限 |
| 9 | 凭证加密存储 | 安全风险 |

### 🟢 P2 - 增强功能 (10项)
| # | 问题 | 影响 |
|---|------|------|
| 10-19 | 见上表 | 功能完整性/可观测性 |

---

## 四、技术债务清单

| 债务项 | 位置 | 描述 | 风险 |
|--------|------|------|------|
| 1 | `crates/llm/src/auth_layered/` | 认证分层模块存在但完成度存疑 | 高 |
| 2 | `crates/core/src/config.rs` | 配置结构体复杂(1000+行)，merge逻辑耦合 | 中 |
| 3 | `crates/auth/` | auth crate存在但可能未被广泛引用 | 高 |
| 4 | 测试覆盖 | 全项目测试文件数量不足 | 高 |
| 5 | `crates/tools/src/grep_tool_test.rs` | 测试文件放在src/而非tests/ | 低 |
| 6 | `.opencode/commands/` | 空目录待实现 | 中 |
| 7 | `crates/llm/src/*.rs` | 18个provider可能存在重复代码 | 中 |

---

## 五、与上次分析对比

| 指标 | 上次(iteration-6) | 本次 | 变化 |
|------|------------------|------|------|
| P0 | 2 | 2 | → |
| P1 | 9 | 9 | → |
| P2 | 11 | 10 | ↓ 减少1项 |
| 完成度 | ~75-80% | ~75-80% | → |

**结论**: 差距分析基本稳定，主要问题集中在认证系统和配置系统。P2问题部分解决但整体进度不明显。

---

## 六、建议优先级

1. **立即处理**: P0认证问题 (OAuth + 分层架构)
2. **短期目标**: P1配置系统完善 (Remote Config, disabled_providers, 凭证加密)
3. **中期目标**: TUI Inspector, Plugin WASM, Share服务
4. **长期目标**: P2增强功能 + 技术债务清理
