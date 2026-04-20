# PRD: project Module

## Module Overview

**Module Name:** `project`
**Type:** Integration
**Source:** `/packages/opencode/src/project/`

## Purpose

Project detection and management. Identifies project type, root directory, and provides project-specific context.

## Functionality

### Core Features

1. **Project Detection**
   - Detect project root
   - Identify project type (Node, Rust, Python, etc.)
   - Read project metadata

2. **Project Types**

   | Type | Detection | Files |
   |------|-----------|-------|
   | Node.js | package.json | npm, yarn, pnpm |
   | Rust | Cargo.toml | cargo |
   | Python | pyproject.toml, requirements.txt | pip, poetry |
   | Go | go.mod | go |
   | And more... | | |

3. **Project Interface**

   ```typescript
   interface Project {
     root: string
     type: ProjectType
     packageManager: PackageManager
     languages: string[]
     config: ProjectConfig
   }
   ```

### Key Files

- Project detection logic
- Project type identification
- Configuration reading

## Acceptance Criteria

1. Project root is correctly identified
2. Project type is detected
3. Package manager is identified
4. Project config is accessible

## Rust Implementation Guidance

The Rust equivalent should:
- Use filesystem operations
- Parse project files
- Use `serde` for config parsing
