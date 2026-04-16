# ratatui-testing Gap Analysis Report

## 1. 差距列表

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **CliTester缺少`capture_stdout()`和`capture_stderr()`方法** | P0 | cli | 添加`capture_stdout()`和`capture_stderr()` fluent方法到`CliTester`，参考PRD的`capture_stdout(&mut self) -> &mut Self`和`capture_stderr(&mut self) -> &mut Self` |
| **PtySimulator的`new()`无参数版本未实现command参数化** | P1 | pty | `new()`目前硬编码`["bash", "-c", "echo ready"]`。应添加`pub fn new() -> Result<Self>`直接调用`new_with_command`或提供默认command |
| **缺少`BufferDiff` builder模式的完整支持 | P1 | diff | 实现链式builder方法，如`ignore_foreground()`, `ignore_background()`, `ignore_attributes()`等fluent API |
| **StateTester缺少直接的状态比较方法** | P1 | state | 添加`compare_state<S>(&self, state: &S) -> Result<StateDiff>`方法直接比较当前状态与快照 |
| **TestDsl的`with_pty()`缺少无参数版本** | P1 | dsl | 当前`with_pty`需要传入command参数。添加`with_pty(self) -> Result<Self>`使用默认命令 |
| **DialogTester模块功能过于简单** | P2 | dialog_tester | 增强`DialogRenderTester`，添加更多对话框渲染验证方法，如验证特定内容存在、边框样式检查等 |
| **缺少对ratatui 0.28+新特性的支持测试** | P2 | tests | 添加针对新版本ratatui的Buffer API、新的Widget特性等的测试用例 |
| **Snapshot模块未实现自动版本管理** | P2 | snapshot | 当前的snapshot只能手动管理。考虑添加基于文件哈希的自动版本检测 |
| **集成测试文件缺少对DialogTester的测试** | P2 | tests | `integration_tests.rs`中未包含对`DialogRenderTester`的集成测试 |
| **缺少对Windows平台的支持检测** | P2 | pty | PTY功能在Windows上可能不工作，应添加平台检测和条件编译提示 |
| **TestDsl的`wait_for`系列方法未使用predicate** | P2 | dsl | `wait_for`等方法未使用`predicates`字段，仅用于`wait_with_predicates` |
| **BufferDiff的`diff_str`方法不支持忽略选项** | P2 | diff | `diff_str`方法直接调用`diff`而非使用`IgnoreOptions`，导致字符串比较不支持忽略颜色/属性 |

---

## 2. P0/P1/P2 问题分类

### P0 阻断性问题

| 问题 | 描述 | 影响 |
|------|------|------|
| **CliTester API不完整** | PRD要求`capture_stdout()`和`capture_stderr()`方法，但当前实现中`CliTester`没有这些方法 | 无法按照PRD的API设计使用，API不对齐 |
| **缺少integration test文件** | PRD要求`tests/`目录下应有`pty_tests.rs`, `buffer_diff_tests.rs`, `state_tests.rs`, `dsl_tests.rs`, `integration_tests.rs`，但缺少`dsl_tests.rs` | 测试覆盖不完整 |

### P1 重要问题

| 问题 | 描述 | 影响 |
|------|------|------|
| **PtySimulator构造函数设计不一致** | `new()`不接受参数但内部硬编码command，`new_with_command()`接受参数。PRD中`new()`应能创建默认PTY | API与PRD不一致 |
| **TestDsl的`with_pty`设计不一致** | PRD中`with_pty()`不需要参数，当前需要传入`&[&str]` | API不对齐 |
| **StateTester缺少直接比较API** | 缺少`compare_state`等直接比较方法 | 功能缺失 |
| **BufferDiff builder方法实现不完整** | `IgnoreOptions`的builder方法未被`BufferDiff`正确使用 | API不完整 |

### P2 改进建议

| 问题 | 描述 | 影响 |
|------|------|------|
| **DialogTester功能单薄** | 仅提供基础border/content检查方法 | 功能有限 |
| **wait_for方法未使用predicates字段** | `predicates`字段只在`wait_with_predicates`中使用 | 代码冗余 |
| **缺少跨平台检测** | Windows平台PTY可能不工作但无提示 | 可移植性问题 |
| **diff_str不支持IgnoreOptions** | 字符串diff不支持忽略颜色/属性 | 功能限制 |
| **Snapshot版本管理缺失** | 没有自动版本检测和管理 | 使用不便 |

---

## 3. 技术债务清单

