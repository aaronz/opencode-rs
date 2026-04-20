# PRD: flag Module

## Module Overview

**Module Name:** `flag`
**Type:** Utility
**Source:** `/packages/opencode/src/flag/`

## Purpose

Feature flags and runtime configuration. Controls feature enablement via environment variables.

## Functionality

### Core Features

1. **Flag Variables**
   - `OPENCODE_ENABLE_EXPERIMENTAL_MODELS` - Enable alpha/beta models
   - `OPENCODE_MODELS_URL` - Custom models.dev URL
   - `OPENCODE_MODELS_PATH` - Local models file path
   - `OPENCODE_DISABLE_MODELS_FETCH` - Disable model fetching
   - `OPENCODE_PURE` - Run without plugins
   - And more...

2. **Flag Interface**

   ```typescript
   const Flag = {
     OPENCODE_ENABLE_EXPERIMENTAL_MODELS: boolean,
     OPENCODE_MODELS_URL: string,
     OPENCODE_MODELS_PATH: string,
     OPENCODE_DISABLE_MODELS_FETCH: boolean,
     OPENCODE_PURE: boolean,
     // ... more flags
   }
   ```

## Acceptance Criteria

1. Flags are properly read from environment
2. Flags control feature behavior correctly
3. Missing flags have sensible defaults

## Rust Implementation Guidance

The Rust equivalent should:
- Use `std::env` for environment variables
- Use `bool` for boolean flags
- Provide default values
