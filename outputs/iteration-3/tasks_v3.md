# OpenCode-RS 任务清单 v3

**版本**: 3.0
**日期**: 2026-04-04
**基于**: spec_v3.md + gap-analysis.md + plan_v3.md
**状态**: 已完成

---

## 1. 任务总览

| Phase | 优先级 | 任务数 | 状态 |
|-------|--------|--------|------|
| Phase 0 | P0 | 2 | 待开始 |
| Phase 1 | P1 | 5 | 待开始 |
| Phase 2 | P2 | 5 | 待开始 |

---

## 2. Phase 0: P0 阻断性问题

### Task 0.1: OPENCODE_TUI_CONFIG 环境变量支持

**ID**: TASK-0.1
**优先级**: P0
**模块**: core/config
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-033
**关联 Constitution**: C-017

**目标**: 实现 `OPENCODE_TUI_CONFIG` 环境变量，允许用户自定义 TUI 配置文件路径。

**子任务**:
- [ ] TASK-0.1.1: 添加环境变量读取逻辑
  - 在 `config.rs` 中读取 `OPENCODE_TUI_CONFIG`
  - 使用 `std::env::var("OPENCODE_TUI_CONFIG")`
- [ ] TASK-0.1.2: 实现路径展开
  - `~` → `$HOME` 展开
  - 绝对路径直接使用
- [ ] TASK-0.1.3: 实现 TUI 配置加载函数 `load_tui_config()`
  - 优先级: `OPENCODE_TUI_CONFIG` → `~/.config/opencode/tui.json` → 内建默认
  - 文件不存在时不报错，使用默认值
- [ ] TASK-0.1.4: 定义独立 `TuiConfig` 结构体
  - 包含 scroll_speed, scroll_acceleration, diff_style, theme, keybinds
  - 实现 `Default` trait
- [ ] TASK-0.1.5: 添加单元测试
  - 环境变量设置/未设置场景
  - `~` 展开测试
  - 文件不存在降级测试

**验收标准**:
- [ ] `OPENCODE_TUI_CONFIG` 环境变量可自定义 TUI 配置路径
- [ ] 路径支持 `~` 展开
- [ ] 未设置时使用默认路径
- [ ] 文件不存在时降级到内建默认
- [ ] 单元测试覆盖所有路径情况

**依赖**: 无

---

### Task 0.2: TUI 配置分离为独立 tui.json 文件

**ID**: TASK-0.2
**优先级**: P0
**模块**: core/config, tui
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-034
**关联 Constitution**: C-017

**目标**: 将 TUI 相关配置从主配置 (opencode.json) 分离到独立的 tui.json 文件。

**子任务**:
- [ ] TASK-0.2.1: 创建独立 TuiConfig 结构体
  - 字段: scroll_speed, scroll_acceleration, diff_style, theme, keybinds
  - `$schema` 声明: `"https://opencode.ai/tui.json"`
  - 实现 `Deserialize` 和 `Serialize`
- [ ] TASK-0.2.2: 从主 Config 标记废弃字段
  - `theme` 标记 `#[deprecated]`
  - `keybinds` 标记 `#[deprecated]`
  - 添加废弃日志
- [ ] TASK-0.2.3: 实现 TUI 配置加载优先级
  - `OPENCODE_TUI_CONFIG` → `~/.config/opencode/tui.json` → 项目 `tui.json` → 内建默认
- [ ] TASK-0.2.4: 实现 TUI 配置与 Runtime 配置独立合并
  - 各自使用 deep_merge，互不影响
  - 禁止 TUI 配置出现 runtime 字段
  - 禁止 Runtime 配置出现 TUI 字段
- [ ] TASK-0.2.5: 实现废弃警告
  - 主配置中检测到 theme/keybinds 时发出 warning
  - 提示迁移到 tui.json
- [ ] TASK-0.2.6: 实现向后兼容
  - 旧配置值自动迁移到 TUI 配置
  - TUI 配置优先于旧配置
