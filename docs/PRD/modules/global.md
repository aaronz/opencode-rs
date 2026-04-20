# PRD: global Module

## Module Overview

**Module Name:** `global`
**Type:** Utility
**Source:** `/packages/opencode/src/global/`

## Purpose

Global state and paths. Provides access to global directories and application state.

## Functionality

### Core Features

1. **Global Paths**

   ```typescript
   const Global = {
     Path: {
       data: string,      // User data directory
       cache: string,     // Cache directory
       config: string,    // Config directory
       log: string,       // Log file location
     }
   }
   ```

2. **Platform Detection**
   - Windows, macOS, Linux detection
   - Home directory resolution
   - XDG Base Directory support

## Acceptance Criteria

1. Paths are correctly resolved
2. Platform detection works
3. Directories are accessible

## Rust Implementation Guidance

The Rust equivalent should:
- Use `dirs` crate for path resolution
- Use `std::env` for environment
- Use platform-specific code when needed

## Test Design

### Unit Tests
- `path_resolution`: Test fallback paths for different OS targets (Windows, macOS, Linux).

### Rust Specifics
- Test via conditional compilation (`#[cfg(target_os = windows)]`) or path mocking.
