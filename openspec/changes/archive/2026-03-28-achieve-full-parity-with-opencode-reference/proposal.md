## Why

当前Rust实现的opencode项目(rust-opencode-port)与TypeScript参考项目(opencode)之间存在功能差距。虽然大部分CLI命令和LLM Provider已经实现，但由于编译错误和CLI路由未完成，导致无法通过目标项目的测试用例。需要补齐这些差距以实现完全的功能对等。

## What Changes

1. **修复编译错误** - 修复schema_validation.rs中的生命周期标注问题
2. **完成CLI命令路由** - 在main.rs中将所有子命令正确连接到实现
3. **修复测试文件** - 修复测试中的临时值生命周期问题
4. **添加工具依赖** - 为TUI测试添加tempfile依赖

## Capabilities

### New Capabilities
- `cli-command-routing`: 完成所有CLI命令的路由连接
- `test-infrastructure`: 修复测试基础设施问题

### Modified Capabilities
- (无需求变更)

## Impact

- **crates/cli/src/main.rs** - 需要添加所有子命令的路由
- **crates/tools/src/schema_validation.rs** - 已修复
- **crates/cli/tests/*.rs** - 已修复
- **crates/tui/Cargo.toml** - 已添加tempfile依赖