- [ ] TASK-0.2.7: 添加 TUI 配置验证
  - 检测 TUI 配置中的 runtime 字段并报错
- [ ] TASK-0.2.8: 添加集成测试
  - TUI 配置加载测试
  - 废弃警告测试
  - 迁移测试

**验收标准**:
- [ ] TUI 配置使用独立 tui.json 文件
- [ ] `$schema: "https://opencode.ai/tui.json"` 正确声明
- [ ] 主配置中旧 TUI 项发出废弃警告
- [ ] TUI 配置加载优先级正确
- [ ] TUI 配置与 Runtime 配置独立合并
- [ ] 旧配置值自动迁移

**依赖**: TASK-0.1

---

## 3. Phase 1: P1 核心功能缺失

### Task 1.1: .opencode/ 目录扫描集成到配置加载

**ID**: TASK-1.1
**优先级**: P1
**模块**: core/config
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-039
**关联 Constitution**: C-013 (细化)

**目标**: 将 `.opencode/` 目录扫描功能集成到配置加载流程中。

**子任务**:
- [ ] TASK-1.1.1: 修改 `load_multi()` 函数
  - 在加载 Project Config 后调用 `load_opencode_directory()`
  - 传递项目目录路径上下文
- [ ] TASK-1.1.2: 实现扫描结果合并
  - 扫描结果按优先级合并到最终配置
  - 遵循 deep_merge 策略
- [ ] TASK-1.1.3: 确保各子目录内容正确注册
  - agents/ → agent 配置
  - commands/ → command 配置
  - modes/ → mode 配置
  - plugins/ → plugin 配置
  - skills/ → skill 配置
  - tools/ → tool 配置
  - themes/ → theme 配置
- [ ] TASK-1.1.4: 实现错误容忍
  - 目录不存在时记录 warning，不阻断加载
  - 权限不足时记录 warning，不阻断加载
- [ ] TASK-1.1.5: 添加集成测试
  - 目录扫描集成测试
  - 配置合并测试

**验收标准**:
- [ ] `load_multi()` 自动调用 `.opencode/` 目录扫描
- [ ] 各子目录内容被正确加载和注册
- [ ] 目录内容与主配置正确合并
- [ ] 扫描失败不阻断配置加载

**依赖**: TASK-1.2 (modes/ 扫描)

---

### Task 1.2: modes/ 目录扫描

**ID**: TASK-1.2
**优先级**: P1
**模块**: core/config/directory_scanner
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-035
**关联 Constitution**: C-013 (细化)

**目标**: 在 `.opencode/` 和 `~/.config/opencode/` 目录扫描中添加 `modes/` 子目录支持。

**子任务**:
- [ ] TASK-1.2.1: 在 DirectoryScanner 中添加 modes/ 目录
  - 扩展 `SCANNABLE_DIRS` 常量或配置
  - 添加 `modes` 到扫描列表
- [ ] TASK-1.2.2: 定义 Mode 结构体
  - `name`: 模式名称
  - `description`: 模式描述
  - `system_prompt`: 系统提示模板
  - `default_agent`: 默认 agent
  - `permission_overrides`: 权限覆盖
- [ ] TASK-1.2.3: 实现 YAML frontmatter + Markdown body 解析
  - 使用 `serde_yaml` 解析 frontmatter
  - 提取 Markdown body 作为 system_prompt
- [ ] TASK-1.2.4: 实现 modes 注册到配置系统
  - 全局 `~/.config/opencode/modes/` 扫描
  - 项目 `.opencode/modes/` 扫描
  - 项目覆盖全局
- [ ] TASK-1.2.5: 添加单元测试
  - 模式文件解析测试
  - 目录扫描测试

**验收标准**:
- [ ] `.opencode/modes/` 目录被正确扫描
- [ ] `~/.config/opencode/modes/` 目录被正确扫描
- [ ] 模式定义文件格式正确解析
- [ ] 扫描结果注册到配置系统

**依赖**: 无

---

### Task 1.3: 配置路径命名统一为 opencode

**ID**: TASK-1.3
**优先级**: P1
**模块**: core/config
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-036
**关联 Constitution**: C-018

