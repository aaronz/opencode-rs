# OpenCode 产品配置 PRD 文档

**文档版本**：1.0  
**创建日期**：2026年3月31日  
**文档状态**：初稿  
**产品名称**：OpenCode 配置管理系统  
**目标读者**：产品经理、开发团队、测试团队

---

## 1. 产品概述

### 1.1 产品背景

OpenCode 是一款由 Anomaly 公司开发的 AI 编程助手，支持多种编程语言和开发环境。为了满足不同用户群体（个人开发者、企业团队）的多样化需求，OpenCode 采用可配置的架构设计，允许用户通过 JSON 配置文件自定义各项功能和行为。

### 1.2 产品定位

OpenCode 配置系统是一个灵活、可扩展的配置管理框架，支持：

- 多层级配置（远程、全局、项目级）
- 多种配置格式（JSON/JSONC）
- 运行时变量替换
- 配置继承与覆盖机制

### 1.3 目标用户

| 用户类型 | 使用场景 | 配置需求 |
|---------|---------|---------|
| 个人开发者 | 本地开发环境配置 | 主题、模型、快捷键 |
| 企业团队 | 统一团队开发规范 | 远程配置、代理、规则 |
| 项目维护者 | 项目特定配置 | 项目级配置、自定义工具 |

---

## 2. 功能需求

### 2.1 配置格式支持

**FR-001：JSON 格式支持**

- OpenCode 必须支持纯 JSON 格式的配置文件
- 配置文件应放置在 `opencode.json` 文件中
- 应包含 Schema 验证支持

**FR-002：JSONC 格式支持**

- OpenCode 必须支持带注释的 JSON（JSONC）格式
- 允许在配置文件中使用 `//` 和 `/* */` 风格的注释
- 示例：`opencode.jsonc`

> **验收标准**：以下配置应被正确解析
> ```jsonc
> {
>   "$schema": "https://opencode.ai/config.json",
>   "model": "anthropic/claude-sonnet-4-5", // 主模型
>   "autoupdate": true,
> }
> ```

### 2.2 配置位置管理

**FR-003：多层级配置加载**

系统必须按照以下优先级顺序加载配置：

1. **远程配置**（`.well-known/opencode`）- 组织级默认值
2. **全局配置**（`~/.config/opencode/opencode.json`）- 用户偏好
3. **自定义配置**（`OPENCODE_CONFIG` 环境变量）- 自定义覆盖
4. **项目配置**（项目中的 `opencode.json`）- 项目特定设置
5. **`.opencode` 目录** - 代理、命令、插件
6. **内联配置**（`OPENCODE_CONFIG_CONTENT` 环境变量）- 运行时覆盖

**FR-004：配置合并机制**

- 配置文件应合并而非替换
- 后续配置仅在键冲突时覆盖前面的配置
- 非冲突设置应全部保留

> **验收标准**：全局配置设置 `autoupdate: true`，项目配置设置 `model: "anthropic/claude-sonnet-4-5"`，最终配置应同时包含这两个设置。

**FR-005：远程配置支持**

- 组织可通过 `.well-known/opencode` 端点提供默认配置
- 用户认证后自动获取远程配置
- 远程配置可被本地配置覆盖

**FR-006：全局配置支持**

- 全局配置文件路径：`~/.config/opencode/opencode.json`
- 用于设置用户级别偏好（主题、提供商、快捷键）

**FR-007：项目级配置支持**

- 项目配置文件路径：项目根目录的 `opencode.json`
- 最高优先级，可覆盖全局和远程配置
- 可安全提交到 Git 仓库

**FR-008：自定义配置路径**

- 支持通过 `OPENCODE_CONFIG` 环境变量指定自定义配置路径
- 优先级位于全局配置和项目配置之间

**FR-009：自定义配置目录**

- 支持通过 `OPENCODE_CONFIG_DIR` 环境变量指定配置目录
- 用于加载代理、命令、模式和插件

### 2.3 Schema 配置

#### 2.3.1 TUI 配置

**FR-010：TUI 滚动设置**

- 支持 `scroll_speed` 自定义滚动速度（默认值：3，最小值：1）
- 支持 `scroll_acceleration.enabled` 启用 macOS 风格滚动加速

**FR-011：差异显示风格**

- 支持 `diff_style` 配置差异渲染方式
- 可选值：`"auto"`（自适应终端宽度）、`"stacked"`（单列显示）

