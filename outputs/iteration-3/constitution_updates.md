# Constitution 更新建议 (v1.3)

## 背景

Iteration-3 Gap Analysis 针对 PRD-OpenCode-Configuration.md 对 Rust 实现进行了全面审查，发现 ~75-80% 的实现完整度。本次 Constitution 更新聚焦于 **Configuration System** 领域的新 P0/P1 问题。

---

## 一、Iteration-3 P0/P1 问题 vs Constitution 覆盖度

### P0 问题对照

| Iteration-3 P0 | Constitution 覆盖 | 状态 |
|---|---|---|
| P0-1: 缺少 `OPENCODE_TUI_CONFIG` 环境变量 | ⚠️ C-012 仅覆盖通用 `{env:VAR}` 替换，未规定此特定 env var | **需新增 C-017** |
| P0-2: TUI 配置未分离为独立 tui.json | ❌ 无任何条款约束 TUI/runtime 配置分离 | **需新增 C-017** |

### P1 问题对照

| Iteration-3 P1 | Constitution 覆盖 | 状态 |
|---|---|---|
| P1-3: 缺少 `modes/` 目录扫描 | ⚠️ C-013 规定目录结构但未枚举 modes/ | **需细化 C-013** |
| P1-4: 配置路径命名 `opencode-rs` vs `opencode` | ❌ 无条款约束路径命名规范 | **需新增 C-018** |
| P1-5: `{file:path}` 不支持 `~` 展开 | ⚠️ C-012 仅覆盖 `{env:VAR}` | **需新增 C-019** |
| P1-6: `{file:path}` 不支持相对路径 | ❌ 无条款约束文件引用语义 | **需新增 C-019** |
| P1-7: `.opencode/` 目录扫描未集成到配置加载 | ⚠️ C-013 规定目录结构但未规定加载集成 | **需细化 C-013** |

---

## 二、新增 Constitution 条款

### 2.1 TUI 配置分离 (C-017) — 覆盖 P0-1 & P0-2

**条款 C-017: TUI 配置与 Runtime 配置分离**

```
1. 配置分离原则:
   a) Runtime 配置 (config.json/config.toml):
      - Provider 配置 (models, API keys, timeout)
      - Agent 配置 (agents, default_agent)
      - Permission 配置 (permissions)
      - MCP 配置 (mcp servers)
      - 项目级配置 (.opencode/config.json)

   b) TUI 配置 (tui.json):
      - 主题配置 (theme, colors, typography)
      - 键盘绑定 (keybinds)
      - TUI 布局偏好 (sidebar, panel sizes)
      - TUI 行为 (auto-save, notification preferences)

2. 独立文件规范:
   a) TUI 配置文件名: tui.json
   b) TUI JSON Schema: "$schema": "https://opencode.ai/tui.json"
   c) Runtime JSON Schema: "$schema": "https://opencode.ai/config.json"
   d) 两套配置使用不同的 $schema URL，确保校验隔离

3. 配置加载优先级 (6 个位置，按优先级降序):
   优先级 1: OPENCODE_CONFIG 环境变量指定的路径
   优先级 2: OPENCODE_TUI_CONFIG 环境变量指定的 TUI 配置路径 (新增)
   优先级 3: 当前工作目录下的 .opencode/config.json
   优先级 4: 当前工作目录下的 opencode.json
   优先级 5: 用户全局配置目录 (~/.config/opencode/config.json)
   优先级 6: 用户全局 TUI 配置 (~/.config/opencode/tui.json) (新增)

4. OPENCODE_TUI_CONFIG 环境变量:
   a) 用途: 自定义 TUI 配置文件路径
   b) 格式: 支持绝对路径和 ~ 展开
   c) 优先级: 高于默认 TUI 配置路径，低于 OPENCODE_CONFIG
   d) 未设置时: 使用默认路径 ~/.config/opencode/tui.json
   e) 文件不存在时: 不报错，使用 TUI 默认值

5. 合并策略:
   a) Runtime 配置与 TUI 配置独立合并，互不影响
   b) 各自遵循 deep_merge 策略 (低优先级 → 高优先级)
   c) 禁止 TUI 配置中出现 runtime 字段 (provider, agent, mcp, permission)
   d) 禁止 Runtime 配置中出现 TUI 字段 (theme, keybinds)

6. 废弃声明:
   a) 主配置中的 theme 字段已废弃，迁移至 tui.json
   b) 主配置中的 keybinds 字段已废弃，迁移至 tui.json
   c) 实现应保留向后兼容读取，但发出 deprecation warning
```

