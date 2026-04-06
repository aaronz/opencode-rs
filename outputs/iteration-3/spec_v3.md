# OpenCode-RS 规格文档 v3

**版本**: 3.0
**日期**: 2026-04-04
**基于**: PRD-OpenCode-Configuration.md (v1.0) + Gap Analysis (iteration-3)
**状态**: 已完成

---

## 1. 文档概述

### 1.1 背景

本规格文档基于 PRD-OpenCode-Configuration.md 产品需求文档和 iteration-3 差距分析报告生成，专注于**配置系统**的完整需求定义。与 v2 相比，v3 新增了配置系统专项差距分析中发现的所有需求项。

### 1.2 目标

- 为每个差距项分配唯一的需求编号 (FR-XXX)
- 按优先级组织需求
- 为每个需求定义验收标准
- 确保新功能有对应的规格定义

### 1.3 参考文档

- **PRD-OpenCode-Configuration.md**: 配置系统产品需求文档 (626 行)
- **gap-analysis.md**: 差距分析报告 (iteration-3, 112 行)
- **spec_v2.md**: 上一版规格文档 (1504 行)

### 1.4 与 v2 的关系

v3 保留 v2 的所有需求 (FR-001 ~ FR-032)，并新增配置系统专项需求 (FR-033 ~ FR-043)。v2 中的 FR-008/FR-021/FR-030 与 v3 新增需求有重叠，v3 提供更细粒度的拆分。

---

## 2. 需求总览

| 优先级 | 数量 | 说明 |
|--------|------|------|
| P0 | 12 | 阻断性问题 (v2: 10 + v3新增: 2) |
| P1 | 17 | 核心功能缺失 (v2: 10 + v3新增: 7) |
| P2 | 17 | 完善性问题 (v2: 12 + v3新增: 5) |

---

## 3. P0 - 阻断性问题

> FR-001 ~ FR-010 继承自 v2，内容不变。以下为 v3 新增 P0 需求。

### FR-033: OPENCODE_TUI_CONFIG 环境变量支持

