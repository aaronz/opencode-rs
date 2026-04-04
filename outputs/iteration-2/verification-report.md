# 迭代验证报告 (Iteration 3)

**项目**: OpenCode-RS (rust-opencode-port)  
**日期**: 2026-04-04  
**验证范围**: PRD 配置系统差距分析 vs 实际实现状态  
**基于**: gap-analysis.md + tasks_v3.md + spec_v3.md + plan_v3.md  
**验证方法**: 3个并行 explore agents + 直接代码审查 (config.rs 1704行, directory_scanner.rs 682行, schema.rs 90行, merge.rs 74行)

---

## P0问题状态

| 问题 | 状态 | 备注 |
|------|------|------|
| P0-1: 缺少 `OPENCODE_TUI_CONFIG` 环境变量 | ❌ **未实现** | 全仓库 grep 零匹配。`load_multi()` 处理 OPENCODE_REMOTE_CONFIG/OPENCODE_CONFIG_CONTENT/OPENCODE_CONFIG/OPENCODE_CONFIG_DIR，但完全没有 OPENCODE_TUI_CONFIG |
| P0-2: TUI 配置未分离为独立 tui.json 文件 | ❌ **未实现** | `TuiConfig` 结构体 (config.rs:828-836) 仍内嵌在主 `Config` 中 (config.rs:148 `pub tui: Option<TuiConfig>`)。无 tui.json 文件加载逻辑。全仓库 grep "tui.json" 零匹配 |

### 详细验证

#### P0-1: OPENCODE_TUI_CONFIG

**验证证据**:
- 全仓库搜索 `OPENCODE_TUI_CONFIG` → 0 个 Rust 源码匹配
- `load_multi()` (config.rs:1035-1104) 仅处理 5 个配置来源，缺少 TUI 配置加载
- `config_path()` (config.rs:1012-1032) 仅处理 `OPENCODE_CONFIG_DIR`，无 TUI 路径逻辑
- PRD §10.3 明确要求支持，tasks_v3.md TASK-0.1 状态为 pending

**结论**: 完全未实现。PRD 验收标准 §10.3 不通过。

#### P0-2: TUI 配置分离

**验证证据**:
- `TuiConfig` 结构体存在但仅包含 scroll_speed/scroll_acceleration/diff_style (config.rs:828-836)
- 缺少 `theme` 和 `keybinds` 字段 (这两个仍在主 Config 中: config.rs:140, 144)
- 无独立 tui.json 文件加载函数
- 无 `$schema: "https://opencode.ai/tui.json"` 声明
- 无废弃警告机制 (theme/keybinds 无 `#[deprecated]` 标记)
- PRD §10.3 明确要求 TUI 配置与 runtime 分离

**结论**: 完全未实现。PRD 验收标准 §10.3 不通过。

---

## P1问题状态

| 问题 | 状态 | 备注 |
|------|------|------|
| P1-1: 缺少 `modes/` 目录扫描 | ✅ **已实现** | `directory_scanner.rs:265-379` 完整实现 `scan_modes()`，支持 YAML frontmatter 解析。`OpencodeDirectoryScan` 包含 `modes` 字段。`load_opencode_directory()` 调用 `scan_all()` 包含 modes |
| P1-2: 配置路径使用 `opencode-rs` 而非 `opencode` | ❌ **未修复** | `config_path()` (config.rs:1022) 使用 `directories::ProjectDirs::from("com", "opencode", "rs")`，fallback 为 `"~/.config/opencode-rs/config.toml"` (config.rs:1031) |
| P1-3: `{file:path}` 不支持 `~` 路径展开 | ❌ **未修复** | `substitute_variables()` (config.rs:992-1007) 直接将 file_path 传给 `std::fs::read_to_string()`，无 `~` 展开逻辑 |
| P1-4: `{file:path}` 不支持相对于配置文件目录 | ❌ **未修复** | 同上，无配置文件目录上下文传递。相对路径仅相对于 CWD 生效 |
| P1-5: `load_opencode_directory()` 未被 `load_multi()` 调用 | ❌ **未修复** | `load_multi()` (config.rs:1035-1104) 加载 .opencode/config.json 文件但**不调用** `load_opencode_directory()` 扫描子目录。全仓库搜索 `load_opencode_directory` 调用点仅发现定义和 re-export，无实际调用 |

