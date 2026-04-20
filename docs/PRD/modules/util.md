# PRD: util Module

## Module Overview

**Module Name:** `util`
**Type:** Utility
**Source:** `/packages/opencode/src/util/`

## Purpose

General utilities and helpers used throughout the application. Includes logging, error handling, filesystem utilities, and common helpers.

## Functionality

### Core Features

1. **Logging** (`Log`)
   - Structured logging with levels (DEBUG, INFO, WARN, ERROR)
   - File output with rotation
   - Console output control
   - Log file location

2. **Error Handling**
   - `NamedError` - Named error types
   - `errorMessage` - Error message extraction
   - Error formatting

3. **Filesystem** (`Filesystem`)
   - File read/write operations
   - Directory operations
   - Path utilities

4. **Common Helpers**
   - `iife` - Immediately invoked function expression
   - `lazy` - Lazy evaluation
   - `retry` - Retry utilities

### Key Components

```typescript
// Logging
interface Log {
  debug(message: string, data?: object): void
  info(message: string, data?: object): void
  warn(message: string, data?: object): void
  error(message: string, data?: object): void
}

// Error
class NamedError extends Error {
  name: string
  code?: string
  data?: Record<string, any>
}

// Filesystem
interface Filesystem {
  read(path: string): Promise<string>
  write(path: string, content: string): Promise<void>
  exists(path: string): Promise<boolean>
  stat(path: string): Promise<Stats>
}
```

### Dependencies

- `bun` - For filesystem operations (Bun runtime)
- `installation` - For version info

## Acceptance Criteria

1. Logging works with proper levels
2. Errors are properly typed
3. Filesystem operations are reliable
4. Helpers are functional

## Rust Implementation Guidance

The Rust equivalent should:
- Use `tracing` for logging
- Use `thiserror` for error types
- Use `tokio::fs` for filesystem
- Use standard library helpers