### 2.2 配置路径命名规范 (C-018) — 覆盖 P1-4

**条款 C-018: 配置路径与目录命名规范**

```
1. 全局配置目录命名:
   a) 必须使用: ~/.config/opencode/ (而非 opencode-rs)
   b) 原因: 与官方 OpenCode 生态保持一致，确保配置可移植
   c) 实现: 使用 directories crate 的 ProjectDirs 统一获取路径

2. 配置文件命名:
   a) 主配置: config.json (首选) 或 config.jsonc (支持注释)
   b) TUI 配置: tui.json
   c) 项目配置: .opencode/config.json
   d) 不推荐使用 .toml 格式 (PRD 要求 JSON/JSONC)

3. 路径获取方式:
   a) 禁止硬编码路径字符串 (如 "~/.config/opencode-rs/config.toml")
   b) 必须使用 directories crate 或等价的平台适配库
   c) macOS: ~/Library/Application Support/opencode/
   d) Linux:   ~/.config/opencode/
   e) Windows: %APPDATA%\opencode\

4. 格式检测:
   a) 优先通过文件扩展名检测 (.json / .jsonc / .toml)
   b) OPENCODE_CONFIG_FORMAT 环境变量为临时兼容方案，不应作为默认
   c) 默认格式: JSONC (支持注释的 JSON)
```

### 2.3 文件引用变量语义 (C-019) — 覆盖 P1-5 & P1-6

**条款 C-019: 配置变量替换语义**

```
1. 支持的变量类型:
   a) {env:VAR_NAME} - 环境变量替换
   b) {file:path}    - 文件内容引用

2. {env:VAR_NAME} 语义:
   a) 环境变量存在: 替换为变量值
   b) 环境变量不存在: 替换为空字符串 "" (而非保留原字符串)
   c) 支持嵌套: {env:PREFIX}_{env:SUFFIX} → 分别替换后拼接

3. {file:path} 语义:
   a) 路径解析优先级:
      i.  绝对路径: 直接使用
      ii. ~ 开头: 展开为用户 home 目录 (如 {file:~/.secrets/api-key})
      iii. ./ 或 ../ 开头: 相对于配置文件所在目录
      iv.  其他: 相对于当前工作目录
   b) 文件不存在: 替换为空字符串 ""，并记录 warning 日志
   c) 文件读取失败 (权限等): 替换为空字符串 ""，并记录 error 日志
   d) 编码: 默认 UTF-8，二进制文件行为未定义

4. 变量替换执行时机:
   a) 必须在 JSON 解析之前，在原始字符串层面执行
   b) load_multi() 所有加载路径必须统一执行变量替换
   c) 禁止部分路径执行替换、部分路径不执行替换

5. 安全约束:
   a) {file:path} 禁止递归引用 (文件内容中不再次解析 {file:} 或 {env:})
   b) 文件大小限制: 单文件引用不超过 1MB
   c) 禁止引用 /proc, /sys, /dev 等特殊文件系统路径
```

---

## 三、现有条款细化

### 3.1 C-013 细化: 目录结构扩展

**原 C-013** 规定了 `.opencode/` 目录结构，需补充以下内容:

```
C-013 补充条款:

1. 完整目录结构 (7 个子目录):
   .opencode/
   ├── config.json          # 项目级配置
   ├── agents/              # Agent 定义 (*.json)
   ├── commands/            # 自定义命令 (*.json)
   ├── skills/              # Skills 定义 (目录或 *.json)
   ├── tools/               # 工具配置 (*.json)
   ├── themes/              # 主题文件 (*.json)
   ├── modes/               # 模式定义 (新增, *.json)  ← P1-3
   └── plugins/             # 插件配置 (*.json)

2. modes/ 目录规范:
   a) 用途: 定义预配置的交互模式 (如 code-review, debug, brainstorm)
   b) 文件格式: JSON，每个模式一个文件
   c) 加载时机: 与 agents/commands/skills 同等优先级，在 load_multi() 中统一扫描
   d) 模式内容: 包含 system prompt 模板、默认 agent、权限覆盖

3. 目录扫描集成:
   a) load_opencode_directory() 必须被 load_multi() 调用
   b) 扫描结果必须合并到最终配置中 (遵循 deep_merge)
   c) 扫描失败 (目录不存在/权限不足) 不阻断配置加载，仅记录 warning
   d) 禁止仅检查 .opencode/config.json 文件而忽略子目录
```