### 详细验证

#### P1-1: modes/ 目录扫描 (已实现 ✅)

**验证证据**:
- `directory_scanner.rs:265` — `pub fn scan_modes(&self, base_path: &Path) -> Vec<ModeInfo>`
- `directory_scanner.rs:52-60` — `ModeInfo` 结构体完整 (name, path, description, system_prompt, default_agent, permission_overrides)
- `directory_scanner.rs:271-272` — YAML frontmatter 解析 regex
- `directory_scanner.rs:382-392` — `scan_all()` 包含 `modes: self.scan_modes(base_path)`
- `directory_scanner.rs:419` — `load_opencode_directory()` 合并 modes 到结果
- 测试覆盖: `test_scan_all` (line 610) 和 `test_scan_modes_with_frontmatter` (line 647) 均通过
- **注意**: modes 扫描功能已实现，但因 P1-5 问题，`load_opencode_directory()` 未被 `load_multi()` 调用，所以实际不会被自动加载

#### P1-2: 配置路径命名 (未修复 ❌)

**验证证据**:
- `config.rs:1022` — `directories::ProjectDirs::from("com", "opencode", "rs")`
- `config.rs:1031` — fallback: `PathBuf::from("~/.config/opencode-rs/config.toml")`
- 未使用 `directories::ProjectDirs::from("ai", "opencode", "opencode")` (tasks_v3.md 要求)
- 默认格式仍为 TOML (非 JSONC)
- `OPENCODE_CONFIG_FORMAT` 仍为临时方案

#### P1-3 & P1-4: {file:path} 路径展开 (未修复 ❌)

**验证证据**:
- `config.rs:992-1007` — `{file:path}` 处理:
  ```rust
  let file_path = &result[start + 6..start + end];
  let replacement = std::fs::read_to_string(file_path)
      .unwrap_or_else(|_| format!("{{file:{}}}", file_path));
  ```
- 无 `~` 展开 (无 `shellexpand` 或 `dirs::home_dir()` 调用)
- 无配置文件目录上下文传递
- 未设置变量时保留原字符串 (非 PRD 要求的空字符串 `""`)

#### P1-5: .opencode/ 目录扫描集成 (未修复 ❌)

**验证证据**:
- `load_multi()` (config.rs:1071-1094) 检查 `.opencode/config.json` 文件
- 但**不调用** `load_opencode_directory()` 扫描 agents/commands/skills/tools/themes/modes 子目录
- `load_opencode_directory()` 函数存在 (directory_scanner.rs:403-438) 但未被任何配置加载路径调用
- 全仓库 grep `load_opencode_directory` 仅 2 处: 定义 + re-export

---

## P2问题状态

| 问题 | 状态 | 备注 |
|------|------|------|
| P2-1: 变量替换覆盖不完整 | ⚠️ **部分** | `load()` 中执行变量替换 (config.rs:940-941)，`load_multi()` 中仅 OPENCODE_CONFIG_CONTENT 执行替换 (config.rs:1049)。global/project config 通过 `load()` 加载时执行替换，但 remote config (config.rs:1041) 未经过变量替换 |
| P2-2: theme/keybinds 未迁移出主配置 | ❌ **未修复** | `theme` (config.rs:144) 和 `keybinds` (config.rs:140) 仍在主 Config 中，无 `#[deprecated]` 标记，无迁移逻辑 |
| P2-3: AgentMapConfig 固定键设计 | ⚠️ **部分可接受** | 使用固定键 (plan/build/general/explore/title/summary/compaction) + `#[serde(flatten)] custom: Option<HashMap<String, AgentConfig>>`。自定义 agent 名称可通过 custom 字段支持，但非完全动态 HashMap |
| P2-4: JSON Schema 远程验证空壳 | ❌ **未修复** | `schema.rs:5-40` 仅检查 port 和 temperature 两个字段。`get_official_schema_url()` 返回 URL 但从未被调用。无远程拉取、缓存、`jsonschema` crate 验证 |
| P2-5: 测试覆盖不足 | ⚠️ **有改善但仍不足** | 核心配置模块有 10 个单元测试 + 17 个配置相关测试 + 11 个集成测试。但缺少: load_multi 集成测试、变量替换完整覆盖、配置合并优先级测试 |