**目标**: 将配置目录路径从 `~/.config/opencode-rs/` 统一为 `~/.config/opencode/`。

**子任务**:
- [ ] TASK-1.3.1: 引入 `directories` crate
  - 添加到 `Cargo.toml`
  - 使用 `ProjectDirs::from("ai", "opencode", "opencode")`
- [ ] TASK-1.3.2: 修改 `config_path()` 函数
  - 使用 `directories` crate 获取配置目录
  - 返回 `config.json` 路径 (非 `.toml`)
- [ ] TASK-1.3.3: 更新所有硬编码路径
  - 搜索 `opencode-rs` 字符串
  - 替换为 `directories` crate 调用
- [ ] TASK-1.3.4: 实现旧路径迁移支持
  - 启动时检查 `~/.config/opencode-rs/` 是否存在
  - 若存在，输出迁移提示
  - 可选自动迁移函数
- [ ] TASK-1.3.5: 修改默认格式为 JSONC
  - 移除 `OPENCODE_CONFIG_FORMAT` 作为默认
  - 默认使用 `.json` / `.jsonc` 扩展名检测
- [ ] TASK-1.3.6: 添加单元测试
  - 路径获取测试 (mock directories)
  - 旧路径迁移测试

**验收标准**:
- [ ] 配置目录路径使用 `directories` crate
- [ ] 无硬编码路径字符串
- [ ] 旧路径迁移提示正常
- [ ] 默认格式为 JSONC

**依赖**: 无

---

### Task 1.4: {file:path} 支持 ~ 路径展开

**ID**: TASK-1.4
**优先级**: P1
**模块**: core/config
**状态**: completed
**预计工期**: 1 天
**关联 FR**: FR-037
**关联 Constitution**: C-019

**目标**: 在 `{file:path}` 变量替换中支持 `~` 开头的路径。

**子任务**:
- [ ] TASK-1.4.1: 修改 `substitute_variables` 函数
  - 在 `{file:...}` 处理分支中检测 `~` 开头
  - 使用 `dirs::home_dir()` 或 `std::env::var("HOME")` 展开
- [ ] TASK-1.4.2: 实现展开规则
  - `~` → `$HOME`
  - `~/` → `$HOME/`
  - 支持 `~` 后接任意路径
- [ ] TASK-1.4.3: 实现错误处理
  - `~` 展开失败时记录 error 日志
  - 替换为空字符串 `""`
- [ ] TASK-1.4.4: 添加单元测试
  - `{file:~/.secrets/api-key}` 正确读取
  - `~` 不存在时的错误处理

**验收标准**:
- [ ] `{file:~/.secrets/api-key}` 正确读取文件
- [ ] `~` 展开为当前用户主目录
- [ ] 展开失败时有明确错误提示

**依赖**: 无

---

### Task 1.5: {file:path} 支持相对于配置文件目录

**ID**: TASK-1.5
**优先级**: P1
**模块**: core/config
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-038
**关联 Constitution**: C-019

**目标**: 在 `{file:path}` 变量替换中支持相对于配置文件所在目录的路径。

**子任务**:
- [ ] TASK-1.5.1: 修改 `substitute_variables` 函数签名
  - 增加 `config_dir: Option<&Path>` 参数
  - 所有调用点传递配置文件目录
- [ ] TASK-1.5.2: 实现相对路径解析规则
  - `./` 开头 → 相对于配置文件所在目录
  - `../` 开头 → 相对于配置文件所在目录的父目录
  - 不以 `/` 或 `~` 开头 → 视为相对路径
- [ ] TASK-1.5.3: 修改 `load_multi()` 传递配置文件目录
  - 每个配置来源记录其所在目录
  - 在变量替换时传递
- [ ] TASK-1.5.4: 添加单元测试
  - `{file:./instructions.md}` 相对于配置文件目录
  - `{file:../shared/config.md}` 支持上级目录
  - 无配置文件目录时的降级行为