**模块**: core/config
**严重程度**: P0
**差距项**: 缺少 `OPENCODE_TUI_CONFIG` 环境变量支持 (Gap #1)

#### 需求描述

实现 `OPENCODE_TUI_CONFIG` 环境变量，允许用户自定义 TUI 配置文件路径。

#### 详细规格

1. **环境变量定义**
   - 变量名: `OPENCODE_TUI_CONFIG`
   - 类型: 文件路径 (绝对路径或 `~` 开头的路径)
   - 优先级: 高于默认 TUI 配置路径

2. **加载逻辑**
   ```
   1. 检查 OPENCODE_TUI_CONFIG 环境变量
   2. 若设置，使用该路径作为 TUI 配置文件
   3. 若未设置，使用默认路径:
      - ~/.config/opencode/tui.json
   4. 若文件不存在，使用内建默认 TUI 配置
   ```

3. **路径展开**
   - 支持 `~` 展开为用户主目录
   - 支持绝对路径

4. **PRD 对应**
   - PRD §4.3 环境变量表明确列出 `OPENCODE_TUI_CONFIG`
   - PRD §10.3 验收标准明确要求支持

#### 验收标准

- [ ] `OPENCODE_TUI_CONFIG` 环境变量可自定义 TUI 配置路径
- [ ] 路径支持 `~` 展开
- [ ] 未设置时使用默认路径
- [ ] 文件不存在时降级到内建默认

---

### FR-034: TUI 配置分离为独立 tui.json 文件

**模块**: core/config, tui
**严重程度**: P0
**差距项**: TUI 配置未分离为独立文件 (Gap #2)

#### 需求描述

将 TUI 相关配置从主配置 (opencode.json) 分离到独立的 tui.json 文件，使用独立 schema。

#### 详细规格

1. **独立文件**
   - 文件名: `tui.json` 或 `tui.jsonc`
   - Schema: `$schema: "https://opencode.ai/tui.json"`
   - 默认路径: `~/.config/opencode/tui.json`

2. **TUI 配置项 (应移至 tui.json)**
   - `scroll_speed`: 滚动速度 (数字)
   - `scroll_acceleration`: 滚动加速 (对象 `{"enabled": true}`)
   - `diff_style`: diff 显示风格
   - `theme`: 主题名称
   - `keybinds`: 自定义快捷键绑定

3. **主配置中应移除的 TUI 相关项**
   - `theme` (在 opencode.json 中已废弃，PRD §9.1)
   - `keybinds` (在 opencode.json 中已废弃，PRD §9.1)
   - `tui` 对象 (在 opencode.json 中已废弃，PRD §9.1)

4. **加载优先级**
   ```
   1. OPENCODE_TUI_CONFIG 环境变量指定路径
   2. ~/.config/opencode/tui.json
   3. 项目目录 tui.json
   4. 内建默认 TUI 配置
   ```

5. **向后兼容**
   - 主配置中的旧 TUI 配置项应发出 deprecation warning
   - 自动迁移提示

6. **PRD 对应**
   - PRD §3.3 TUI 专用配置
   - PRD §5.16 TUI 配置
   - PRD §9.1 迁移说明
   - PRD §10.3 验收标准

#### 验收标准

- [ ] TUI 配置使用独立 tui.json 文件
- [ ] `$schema: "https://opencode.ai/tui.json"` 正确声明
- [ ] 主配置中旧 TUI 项发出废弃警告
- [ ] TUI 配置加载优先级正确

---

## 4. P1 - 核心功能缺失

> FR-011 ~ FR-020, FR-032 继承自 v2，内容不变。以下为 v3 新增 P1 需求。

### FR-035: modes/ 目录扫描

**模块**: core/config/directory_scanner
**严重程度**: P1
**差距项**: 缺少 `modes/` 目录扫描 (Gap #3)

#### 需求描述

在 `.opencode/` 和 `~/.config/opencode/` 目录扫描中添加 `modes/` 子目录支持。

#### 详细规格

1. **目录结构**
   ```
   .opencode/
   └── modes/
       └── *.md    # 模式定义文件
   ```

2. **扫描规则**
   - 扫描 `.opencode/modes/` 目录下所有 `.md` 文件
   - 扫描 `~/.config/opencode/modes/` 目录下所有 `.md` 文件
   - 支持 YAML frontmatter + Markdown body 格式

3. **模式定义格式**
   ```yaml
   ---
   name: code-review
   description: Code review mode with strict checks
   ---
   Mode instructions here...
   ```

4. **与 DirectoryScanner 集成**
   - 当前 `DirectoryScanner` 扫描: agents/commands/plugins/skills/tools/themes/
   - 需要添加: **modes/**

5. **PRD 对应**
   - PRD §4.2 目录结构约定明确列出 `modes/`

#### 验收标准

- [ ] `.opencode/modes/` 目录被正确扫描
- [ ] `~/.config/opencode/modes/` 目录被正确扫描
- [ ] 模式定义文件格式正确解析
- [ ] 扫描结果注册到配置系统

---

### FR-036: 配置路径命名统一为 opencode

**模块**: core/config
**严重程度**: P1
**差距项**: 配置路径使用 `opencode-rs` 而非 `opencode` (Gap #4)

#### 需求描述

将配置目录路径从 `~/.config/opencode-rs/` 统一为 `~/.config/opencode/`，与官方 OpenCode 生态保持一致。

#### 详细规格

1. **路径变更**
   - 旧路径: `~/.config/opencode-rs/config.toml`
   - 新路径: `~/.config/opencode/config.json` (或 `.jsonc`)

2. **影响范围**
   - `config_path()` 函数返回值
   - TUI 配置默认路径
   - 所有引用 `~/.config/opencode-rs/` 的硬编码

3. **迁移支持**
   - 启动时检查旧路径是否存在
   - 若存在，提示用户迁移到新路径
   - 可选自动迁移

4. **使用 directories crate**
   - 使用 `directories::ProjectDirs` 统一路径管理
   - 避免硬编码路径字符串

5. **PRD 对应**
   - PRD §4.1 优先级表使用 `~/.config/opencode/`

#### 验收标准

- [ ] 配置目录路径为 `~/.config/opencode/`
- [ ] 使用 `directories` crate 管理路径
- [ ] 无硬编码路径字符串
- [ ] 旧路径迁移提示正常

---

### FR-037: {file:path} 支持 ~ 路径展开

**模块**: core/config
**严重程度**: P1
**差距项**: `{file:path}` 不支持 `~` 路径展开 (Gap #5)

#### 需求描述

在 `{file:path}` 变量替换中支持 `~` 开头的路径，自动展开为用户主目录。

#### 详细规格

1. **语法**
   ```jsonc
   {
     "provider": {
       "openai": {
         "options": {
           "apiKey": "{file:~/.secrets/openai-key}"
         }
       }
     }
   }
   ```

2. **展开规则**
   - `~` → 当前用户主目录 (`$HOME`)
   - `~/` → `$HOME/`
   - 支持 `~` 后接任意路径

3. **实现位置**
   - `substitute_variables` 函数中的 `{file:...}` 处理分支
   - 在 `std::fs::read_to_string` 之前展开路径

4. **PRD 对应**
   - PRD §6.2 文件引用明确说明支持 `~` 开头

#### 验收标准

- [ ] `{file:~/.secrets/api-key}` 正确读取文件
- [ ] `~` 展开为当前用户主目录
- [ ] 展开失败时有明确错误提示

---

### FR-038: {file:path} 支持相对于配置文件目录

**模块**: core/config
**严重程度**: P1
**差距项**: `{file:path}` 不支持相对于配置文件目录 (Gap #6)

#### 需求描述

在 `{file:path}` 变量替换中支持相对于配置文件所在目录的路径。

#### 详细规格

1. **语法**
   ```jsonc
   // 假设配置文件位于 /project/.opencode/config.json
   {
     "instructions": ["{file:./custom-instructions.md}"]
     // 解析为 /project/.opencode/custom-instructions.md
   }
   ```

2. **解析规则**
   - `./` 开头 → 相对于配置文件所在目录
   - `../` 开头 → 相对于配置文件所在目录的父目录
   - 不以 `/` 或 `~` 开头 → 视为相对路径

3. **实现要求**
   - `substitute_variables` 需要知道配置文件的目录路径
   - 在 `load_multi()` 中传递配置文件目录上下文

4. **PRD 对应**
   - PRD §6.2 明确说明支持相对于配置文件目录的路径

#### 验收标准

- [ ] `{file:./instructions.md}` 相对于配置文件目录解析
- [ ] `{file:../shared/config.md}` 支持上级目录引用
- [ ] 相对路径解析在 `load_multi()` 中正确工作

---

### FR-039: .opencode/ 目录扫描集成到配置加载

**模块**: core/config
**严重程度**: P1
**差距项**: `load_opencode_directory()` 未被 `load_multi()` 调用 (Gap #12)

#### 需求描述

将 `.opencode/` 目录扫描功能集成到配置加载流程中，确保 agents/commands/skills/tools/themes/modes 等目录内容被自动加载。

#### 详细规格

1. **当前问题**
   - `load_opencode_directory()` 函数存在但独立于配置加载流程
   - `load_multi()` 只检查 `.opencode/config.json` 文件
   - 不扫描子目录 (agents/, commands/, skills/, etc.)

2. **集成方案**
   ```
   load_multi() 流程:
   1. 加载 Remote Config
   2. 加载 Global Config (~/.config/opencode/)
   3. 加载 Custom Config (OPENCODE_CONFIG)
   4. 加载 Project Config (opencode.json)
   5. → 调用 load_opencode_directory() 扫描 .opencode/ 子目录
   6. 加载 Inline Config (OPENCODE_CONFIG_CONTENT)
   7. 按优先级合并
   ```

3. **扫描内容**
   - `agents/` → 注册到 agent 配置
   - `commands/` → 注册到 command 配置
   - `modes/` → 注册到 mode 配置 (FR-035)
   - `plugins/` → 注册到 plugin 配置
   - `skills/` → 注册到 skill 配置
   - `tools/` → 注册到 tool 配置
   - `themes/` → 注册到 theme 配置

4. **PRD 对应**
   - PRD §4.1 优先级 #5: `.opencode/` 目录
   - PRD §4.2 目录结构约定

#### 验收标准

- [ ] `load_multi()` 自动调用 `.opencode/` 目录扫描
- [ ] agents/commands/modes/plugins/skills/tools/themes 内容被加载
- [ ] 目录内容与主配置正确合并
- [ ] 同名配置按优先级覆盖

---

## 5. P2 - 完善性问题

> FR-021 ~ FR-031 继承自 v2，内容不变。以下为 v3 新增 P2 需求。

### FR-040: 变量替换覆盖完整性

**模块**: core/config
**严重程度**: P2
**差距项**: 变量替换在 JSON 序列化层面而非原始字符串 (Gap #7)

#### 需求描述

确保所有配置加载路径都正确执行变量替换，避免部分路径遗漏。

#### 详细规格

1. **当前问题**
   - `load()` 中先替换变量再解析 JSON
   - `load_multi()` 中只有部分路径做了变量替换
   - 变量替换在 JSON 序列化层面进行，可能导致复杂情况出错

2. **统一替换策略**
   - 所有配置来源在解析前执行变量替换
   - 替换在原始字符串层面进行
   - 替换后再解析为 JSON

3. **未设置变量处理**
   - PRD §10.2 要求: 未设置的变量替换为空字符串
   - 当前行为: 保留原 `{env:VAR}` 字符串
   - 需要修正为空字符串

4. **PRD 对应**
   - PRD §6.1 环境变量替换
   - PRD §6.2 文件引用
   - PRD §10.2 验收标准

#### 验收标准

- [ ] 所有配置加载路径执行变量替换
- [ ] 未设置变量替换为空字符串 (非保留原字符串)
- [ ] 变量替换在原始字符串层面进行
- [ ] 嵌套/复杂变量替换正确处理

---

### FR-041: theme/keybinds 从主配置迁移

**模块**: core/config
**严重程度**: P2
**差距项**: `theme` 和 `keybinds` 仍在主配置中 (Gap #8)

#### 需求描述

将 `theme` 和 `keybinds` 配置项从主 Config 结构体迁移到 TUI 配置，与 PRD §9.1 废弃声明保持一致。

#### 详细规格

1. **废弃声明 (PRD §9.1)**
   - `theme` (在 opencode.json) → 移至 tui.json
   - `keybinds` (在 opencode.json) → 移至 tui.json

2. **迁移策略**
   - 主配置中保留字段但标记 `#[deprecated]`
   - 加载时发出 warning
   - 值自动迁移到 TUI 配置
   - 下一版本移除

3. **TUI 配置中新增**
   - `theme`: 主题名称或路径
   - `keybinds`: 自定义快捷键对象

4. **PRD 对应**
   - PRD §9.1 迁移说明

#### 验收标准

- [ ] 主配置中 theme/keybinds 发出废弃警告
- [ ] 旧配置值自动迁移到 TUI 配置
- [ ] TUI 配置正确加载 theme/keybinds

---

### FR-042: AgentMapConfig 完全动态 HashMap

**模块**: core/config
**严重程度**: P2
**差距项**: `AgentMapConfig` 使用固定键而非完全动态 HashMap (Gap #10)

#### 需求描述

将 `AgentMapConfig` 从固定键设计改为完全动态的 HashMap，支持任意 agent 名称。

#### 详细规格

1. **当前问题**
   - 预设固定键: plan/build/general/explore
   - 使用 `#[serde(flatten)]` 处理 custom agents
   - 非标准 agent 名可能解析失败

2. **PRD 示例**
   ```jsonc
   {
     "agent": {
       "code-reviewer": { ... },
       "security-reviewer": { ... },
       "custom-anything": { ... }
     }
   }
   ```

3. **目标设计**
   - 使用 `HashMap<String, AgentConfig>` 完全动态
   - 支持任意 agent 名称
   - 无预设键限制

4. **PRD 对应**
   - PRD §5.4 Agents 配置示例使用任意名称

#### 验收标准

- [ ] 支持任意 agent 名称作为 key
- [ ] 无固定键限制
- [ ] 自定义 agent 正确加载

---

### FR-043: JSON Schema 远程验证实现

**模块**: core/config/schema
**严重程度**: P2
**差距项**: 缺少配置文件的 JSON Schema 远程验证 (Gap #11)

#### 需求描述

实现真正的 JSON Schema 远程验证，从 `https://opencode.ai/config.json` 拉取 schema 并验证配置。

#### 详细规格

1. **当前问题**
   - `validate_json_schema` 是空壳实现
   - 只检查 port 和 temperature 两个字段
   - 未实际拉取和使用 JSON Schema

2. **实现方案**
   ```
   1. 读取配置文件中的 $schema 字段
   2. 尝试从远程 URL 拉取 schema
   3. 失败时使用本地缓存 (~/.config/opencode/schemas/)
   4. 缓存失败时使用内建 fallback schema
   5. 使用 jsonschema crate 验证配置
   6. 输出详细验证错误
   ```

3. **Schema 来源**
   - 远程: `$schema` 声明的 URL
   - 本地缓存: `~/.config/opencode/schemas/config.json`
   - 内建: 编译时嵌入的默认 schema

4. **离线模式**
   - 网络不可用时使用缓存/内建 schema
   - 不阻断配置加载

5. **PRD 对应**
   - PRD §3.2 Schema 声明

#### 验收标准

- [ ] 从远程 URL 拉取 JSON Schema
- [ ] 本地缓存机制正常
- [ ] 内建 fallback schema 可用
- [ ] 验证错误提示详细
- [ ] 离线模式不阻断配置加载

---

## 6. 技术债务清单

> 以下技术债务项需要在后续迭代中解决。

| 债务项 | 位置 | 描述 | 关联 FR |
|--------|------|------|---------|
| **TOML vs JSON 格式分裂** | `config.rs:1012-1031` | `config_path()` 默认返回 `.toml`，但 PRD 要求 JSON/JSONC | FR-036 |
| **硬编码路径** | `config.rs:1031` | `"~/.config/opencode-rs/config.toml"` 硬编码 | FR-036 |
| **变量替换实现粗糙** | `config.rs:972-1009` | 字符串替换对嵌套/复杂情况可能出错 | FR-040 |
| **merge_configs 通过 JSON 中转** | `merge.rs:22-29` | 序列化→deep_merge→反序列化，丢失类型信息 | FR-021 |
| **fetch_remote_config 同步包装异步** | `config.rs:1107-1109` | 同步函数中创建 tokio runtime | - |
| **TimeoutConfig 枚举命名** | `config.rs:469-474` | `Disabled(bool)` 语义不清 | - |
| **PermissionConfig 大量重复字段** | `config.rs:628-697` | 应考虑宏生成或统一结构 | - |
| **Schema 验证空壳** | `schema.rs:5-40` | 只检查 2 个字段 | FR-043 |
| **DirectoryScanner 未使用 glob** | `directory_scanner.rs` | 手动 read_dir，不支持 glob 模式 | FR-035 |
| **测试覆盖不足** | `core/tests/` | 仅 2 个测试文件，缺少集成测试 | - |

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
| disabled_providers 优先级 | 10.4 | ✅ | - | `is_provider_enabled()` 正确 |
| 自定义 agent 配置 | 10.5 | ✅ | FR-042 | AgentConfig 完整，但 AgentMapConfig 需改为动态 |
| default_agent 设置 | 10.5 | ✅ | - | 字段存在且被 env 覆盖 |
| 命令模板变量替换 | 10.5 | ⚠️ | FR-004 | 命令模板变量替换未明确实现 |
| permission 配置 | 10.6 | ✅ | - | `PermissionConfig` 完整 |
| API Key 文件引用 | 10.6 | ⚠️ | FR-037, FR-038 | 依赖 `{file:path}`，但该功能不完整 |

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
| config/tui | FR-019 | scroll_acceleration 结构修复 | P1 | v2 |
| config/tui | FR-020 | keybinds 自定义绑定 | P1 | v2 |
| config/tui | FR-031 | theme 路径解析增强 | P2 | v2 |
| schema | FR-018 | TUI Schema 验证 | P1 | v2 |
| plugin | FR-002 | Plugin System | P0 | v2 |
| mcp | FR-005 | MCP 工具接入 | P0 | v2 |
| server | FR-006 | TUI 快捷输入解析器 | P0 | v2 |
| server | FR-007 | Session Fork | P0 | v2 |
| server | FR-011 | Server API 完善 | P1 | v2 |
| storage | FR-032 | Snapshot 元数据完善 | P1 | v2 |
| storage/permission | FR-016 | Permission 审计记录 | P1 | v2 |
| lsp | FR-013 | LSP 功能增强 | P1 | v2 |
| auth | FR-015 | 凭证加密存储 | P1 | v2 |
| auth | FR-029 | OAuth 登录预留 | P2 | v2 |
| tui | FR-017 | TUI Token/Cost 显示 | P1 | v2 |
| tui | FR-023 | TUI 布局切换 | P2 | v2 |
| tui | FR-024 | TUI 右栏功能完善 | P2 | v2 |
| tui | FR-025 | TUI Patch 预览展开 | P2 | v2 |
| tui | FR-026 | Web UI | P2 | v2 |
| - | FR-027 | IDE 扩展预留 | P2 | v2 |
| git | FR-028 | GitHub 集成预留 | P2 | v2 |

### 按优先级分组

| 优先级 | FR 编号 |
|--------|---------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010, FR-033, FR-034 |
| P1 | FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020, FR-032, FR-035, FR-036, FR-037, FR-038, FR-039 |
| P2 | FR-021, FR-022, FR-023, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029, FR-030, FR-031, FR-040, FR-041, FR-042, FR-043 |

---

## 9. 实施建议

### Phase 1: P0 阻断性问题 (当前优先级)

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
6. **FR-011 Server API** - API 完整性
7. **FR-013 LSP 功能增强** - 开发体验
8. **FR-012 Share 功能** - 协作能力
9. **FR-015 凭证加密存储** - 安全合规
10. **FR-014 插件事件总线** - 事件系统
11. **FR-016 Permission 审计记录** - 权限追踪
12. **FR-017 TUI Token/Cost 显示** - 成本感知
13. **FR-018 TUI Schema 验证** - 配置验证增强
14. **FR-019 scroll_acceleration 结构修复** - 类型修正
15. **FR-020 keybinds 自定义绑定** - 绑定扩展
16. **FR-032 Snapshot 元数据完善** - 数据完整性

### Phase 3: P2 完善性

1. **FR-040 变量替换覆盖完整性** - 配置系统完善
2. **FR-041 theme/keybinds 迁移** - 废弃声明一致性
3. **FR-042 AgentMapConfig 动态 HashMap** - 灵活性
4. **FR-043 JSON Schema 远程验证** - 配置校验
5. **FR-021 配置系统** - 配置灵活性
6. **FR-022 Session Summarize** - 会话管理
7. **FR-023 TUI 布局切换** - UI 增强
8. **FR-024 TUI 右栏功能完善** - 面板功能
9. **FR-025 TUI Patch 预览展开** - Diff 交互
10. **FR-026 Web UI** - 多端支持
11. **FR-027 IDE 扩展预留** - 生态扩展
12. **FR-028 GitHub 集成预留** - DevOps 集成
13. **FR-029 OAuth 登录预留** - 认证扩展
14. **FR-030 废弃字段清理** - 代码清理
15. **FR-031 theme 路径解析增强** - 主题功能增强

---

## 10. 附录

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

### C. 配置系统状态

| 配置项 | 实现状态 | 关联 FR | 备注 |
|--------|----------|---------|------|
| JSON/JSONC 格式 | ✅ 完整 | - | jsonc.rs |
| 配置合并 | ✅ 完整 | FR-021 | merge.rs |
| Remote Config | ⚠️ 部分 | FR-008 | fetch_remote_config 同步包装异步 |
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

### D. v2 → v3 变更摘要

| 变更项 | 说明 |
|--------|------|
| 新增 FR-033 | OPENCODE_TUI_CONFIG 环境变量支持 (P0) |
| 新增 FR-034 | TUI 配置分离为独立 tui.json (P0) |
| 新增 FR-035 | modes/ 目录扫描 (P1) |
| 新增 FR-036 | 配置路径命名统一为 opencode (P1) |
| 新增 FR-037 | {file:path} ~ 路径展开 (P1) |
| 新增 FR-038 | {file:path} 相对路径支持 (P1) |
| 新增 FR-039 | .opencode/ 目录扫描集成到配置加载 (P1) |
| 新增 FR-040 | 变量替换覆盖完整性 (P2) |
| 新增 FR-041 | theme/keybinds 从主配置迁移 (P2) |
| 新增 FR-042 | AgentMapConfig 完全动态 HashMap (P2) |
| 新增 FR-043 | JSON Schema 远程验证实现 (P2) |
| 新增 §6 | 技术债务清单 |
| 新增 §7 | PRD §10 验收标准对照表 |
| 新增 §10.C | 配置系统状态表 |
| 新增 §10.D | v2 → v3 变更摘要 |

---

**文档状态**: 草稿
**下一步**: 基于本规格文档创建迭代计划
