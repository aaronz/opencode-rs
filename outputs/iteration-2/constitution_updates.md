# Constitution 更新建议

## 分析结论

**Constitution 不存在**。需要为 OpenCode-RS 项目创建 Constitution 以指导后续开发决策。

基于 Gap Analysis 识别出的 P0 阻断性问题，需要建立设计原则确保后续实现的一致性与可扩展性。

---

## 一、Constitution 框架

### 1.1 核心设计原则

| 原则 | 描述 | 优先级 |
|------|------|--------|
| 本地优先 | 所有数据默认存储在用户本地，不强制云端依赖 | P0 |
| 运行时抽象 | Agent Runtime 为核心，Client 可替换 | P0 |
| 权限最小化 | 默认拒绝，按需授权，可审计 | P0 |
| 可扩展架构 | 插件化加载，热更新能力 | P1 |
| 可观测性 | 结构化日志 + tracing，Session 可追溯 | P1 |

### 1.2 模块边界原则

```
┌─────────────────────────────────────────────────────┐
│                    Client Layer                      │
│   (TUI / Server / Web / IDE / SDK)                  │
├─────────────────────────────────────────────────────┤
│                   Core Runtime                       │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │
│  │ Context │ │ Session │ │ Agent   │ │Permission│   │
│  │ Engine  │ │ Engine  │ │ Engine  │ │ Engine  │    │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘    │
├─────────────────────────────────────────────────────┤
│                   Adapter Layer                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐    │
│  │ Tool    │ │ LLM     │ │ MCP     │ │ LSP     │    │
│  │ Runtime │ │ Gateway │ │ Bridge  │ │ Client  │    │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘    │
├─────────────────────────────────────────────────────┤
│                  Plugin System                       │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐               │
│  │  WASM   │ │ Skills   │ │Commands │               │
│  │ Runtime │ │ Loader   │ │ Parser  │               │
│  └─────────┘ └─────────┘ └─────────┘               │
└─────────────────────────────────────────────────────┘
```

---

## 二、P0 问题对应的 Constitution 条款

### 2.1 Context Engine (Token Budget + Ranking + Compaction)

**条款 C-001: Context 管理原则**

```
1. 每个 Session 必须维护独立的 Context Budget
2. Context Budget 默认值为 128K tokens，可配置
3. 超过 80% budget 时触发 compaction 评估
4. Compaction 策略优先级: summarize > compress > prune
5. 保留最近 N 轮完整对话 + 关键决策点
6. 用户可通过 @file 显式注入高优先级 Context
```

**条款 C-002: Context Ranking 算法**

```
1. 采用多维度评分: recency (0.4) + relevance (0.3) + importance (0.3)
2. Relevance 通过 embedding similarity 计算
3. Importance 基于工具调用结果、错误状态、用户确认标记
4. Ranking 在每次 LLM 调用前重新计算
5. 支持用户手动 boost 特定消息
```

### 2.2 Plugin System (WASM Runtime + Event Bus)

**条款 C-003: Plugin 架构原则**

```
1. Plugin 必须通过 WASM 加载，隔离执行环境
2. Plugin 生命周期: load -> init -> active -> unload
3. 每个 Plugin 拥有独立的 filesystem namespace 和 network policy
4. Plugin 权限通过 Capability 声明，运行时授予
5. 禁止 Plugin 直接访问 host 进程内存
```

**条款 C-004: Event Bus 设计**

```
1. Event Bus 采用发布-订阅模式
2. 事件类型: session_start, session_end, tool_call, tool_result,
             message, error, permission_request
3. Plugin 可订阅感兴趣的事件，但不可阻塞主流程
4. 事件携带结构化 payload，支持异步处理
5. Event History 保留最近 1000 条事件，供审计使用
```

### 2.3 Skills System (Loader + Semantic Matching)

**条款 C-005: Skills 加载原则**

```
1. Skills 定义为 YAML/JSON 配置文件，放置在 ~/.opencode/skills/
2. Skill 包含: name, description, triggers, actions, examples
3. 延迟加载: Session 启动时扫描 skill 目录，不预加载
4. 支持 skill 依赖声明，避免循环依赖
5. 内置 Skill 优先级高于用户自定义 Skill
```

**条款 C-006: Semantic Matching 算法**

```
1. 用户输入先进行 intent classification
2. Matching 策略: exact > prefix > fuzzy > semantic
3. Semantic matching 使用轻量 embedding (128维)
4. 每个 Skill 可指定 required_context_fields
5. Matching 结果附带 confidence score，<0.6 不触发
```

### 2.4 Commands System (Parser + Template Expansion)

**条款 C-007: Commands 设计原则**

```
1. Command 以 '/' 前缀识别，如 /refactor, /test, /explain
2. Command 定义支持参数: /command [arg1] [--flag value]
3. 支持 command alias 和 hotkey 映射
4. Command 可模板化，包含占位符替换
5. 内置 Commands 不可覆盖，用户可定义同名自定义
```

**条款 C-008: Template Expansion**

```
1. Template 语法: {{variable}} 或 {{#each}}...{{/each}}
2. 支持 context 变量: {{session.id}}, {{model.name}}, {{cursor.file}}
3. 支持条件: {{#if condition}}...{{/if}}
4. Template 预验证，错误模板拒绝注册
5. Expansion 在 command 执行前完成
```

### 2.5 MCP Integration (Schema Cache + Permission)

**条款 C-009: MCP 工具接入原则**

```
1. MCP Server 通过 stdio 或 HTTP 连接
2. 每个 MCP 工具必须声明 schema (input/output)
3. Schema 本地缓存，有效期 24 小时
4. MCP 工具调用经过 Permission Engine 审核
5. MCP 工具元信息 (description, examples) 延迟注入，按需加载
```

**条款 C-010: MCP Permission 集成**

```
1. MCP 工具属于 external tool 类型
2. 首次调用提示用户授权，并记住选择
3. 支持按工具粒度的 allow/deny
4. 支持 time-based expiration (e.g., 1 hour)
5. MCP 工具调用日志包含 tool_name, duration, success/failure
```

---

## 三、实施优先级

| 优先级 | 组件 | Constitution 条款 | 预估工作量 |
|--------|------|-------------------|------------|
| P0 | Context Engine | C-001, C-002 | 2-3 周 |
| P0 | Commands System | C-007, C-008 | 1-2 周 |
| P0 | Skills System | C-005, C-006 | 2 周 |
| P1 | Plugin System | C-003, C-004 | 3-4 周 |
| P1 | MCP Integration | C-009, C-010 | 2 周 |

---

## 四、验证清单

实现新功能时需验证:

- [ ] 是否符合 C-001~C-010 中的对应条款?
- [ ] 是否引入新的模块间依赖? 是否可解耦?
- [ ] 是否有硬编码值需提取为配置?
- [ ] 错误处理是否符合 Permission Engine 原则?
- [ ] 是否有对应的测试用例覆盖?

---

## 五、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0 | 2026-04-04 | 初始版本，覆盖 P0 问题 |

---

*本文档作为 OpenCode-RS 项目的 Constitution，后续迭代需保持向后兼容。*