#### 2.3.2 服务器配置

**FR-012：服务器端口和主机名**

- 支持 `server.port` 配置监听端口
- 支持 `server.hostname` 配置监听主机名

**FR-013：mDNS 服务发现**

- 支持 `server.mdns` 启用 mDNS 服务发现
- 支持 `server.mdnsDomain` 自定义 mDNS 域名

**FR-014：CORS 配置**

- 支持 `server.cors` 配置允许的 CORS 来源

#### 2.3.3 工具配置

**FR-015：工具启用/禁用**

- 支持通过 `tools` 选项管理 LLM 工具
- 可禁用 `write`、`edit`、`bash` 等工具

#### 2.3.4 模型配置

**FR-016：主模型配置**

- 支持 `model` 选项设置主模型
- 示例：`"anthropic/claude-sonnet-4-5"`

**FR-017：小模型配置**

- 支持 `small_model` 选项为轻量级任务配置单独模型
- 用于标题生成等任务

**FR-018：提供商选项配置**

- 支持配置 `timeout` 请求超时（毫秒，默认：300000）
- 支持配置 `setCacheKey` 设置缓存键
- 支持 Amazon Bedrock 的 AWS 特定配置（region、profile、endpoint）

#### 2.3.5 主题配置

**FR-019：主题选择**

- 支持 `theme` 选项设置 UI 主题

#### 2.3.6 代理配置

**FR-020：自定义代理定义**

- 支持通过 `agent` 选项定义专用代理
- 包含 description、model、prompt、tools 配置
- 支持使用 Markdown 文件定义（`~/.config/opencode/agents/` 或 `.opencode/agents/`）

**FR-021：默认代理设置**

- 支持 `default_agent` 选项设置默认代理
- 可选内置代理（`build`、`plan`）或自定义代理

#### 2.3.7 分享配置

**FR-022：分享功能配置**

- 支持 `share` 选项配置分享功能
- 可选值：`"manual"`（手动）、`"auto"`（自动）、`"disabled"`（禁用）

#### 2.3.8 命令配置

**FR-023：自定义命令定义**

- 支持通过 `command` 选项定义自定义命令
- 包含 template、description、agent、model 配置
- 支持使用 Markdown 文件定义（`~/.config/opencode/commands/` 或 `.opencode/commands/`）

#### 2.3.9 快捷键配置

**FR-024：快捷键自定义**

- 支持通过 `keybinds` 选项自定义快捷键

#### 2.3.10 自动更新配置

**FR-025：自动更新设置**

- 支持 `autoupdate` 选项控制自动更新
- 可选值：`true`（启用）、`false`（禁用）、`"notify"`（通知）

#### 2.3.11 格式化程序配置

**FR-026：格式化程序设置**

- 支持通过 `formatter` 选项配置代码格式化程序
- 支持自定义格式化程序命令和环境变量

#### 2.3.12 权限配置

**FR-027：工具权限控制**

- 支持通过 `permission` 选项控制工具权限
- 可选值：`"allow"`（允许）、`"ask"`（询问）、`"deny"`（拒绝）

#### 2.3.13 压缩配置

**FR-028：上下文压缩设置**

- 支持 `compaction.auto` 自动压缩会话
- 支持 `compaction.prune` 删除旧工具输出
- 支持 `compaction.reserved` Token 缓冲区设置

#### 2.3.14 文件监视器配置

**FR-029：文件监视忽略模式**

- 支持通过 `watcher.ignore` 配置忽略模式
- 使用 glob 语法

#### 2.3.15 MCP 服务器配置

**FR-030：MCP 服务器集成**

- 支持通过 `mcp` 选项配置 MCP 服务器

#### 2.3.16 插件配置

**FR-031：插件支持**

- 支持通过 `plugin` 选项加载 npm 插件
- 支持 `.opencode/plugins/` 和 `~/.config/opencode/plugins/` 目录

#### 2.3.17 指令配置

**FR-032：模型指令设置**

- 支持 `instructions` 选项配置模型指令
- 接受文件路径和 glob 模式数组

#### 2.3.18 提供商管理

**FR-033：禁用提供商**

- 支持 `disabled_providers` 选项禁用指定提供商

**FR-034：启用提供商白名单**

- 支持 `enabled_providers` 选项设置提供商白名单
- `disabled_providers` 优先于 `enabled_providers`

#### 2.3.19 实验性功能

**FR-035：实验性功能开关**

