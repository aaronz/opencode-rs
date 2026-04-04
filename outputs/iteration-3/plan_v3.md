# OpenCode-RS 实现计划 v3

**版本**: 3.0
**日期**: 2026-04-04
**基于**: spec_v3.md + gap-analysis.md + constitution_updates.md
**状态**: 草稿

---

## 1. 计划概述

### 1.1 目标

基于 iteration-3 差距分析报告中识别的 ~75-80% 实现完整度，修复所有 P0/P1/P2 差距项，将配置系统提升至 PRD 合规水平。

### 1.2 差距优先级总览

| Priority | 差距数 | 目标 |
|----------|--------|------|
| P0 | 2 | 本周 (TUI 配置分离 + OPENCODE_TUI_CONFIG) |
| P1 | 5 | 下周 (目录扫描/路径/变量替换) |
| P2 | 5 | Sprint 内 (完善性/技术债务) |

### 1.3 阶段划分

| Phase | 优先级 | 任务数 | 目标 |
|-------|--------|--------|------|
| Phase 0 | P0 | 2 | TUI 配置分离 + OPENCODE_TUI_CONFIG 环境变量 |
| Phase 1 | P1 | 5 | 目录扫描集成/路径命名统一/{file:path} 增强 |
| Phase 2 | P2 | 5 | 变量替换完整性/AgentMap 动态化/Schema 验证 |

---

## 2. Phase 0: P0 阻断性问题

### 2.1 目标

修复 PRD §10.3 验收标准中明确要求的两项阻断性问题：TUI 配置与 runtime 配置分离，以及 `OPENCODE_TUI_CONFIG` 环境变量支持。

### 2.2 任务清单

#### Task 0.1: OPENCODE_TUI_CONFIG 环境变量支持 (FR-033)

**目标**: 实现 `OPENCODE_TUI_CONFIG` 环境变量，允许用户自定义 TUI 配置文件路径。

**子任务**:
1. 在 `config.rs` 中添加 `OPENCODE_TUI_CONFIG` 环境变量读取逻辑
2. 实现路径展开 (`~` → `$HOME`)
3. 实现 TUI 配置加载函数 `load_tui_config()`:
   - 优先使用 `OPENCODE_TUI_CONFIG` 指定路径
   - 降级到 `~/.config/opencode/tui.json`
   - 文件不存在时使用内建默认 TUI 配置
4. 实现 TUI 配置结构体 `TuiConfig` 独立加载
5. 添加单元测试覆盖路径展开和降级逻辑

**验收标准**:
- [ ] `OPENCODE_TUI_CONFIG` 环境变量可自定义 TUI 配置路径
- [ ] 路径支持 `~` 展开
- [ ] 未设置时使用默认路径
- [ ] 文件不存在时降级到内建默认
- [ ] 单元测试覆盖所有路径情况

**预计工期**: 2 天

---

#### Task 0.2: TUI 配置分离为独立 tui.json 文件 (FR-034)

**目标**: 将 TUI 相关配置从主配置 (opencode.json) 分离到独立的 tui.json 文件，使用独立 schema。

**子任务**:
1. 创建独立 `TuiConfig` 结构体，包含:
   - `scroll_speed`: 滚动速度
   - `scroll_acceleration`: 滚动加速
   - `diff_style`: diff 显示风格
   - `theme`: 主题名称
   - `keybinds`: 自定义快捷键绑定
   - `$schema`: 声明 `"https://opencode.ai/tui.json"`
2. 从主 `Config` 结构体中标记 `theme`/`keybinds` 为 `#[deprecated]`
3. 实现 TUI 配置加载优先级:
   - `OPENCODE_TUI_CONFIG` → `~/.config/opencode/tui.json` → 项目 `tui.json` → 内建默认
4. 实现 TUI 配置与 Runtime 配置独立合并 (互不影响)
5. 实现废弃警告: 主配置中的旧 TUI 配置项发出 deprecation warning
6. 实现向后兼容: 旧配置值自动迁移到 TUI 配置
7. 添加 TUI 配置验证 (禁止出现 runtime 字段)
8. 添加集成测试

**验收标准**:
- [ ] TUI 配置使用独立 tui.json 文件
- [ ] `$schema: "https://opencode.ai/tui.json"` 正确声明
- [ ] 主配置中旧 TUI 项发出废弃警告
- [ ] TUI 配置加载优先级正确
- [ ] TUI 配置与 Runtime 配置独立合并
- [ ] 旧配置值自动迁移