---

## 四、条款更新映射

### 新增条款 (v1.3)

| 条款 | 模块 | 覆盖问题 |
|------|------|----------|
| C-017 | TUI 配置分离 | P0-1 (OPENCODE_TUI_CONFIG), P0-2 (tui.json 分离) |
| C-018 | 配置路径命名 | P1-4 (opencode-rs vs opencode) |
| C-019 | 文件引用变量 | P1-5 (~ 展开), P1-6 (相对路径), P1-7 (变量替换覆盖) |

### 细化条款

| 条款 | 变更类型 | 说明 |
|------|----------|------|
| C-013 | 细化 | 新增 modes/ 目录、目录扫描集成要求 |

### 保持不变

| 条款 | 说明 |
|------|------|
| C-001 | 已废止 (被 C-016 替代) |
| C-002 ~ C-012 | 不受本次更新影响 |
| C-014 ~ C-016 | v1.2 新增，保持不变 |

---

## 五、验证清单 (v1.3 新增)

### C-017: TUI 配置分离
- [ ] tui.json 是否使用独立 $schema: "https://opencode.ai/tui.json"
- [ ] OPENCODE_TUI_CONFIG 环境变量是否被正确读取
- [ ] TUI 配置与 Runtime 配置是否独立合并
- [ ] 主配置中的 theme/keybinds 是否发出 deprecation warning
- [ ] TUI 配置中是否禁止出现 runtime 字段

### C-018: 配置路径命名
- [ ] 配置目录是否使用 ~/.config/opencode/ (非 opencode-rs)
- [ ] 是否使用 directories crate 而非硬编码路径
- [ ] 默认格式是否为 JSONC

### C-019: 文件引用变量
- [ ] {env:VAR} 未设置时是否替换为空字符串
- [ ] {file:~/.path} 是否正确展开 ~ 为 home 目录
- [ ] {file:./relative.md} 是否相对于配置文件目录解析
- [ ] 变量替换是否在 JSON 解析前执行
- [ ] load_multi() 所有路径是否统一执行变量替换

### C-013 细化
- [ ] modes/ 目录是否被扫描
- [ ] load_opencode_directory() 是否被 load_multi() 调用
- [ ] 目录扫描失败是否不阻断配置加载

---

## 六、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0 | 2026-04-04 | 初始版本，覆盖 Context/Plugin/Skills/Commands/MCP |
| 1.1 | 2026-04-04 | 新增 Config System 条款 (C-011, C-012, C-013) |
| 1.2 | 2026-04-04 | 新增 TUI Input Parser (C-014), Session Fork (C-015), Context Token Budget (C-016) |
| **1.3** | **2026-04-04** | **新增 TUI 配置分离 (C-017), 路径命名 (C-018), 文件引用变量 (C-019), 细化 C-013** |

---

## 七、设计决策约束 (更新版)

| 设计决策 | 必须遵循条款 |
|----------|-------------|
| Config 系统实现 | C-011, C-012, C-013 (含 modes/), C-017, C-018, C-019 |
| TUI 配置实现 | C-014, C-017 |
| 变量替换实现 | C-012, C-019 |
| 目录扫描实现 | C-013 (细化版) |
| 路径获取实现 | C-018 |

---

## 八、P2 技术债务 — Constitution 建议 (非约束性)

以下 P2 问题暂不纳入 Constitution 约束，但建议后续迭代关注:

| P2 问题 | 建议 | 理由 |
|---------|------|------|
| JSON Schema 远程验证未实现 | 后续版本增加 C-020 | 当前为功能缺失，非设计约束 |
| AgentMapConfig 固定键 | 后续版本修订 C-011 | 当前可用 custom flatten 绕过 |
| merge_configs JSON 中转 | 代码重构关注 | 实现细节，非架构约束 |
| fetch_remote_config 同步包装 | 代码重构关注 | 实现细节 |
| 测试覆盖不足 | 工程实践关注 | 属于工程质量，非设计约束 |

---

*本文档作为 OpenCode-RS 项目的 Constitution v1.3 更新建议，聚焦 Configuration System 领域的 P0/P1 问题覆盖。*
