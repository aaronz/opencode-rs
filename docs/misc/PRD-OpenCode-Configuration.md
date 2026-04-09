# OpenCode 配置系统 PRD

**文档版本**: 1.0  
**日期**: 2026-04-04  
**状态**: 完成

---

## 1. 概述

### 1.1 目的

本文档描述 OpenCode 配置系统的产品需求，涵盖配置格式、优先级、存储位置及完整配置项 schema。

### 1.2 适用范围

本文档适用于：
- OpenCode 配置文件的创建和维护
- 企业级 OpenCode 部署配置
- 自定义 agents、commands、plugins 的配置需求

---

## 2. 产品概述

OpenCode 是一个开源 AI 编码代理，支持终端界面、桌面应用和 IDE 扩展三种使用方式。

**核心特性**：
- 支持 JSON 和 JSONC（带注释的 JSON）配置格式
- 多层级配置合并机制
- 支持远程配置（Remote Config）
- 变量替换（环境变量、文件引用）

---

## 3. 配置格式

### 3.1 支持格式

| 格式 | 说明 | 文件扩展名 |
|------|------|-----------|
| JSON | 标准 JSON 格式 | `.json` |
| JSONC | 带注释的 JSON | `.jsonc` |

### 3.2 Schema 声明

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  // 配置文件内容
}
```

### 3.3 TUI 专用配置

TUI 相关配置使用独立 schema：

```jsonc
{
  "$schema": "https://opencode.ai/tui.json",
  "scroll_speed": 3,
  "theme": "tokyonight"
}
```

---

## 4. 配置位置与优先级

### 4.1 优先级顺序（从低到高）

| 优先级 | 位置 | 说明 |
|--------|------|------|
| 1 | Remote Config | `.well-known/opencode` - 组织级默认配置 |
| 2 | Global Config | `~/.config/opencode/opencode.json` - 用户全局配置 |
| 3 | Custom Config | `OPENCODE_CONFIG` 环境变量指定的路径 |
| 4 | Project Config | 项目根目录的 `opencode.json` |
| 5 | .opencode 目录 | `.opencode/` - agents, commands, plugins |
| 6 | Inline Config | `OPENCODE_CONFIG_CONTENT` 环境变量 |

**重要**：配置文件是**合并**而非替换，后加载的配置仅覆盖冲突键。

### 4.2 目录结构约定

`.opencode/` 和 `~/.config/opencode/` 使用**复数**命名：
- `agents/` - 自定义 agents
- `commands/` - 自定义命令
- `modes/` - 自定义模式
- `plugins/` - 插件
- `skills/` - 技能
- `tools/` - 工具
- `themes/` - 主题

### 4.3 环境变量

| 变量名 | 说明 |
|--------|------|
| `OPENCODE_CONFIG` | 自定义配置文件路径 |
| `OPENCODE_CONFIG_DIR` | 自定义配置目录路径 |
| `OPENCODE_TUI_CONFIG` | 自定义 TUI 配置文件路径 |
| `OPENCODE_CONFIG_CONTENT` | 内联配置内容（运行时覆盖） |

---

## 5. 配置 Schema 详解

### 5.1 Server 配置

```jsonc
{
  "server": {
    "port": 4096,              // 端口号
    "hostname": "0.0.0.0",      // 监听地址
    "mdns": true,              // 启用 mDNS 服务发现
    "mdnsDomain": "myproject.local",  // mDNS 域名
    "cors": ["http://localhost:5173"]  // CORS 允许的源
  }
}
```

### 5.2 Tools 配置

```jsonc
{
  "tools": {
    "write": false,   // 禁用 write 工具
    "bash": false     // 禁用 bash 工具
  }
}
```

### 5.3 Models 配置

```jsonc
{
  "provider": {},                    // Provider 配置
  "model": "anthropic/claude-sonnet-4-5",     // 主模型
  "small_model": "anthropic/claude-haiku-4-5"  // 轻量级模型（用于标题生成等）
}
```

**Provider 通用选项**：

```jsonc
{
  "provider": {
    "anthropic": {
      "options": {
        "timeout": 600000,        // 请求超时（毫秒），默认 300000
        "chunkTimeout": 30000,    // 流式响应块超时
        "setCacheKey": true       // 确保设置缓存键
      }
    }
  }
}
```

**Amazon Bedrock 专属配置**：

```jsonc
{
  "provider": {
    "amazon-bedrock": {
      "options": {
        "region": "us-east-1",                              // AWS 区域
        "profile": "my-aws-profile",                        // AWS 凭据配置文件名
        "endpoint": "https://bedrock-runtime.xxx.amazonaws.com"  // VPC 端点
      }
    }
  }
}
```

### 5.4 Agents 配置

```jsonc
{
  "agent": {
    "code-reviewer": {
      "description": "Reviews code for best practices",
      "model": "anthropic/claude-sonnet-4-5",
      "prompt": "You are a code reviewer...",
      "tools": {
        "write": false,
        "edit": false
      }
    }
  },
  "default_agent": "plan"  // 默认 agent
}
```

### 5.5 Commands 配置

```jsonc
{
  "command": {
    "test": {
      "template": "Run the full test suite...",
      "description": "Run tests with coverage",
      "agent": "build",
      "model": "anthropic/claude-haiku-4-5"
    }
  }
}
```

### 5.6 Share 配置

```jsonc
{
  "share": "manual"  // "manual" | "auto" | "disabled"
}
```

### 5.7 Formatters 配置

```jsonc
{
  "formatter": {
    "prettier": {
      "disabled": true
    },
    "custom-prettier": {
      "command": ["npx", "prettier", "--write", "$FILE"],
      "environment": {
        "NODE_ENV": "development"
      },
      "extensions": [".js", ".ts", ".jsx", ".tsx"]
    }
  }
}
```

### 5.8 Permissions 配置

```jsonc
{
  "permission": {
    "edit": "ask",   // edit 工具需要用户批准
    "bash": "ask"    // bash 工具需要用户批准
  }
}
```

权限值：`"allow"` | `"deny"` | `"ask"`

### 5.9 Compaction 配置

```jsonc
{
  "compaction": {
    "auto": true,      // 自动压缩会话
    "prune": true,     // 移除旧工具输出以节省 token
    "reserved": 10000  // 压缩时保留的 token 缓冲
  }
}
```

### 5.10 Watcher 配置

```jsonc
{
  "watcher": {
    "ignore": [
      "node_modules/**",
      "dist/**",
      ".git/**"
    ]
  }
}
```

### 5.11 MCP Servers 配置

```jsonc
{
  "mcp": {
    // MCP 服务器配置
  }
}
```

### 5.12 Plugins 配置

```jsonc
{
  "plugin": [
    "opencode-helicone-session",
    "@my-org/custom-plugin"
  ]
}
```

### 5.13 Instructions 配置

```jsonc
{
  "instructions": [
    "CONTRIBUTING.md",
    "docs/guidelines.md",
    ".cursor/rules/*.md"
  ]
}
```

### 5.14 Provider 控制配置

```jsonc
{
  "disabled_providers": ["openai", "gemini"],    // 禁用的 providers
  "enabled_providers": ["anthropic", "openai"]  // 白名单模式
}
```

**注意**：`disabled_providers` 优先级高于 `enabled_providers`

### 5.15 其他配置项

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `autoupdate` | `boolean \| "notify"` | `true` | 自动更新 |
| `snapshot` | `boolean` | `true` | 启用快照（用于撤销） |
| `experimental` | `object` | `{}` | 实验性功能 |

### 5.16 TUI 配置

```jsonc
{
  "$schema": "https://opencode.ai/tui.json",
  "scroll_speed": 3,
  "scroll_acceleration": {
    "enabled": true
  },
  "diff_style": "auto",
  "theme": "tokyonight",
  "keybinds": {}
}
```

---

## 6. 变量替换

### 6.1 环境变量

```jsonc
{
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

### 6.2 文件引用

```jsonc
{
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

文件路径支持：
- 相对于配置文件目录
- 绝对路径（以 `/` 或 `~` 开头）

---

## 7. 使用场景

### 7.1 个人开发者配置

**目标**：设置用户级别的默认模型和 provider

```jsonc
// ~/.config/opencode/opencode.json
{
  "$schema": "https://opencode.ai/config.json",
  "model": "anthropic/claude-sonnet-4-5",
  "autoupdate": true
}
```

### 7.2 企业组织配置

**目标**：通过 Remote Config 统一管理团队配置

```jsonc
// .well-known/opencode (服务端)
{
  "mcp": {
    "jira": {
      "type": "remote",
      "url": "https://jira.example.com/mcp",
      "enabled": false
    }
  }
}
```

### 7.3 项目特定配置

**目标**：为特定项目配置 formatter 和 instructions

```jsonc
// project/opencode.json
{
  "$schema": "https://opencode.ai/config.json",
  "formatter": {
    "prettier": {
      "disabled": true
    },
    "custom-eslint": {
      "command": ["npx", "eslint", "--fix", "$FILE"],
      "extensions": [".ts", ".tsx"]
    }
  },
  "instructions": ["./CODE_GUIDELINES.md"]
}
```

### 7.4 自定义 Agent 配置

**目标**：创建专门的代码审查 agent

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "agent": {
    "security-reviewer": {
      "description": "Security-focused code reviewer",
      "model": "anthropic/claude-sonnet-4-5",
      "prompt": "You are a security expert. Focus on vulnerabilities...",
      "tools": {
        "write": false,
        "edit": false,
        "bash": false
      }
    }
  },
  "default_agent": "security-reviewer"
}
```

---

## 8. 远程配置（Remote Config）

### 8.1 工作原理

1. 组织在 `.well-known/opencode` 端点提供默认配置
2. 用户认证支持该功能的 provider 时自动拉取
3. 作为基础层加载，后续配置可覆盖

### 8.2 应用场景

- 统一管理组织内的 MCP 服务器
- 强制开启特定安全策略
- 预配置支持的模型列表

---

## 9. 迁移说明

### 9.1 已废弃的配置

| 旧键 | 状态 | 迁移方式 |
|------|------|---------|
| `theme` (在 opencode.json) | 已废弃 | 移至 `tui.json` |
| `keybinds` (在 opencode.json) | 已废弃 | 移至 `tui.json` |
| `tui` (在 opencode.json) | 已废弃 | 移至 `tui.json` |
| `mode` | 已废弃 | 使用 `agent` 配置 |

**说明**：旧配置在可能时会自动迁移。

---

## 10. 验收标准

### 10.1 配置加载

- [ ] 支持 JSON 和 JSONC 格式
- [ ] 配置合并逻辑正确（later overrides earlier）
- [ ] 所有 6 个配置位置按优先级加载

### 10.2 变量替换

- [ ] `{env:VARIABLE_NAME}` 正确替换环境变量
- [ ] `{file:path}` 正确读取文件内容
- [ ] 未设置的变量替换为空字符串

### 10.3 TUI 配置

- [ ] TUI 专用配置与 runtime 配置分离
- [ ] 支持 `OPENCODE_TUI_CONFIG` 自定义路径

### 10.4 Provider 配置

- [ ] 通用 timeout、chunkTimeout、setCacheKey 选项生效
- [ ] Amazon Bedrock region、profile、endpoint 配置生效
- [ ] disabled_providers 优先级高于 enabled_providers

### 10.5 Agents & Commands

- [ ] 自定义 agent 配置正确加载
- [ ] default_agent 设置生效
- [ ] 命令模板中的变量替换正确

### 10.6 权限与安全

- [ ] permission 配置正确控制工具访问
- [ ] API Key 等敏感信息支持文件引用

---

## 11. 参考链接

- [官方配置文档](https://opencode.ai/docs/config/)
- [中文配置文档](https://opencode.ai/docs/zh-cn/config/)
- [Config Schema](https://opencode.ai/config.json)
- [TUI Schema](https://opencode.ai/tui.json)

---

## 12. 附录

### 12.1 完整配置示例

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  
  // 模型配置
  "model": "anthropic/claude-sonnet-4-5",
  "small_model": "anthropic/claude-haiku-4-5",
  
  // Provider 配置
  "provider": {
    "anthropic": {
      "options": {
        "timeout": 600000,
        "chunkTimeout": 30000,
        "setCacheKey": true
      }
    }
  },
  
  // 服务器配置
  "server": {
    "port": 4096,
    "hostname": "0.0.0.0",
    "mdns": true,
    "cors": ["http://localhost:5173"]
  },
  
  // 工具配置
  "tools": {
    "write": true,
    "bash": true
  },
  
  // 权限配置
  "permission": {
    "edit": "ask",
    "bash": "ask"
  },
  
  // 自动更新
  "autoupdate": true,
  
  // 快照
  "snapshot": true,
  
  // 压缩配置
  "compaction": {
    "auto": true,
    "prune": true,
    "reserved": 10000
  },
  
  // 文件监视器忽略
  "watcher": {
    "ignore": ["node_modules/**", "dist/**"]
  },
  
  // MCP 服务器
  "mcp": {},
  
  // 插件
  "plugin": [],
  
  // 指令文件
  "instructions": [],
  
  // 实验性功能
  "experimental": {}
}
```

### 12.2 TUI 完整配置示例

```jsonc
{
  "$schema": "https://opencode.ai/tui.json",
  
  "scroll_speed": 3,
  "scroll_acceleration": {
    "enabled": true
  },
  "diff_style": "auto",
  "theme": "tokyonight",
  "keybinds": {}
}
```