**预计工期**: 3 天

---

### 2.3 Phase 0 并行策略

**顺序执行**: Task 0.1 → Task 0.2 (Task 0.2 依赖 Task 0.1 的环境变量支持)

---

## 3. Phase 1: P1 核心功能缺失

### 3.1 目标

修复目录扫描、路径命名、变量替换等核心功能缺失，确保配置系统完整性。

### 3.2 任务清单

#### Task 1.1: .opencode/ 目录扫描集成到配置加载 (FR-039)

**目标**: 将 `.opencode/` 目录扫描功能集成到配置加载流程中。

**子任务**:
1. 修改 `load_multi()` 函数，在加载 Project Config 后调用 `load_opencode_directory()`
2. 确保扫描结果按优先级合并到最终配置:
   - Remote Config → Global Config → Custom Config → Project Config → **.opencode/ 目录** → Inline Config
3. 确保各子目录内容正确注册:
   - `agents/` → agent 配置
   - `commands/` → command 配置
   - `modes/` → mode 配置
   - `plugins/` → plugin 配置
   - `skills/` → skill 配置
   - `tools/` → tool 配置
   - `themes/` → theme 配置
4. 扫描失败不阻断配置加载，仅记录 warning
5. 添加集成测试

**验收标准**:
- [ ] `load_multi()` 自动调用 `.opencode/` 目录扫描
- [ ] 各子目录内容被正确加载和注册
- [ ] 目录内容与主配置正确合并
- [ ] 扫描失败不阻断配置加载

**预计工期**: 2 天

---

#### Task 1.2: modes/ 目录扫描 (FR-035)

**目标**: 在 `.opencode/` 和 `~/.config/opencode/` 目录扫描中添加 `modes/` 子目录支持。

**子任务**:
1. 在 `DirectoryScanner` 中添加 `modes/` 目录扫描
2. 定义 Mode 结构体:
   - `name`: 模式名称
   - `description`: 模式描述
   - `system_prompt`: 系统提示模板
   - `default_agent`: 默认 agent
   - `permission_overrides`: 权限覆盖
3. 实现 YAML frontmatter + Markdown body 格式解析
4. 实现 modes 注册到配置系统
5. 同时扫描全局 `~/.config/opencode/modes/` 和项目 `.opencode/modes/`
6. 添加单元测试

**验收标准**:
- [ ] `.opencode/modes/` 目录被正确扫描
- [ ] `~/.config/opencode/modes/` 目录被正确扫描
- [ ] 模式定义文件格式正确解析
- [ ] 扫描结果注册到配置系统

**预计工期**: 2 天

---

#### Task 1.3: 配置路径命名统一为 opencode (FR-036)

**目标**: 将配置目录路径从 `~/.config/opencode-rs/` 统一为 `~/.config/opencode/`。

**子任务**:
1. 使用 `directories` crate 的 `ProjectDirs` 统一路径管理
2. 修改 `config_path()` 返回 `~/.config/opencode/config.json`
3. 更新所有引用 `opencode-rs` 的硬编码路径
4. 实现旧路径迁移支持:
   - 启动时检查 `~/.config/opencode-rs/` 是否存在
   - 若存在，提示用户迁移到新路径
   - 可选自动迁移