**验收标准**:
- [ ] `{file:./instructions.md}` 相对于配置文件目录解析
- [ ] `{file:../shared/config.md}` 支持上级目录引用
- [ ] 相对路径解析在 `load_multi()` 中正确工作

**依赖**: 无

---

## 4. Phase 2: P2 完善性问题

### Task 2.1: 变量替换覆盖完整性

**ID**: TASK-2.1
**优先级**: P2
**模块**: core/config
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-040
**关联 Constitution**: C-019

**目标**: 确保所有配置加载路径都正确执行变量替换。

**子任务**:
- [ ] TASK-2.1.1: 审查 `load_multi()` 所有配置加载路径
  - 列出所有来源: remote, global, custom, project, opencode-dir, inline
  - 确认每个来源是否执行变量替换
- [ ] TASK-2.1.2: 统一变量替换执行点
  - 在 JSON 解析前对原始字符串执行替换
  - 确保所有路径使用同一替换函数
- [ ] TASK-2.1.3: 修正未设置变量处理
  - `{env:VAR}` 未设置时替换为 `""`
  - 当前行为 (保留原字符串) 改为空字符串
- [ ] TASK-2.1.4: 处理嵌套/复杂变量替换
  - `{env:PREFIX}_{env:SUFFIX}` 分别替换
  - 嵌套 `{file:{env:PATH}}` 递归替换 (可选)
- [ ] TASK-2.1.5: 添加集成测试
  - 所有配置来源的变量替换测试
  - 未设置变量测试
  - 嵌套变量测试

**验收标准**:
- [ ] 所有配置加载路径执行变量替换
- [ ] 未设置变量替换为空字符串
- [ ] 变量替换在原始字符串层面进行
- [ ] 嵌套/复杂变量替换正确处理

**依赖**: TASK-1.4, TASK-1.5

---

### Task 2.2: theme/keybinds 从主配置迁移

**ID**: TASK-2.2
**优先级**: P2
**模块**: core/config
**状态**: completed
**预计工期**: 1 天
**关联 FR**: FR-041
**关联 Constitution**: C-017

**目标**: 将 `theme` 和 `keybinds` 配置项从主 Config 结构体迁移到 TUI 配置。

**子任务**:
- [ ] TASK-2.2.1: 在主 Config 中标记废弃字段
  - `theme` 添加 `#[deprecated(since = "3.0.0", note = "Move to tui.json")]`
  - `keybinds` 添加 `#[deprecated(...)]`
- [ ] TASK-2.2.2: 实现废弃警告日志
  - 加载时检测到 theme/keybinds 输出 warning
  - 提示迁移到 tui.json
- [ ] TASK-2.2.3: 实现自动迁移
  - 旧配置值合并到 TUI 配置
  - TUI 配置值优先
- [ ] TASK-2.3.4: 在 TuiConfig 中添加对应字段
  - `theme`: 主题名称或路径
  - `keybinds`: 自定义快捷键对象
- [ ] TASK-2.2.5: 添加单元测试
  - 废弃警告测试
  - 自动迁移测试

**验收标准**:
- [ ] 主配置中 theme/keybinds 发出废弃警告
- [ ] 旧配置值自动迁移到 TUI 配置
- [ ] TUI 配置正确加载 theme/keybinds

**依赖**: TASK-0.2 (TUI 配置分离)

---

### Task 2.3: AgentMapConfig 完全动态 HashMap

**ID**: TASK-2.3
**优先级**: P2
**模块**: core/config
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-042

**目标**: 将 `AgentMapConfig` 从固定键设计改为完全动态的 HashMap。

**子任务**:
- [ ] TASK-2.3.1: 移除预设固定键
  - 移除 plan/build/general/explore 预设
  - 移除 `#[serde(flatten)]` custom 字段
- [ ] TASK-2.3.2: 使用完全动态 HashMap
  - `agents: HashMap<String, AgentConfig>`
  - 实现自定义 `Deserialize` 和 `Serialize`
- [ ] TASK-2.3.3: 确保 default_agent 兼容
  - `default_agent` 字段指向 HashMap 中的 key
  - 验证 default_agent 存在于 agents 中
