I detect **investigation/analysis** intent — comparing implementation against PRD requirements. My approach: parallel exploration of codebase structure + PRD reading → synthesize gap analysis.
Now let me explore the implementation crates in detail:
Now let me explore the config implementation in detail and the Cargo.toml for the full project structure:
Now let me look at some more key implementation details and existing gap analysis docs:
Now let me check the server routes and the CLI commands to understand API completeness:
Now let me check the TUI config implementation more deeply and look at the `load_opencode_directory` function usage, the `.opencode` directory handling, and the `OPENCODE_TUI_CONFIG` env var support:
Now let me do a few more targeted checks before finalizing the analysis:
Now I have comprehensive context. Let me synthesize the gap analysis.

---

# 差距分析报告：PRD vs Rust 实现

## 执行摘要

| 指标 | 值 |
|------|-----|
| PRD 版本 | 1.0 (PRD-OpenCode-Configuration.md) |
| 实现完整度 | ~75-80% |
| 核心配置系统 | 已实现大部分 |
| 关键缺失 | TUI 配置分离、OPENCODE_TUI_CONFIG、modes/ 目录扫描 |

---

## 差距列表

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **缺少 `OPENCODE_TUI_CONFIG` 环境变量支持** | P0 | core/config | PRD §4.3 明确要求，`config.rs` 中完全缺失该 env var 处理 |
| **TUI 配置未分离为独立文件 (tui.json)** | P0 | core/config, tui | PRD §3.3 要求 TUI 使用独立 `$schema: "https://opencode.ai/tui.json"` 和独立文件，当前 `TuiConfig` 内嵌在主配置中 |
| **缺少 `modes/` 目录扫描** | P1 | core/config/directory_scanner | PRD §4.2 要求 `.opencode/modes/` 目录，`DirectoryScanner` 只扫描 agents/commands/plugins/skills/tools/themes/，缺少 modes/ |
| **配置路径使用 `opencode-rs` 而非 `opencode`** | P1 | core/config | `config_path()` 返回 `~/.config/opencode-rs/` 而非 PRD 要求的 `~/.config/opencode/` |
| **`{file:path}` 不支持 `~` 路径展开** | P1 | core/config | PRD §6.2 明确说明文件路径支持 `~` 开头，`substitute_variables` 中直接 `std::fs::read_to_string(file_path)` 未展开 |
| **`{file:path}` 不支持相对于配置文件目录** | P1 | core/config | PRD §6.2 要求支持相对于配置文件目录的路径，当前只支持绝对路径 |
| **变量替换在 JSON 序列化层面而非原始字符串** | P2 | core/config | `substitute_variables` 在 `load()` 中先替换再解析，但 `load_multi()` 中只有部分路径做了变量替换 |
| **`theme` 和 `keybinds` 仍在主配置中（未迁移到 tui.json）** | P2 | core/config | PRD §9.1 声明旧配置已废弃应移至 tui.json，但当前实现仍在 `Config` 结构体中保留 |
| **缺少 `mcp` 配置中 `timeout` 的全局默认值支持** | P2 | core/config | PRD §5.11 MCP 配置为注释占位，`McpConfig` 有 timeout 但无全局默认 |
| **`AgentMapConfig` 使用固定键而非完全动态 HashMap** | P2 | core/config | PRD §5.4 示例使用任意 agent 名称（如 `code-reviewer`），当前 `AgentMapConfig` 预设了 plan/build/general/explore 等固定键 + custom flatten，可能导致非标准 agent 名解析问题 |
| **缺少配置文件的 JSON Schema 远程验证** | P2 | core/config/schema | `validate_json_schema` 是空壳实现（只检查 port 和 temperature），未实际拉取 `https://opencode.ai/config.json` 验证 |
| **`load_opencode_directory()` 未被 `load_multi()` 调用** | P1 | core/config | `.opencode/` 目录扫描功能存在但独立于配置加载流程，`load_multi()` 中只检查 `.opencode/config.json` 文件，不扫描子目录 |
| **测试覆盖不足** | P2 | tests | `core/tests/` 仅 2 个测试文件，`tests/` 根目录为空，缺少配置加载集成测试 |

---

## P0/P1/P2 问题分类

### 🔴 P0 — 阻塞发布

| # | 问题 | 影响 |
|---|------|------|
| 1 | **缺少 `OPENCODE_TUI_CONFIG` 环境变量** | 无法自定义 TUI 配置文件路径，PRD 验收标准 §10.3 明确要求 |
| 2 | **TUI 配置未分离为独立 tui.json 文件** | PRD 核心设计要求，TUI 配置与 runtime 配置应分离 |

### 🟡 P1 — 重要功能缺失