5. 使用平台适配路径:
   - macOS: `~/Library/Application Support/opencode/`
   - Linux: `~/.config/opencode/`
   - Windows: `%APPDATA%\opencode\`
6. 默认格式改为 JSONC (非 TOML)
7. 添加单元测试

**验收标准**:
- [ ] 配置目录路径为 `~/.config/opencode/` (或平台等效路径)
- [ ] 使用 `directories` crate 管理路径
- [ ] 无硬编码路径字符串
- [ ] 旧路径迁移提示正常

**预计工期**: 2 天

---

#### Task 1.4: {file:path} 支持 ~ 路径展开 (FR-037)

**目标**: 在 `{file:path}` 变量替换中支持 `~` 开头的路径。

**子任务**:
1. 在 `substitute_variables` 函数的 `{file:...}` 处理分支中添加 `~` 展开
2. 实现规则:
   - `~` → `$HOME`
   - `~/` → `$HOME/`
   - 支持 `~` 后接任意路径
3. 展开失败时提供明确错误提示
4. 添加单元测试覆盖:
   - `{file:~/.secrets/api-key}` 正确读取
   - `~` 不存在时的错误处理

**验收标准**:
- [ ] `{file:~/.secrets/api-key}` 正确读取文件
- [ ] `~` 展开为当前用户主目录
- [ ] 展开失败时有明确错误提示

**预计工期**: 1 天

---

#### Task 1.5: {file:path} 支持相对于配置文件目录 (FR-038)

**目标**: 在 `{file:path}` 变量替换中支持相对于配置文件所在目录的路径。

**子任务**:
1. 修改 `substitute_variables` 函数签名，接收配置文件目录上下文
2. 实现相对路径解析规则:
   - `./` 开头 → 相对于配置文件所在目录
   - `../` 开头 → 相对于配置文件所在目录的父目录
   - 不以 `/` 或 `~` 开头 → 视为相对路径
3. 在 `load_multi()` 中传递配置文件目录上下文
4. 添加单元测试覆盖:
   - `{file:./instructions.md}` 相对于配置文件目录
   - `{file:../shared/config.md}` 支持上级目录

**验收标准**:
- [ ] `{file:./instructions.md}` 相对于配置文件目录解析
- [ ] `{file:../shared/config.md}` 支持上级目录引用
- [ ] 相对路径解析在 `load_multi()` 中正确工作

**预计工期**: 2 天

---

### 3.3 Phase 1 并行策略

**可并行**:
- Task 1.2 (modes/) 和 Task 1.3 (路径命名) 可并行
- Task 1.4 (~ 展开) 和 Task 1.5 (相对路径) 可并行
- Task 1.1 (目录扫描集成) 依赖 Task 1.2 (modes/ 扫描)

**依赖关系**:
```
Task 1.2 ──┐
           ├──→ Task 1.1
Task 1.3 ──┘

Task 1.4 ──┐
           └──→ (独立，可与 1.2/1.3 并行)
Task 1.5 ──┘
```

---

## 4. Phase 2: P2 完善性问题

### 4.1 目标

解决变量替换覆盖完整性、AgentMap 动态化、Schema 验证等技术债务。

### 4.2 任务清单

#### Task 2.1: 变量替换覆盖完整性 (FR-040)

**目标**: 确保所有配置加载路径都正确执行变量替换。

**子任务**:
1. 审查 `load_multi()` 中所有配置加载路径，确保统一执行变量替换
2. 修正未设置变量处理: 替换为空字符串 `""` (非保留原 `{env:VAR}` 字符串)
3. 确保变量替换在原始字符串层面进行 (JSON 解析前)
4. 处理嵌套/复杂变量替换情况
5. 添加集成测试覆盖所有变量替换场景

**验收标准**:
- [ ] 所有配置加载路径执行变量替换
- [ ] 未设置变量替换为空字符串
- [ ] 变量替换在原始字符串层面进行
- [ ] 嵌套/复杂变量替换正确处理

**预计工期**: 2 天

---

#### Task 2.2: theme/keybinds 从主配置迁移 (FR-041)

**目标**: 将 `theme` 和 `keybinds` 配置项从主 Config 结构体迁移到 TUI 配置。

**子任务**:
1. 在主配置 `Config` 结构体中标记 `theme`/`keybinds` 为 `#[deprecated]`
2. 加载时发出 warning 日志
3. 实现自动迁移: 旧配置值合并到 TUI 配置
4. 在 TUI 配置中添加 `theme` 和 `keybinds` 字段
5. 添加单元测试

**验收标准**:
- [ ] 主配置中 theme/keybinds 发出废弃警告
- [ ] 旧配置值自动迁移到 TUI 配置
- [ ] TUI 配置正确加载 theme/keybinds

**预计工期**: 1 天

---

#### Task 2.3: AgentMapConfig 完全动态 HashMap (FR-042)

**目标**: 将 `AgentMapConfig` 从固定键设计改为完全动态的 HashMap。

**子任务**:
1. 移除预设固定键 (plan/build/general/explore)
2. 使用 `HashMap<String, AgentConfig>` 完全动态
3. 实现自定义 `Deserialize` 支持任意 agent 名称
4. 确保 `default_agent` 字段与动态 map 兼容
5. 添加单元测试覆盖任意 agent 名称

**验收标准**:
- [ ] 支持任意 agent 名称作为 key
- [ ] 无固定键限制
- [ ] 自定义 agent 正确加载

**预计工期**: 2 天

---

#### Task 2.4: JSON Schema 远程验证实现 (FR-043)

**目标**: 实现真正的 JSON Schema 远程验证。