### 详细验证

#### P2-1: 变量替换覆盖

**验证证据**:
- `load()` (config.rs:940-941): `let content = Self::substitute_variables(&content);` ✅
- `load_multi()`:
  - Priority 2 (env-content, config.rs:1049): `let content = Self::substitute_variables(&content);` ✅
  - Priority 3 (env-path, config.rs:1059): `Self::load(&path)` → 内部调用 substitute_variables ✅
  - Priority 4 (global, config.rs:1067): `Self::load(&global_path)` → 内部调用 substitute_variables ✅
  - Priority 5 (project, config.rs:1079/1088): `Self::load(&project_config)` → 内部调用 substitute_variables ✅
  - Priority 1 (remote, config.rs:1041): `Self::parse_config_content(&content, "json")` → **未经过 substitute_variables** ❌
- 未设置变量行为: `test_substitute_variables_missing_env` (config.rs:1652-1657) 确认保留原字符串，非 PRD 要求的空字符串

#### P2-3: AgentMapConfig

**验证证据** (config.rs:260-285):
```rust
pub struct AgentMapConfig {
    pub plan: Option<AgentConfig>,
    pub build: Option<AgentConfig>,
    pub general: Option<AgentConfig>,
    pub explore: Option<AgentConfig>,
    pub title: Option<AgentConfig>,
    pub summary: Option<AgentConfig>,
    pub compaction: Option<AgentConfig>,
    #[serde(flatten)]
    pub custom: Option<HashMap<String, AgentConfig>>,
}
```
- 预设 7 个固定键 + flatten custom HashMap
- 自定义 agent 名称可通过 JSON 顶层键映射到 custom 字段
- 非 PRD 示例中的完全动态 `HashMap<String, AgentConfig>`
- 功能上可支持任意 agent 名，但序列化/反序列化路径不同 (固定键 vs flatten)

#### P2-5: 测试覆盖

**验证证据**:
- **core/src/config.rs 单元测试** (10 个):
  - test_default_config, test_provider_enabled
  - test_substitute_variables_env/missing_env/multiple
  - test_scroll_acceleration (4 个变体)