- 支持 `experimental` 键包含实验性选项

### 2.4 变量替换

**FR-036：环境变量替换**

- 支持使用 `{env:VARIABLE_NAME}` 语法引用环境变量
- 未设置的环境变量替换为空字符串

**FR-037：文件内容替换**

- 支持使用 `{file:path/to/file}` 语法引用文件内容
- 支持相对路径和绝对路径（以 `/` 或 `~` 开头）

---

## 3. 非功能需求

### 3.1 性能要求

| 指标 | 要求 |
|-----|------|
| 配置加载时间 | 单个配置文件加载时间 < 100ms |
| 配置合并性能 | 多层级配置合并时间 < 200ms |
| 内存占用 | 配置模块内存占用 < 50MB |

### 3.2 兼容性要求

- 支持 macOS、Windows、Linux 三大主流平台
- 支持 Node.js 18+ 运行环境
- 配置文件格式兼容 JSON 和 JSONC

### 3.3 安全要求

- API 密钥等敏感信息应支持通过文件引用（`{file:}`）管理
- 不应在日志中打印敏感配置信息
- 远程配置获取应支持 HTTPS 协议

### 3.4 可扩展性要求

- 配置 Schema 应支持向后兼容
- 新增配置项不应影响现有配置
- 插件系统应支持第三方扩展

---

## 4. 用户流程

### 4.1 首次配置流程

```
用户安装 OpenCode
    ↓
启动 OpenCode（无配置文件）
    ↓
加载默认配置（内嵌默认值）
    ↓
检查远程配置（已认证用户）
    ↓
加载全局配置（如存在）
    ↓
加载项目配置（如存在）
    ↓
合并所有配置
    ↓
启动完成
```

### 4.2 配置修改流程

```
用户编辑配置文件
    ↓
保存配置文件
    ↓
重启 OpenCode / 重新加载配置
    ↓
验证新配置
    ↓
配置生效
```

---

## 5. 配置示例

### 5.1 最小配置

```json
{
  "$schema": "https://opencode.ai/config.json"
}
```

### 5.2 完整配置示例

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "model": "anthropic/claude-sonnet-4-5",
  "small_model": "anthropic/claude-haiku-4-5",
  "autoupdate": true,
  "tui": {
    "scroll_speed": 3,
    "scroll_acceleration": {
      "enabled": true
    },
    "diff_style": "auto"
  },
  "server": {
    "port": 4096,
    "hostname": "0.0.0.0",
    "mdns": true,
    "cors": ["http://localhost:5173"]
  },
  "tools": {
    "write": true,
    "edit": true,
    "bash": true
  },
  "theme": "",
  "default_agent": "build",
  "share": "manual",
  "compaction": {
    "auto": true,
    "prune": true,
    "reserved": 10000
  },
  "watcher": {
    "ignore": ["node_modules/**", "dist/**", ".git/**"]
  },
  "permission": {
    "edit": "allow",
    "bash": "ask"
  },
  "provider": {
    "anthropic": {
      "options": {
        "timeout": 600000
      }
    }
  }
}
```

### 5.3 环境变量引用示例

```json
{
  "$schema": "https://opencode.ai/config.json",
  "model": "{env:OPENCODE_MODEL}",
  "provider": {
    "anthropic": {
      "options": {
        "apiKey": "{env:ANTHROPIC_API_KEY}"
      }
    }
  }
}
```

### 5.4 文件引用示例

```json
{
  "$schema": "https://opencode.ai/config.json",
  "instructions": ["./custom-instructions.md"],
  "provider": {
    "openai": {
      "options": {
        "apiKey": "{file:~/.secrets/openai-key}"
      }
    }
  }
}
```

---

## 6. 术语表

| 术语 | 定义 |
|-----|------|
| JSONC | 带注释的 JSON 格式 |
| Schema | 配置文件的结构定义 |
| mDNS | 多播 DNS，用于局域网服务发现 |
| CORS | 跨域资源共享 |
| MCP | Model Context Protocol，模型上下文协议 |
| TUI | 终端用户界面 |

---

## 7. 参考资料

- OpenCode 官方配置文档：https://opencode.ai/docs/zh-cn/config/
- 配置 Schema：https://opencode.ai/config.json

---

## 8. 修订历史

| 版本 | 日期 | 修订内容 | 修订人 |
|-----|------|---------|--------|
| 1.0 | 2026-03-31 | 初稿创建 | - |