**子任务**:
1. 读取配置文件中的 `$schema` 字段
2. 实现远程 schema 拉取 (从 `https://opencode.ai/config.json`)
3. 实现本地缓存机制 (`~/.config/opencode/schemas/`)
4. 实现内建 fallback schema (编译时嵌入)
5. 使用 `jsonschema` crate 验证配置
6. 实现离线模式 (网络不可用时使用缓存/内建 schema)
7. 输出详细验证错误信息
8. 添加单元测试

**验收标准**:
- [ ] 从远程 URL 拉取 JSON Schema
- [ ] 本地缓存机制正常
- [ ] 内建 fallback schema 可用
- [ ] 验证错误提示详细
- [ ] 离线模式不阻断配置加载

**预计工期**: 3 天

---

#### Task 2.5: 技术债务清理

**目标**: 解决 gap analysis 中识别的关键技术债务。

**子任务**:
1. **merge_configs 优化**: 避免 JSON 中转，直接 deep_merge on serde_json::Value
2. **fetch_remote_config 异步化**: 将同步包装改为纯异步
3. **TimeoutConfig 枚举重命名**: `Disabled(bool)` → `NoTimeout`
4. **DirectoryScanner glob 支持**: 使用 `glob` crate 支持 `*.md` 批量匹配
5. **测试文件命名规范化**: 统一为 `*_test.rs` 或使用 `tests/` 集成测试目录

**验收标准**:
- [ ] merge_configs 不再通过 JSON 中转
- [ ] fetch_remote_config 为纯异步
- [ ] TimeoutConfig 枚举命名清晰
- [ ] DirectoryScanner 支持 glob 模式
- [ ] 测试文件命名规范

**预计工期**: 2 天

---

### 4.3 Phase 2 并行策略

**可并行**:
- Task 2.2 (theme/keybinds 迁移) 和 Task 2.3 (AgentMap 动态化) 可并行
- Task 2.5 (技术债务清理) 可与其他任务并行
- Task 2.1 (变量替换) 和 Task 2.4 (Schema 验证) 可并行

**依赖关系**:
```
Task 2.2 依赖 Phase 0 Task 0.2 (TUI 配置分离)
Task 2.1 依赖 Phase 1 Task 1.4, 1.5 ({file:path} 增强)
Task 2.3, 2.4, 2.5 可并行
```

---

## 5. 资源分配

### 5.1 人力估算

| Phase | 任务数 | 预计工期 | 总人天 |
|-------|--------|----------|--------|
| Phase 0 | 2 | 5 天 | 5 人天 |
| Phase 1 | 5 | 9 天 | 9 人天 |
| Phase 2 | 5 | 10 天 | 10 人天 |
| **总计** | **12** | **24 天** | **24 人天** |

### 5.2 并行机会

- Phase 0: 顺序执行 (0.1 → 0.2)
- Phase 1: Task 1.2+1.3 并行，Task 1.4+1.5 并行
- Phase 2: Task 2.2+2.3+2.4+2.5 可并行 (依赖满足后)

---

## 6. 里程碑

| Milestone | Phase | 任务 | 预计完成 |
|-----------|-------|------|----------|
| M0 | Phase 0 | P0 阻断性问题 (TUI 配置分离) | Week 1 |
| M1 | Phase 1 | P1 核心功能 (目录/路径/变量) | Week 3 |
| M2 | Phase 2 | P2 完善性 (Schema/动态化/债务) | Week 5 |

---

## 7. 验收流程

### 7.1 每个任务的验收标准

1. 功能正常运行
2. 错误处理正确
3. 性能满足要求
4. 文档完整
5. 测试覆盖

### 7.2 阶段验收

- Phase 0 完成后进行 TUI 配置分离评审
- Phase 1 完成后进行配置系统完整性评审
- Phase 2 完成后进行代码质量评审

---

## 8. Constitution 合规性

本计划遵循 Constitution v1.3 新增条款:

| 条款 | 覆盖任务 | 说明 |
|------|----------|------|
| C-017 | Task 0.1, 0.2, 2.2 | TUI 配置与 Runtime 配置分离 |
| C-018 | Task 1.3 | 配置路径与目录命名规范 |
| C-019 | Task 1.4, 1.5, 2.1 | 配置变量替换语义 |
| C-013 (细化) | Task 1.1, 1.2 | 目录结构扩展 (modes/) + 扫描集成 |

---

**文档状态**: 草稿
**下一步**: 基于本计划创建 tasks_v3.md 任务清单