| 技术债务 | 描述 | 优先级 |
|----------|------|--------|
| **CliTester temp_dir字段未使用** | `CliTester`结构体有`temp_dir: Option<TempDir>`字段但未在`working_dir`中使用，导致temp_dir永远为None | P1 |
| **TestDsl中未使用的`predicates`字段** | `TestDsl`有`predicates: Vec<WaitPredicate>`字段但`wait_for`等方法未使用它 | P2 |
| **DialogTester中`allow(dead_code)`警告** | `assert_render_result`和`assert_empty_state`标记为`#[allow(dead_code)]`但被公开使用 | P2 |
| **硬编码的sleep时间** | PTY测试中使用硬编码的`Duration::from_millis(100)`等值 | P2 |
| **重复的测试辅助函数** | `tests/buffer_diff_tests.rs`和`src/diff.rs`都有`create_buffer`辅助函数 | P2 |
| **未导出的ChildProcess** | `ChildProcess`在`cli.rs`中定义但未在lib.rs中导出 | P2 |
| **SNAPSHOT_DIR硬编码** | snapshot目录名为`"snapshots"`硬编码在代码中 | P3 |

---

## 4. 实现进度总结

### 模块实现状态

| 模块 | 状态 | 进度 | 说明 |
|------|------|------|------|
| **PtySimulator** | ✅ 已实现 | 95% | 核心功能完整，缺少平台检测 |
| **BufferDiff** | ✅ 已实现 | 90% | 完整实现，diff_str未使用IgnoreOptions |
| **StateTester** | ✅ 已实现 | 85% | 缺少直接compare_state方法 |
| **TestDsl** | ✅ 已实现 | 85% | 缺少无参数with_pty，wait_for未使用predicates |
| **CliTester** | ⚠️ 部分实现 | 60% | 缺少capture_stdout/capture_stderr fluent方法 |
| **Snapshot** | ✅ 已实现 | 80% | 基础功能完整，缺少版本管理 |
| **DialogTester** | ⚠️ 部分实现 | 50% | 功能过于简单，仅有基础检查方法 |

### 测试覆盖状态

| 测试文件 | 状态 | 测试数量 |
|----------|------|----------|
| `tests/pty_tests.rs` | ✅ 存在 | 11 tests |
| `tests/buffer_diff_tests.rs` | ✅ 存在 | 35+ tests |
| `tests/state_tests.rs` | ✅ 存在 | 33 tests |
| `tests/dsl_tests.rs` | ❌ 缺失 | 0 tests |
| `tests/integration_tests.rs` | ✅ 存在 | 28 tests |
| `tests/dsl_integration_tests.rs` | ✅ 存在 | - |

### 依赖状态

| 依赖 | 状态 | 说明 |
|------|------|------|
| ratatui | ✅ 0.28 | 符合PRD要求 |
| crossterm | ✅ 0.28 (optional) | 符合PRD要求 |
| portable-pty | ✅ 0.8 | 符合PRD要求 |
| anyhow | ✅ 1.0 | 符合PRD要求 |
| thiserror | ✅ 2.0 | 符合PRD要求 |
| serde | ✅ 1.0 | 符合PRD要求 |
| serde_json | ✅ 1.0 | 符合PRD要求 |
| tempfile | ✅ 3.14 | 符合PRD要求 |
| tokio | ✅ 1.45 | 符合PRD要求 |
| similar-asserts | ❌ 未添加 | 在dev-dependencies中缺失 |

### 文件结构对比

| PRD要求 | 实际存在 | 状态 |
|---------|----------|------|
| `src/lib.rs` | ✅ 存在 | 正常 |
| `src/pty.rs` | ✅ 存在 | 正常 |
| `src/diff.rs` | ✅ 存在 | 正常 |
| `src/state.rs` | ✅ 存在 | 正常 |
| `src/dsl.rs` | ✅ 存在 | 正常 |
| `src/cli.rs` | ✅ 存在 | 正常 |
| `src/snapshot.rs` | ✅ 存在 | 正常 |
| `tests/pty_tests.rs` | ✅ 存在 | 正常 |
| `tests/buffer_diff_tests.rs` | ✅ 存在 | 正常 |
| `tests/state_tests.rs` | ✅ 存在 | 正常 |
| `tests/dsl_tests.rs` | ❌ 不存在 | **缺失** |
| `tests/integration_tests.rs` | ✅ 存在 | 正常 |

---

## 5. 建议修复顺序

1. **P0 - 立即修复**
   - 创建`tests/dsl_tests.rs`文件
   - 为`CliTester`添加`capture_stdout()`和`capture_stderr()`方法

2. **P1 - 高优先级**
   - 统一`PtySimulator::new()`和`new_with_command()`设计
   - 为`TestDsl`添加无参数`with_pty()`版本
   - 添加`StateTester::compare_state()`方法
   - 修复`CliTester`的`temp_dir`字段使用

3. **P2 - 中优先级**
   - 增强`DialogTester`功能
   - 修复`diff_str`的IgnoreOptions支持
   - 添加跨平台检测
   - 重构`wait_for`方法使用`predicates`字段

4. **P3 - 低优先级**
   - 添加`similar-asserts`到dev-dependencies
   - 提取公共测试辅助函数
   - 导出`ChildProcess`类型