- **core/tests/** (2 个文件):
  - session_test.rs — 含 test_config_default
  - filesystem_test.rs — 无配置相关测试
- **集成测试** (11 个配置相关):
  - cli/tests/e2e_settings_workflows.rs (5 个)
  - cli/tests/e2e_model_workflows.rs (5 个)
  - ratatui-testing/tests/integration.rs (1 个)
- **缺失**:
  - ❌ `load_multi()` 完整优先级集成测试
  - ❌ `{file:path}` 变量替换测试
  - ❌ 配置合并 (merge_configs) 集成测试
  - ❌ JSON Schema 验证测试
  - ❌ TUI 配置加载测试
  - ❌ .opencode/ 目录扫描集成测试

---

## 技术债务状态 (对比 gap-analysis.md)

| 债务项 | 状态 | 备注 |
|--------|------|------|
| TOML vs JSON 格式分裂 | ❌ 仍存在 | `config_path()` 默认返回 `.toml` (config.rs:1017, 1027) |
| 硬编码路径 | ❌ 仍存在 | `"~/.config/opencode-rs/config.toml"` 硬编码 (config.rs:1031) |
| 变量替换实现粗糙 | ❌ 仍存在 | `while let Some(start) = result.find(...)` 字符串替换 (config.rs:972-1009) |
| merge_configs 通过 JSON 中转 | ❌ 仍存在 | merge.rs:22-29 序列化→deep_merge→反序列化 |
| fetch_remote_config 同步包装异步 | ❌ 仍存在 | config.rs:1107-1109 `tokio::runtime::Runtime::new().unwrap().block_on()` |
| TimeoutConfig 枚举命名 | 未验证 | 需进一步检查 |
| PermissionConfig 重复字段 | 未验证 | 需进一步检查 |
| Schema 验证空壳 | ❌ 仍存在 | schema.rs:5-40 仅检查 2 个字段 |
| DirectoryScanner 未使用 glob | ⚠️ 部分 | 手动 `read_dir` 遍历，但 modes 扫描已实现 |
| 测试文件命名 | ⚠️ 部分 | core/tests/ 有 2 个文件，cli/tests/ 有 24 个 e2e 测试 |

---

## Constitution合规性

| 条款 | 覆盖任务 | 当前状态 |
|------|----------|----------|
| C-017 (TUI/Runtime 配置分离) | Task 0.1, 0.2, 2.2 | ❌ 未实现 |
| C-018 (配置路径与目录命名) | Task 1.3 | ❌ 仍使用 opencode-rs |
| C-019 (配置变量替换语义) | Task 1.4, 1.5, 2.1 | ❌ ~ 和相对路径未实现 |
| C-013 (目录结构扩展 modes/) | Task 1.1, 1.2 | ✅ modes/ 扫描已实现，但未集成到 load_multi |

---

## PRD完整度

### 配置系统实现概况

| 模块 | 状态 | 完成度 |
|------|------|--------|
| JSON/JSONC 解析 | ✅ | 100% |
| 配置合并 (deep_merge) | ✅ | 100% |
| Remote Config | ⚠️ | ~80% (缺变量替换) |
| Global Config | ⚠️ | ~80% (路径命名不一致) |
| OPENCODE_CONFIG | ✅ | 100% |
| OPENCODE_CONFIG_CONTENT | ✅ | 100% |
| OPENCODE_TUI_CONFIG | ❌ | 0% |
| Project Config | ✅ | 100% |
| .opencode/ 目录扫描 | ⚠️ | ~60% (扫描实现完整，但未集成到加载流程) |
| {env:VAR} 变量替换 | ✅ | 100% (但未设置变量行为不符合 PRD) |
| {file:path} 文件引用 | ⚠️ | ~40% (不支持 ~ 和相对路径) |
| TUI 配置分离 | ❌ | 0% |
| Schema 验证 | ⚠️ | ~20% (空壳实现) |
| Agent 配置 | ✅ | ~90% (AgentMapConfig 非完全动态) |
| Command 配置 | ✅ | 100% |
| Permission 配置 | ✅ | 100% |
| Provider 配置 | ✅ | 100% |
| MCP 配置 | ⚠️ | ~80% |

**整体配置系统完整度: ~70%** (较 iteration-3 gap-analysis 估计的 75-80% 略低，因发现更多未实现项)

### PRD §10 验收标准对照

| 验收项 | PRD § | 状态 | 备注 |
|--------|-------|------|------|
| JSON/JSONC 格式支持 | 10.1 | ✅ | jsonc.rs 完整实现 |
| 配置合并逻辑 | 10.1 | ✅ | merge.rs deep_merge 实现 |
| 6 个配置位置按优先级加载 | 10.1 | ❌ | 缺少 OPENCODE_TUI_CONFIG，.opencode 目录扫描未集成。实际仅 4/6 完整 |
| `{env:VAR}` 变量替换 | 10.2 | ⚠️ | 实现正确但未设置变量保留原字符串 (PRD 要求空字符串) |
| `{file:path}` 正确读取文件 | 10.2 | ❌ | 不支持 `~` 和相对路径 |
| 未设置变量替换为空字符串 | 10.2 | ❌ | 当前保留原 `{env:VAR}` 字符串 |
| TUI 配置与 runtime 分离 | 10.3 | ❌ | 未实现独立 tui.json |
| `OPENCODE_TUI_CONFIG` 自定义路径 | 10.3 | ❌ | 完全缺失 |
| Provider timeout/chunkTimeout/setCacheKey | 10.4 | ✅ | ProviderOptions 完整 |
| Amazon Bedrock 配置 | 10.4 | ✅ | awsRegion/awsProfile/awsEndpoint |
| disabled_providers 优先级 | 10.4 | ✅ | is_provider_enabled() 逻辑正确 |
| 自定义 agent 配置 | 10.5 | ✅ | AgentConfig 完整 |
| default_agent 设置 | 10.5 | ✅ | 字段存在且被 env 覆盖 |
| permission 配置 | 10.6 | ✅ | PermissionConfig 完整 |
| API Key 文件引用 | 10.6 | ❌ | 依赖 {file:path}，但该功能不完整 |

**PRD 验收通过率: 8/15 (53%)**

---

## 遗留问题

### 高优先级 (P0 — 阻塞发布)

1. **OPENCODE_TUI_CONFIG 环境变量完全缺失** — PRD §10.3 明确要求，tasks_v3.md TASK-0.1 pending
2. **TUI 配置未分离为独立 tui.json** — PRD 核心架构要求，tasks_v3.md TASK-0.2 pending

### 中优先级 (P1 — 重要功能缺失)

3. **`.opencode/` 目录扫描未集成到配置加载** — modes/agents/commands/skills 等目录内容不会被自动加载
4. **配置路径命名不一致** — `opencode-rs` vs `opencode`，与官方生态不兼容
5. **`{file:path}` 不支持 `~` 展开** — 用户无法使用 `{file:~/.secrets/api-key}`
6. **`{file:path}` 不支持相对路径** — 无法使用 `{file:./instructions.md}`

### 低优先级 (P2 — 改进/技术债务)

7. 变量替换未设置时保留原字符串 (应为空字符串)
8. theme/keybinds 未标记废弃且未迁移
9. JSON Schema 远程验证空壳
10. AgentMapConfig 非完全动态 HashMap
11. merge_configs 通过 JSON 中转
12. fetch_remote_config 同步包装异步
13. 测试覆盖不足 (缺少 load_multi 集成测试、{file:path} 测试)

---

## 下一步建议

### 立即行动 (本周 — Phase 0)

1. **实现 OPENCODE_TUI_CONFIG 环境变量** (TASK-0.1)
   - 读取环境变量 → 路径展开 → 加载 TUI 配置 → 降级到默认
   - 预计工期: 2 天

2. **TUI 配置分离为独立 tui.json** (TASK-0.2)
   - 创建独立 TuiConfig 结构体 (含 theme/keybinds)
   - 实现加载优先级: OPENCODE_TUI_CONFIG → ~/.config/opencode/tui.json → 项目 tui.json → 内建默认
   - 标记主配置中 theme/keybinds 为 deprecated
   - 预计工期: 3 天

### 后续行动 (下周 — Phase 1)

3. **集成 .opencode/ 目录扫描到 load_multi()** (TASK-1.1)
   - 在加载 Project Config 后调用 load_opencode_directory()
   - 扫描结果合并到最终配置

4. **{file:path} 增强** (TASK-1.4 + TASK-1.5)
   - 添加 ~ 路径展开
   - 添加相对于配置文件目录的支持
   - 修正未设置变量为空字符串

5. **配置路径命名统一** (TASK-1.3)
   - 使用 directories crate 统一管理
   - 从 opencode-rs 迁移到 opencode
   - 默认格式改为 JSONC

### 技术债务清理 (Sprint 内 — Phase 2)

6. 变量替换覆盖完整性 (TASK-2.1)
7. theme/keybinds 迁移 (TASK-2.2)
8. AgentMapConfig 动态化 (TASK-2.3)
9. JSON Schema 远程验证 (TASK-2.4)
10. 技术债务清理 (TASK-2.5)

---

## 总结

**实现完成度: ~70%**

| 维度 | 状态 |
|------|------|
| P0 问题 | ❌ 2/2 未实现 |
| P1 问题 | ✅ 1/5 已实现 (modes/ 扫描), ❌ 4/5 未修复 |
| P2 问题 | ⚠️ 2/5 部分, ❌ 3/5 未修复 |
| PRD 验收通过率 | 53% (8/15) |
| Constitution 合规 | 1/4 条款合规 |
| 技术债务 | 0/10 已清理 |

**关键发现**:
- modes/ 目录扫描功能已完整实现 (directory_scanner.rs)，但因未集成到 load_multi() 而实际不可用
- 配置系统基础架构 (JSONC 解析、deep_merge、环境变量替换) 已实现且可用
- P0 问题 (TUI 配置分离 + OPENCODE_TUI_CONFIG) 完全未开始，阻塞 PRD §10.3 验收
- 测试覆盖有基础但严重不足，特别是配置加载集成测试

**tasks_v3.md 任务状态**: 全部 12 个任务均为 pending，无一个开始实施。

---

*Generated: 2026-04-04*  
*验证方法: 3个并行 explore agents + 直接代码审查 (4个核心文件, 2550+ 行)*
