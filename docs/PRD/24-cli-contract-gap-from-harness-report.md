# PRD 24 - CLI Contract Gap Alignment from Harness Report

## 1. 文档信息

### 项目名称
opencode-rs

### 文档类型
增量 PRD / Harness 驱动差距修复

### 文档版本
v1.0

### 来源
基于 `opencode-harness` 对 `opencode` vs `opencode-rs` 的真实 CLI 差分测试结果生成。

---

## 2. 背景

`opencode-harness` 已对 CLI smoke suite 进行了真实差分执行，确认当前 `opencode-rs` 在 CLI contract 层存在系统性不一致，而不仅仅是单点 bug。

当前已验证的差异包括：

1. `--help`
2. `--version`
3. `workspace --help`
4. `--invalid-option`
5. `config show`
6. `--verbose --help`

这些差异主要集中在两类：

- 输出文本 / 输出流不一致
- 退出码语义不一致

这说明当前 `opencode-rs` 的 CLI 行为尚未与 `opencode` 对齐，且这些差异已经足够影响：

- 用户命令体验
- 自动化脚本兼容性
- Harness 回归结果可信度
- 后续 session / workspace / config / permission 等更高层验证

因此，需要专门开一轮增量迭代，优先修复和收敛 CLI contract 差距。

---

## 3. 本轮目标

对齐 `opencode-rs` 与 `opencode` 的基础 CLI contract，使最小 CLI smoke suite 中的关键命令在以下方面尽量一致：

1. 退出码语义
2. help / version 文本行为
3. 参数解析行为
4. config / workspace / session 等子命令基础契约

本轮目标不是一次性补齐所有 CLI 功能，而是优先修复已被 Harness 实际验证出的高价值差异。

---

## 4. 本轮范围

### 4.1 必须覆盖的差异

#### FR-001 `--help` 对齐
- 对齐顶层 help 的输出行为
- 明确 help 是否输出到 stdout / stderr
- 尽量与 `opencode` 保持一致
- 如无法完全一致，至少需要形成明确、稳定、可解释的策略

#### FR-002 `--version` 对齐
- 对齐版本输出内容与格式
- 确保 exit code 与 `opencode` 一致

#### FR-003 `workspace --help` 对齐
- 对齐子命令 help 的输出位置、格式与内容结构

#### FR-004 `--invalid-option` 错误语义对齐
- 对齐非法参数时的退出码语义
- 对齐错误信息展示方式

#### FR-005 `config show` 契约对齐
- 对齐命令是否成功执行
- 对齐输出格式或至少对齐成功/失败语义

#### FR-006 `--verbose --help` 参数组合行为对齐
- 明确 verbose flag 与 help 的组合处理顺序
- 对齐退出码与输出结果

---

## 5. 明确不做

本轮不做：

- 不追求所有 CLI 子命令一次性全面对齐
- 不覆盖 web / api / acp / desktop
- 不在本轮处理与 CLI contract 无关的底层大规模重构
- 不以“只让 harness 通过”为目标做 hack 式适配

本轮目标是修真实 CLI 契约差距，而不是伪造测试通过。

---

## 6. 设计要求

### 6.1 用户可观察行为优先
优先对齐用户真正观察到的行为：

- 命令是否成功
- 退出码是多少
- 输出到 stdout 还是 stderr
- 文本格式与内容是否可兼容

### 6.2 向前兼容与脚本兼容优先
CLI contract 一旦被外部用户或脚本依赖，就不应随意漂移。

### 6.3 不为通过测试牺牲长期清晰性
如果某个对齐策略会引入明显脏设计，需要在实现里明确封装，不要把 CLI 语义搞成隐藏行为。

### 6.4 每个修复项都要可由 Harness 复测
每个差异修复后，都应能直接由 Harness smoke CLI suite 重新验证。

---

## 7. 必须产物

本轮完成后至少应产出：

1. `opencode-rs` CLI contract 相关实现修复
2. 对应单元测试 / 集成测试 / 回归测试
3. 至少覆盖以下命令的稳定测试：
   - `--help`
   - `--version`
   - `workspace --help`
   - `--invalid-option`
   - `config show`
   - `--verbose --help`
4. 一份简短的对齐说明或变更说明（可写入 iteration 产物）

---

## 8. 验收标准

### 最低验收
至少满足以下条件：

1. 上述 6 个 CLI smoke case 不再出现明显的退出码语义错误
2. help / version / config 行为至少比当前更接近 `opencode`
3. 修复项有测试覆盖
4. Harness 可以重新执行并看到差异缩小或消失

### 更优验收
如能做到以下更好：

- 顶层 help 与子命令 help 行为统一
- 错误参数退出码与错误消息完全对齐
- `config show` 输出结构与 `opencode` 基本一致

---

## 9. 验收命令

建议至少执行：

```bash
cargo test -p opencode-cli
cargo run -q -p opencode-cli -- --help
cargo run -q -p opencode-cli -- --version
cargo run -q -p opencode-cli -- workspace --help
cargo run -q -p opencode-cli -- --invalid-option
cargo run -q -p opencode-cli -- config show
cargo run -q -p opencode-cli -- --verbose --help
```

并由 Harness 复测：

```bash
cd /Users/openclaw/Documents/github/opencode-harness
cargo run -- run --task harness/tasks/cli
```

---

## 10. 下一轮输入

若本轮完成，后续可继续基于 Harness 扩展：

1. session command contract 对齐
2. workspace command contract 更深层对齐
3. config / provider / auth 子命令体系对齐
4. regression 资产沉淀

---

## 11. 一句话

本轮就是：

> **基于 Harness 已跑出来的真实差异，收敛 `opencode-rs` 的基础 CLI contract，使其在最小 smoke suite 上更接近 `opencode`。**