- [ ] TASK-2.3.4: 添加单元测试
  - 任意 agent 名称解析测试
  - default_agent 验证测试
  - 空 agents map 测试

**验收标准**:
- [ ] 支持任意 agent 名称作为 key
- [ ] 无固定键限制
- [ ] 自定义 agent 正确加载

**依赖**: 无

---

### Task 2.4: JSON Schema 远程验证实现

**ID**: TASK-2.4
**优先级**: P2
**模块**: core/config/schema
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-043

**目标**: 实现真正的 JSON Schema 远程验证。

**子任务**:
- [ ] TASK-2.4.1: 读取配置文件中的 `$schema` 字段
  - 解析 JSON 获取 `$schema` URL
  - 验证 URL 格式
- [ ] TASK-2.4.2: 实现远程 schema 拉取
  - 使用 `reqwest` 或 `ureq` 发起 HTTP GET
  - 设置超时 (5 秒)
  - 处理网络错误
- [ ] TASK-2.4.3: 实现本地缓存机制
  - 缓存目录: `~/.config/opencode/schemas/`
  - 缓存文件名: URL hash 或路径提取
  - 缓存过期策略 (可选)
- [ ] TASK-2.4.4: 实现内建 fallback schema
  - 编译时嵌入默认 schema (使用 `include_str!`)
  - 网络不可用且无缓存时使用
- [ ] TASK-2.4.5: 使用 `jsonschema` crate 验证
  - 添加 `jsonschema` 依赖
  - 实现 `validate_json_schema(schema, config)` 函数
  - 输出详细验证错误 (路径 + 错误信息)
- [ ] TASK-2.4.6: 实现离线模式
  - 网络不可用时使用缓存
  - 无缓存时使用内建 schema
  - 不阻断配置加载
- [ ] TASK-2.4.7: 添加单元测试
  - 远程拉取测试 (mock HTTP)
  - 缓存测试
  - 验证错误输出测试
  - 离线模式测试

**验收标准**:
- [ ] 从远程 URL 拉取 JSON Schema
- [ ] 本地缓存机制正常
- [ ] 内建 fallback schema 可用
- [ ] 验证错误提示详细
- [ ] 离线模式不阻断配置加载

**依赖**: 无

---

### Task 2.5: 技术债务清理

**ID**: TASK-2.5
**优先级**: P2
**模块**: core/config (多处)
**状态**: completed
**预计工期**: 2 天
**关联 FR**: 技术债务清单

**目标**: 解决 gap analysis 中识别的关键技术债务。

**子任务**:
- [ ] TASK-2.5.1: merge_configs 优化
  - 避免 JSON 中转 (序列化→deep_merge→反序列化)
  - 直接在 `serde_json::Value` 上 deep_merge
- [ ] TASK-2.5.2: fetch_remote_config 异步化
  - 移除 `tokio::runtime::Runtime::new().unwrap().block_on()`
  - 改为纯异步函数 `async fn fetch_remote_config()`
  - 更新所有调用点为 `.await`
- [ ] TASK-2.5.3: TimeoutConfig 枚举重命名
  - `Disabled(bool)` → `NoTimeout`
  - 更新所有匹配点
- [ ] TASK-2.5.4: DirectoryScanner glob 支持
  - 添加 `glob` crate 依赖
  - 支持 `*.md` 等 glob 模式匹配
  - 保持向后兼容 (手动 read_dir 仍可用)
- [ ] TASK-2.5.5: 测试文件命名规范化
  - 统一测试文件命名
  - 移动集成测试到 `tests/` 目录

**验收标准**:
- [ ] merge_configs 不再通过 JSON 中转
- [ ] fetch_remote_config 为纯异步
- [ ] TimeoutConfig 枚举命名清晰
- [ ] DirectoryScanner 支持 glob 模式
- [ ] 测试文件命名规范

**依赖**: 无 (可与其他任务并行)

---

## 5. 任务状态追踪