| # | 问题 | 影响 |
|---|------|------|
| 3 | **缺少 `modes/` 目录扫描** | `.opencode/modes/` 目录不被识别 |
| 4 | **配置路径命名不一致 (`opencode-rs` vs `opencode`)** | 与官方 OpenCode 生态不兼容 |
| 5 | **`{file:path}` 不支持 `~` 展开** | 用户无法使用 `{file:~/.secrets/api-key}` 语法 |
| 6 | **`{file:path}` 不支持相对路径** | 无法使用 `{file:./instructions.md}` 引用相对文件 |
| 7 | **`.opencode/` 目录扫描未集成到配置加载** | agents/commands/skills 等目录内容不会被自动加载到配置中 |

### 🟢 P2 — 改进/技术债务

| # | 问题 | 影响 |
|---|------|------|
| 8 | `theme`/`keybinds` 未迁移出主配置 | 与 PRD 废弃声明不一致 |
| 9 | JSON Schema 远程验证未实现 | 配置校验不完整 |
| 10 | `AgentMapConfig` 固定键设计 | 自定义 agent 名称受限 |
| 11 | 测试覆盖不足 | 配置系统无集成测试保障 |
| 12 | 变量替换覆盖不完整 | 部分配置加载路径可能遗漏变量替换 |

---

## 技术债务清单

| 债务项 | 位置 | 描述 |
|--------|------|------|
| **TOML vs JSON 格式分裂** | `config.rs:1012-1031` | `config_path()` 默认返回 `.toml`，但 PRD 要求 JSON/JSONC。`OPENCODE_CONFIG_FORMAT` 是临时方案 |
| **硬编码路径** | `config.rs:1031` | `"~/.config/opencode-rs/config.toml"` 硬编码，应使用 `directories` crate 统一 |
| **变量替换实现粗糙** | `config.rs:972-1009` | 使用 `while let Some(start) = result.find(...)` 字符串替换，对嵌套/复杂情况可能出错 |
| **merge_configs 通过 JSON 中转** | `merge.rs:22-29` | 先序列化到 JSON 再 deep_merge 再反序列化，丢失类型信息且性能差 |
| **`fetch_remote_config` 同步包装异步** | `config.rs:1107-1109` | 在同步函数中创建 `tokio::runtime::Runtime`，可能导致线程阻塞 |
| **`TimeoutConfig` 枚举命名** | `config.rs:469-474` | `Disabled(bool)` 变体语义不清，应为 `NoTimeout` |
| **`PermissionConfig` 大量重复字段** | `config.rs:628-697` | 每个权限字段单独定义，应考虑宏生成或统一结构 |
| **Schema 验证空壳** | `schema.rs:5-40` | `validate_json_schema` 只检查 2 个字段，未使用实际 JSON Schema |
| **`DirectoryScanner` 未使用 glob** | `directory_scanner.rs` | 手动 `read_dir` 遍历，不支持 glob 模式（如 PRD 要求的 `*.md` 批量匹配） |
| **测试文件命名不规范** | `core/tests/` | `filesystem_test.rs` 使用 `_test` 后缀而非 Rust 惯用的 `_test` 或集成测试目录结构 |

---

## 验收标准对照

| 验收项 | PRD § | 状态 | 备注 |
|--------|-------|------|------|
| JSON/JSONC 格式支持 | 10.1 | ✅ | `jsonc.rs` 完整实现 |
| 配置合并逻辑 | 10.1 | ✅ | `merge.rs` deep_merge 实现 |
| 6 个配置位置按优先级加载 | 10.1 | ⚠️ | 缺少 `OPENCODE_TUI_CONFIG`，实际 5/6 |
| `{env:VAR}` 变量替换 | 10.2 | ✅ | 实现正确 |
| `{file:path}` 文件引用 | 10.2 | ⚠️ | 不支持 `~` 和相对路径 |
| 未设置变量替换为空字符串 | 10.2 | ❌ | 当前保留原 `{env:VAR}` 字符串而非空串 |
| TUI 配置与 runtime 分离 | 10.3 | ❌ | 未实现独立 tui.json |
| `OPENCODE_TUI_CONFIG` 自定义路径 | 10.3 | ❌ | 完全缺失 |
| Provider timeout/chunkTimeout/setCacheKey | 10.4 | ✅ | `ProviderOptions` 完整 |
| Amazon Bedrock 配置 | 10.4 | ✅ | awsRegion/awsProfile/awsEndpoint |
| disabled_providers 优先级 | 10.4 | ✅ | `is_provider_enabled()` 逻辑正确 |
| 自定义 agent 配置 | 10.5 | ✅ | `AgentConfig` 完整 |
| default_agent 设置 | 10.5 | ✅ | 字段存在且被 env 覆盖 |
| 命令模板变量替换 | 10.5 | ⚠️ | 命令模板中的变量替换未明确实现 |
| permission 配置 | 10.6 | ✅ | `PermissionConfig` 完整 |
| API Key 文件引用 | 10.6 | ⚠️ | 依赖 `{file:path}`，但该功能不完整 |
