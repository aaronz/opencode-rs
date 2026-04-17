# Baseline Performance Metrics

**Task:** P2-008
**Date:** 2026-04-17
**Status:** Documented

## Overview

This document contains the baseline performance metrics for the OpenCode RS benchmark suite. These metrics are used for regression detection in CI to ensure performance does not degrade across iterations.

## Benchmark Suite Structure

The benchmark suite is located in `opencode-benches/` and uses the `criterion` benchmarking library.

### Benchmark Groups

| Group | Description | Benchmarks | Status |
|-------|-------------|------------|--------|
| `session_operations` | Session lifecycle and message handling | 4 | Pending |
| `tool_registry` | Tool registry operations | 2 | Pending |
| `storage` | Session storage operations | 5 | Pending |
| `config_parsing` | Configuration parsing | 4 | Complete |
| `token_counting` | Token counting operations | 7 | Complete |
| `plugin_manager` | Plugin management operations | 10 | Pending |
| `jsonc_parsing` | JSONC parsing performance | 6 | Complete |

## Session Operations Benchmarks

### Group: `session_operations`

| Benchmark | Description | Baseline (ns) | Tolerance |
|-----------|-------------|---------------|----------|
| `session_new` | Create new empty session | TBD | ±20% |
| `session_with_messages` | Create session with 100 messages | TBD | ±20% |
| `session_save_load` | Save and load session with 50 messages | TBD | ±20% |
| `session_1000_messages` | Create session with 1000 messages | TBD | ±20% |

## Tool Registry Benchmarks

### Group: `tool_registry`

| Benchmark | Description | Baseline (ns) | Tolerance |
|-----------|-------------|---------------|----------|
| `registry_new` | Create new tool registry | TBD | ±20% |
| `registry_is_disabled` | Check if tool is disabled | TBD | ±20% |

## Storage Operations Benchmarks

### Group: `storage`

| Benchmark | Description | Baseline (ns) | Tolerance |
|-----------|-------------|---------------|----------|
| `storage_create_small_session` | Create session with 10 message pairs | TBD | ±20% |
| `storage_create_medium_session` | Create session with 100 message pairs | TBD | ±20% |
| `storage_create_large_session` | Create session with 500 message pairs | TBD | ±20% |
| `session_message_iteration` | Iterate through 1000 messages | TBD | ±20% |
| `session_message_rev_iteration` | Reverse iterate through 1000 messages | TBD | ±20% |

## Config Parsing Benchmarks

### Group: `config_parsing`

| Benchmark | Description | Baseline (ns) | Tolerance |
|-----------|-------------|---------------|----------|
| `config_parse_minimal` | Parse minimal config | 371.64 | ±20% |
| `config_parse_with_permission` | Parse config with permissions | 988.57 | ±20% |
| `config_parse_full` | Parse full config | 2741.32 | ±20% |
| `config_parse_large_agent_section` | Parse config with 20 agents | 20161.62 | ±20% |

## Token Counting Benchmarks

### Group: `token_counting`

| Benchmark | Description | Baseline (ns) | Tolerance |
|-----------|-------------|---------------|----------|
| `token_count_empty_string` | Count tokens in empty string | 630.59 | ±20% |
| `token_count_short_text` | Count tokens in "Hello, world!" | 3860.32 | ±20% |
| `token_count_medium_text` | Count tokens in medium sentence | 17604.64 | ±20% |
| `token_count_long_code_snippet` | Count tokens in 1000-line code | 50367.37 | ±20% |
| `token_count_messages_10` | Count tokens in 10 messages | 84744.49 | ±20% |
| `token_count_messages_100` | Count tokens in 100 messages | 841122.39 | ±20% |
| `token_count_messages_500` | Count tokens in 500 messages | 4211125.62 | ±20% |

## Plugin Manager Benchmarks

### Group: `plugin_manager`

| Benchmark | Description | Baseline (ns) | Tolerance |
|-----------|-------------|---------------|----------|
| `plugin_manager_new` | Create new plugin manager | TBD | ±20% |
| `plugin_manager_register_single` | Register 1 plugin | TBD | ±20% |
| `plugin_manager_register_10_plugins` | Register 10 plugins | TBD | ±20% |
| `plugin_manager_register_50_plugins` | Register 50 plugins | TBD | ±20% |
| `plugin_manager_get_plugin_existing` | Get existing plugin | TBD | ±20% |
| `plugin_manager_get_plugin_nonexistent` | Get nonexistent plugin | TBD | ±20% |
| `plugin_manager_on_start_all` | Call on_start on all 10 plugins | TBD | ±20% |
| `plugin_manager_on_tool_call_all` | Call on_tool_call on all 10 plugins | TBD | ±20% |
| `plugin_manager_on_message_all` | Call on_message on all 10 plugins | TBD | ±20% |
| `plugin_manager_shutdown` | Shutdown manager with 10 plugins | TBD | ±20% |

## JSONC Parsing Benchmarks

### Group: `jsonc_parsing`

| Benchmark | Description | Baseline (ns) | Tolerance |
|-----------|-------------|---------------|----------|
| `parse_minimal_jsonc_current_json5` | Parse minimal JSONC (json5) | 421.23 | ±20% |
| `parse_minimal_jsonc_jsonc_parser` | Parse minimal JSONC (jsonc-parser) | 292.27 | ±20% |
| `parse_full_config_jsonc_current_json5` | Parse full config JSONC (json5) | 4369.20 | ±20% |
| `parse_full_config_jsonc_jsonc_parser` | Parse full config JSONC (jsonc-parser) | 2913.65 | ±20% |
| `parse_large_config_jsonc_current_json5` | Parse large config JSONC (json5) | 45232.68 | ±20% |
| `parse_large_config_jsonc_jsonc_parser` | Parse large config JSONC (jsonc-parser) | 44936.67 | ±20% |

## Running Benchmarks

To run the benchmark suite:

```bash
cd opencode-rust
cargo bench
```

To run a specific benchmark group:

```bash
cargo bench -- session_operations
cargo bench -- tool_registry
cargo bench -- storage
cargo bench -- config_parsing
cargo bench -- token_counting
cargo bench -- plugin_manager
cargo bench -- jsonc_parsing
```

## Regression Detection

CI uses criterion's built-in regression detection. When benchmarks are run:

1. Criterion compares new measurements against stored baseline
2. If mean time deviates beyond tolerance, a regression is flagged
3. Regression thresholds are set at ±20% by default

## Updating Baselines

To update baselines after intentional changes:

```bash
cargo bench -- --baseline updated
```

To compare against a specific baseline:

```bash
cargo bench -- --baseline master
```

## Notes

- Benchmarks run in release mode (`--release`)
- Measurement time varies by benchmark (3-5 seconds per benchmark)
- Platform: macOS ARM64 (Apple Silicon)
- Rust version: 1.70+
- All times are in nanoseconds (ns)

## Change Log

| Date | Changes |
|------|---------|
| 2026-04-17 | Initial documentation created with baseline values for config_parsing, token_counting, and jsonc_parsing benchmarks |