| Phase | Task ID | 任务名称 | 状态 | 优先级 | 预计工期 | 关联 FR |
|-------|---------|----------|------|--------|----------|---------|
| 0 | TASK-0.1 | OPENCODE_TUI_CONFIG 环境变量 | ⬜ pending | P0 | 2d | FR-033 |
| 0 | TASK-0.2 | TUI 配置分离为独立文件 | ⬜ pending | P0 | 3d | FR-034 |
| 1 | TASK-1.1 | .opencode/ 目录扫描集成 | ⬜ pending | P1 | 2d | FR-039 |
| 1 | TASK-1.2 | modes/ 目录扫描 | ⬜ pending | P1 | 2d | FR-035 |
| 1 | TASK-1.3 | 配置路径命名统一 | ⬜ pending | P1 | 2d | FR-036 |
| 1 | TASK-1.4 | {file:path} ~ 路径展开 | ⬜ pending | P1 | 1d | FR-037 |
| 1 | TASK-1.5 | {file:path} 相对路径支持 | ⬜ pending | P1 | 2d | FR-038 |
| 2 | TASK-2.1 | 变量替换覆盖完整性 | ⬜ pending | P2 | 2d | FR-040 |
| 2 | TASK-2.2 | theme/keybinds 迁移 | ⬜ pending | P2 | 1d | FR-041 |
| 2 | TASK-2.3 | AgentMapConfig 动态化 | ⬜ pending | P2 | 2d | FR-042 |
| 2 | TASK-2.4 | JSON Schema 远程验证 | ⬜ pending | P2 | 3d | FR-043 |
| 2 | TASK-2.5 | 技术债务清理 | ⬜ pending | P2 | 2d | 技术债务 |

**总计**: 12 tasks, 24 人天 | ⬜ 0 completed | 12 pending

---

## 6. 依赖关系图

```
Phase 0 (P0):
  TASK-0.1 (OPENCODE_TUI_CONFIG)
      ↓
  TASK-0.2 (TUI 配置分离)

Phase 1 (P1):
  TASK-1.2 (modes/) ──┐
                      ├──→ TASK-1.1 (目录扫描集成)
  TASK-1.3 (路径命名)─┘

  TASK-1.4 (~ 展开) ──┐ (可与 1.2/1.3 并行)
                      └──→ (独立)
  TASK-1.5 (相对路径)─┘

Phase 2 (P2):
  TASK-0.2 ──→ TASK-2.2 (theme/keybinds 迁移)
  TASK-1.4 ──┐
             ├──→ TASK-2.1 (变量替换完整性)
  TASK-1.5 ──┘

  TASK-2.3 (AgentMap 动态化)  (独立)
  TASK-2.4 (Schema 验证)      (独立)
  TASK-2.5 (技术债务)         (独立，可并行)
```

---

## 7. 验收检查清单

每个任务完成后需满足:

- [ ] 功能正常运行
- [ ] 错误处理正确
- [ ] 性能满足要求
- [ ] 文档完整
- [ ] 测试覆盖
- [ ] `cargo check --workspace` 无错误
- [ ] `cargo test` 新增测试通过

---

## 8. PRD 验收标准对照

| PRD 验收项 | § | 关联 Task | 状态 |
|-----------|---|-----------|------|
| 6 个配置位置按优先级加载 | 10.1 | TASK-0.1, TASK-1.1 | ⬜ 待修复 |
| `{file:path}` 正确读取文件 | 10.2 | TASK-1.4, TASK-1.5, TASK-2.1 | ⬜ 待修复 |
| 未设置变量替换为空字符串 | 10.2 | TASK-2.1 | ⬜ 待修复 |
| TUI 配置与 runtime 分离 | 10.3 | TASK-0.2 | ⬜ 待修复 |
| `OPENCODE_TUI_CONFIG` 自定义路径 | 10.3 | TASK-0.1 | ⬜ 待修复 |

---

**文档状态**: 草稿
**下一步**: 开始 Phase 0 实施 (TASK-0.1: OPENCODE_TUI_CONFIG 环境变量)
