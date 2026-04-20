# PRD: config Module

## Module Overview

**Module Name:** `config`
**Type:** Core
**Source:** `/packages/opencode/src/config/`

## Purpose

Configuration management with schema validation. Handles `opencode.json` configuration, environment variables, and defaults.

## Functionality

### Core Features

1. **Config File**
   - `opencode.json` in project directory
   - Hierarchical config (project > user > defaults)
   - JSON Schema validation

2. **Environment Variables**
   - `OPENCODE_*` prefixed variables
   - Provider-specific env vars
   - Config override capability

3. **Configuration Options**

   ```typescript
   interface Config {
     provider?: {
       [providerId: string]: {
         name?: string
         api?: string
         env?: string[]
         options?: Record<string, any>
         models?: {
           [modelId: string]: ModelConfig
         }
       }
     }
     enabled_providers?: string[]
     disabled_providers?: string[]
     agent?: {
       maxIterations?: number
       timeout?: number
     }
     lsp?: {
       enabled?: boolean
       path?: string
     }
   }
   ```

### Configuration Precedence

1. Command line arguments (highest)
2. `opencode.json` in current directory
3. `opencode.json` in home directory
4. Environment variables
5. Default values (lowest)

### Key Files

- Config schema definitions
- Config loading and merging
- Model ID parsing
- Provider configuration

### Configuration Example

```json
{
  "provider": {
    "openai": {
      "options": {
        "headers": {}
      }
    },
    "anthropic": {
      "options": {
        "headers": {
          "anthropic-beta": "interleaved-thinking-2025-05-14"
        }
      }
    }
  },
  "enabled_providers": ["anthropic", "openai"],
  "disabled_providers": [],
  "agent": {
    "maxIterations": 100
  }
}
```

## Dependencies

- `zod` - Schema validation
- `env` - Environment handling

## Acceptance Criteria

1. Config file is properly validated
2. Environment variables override file config
3. Missing config uses sensible defaults
4. Schema validation provides clear errors

## Rust Implementation Guidance

The Rust equivalent should:
- Use `serde` for JSON parsing
- Use `zod` or custom validation
- Use `dirs` for home directory
- Use `env` for environment variables
