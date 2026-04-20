# PRD: env Module

## Module Overview

**Module Name:** `env`
**Type:** Utility
**Source:** `/packages/opencode/src/env/`

## Purpose

Environment variable handling. Provides typed access to environment variables.

## Functionality

### Core Features

1. **Env Service**
   - Get environment variable
   - Get all environment variables
   - Set environment variable (process-level)

2. **Interface**

   ```typescript
   interface Env {
     get(key: string): Effect<string | undefined>
     all(): Effect<Record<string, string | undefined>>
     set(key: string, value: string): Effect<void>
   }
   ```

## Acceptance Criteria

1. Environment variables are read correctly
2. All variables can be retrieved
3. Setting variables works (when allowed)

## Rust Implementation Guidance

The Rust equivalent should:
- Use `std::env` for reading
- Consider process-level only setting
- Use proper error handling

## Test Design

### Unit Tests
- `env_overrides`: Test safe fetching of variables with fallbacks.

### Rust Specifics
- Test thread-safe environment variable reads (avoiding `std::env::set_var` in multi-threaded tests).
