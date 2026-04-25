# PRD: Non-Functional Requirements

## Overview

This document specifies non-functional requirements (NFRs) for OpenCode RS, covering performance, scalability, reliability, security, and usability. These requirements ensure the system meets quality standards beyond functional specifications.

---

## Performance Requirements

### Session Management

| Requirement | Target | Measurement Method |
|-------------|--------|-------------------|
| Session creation latency | < 100ms (local) | Benchmark test |
| Session load latency | < 50ms (local SSD) | Benchmark test |
| Session save latency | < 100ms (local) | Benchmark test |
| Message append throughput | > 1000 msg/s | Load test |

### Tool Execution

| Requirement | Target | Measurement Method |
|-------------|--------|-------------------|
| Tool call overhead | < 5ms | Microbenchmark |
| Read tool (1KB file) | < 10ms | Integration test |
| Write tool (1KB file) | < 15ms | Integration test |
| Grep tool (10K files) | < 500ms | Integration test |

### LLM Provider

| Requirement | Target | Measurement Method |
|-------------|--------|-------------------|
| Provider API latency | P95 < 5s | End-to-end test |
| Retry success rate | > 95% | Integration test |
| Token budget tracking accuracy | ±5% | Unit test |

### Memory Usage

| Requirement | Target | Measurement Method |
|-------------|--------|-------------------|
| Idle memory (no session) | < 50MB | Memory profiler |
| Active session memory | < 200MB | Memory profiler |
| Memory per concurrent session | +50MB | Load test |

---

## Scalability Requirements

### Concurrent Sessions

| Requirement | Target | Notes |
|-------------|--------|-------|
| Max concurrent sessions | 10 | Per process |
| Session fork overhead | < 20ms | Benchmark |
| Cross-session message passing | Supported | via bus |

### Message History

| Requirement | Target | Notes |
|-------------|--------|-------|
| Max messages per session | 100,000 | Configurable |
| Max context tokens | 200,000 | Provider-dependent |
| Compaction ratio | 10:1 | Configurable |

### Storage

| Requirement | Target | Notes |
|-------------|--------|-------|
| SQLite database size | Unlimited | File system limited |
| Sessions directory size | Unlimited | File system limited |
| Concurrent DB connections | 10 | Connection pool |

---

## Reliability Requirements

### Process Reliability

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| Process crash recovery | 100% | Crash test |
| Session recovery after crash | 100% | Crash test |
| No data loss on graceful shutdown | 100% | Shutdown test |

### Session State Machine

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| Valid state transitions enforced | 100% | Unit test |
| Invalid transitions rejected | 100% | Unit test |
| State persistence across restarts | 100% | Integration test |

### Error Handling

| Requirement | Target | Measurement |
|-------------|--------|-------------|
| Uncaught exceptions logged | 100% | Integration test |
| Graceful degradation on provider failure | 100% | Chaos test |
| Error codes consistently defined | 100% | Code review |

---

## Security Requirements

### Authentication

| Requirement | Target | Implementation |
|-------------|--------|----------------|
| API key storage | Encrypted at rest | Keychain/system vault |
| Credential sanitization in exports | 100% | Unit test |
| Token expiration enforcement | 100% | Unit test |

### Authorization

| Requirement | Target | Implementation |
|-------------|--------|----------------|
| Permission model enforced | 100% | Integration test |
| Filesystem boundary respect | 100% | Security test |
| Plugin capability isolation | 100% | WASM sandbox |

### Credential Patterns (Redacted in Exports)

```
sk-*                    # OpenAI API keys
ghp_*                   # GitHub tokens
xoxb-*                  # Slack tokens
gho_*                   # GitHub OAuth
api_key=                # Generic API key pattern
password=               # Password pattern
token=                  # Token pattern
```

### SQL Injection Prevention

| Pattern | Action |
|---------|--------|
| DROP TABLE | Redact |
| DELETE FROM | Redact |
| INSERT INTO | Redact |
| DROP DATABASE | Redact |

---

## Usability Requirements

### CLI

| Requirement | Target |
|-------------|--------|
| Command help availability | All commands |
| Error messages actionable | 100% |
| Tab completion | Supported |
| Keyboard navigation in TUI | Full support |

### Error Messages

| Requirement | Standard |
|-------------|----------|
| Actionable resolution | Every error |
| Error code reference | Every error |
| Context in message | Every error |

### Accessibility (TUI)

| Requirement | Target |
|-------------|--------|
| Color contrast ratio | 4.5:1 minimum |
| Screen reader compatible | Via aria attributes |
| Keyboard-only operation | Full support |

---

## Compatibility Requirements

### Platforms

| Platform | Support Level |
|----------|--------------|
| macOS 12+ | Primary |
| Linux (glibc 2.31+) | Primary |
| Windows 10+ | Secondary |
| wasm32-wasip1 | Plugin runtime |

### Rust Version

| Version | Support |
|---------|---------|
| Rust 1.75+ | Required |
| Edition 2021 | Required |

### Dependencies

| Dependency | Minimum Version |
|------------|----------------|
| tokio | 1.45 |
| serde | 1.0 |
| rusqlite | 0.31 |

---

## Observability Requirements

### Logging

| Requirement | Target |
|-------------|--------|
| Structured JSON logs | All components |
| Log levels | trace, debug, info, warn, error |
| Context propagation | 100% |
| Sensitive data redaction | 100% |

### Metrics

| Category | Metrics |
|----------|---------|
| Session | created, active, completed, failed |
| Tool | invoked, success, failure, latency |
| Provider | requests, tokens, errors, latency |
| System | memory, cpu, connections |

### Tracing

| Requirement | Implementation |
|-------------|---------------|
| Distributed tracing | Via tracing crate |
| Span context | Propagated |
| Trace ID in logs | Every log entry |

---

## Testing Requirements

### Test Coverage

| Category | Target |
|----------|--------|
| Unit tests | 80%+ per crate |
| Integration tests | Core workflows |
| E2E tests | Critical paths |

### Performance Testing

| Requirement | Frequency |
|-------------|-----------|
| Benchmarks | Every PR |
| Load tests | Weekly |
| Stress tests | Monthly |

---

## Cross-References

| Topic | Reference |
|-------|-----------|
| Error codes | [ERROR_CODE_CATALOG.md](./ERROR_CODE_CATALOG.md) |
| Session lifecycle | [system/01-core-architecture.md](./system/01-core-architecture.md) |
| Tool registry | [modules/tool.md](./modules/tool.md) |
| Provider abstraction | [modules/provider.md](./modules/provider.md) |

---

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2026-04-26 | Initial creation | AI |
