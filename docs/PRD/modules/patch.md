# Module PRD: patch (Apply Patch Tool)

## 1. Module Overview

| Field | Value |
|-------|-------|
| **Module Name** | patch |
| **Source Path** | `packages/opencode/src/patch/` |
| **Type** | Utility / Parser + Filesystem Tool |
| **Rust Crate** | `opencode-patch` (or within `opencode-tools`) |
| **Purpose** | Implements the `apply_patch` tool that LLMs can invoke to make structured, multi-file edits via a custom patch format. Parses a domain-specific patch language, resolves file changes, applies them to the filesystem, and returns affected paths. Also provides a verified variant that computes changes without writing files. |

## 2. Functionality

The patch module provides:

1. **Patch Format Parser** (`parsePatch`): Parses the custom `*** Begin Patch / *** End Patch` format into a typed list of hunks. Supports three hunk types:
   - `add` — create a new file with given content
   - `delete` — remove an existing file
   - `update` — apply line-level changes to an existing file (with optional move)

2. **Heredoc Stripping** (`stripHeredoc`): Unwraps bash heredoc syntax (`cat <<'EOF' ... EOF`) before parsing.

3. **Hunk Application** (`applyHunksToFiles`): Applies parsed hunks to the filesystem — creates directories, writes files, and deletes files.

4. **Fuzzy Line Matching** (`seekSequence`): Locates old lines in a file using a 4-pass fuzzy matching strategy:
   - Pass 1: Exact match
   - Pass 2: Trailing whitespace trimmed
   - Pass 3: Both ends trimmed
   - Pass 4: Unicode punctuation normalized to ASCII equivalents

5. **Shell Command Detection** (`maybeParseApplyPatch`): Detects whether a shell command is an `apply_patch` invocation. Supports:
   - Direct: `apply_patch <patch_text>`
   - Alias: `applypatch <patch_text>`
   - Bash heredoc: `bash -lc 'apply_patch <<EOF ... EOF'`

6. **Verified Variant** (`maybeParseApplyPatchVerified`): Like `maybeParseApplyPatch` but also reads current file contents and pre-computes the changes (without writing), returning an `ApplyPatchAction` with the full diff.

7. **Implicit Invocation Detection**: Detects when a raw patch text is passed without the `apply_patch` command wrapper and returns a `CorrectnessError`.

### Patch Format Grammar

```
*** Begin Patch
*** Add File: <path>
+<line content>
+<line content>
...
*** Delete File: <path>
*** Update File: <path>
[*** Move to: <new-path>]
@@ [<context string>]
 <context line>
-<removed line>
+<added line>
*** End Patch
```

Line prefix semantics in update hunks:
- ` ` (space) → keep line (appears in both old and new)
- `-` → remove line (old only)
- `+` → add line (new only)

## 3. API Surface

```typescript
// Schema
export const PatchSchema = z.object({ patchText: z.string() })
export type PatchParams = z.infer<typeof PatchSchema>

// Core types
export type Hunk =
  | { type: "add"; path: string; contents: string }
  | { type: "delete"; path: string }
  | { type: "update"; path: string; move_path?: string; chunks: UpdateFileChunk[] }

export interface UpdateFileChunk {
  old_lines: string[]
  new_lines: string[]
  change_context?: string   // @@ context string for anchoring
  is_end_of_file?: boolean  // if true, match from end of file
}

export interface ApplyPatchArgs {
  patch: string
  hunks: Hunk[]
  workdir?: string
}

export interface ApplyPatchAction {
  changes: Map<string, ApplyPatchFileChange>  // absolute path → change
  patch: string
  cwd: string
}

export type ApplyPatchFileChange =
  | { type: "add"; content: string }
  | { type: "delete"; content: string }
  | { type: "update"; unified_diff: string; move_path?: string; new_content: string }

export interface AffectedPaths {
  added: string[]
  modified: string[]
  deleted: string[]
}

export enum ApplyPatchError {
  ParseError = "ParseError",
  IoError = "IoError",
  ComputeReplacements = "ComputeReplacements",
  ImplicitInvocation = "ImplicitInvocation",
}

export enum MaybeApplyPatch {
  Body = "Body",
  ShellParseError = "ShellParseError",
  PatchParseError = "PatchParseError",
  NotApplyPatch = "NotApplyPatch",
}

export enum MaybeApplyPatchVerified {
  Body = "Body",
  ShellParseError = "ShellParseError",
  CorrectnessError = "CorrectnessError",
  NotApplyPatch = "NotApplyPatch",
}

// Functions
export function parsePatch(patchText: string): { hunks: Hunk[] }
export function maybeParseApplyPatch(argv: string[]):
  | { type: MaybeApplyPatch.Body; args: ApplyPatchArgs }
  | { type: MaybeApplyPatch.PatchParseError; error: Error }
  | { type: MaybeApplyPatch.NotApplyPatch }

export function deriveNewContentsFromChunks(filePath: string, chunks: UpdateFileChunk[]): {
  unified_diff: string
  content: string
}

export async function applyHunksToFiles(hunks: Hunk[]): Promise<AffectedPaths>
export async function applyPatch(patchText: string): Promise<AffectedPaths>
export async function maybeParseApplyPatchVerified(argv: string[], cwd: string): Promise<
  | { type: MaybeApplyPatchVerified.Body; action: ApplyPatchAction }
  | { type: MaybeApplyPatchVerified.CorrectnessError; error: Error }
  | { type: MaybeApplyPatchVerified.NotApplyPatch }
>
```

## 4. Data Structures

### Replacement Plan (Internal)

The `computeReplacements()` function returns a list of splice operations:
```typescript
type Replacement = [startIdx: number, deleteCount: number, newLines: string[]]
```

Replacements are:
1. Computed forward (incrementing `lineIndex` after each match)
2. Sorted by start index
3. Applied in **reverse** order to avoid index shifting

### Fuzzy Matching Passes

| Pass | Comparator |
|------|-----------|
| 1 | Exact: `a === b` |
| 2 | Rstrip: `a.trimEnd() === b.trimEnd()` |
| 3 | Trim: `a.trim() === b.trim()` |
| 4 | Normalized: `normalizeUnicode(a.trim()) === normalizeUnicode(b.trim())` |

### Unicode Normalization Map

| Unicode | ASCII |
|---------|-------|
| `\u2018\u2019\u201A\u201B` | `'` |
| `\u201C\u201D\u201E\u201F` | `"` |
| `\u2010-\u2015` | `-` |
| `\u2026` | `...` |
| `\u00A0` | ` ` |

## 5. Dependencies

| Dependency | Purpose |
|------------|---------|
| `zod` | Schema validation for patch input |
| `path` | Path resolution and manipulation |
| `fs/promises` | Async file operations (read, write, unlink, mkdir) |
| `fs` (readFileSync) | Synchronous file reads for `deriveNewContentsFromChunks` |
| `@/util` (Log) | Logging |

## 6. Acceptance Criteria

### Parser
- [ ] `parsePatch` correctly parses `*** Add File:` hunks with `+` prefixed lines
- [ ] `parsePatch` correctly parses `*** Delete File:` hunks
- [ ] `parsePatch` correctly parses `*** Update File:` hunks with `@@` chunk markers
- [ ] `parsePatch` correctly parses `*** Move to:` directive on update hunks
- [ ] `parsePatch` handles multiple hunks in a single patch
- [ ] `parsePatch` throws on missing `*** Begin Patch` or `*** End Patch` markers
- [ ] `parsePatch` strips bash heredoc syntax before parsing

### Line Matching
- [ ] Exact line match found on Pass 1
- [ ] Trailing-whitespace-only differences found on Pass 2
- [ ] Surrounding-whitespace differences found on Pass 3
- [ ] Unicode quote/dash differences found on Pass 4
- [ ] `is_end_of_file: true` tries matching from end of file first
- [ ] Returns error when old lines cannot be found in the file

### Filesystem Application
- [ ] `applyPatch` creates new files with parent directories
- [ ] `applyPatch` deletes existing files
- [ ] `applyPatch` updates files with correct content
- [ ] `applyPatch` moves files (write to new path, delete old path)
- [ ] `applyPatch` handles multiple hunks atomically (all or nothing? best-effort in TS)
- [ ] `applyPatch` ensures trailing newline in updated files
- [ ] Throws when no hunks provided

### Command Detection
- [ ] `maybeParseApplyPatch` detects `["apply_patch", "<patch>"]`
- [ ] `maybeParseApplyPatch` detects `["applypatch", "<patch>"]`
- [ ] `maybeParseApplyPatch` detects bash heredoc form
- [ ] `maybeParseApplyPatch` returns `NotApplyPatch` for other commands
- [ ] `maybeParseApplyPatchVerified` returns `CorrectnessError` for raw patch text

## 7. Rust Implementation Guidance

### Crate
`opencode-patch` crate in `opencode-rust/crates/patch/`

### Key Crates
- `std::fs` / `tokio::fs` — file operations
- `regex` — Unicode normalization patterns
- `thiserror` — error types
- `serde` — serialization of types
- `tracing` — logging

### Recommended Approach

```rust
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Hunk {
    Add { path: PathBuf, contents: String },
    Delete { path: PathBuf },
    Update { path: PathBuf, move_path: Option<PathBuf>, chunks: Vec<UpdateFileChunk> },
}

#[derive(Debug, Clone)]
pub struct UpdateFileChunk {
    pub old_lines: Vec<String>,
    pub new_lines: Vec<String>,
    pub change_context: Option<String>,
    pub is_end_of_file: bool,
}

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("Invalid patch format: missing Begin/End markers")]
    MissingMarkers,
    #[error("Failed to find expected lines in {path}:\n{lines}")]
    LinesNotFound { path: PathBuf, lines: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No files were modified")]
    NoHunks,
}

pub struct AffectedPaths {
    pub added: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
}

// Parser
pub fn parse_patch(patch_text: &str) -> Result<Vec<Hunk>, PatchError> {
    let cleaned = strip_heredoc(patch_text.trim());
    let lines: Vec<&str> = cleaned.lines().collect();
    let begin_idx = lines.iter().position(|l| l.trim() == "*** Begin Patch")
        .ok_or(PatchError::MissingMarkers)?;
    let end_idx = lines.iter().position(|l| l.trim() == "*** End Patch")
        .ok_or(PatchError::MissingMarkers)?;
    if begin_idx >= end_idx { return Err(PatchError::MissingMarkers); }

    let mut hunks = Vec::new();
    let mut i = begin_idx + 1;
    while i < end_idx {
        if let Some(hunk) = parse_hunk(&lines, &mut i, end_idx)? {
            hunks.push(hunk);
        } else {
            i += 1;
        }
    }
    Ok(hunks)
}

// Fuzzy line matching (4-pass)
fn seek_sequence(lines: &[&str], pattern: &[&str], start: usize, eof: bool) -> Option<usize> {
    // Pass 1: exact
    if let Some(i) = try_match(lines, pattern, start, |a, b| a == b, eof) { return Some(i); }
    // Pass 2: rstrip
    if let Some(i) = try_match(lines, pattern, start, |a, b| a.trim_end() == b.trim_end(), eof) { return Some(i); }
    // Pass 3: trim
    if let Some(i) = try_match(lines, pattern, start, |a, b| a.trim() == b.trim(), eof) { return Some(i); }
    // Pass 4: normalize unicode
    try_match(lines, pattern, start,
        |a, b| normalize_unicode(a.trim()) == normalize_unicode(b.trim()), eof)
}

fn normalize_unicode(s: &str) -> String {
    s.chars().map(|c| match c {
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
        '\u{2010}'..='\u{2015}' => '-',
        '\u{2026}' => '.', // ellipsis → "..." (would need special handling for 3 chars)
        '\u{00A0}' => ' ',
        c => c,
    }).collect()
}

// Apply hunks to filesystem
pub async fn apply_patch(patch_text: &str) -> Result<AffectedPaths, PatchError> {
    let hunks = parse_patch(patch_text)?;
    apply_hunks(&hunks).await
}

pub async fn apply_hunks(hunks: &[Hunk]) -> Result<AffectedPaths, PatchError> {
    if hunks.is_empty() { return Err(PatchError::NoHunks); }
    let mut result = AffectedPaths::default();
    for hunk in hunks {
        match hunk {
            Hunk::Add { path, contents } => {
                if let Some(dir) = path.parent() {
                    tokio::fs::create_dir_all(dir).await?;
                }
                tokio::fs::write(path, contents).await?;
                result.added.push(path.clone());
            }
            Hunk::Delete { path } => {
                tokio::fs::remove_file(path).await?;
                result.deleted.push(path.clone());
            }
            Hunk::Update { path, move_path, chunks } => {
                let update = derive_new_contents(path, chunks)?;
                let target = move_path.as_ref().unwrap_or(path);
                if let Some(dir) = target.parent() {
                    tokio::fs::create_dir_all(dir).await?;
                }
                tokio::fs::write(target, &update.content).await?;
                if move_path.is_some() {
                    tokio::fs::remove_file(path).await?;
                }
                result.modified.push(target.clone());
            }
        }
    }
    Ok(result)
}
```

### Command Detection in Rust

```rust
pub enum MaybeApplyPatch {
    Body(ApplyPatchArgs),
    PatchParseError(PatchError),
    NotApplyPatch,
}

pub fn maybe_parse_apply_patch(argv: &[&str]) -> MaybeApplyPatch {
    let commands = ["apply_patch", "applypatch"];
    if argv.len() == 2 && commands.contains(&argv[0]) {
        match parse_patch(argv[1]) {
            Ok(hunks) => MaybeApplyPatch::Body(ApplyPatchArgs {
                patch: argv[1].to_string(), hunks, workdir: None,
            }),
            Err(e) => MaybeApplyPatch::PatchParseError(e),
        }
    } else {
        MaybeApplyPatch::NotApplyPatch
    }
}
```

## 8. Test Design

Tests are in `test/patch/patch.test.ts`. They use a temporary directory for file operations.

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // parsePatch tests
    // 1. Simple add file patch
    #[test]
    fn parse_add_file_patch() {
        let patch = "*** Begin Patch\n*** Add File: test.txt\n+Hello World\n*** End Patch";
        let hunks = parse_patch(patch).unwrap();
        assert_eq!(hunks.len(), 1);
        assert!(matches!(&hunks[0], Hunk::Add { path, contents }
            if path.to_str() == Some("test.txt") && contents == "Hello World"));
    }

    // 2. Delete file patch
    #[test]
    fn parse_delete_file_patch() {
        let patch = "*** Begin Patch\n*** Delete File: old.txt\n*** End Patch";
        let hunks = parse_patch(patch).unwrap();
        assert_eq!(hunks.len(), 1);
        assert!(matches!(&hunks[0], Hunk::Delete { path } if path.to_str() == Some("old.txt")));
    }

    // 3. Multiple hunks
    #[test]
    fn parse_multiple_hunks() {
        let patch = "*** Begin Patch\n*** Add File: new.txt\n+New content\n*** Delete File: old.txt\n*** End Patch";
        let hunks = parse_patch(patch).unwrap();
        assert_eq!(hunks.len(), 2);
        assert!(matches!(&hunks[0], Hunk::Add { .. }));
        assert!(matches!(&hunks[1], Hunk::Delete { .. }));
    }

    // 4. File move operation
    #[test]
    fn parse_file_move() {
        let patch = "*** Begin Patch\n*** Update File: old.txt\n*** Move to: new.txt\n@@\n-old\n+new\n*** End Patch";
        let hunks = parse_patch(patch).unwrap();
        assert_eq!(hunks.len(), 1);
        if let Hunk::Update { move_path, .. } = &hunks[0] {
            assert_eq!(move_path.as_ref().and_then(|p| p.to_str()), Some("new.txt"));
        } else {
            panic!("expected update hunk");
        }
    }

    // 5. Invalid patch format
    #[test]
    fn parse_invalid_patch_errors() {
        let result = parse_patch("This is not a patch");
        assert!(result.is_err());
    }

    // maybeParseApplyPatch tests
    // 6. Direct apply_patch command
    #[test]
    fn detect_apply_patch_command() {
        let patch = "*** Begin Patch\n*** Add File: t.txt\n+x\n*** End Patch";
        let result = maybe_parse_apply_patch(&["apply_patch", patch]);
        assert!(matches!(result, MaybeApplyPatch::Body(_)));
    }

    // 7. Applypatch alias
    #[test]
    fn detect_applypatch_alias() {
        let patch = "*** Begin Patch\n*** Add File: t.txt\n+x\n*** End Patch";
        let result = maybe_parse_apply_patch(&["applypatch", patch]);
        assert!(matches!(result, MaybeApplyPatch::Body(_)));
    }

    // 8. Non-patch command returns NotApplyPatch
    #[test]
    fn non_patch_command_returns_not_apply_patch() {
        let result = maybe_parse_apply_patch(&["echo", "hello"]);
        assert!(matches!(result, MaybeApplyPatch::NotApplyPatch));
    }

    // applyPatch filesystem tests (async, require tempdir)
    // 9. Add new file
    #[tokio::test]
    async fn apply_patch_adds_new_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("new-file.txt");
        let patch = format!("*** Begin Patch\n*** Add File: {}\n+Hello World\n+Line 2\n*** End Patch",
            file.display());
        let result = apply_patch(&patch).await.unwrap();
        assert_eq!(result.added.len(), 1);
        let content = std::fs::read_to_string(&result.added[0]).unwrap();
        assert_eq!(content, "Hello World\nLine 2");
    }

    // 10. Delete existing file
    #[tokio::test]
    async fn apply_patch_deletes_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("to-delete.txt");
        std::fs::write(&file, "delete me").unwrap();
        let patch = format!("*** Begin Patch\n*** Delete File: {}\n*** End Patch", file.display());
        let result = apply_patch(&patch).await.unwrap();
        assert_eq!(result.deleted.len(), 1);
        assert!(!file.exists());
    }

    // 11. Update existing file
    #[tokio::test]
    async fn apply_patch_updates_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("to-update.txt");
        std::fs::write(&file, "line 1\nline 2\nline 3\n").unwrap();
        let patch = format!("*** Begin Patch\n*** Update File: {}\n@@\n line 1\n-line 2\n+line 2 updated\n line 3\n*** End Patch",
            file.display());
        let result = apply_patch(&patch).await.unwrap();
        assert_eq!(result.modified.len(), 1);
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "line 1\nline 2 updated\nline 3\n");
    }

    // 12. Move and update file
    #[tokio::test]
    async fn apply_patch_moves_file() {
        let tmp = TempDir::new().unwrap();
        let old = tmp.path().join("old.txt");
        let new = tmp.path().join("new.txt");
        std::fs::write(&old, "old content\n").unwrap();
        let patch = format!("*** Begin Patch\n*** Update File: {}\n*** Move to: {}\n@@\n-old content\n+new content\n*** End Patch",
            old.display(), new.display());
        let result = apply_patch(&patch).await.unwrap();
        assert!(!old.exists());
        assert!(new.exists());
        let content = std::fs::read_to_string(&new).unwrap();
        assert_eq!(content, "new content\n");
    }

    // 13. Create parent directories when adding files
    #[tokio::test]
    async fn apply_patch_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("deep").join("nested").join("file.txt");
        let patch = format!("*** Begin Patch\n*** Add File: {}\n+Content\n*** End Patch", file.display());
        let result = apply_patch(&patch).await.unwrap();
        assert_eq!(result.added.len(), 1);
        assert!(file.exists());
    }

    // 14. Multiple operations in one patch
    #[tokio::test]
    async fn apply_patch_multiple_operations() {
        let tmp = TempDir::new().unwrap();
        let f1 = tmp.path().join("file1.txt");
        let f2 = tmp.path().join("file2.txt");
        let f3 = tmp.path().join("file3.txt");
        std::fs::write(&f1, "content 1").unwrap();
        std::fs::write(&f2, "content 2").unwrap();
        let patch = format!(
            "*** Begin Patch\n*** Add File: {}\n+new\n*** Update File: {}\n@@\n-content 1\n+updated 1\n*** Delete File: {}\n*** End Patch",
            f3.display(), f1.display(), f2.display()
        );
        let result = apply_patch(&patch).await.unwrap();
        assert_eq!(result.added.len(), 1);
        assert_eq!(result.modified.len(), 1);
        assert_eq!(result.deleted.len(), 1);
    }

    // Edge cases
    // 15. Empty file update
    #[tokio::test]
    async fn apply_patch_empty_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("empty.txt");
        std::fs::write(&file, "").unwrap();
        let patch = format!("*** Begin Patch\n*** Update File: {}\n@@\n+First line\n*** End Patch", file.display());
        let result = apply_patch(&patch).await.unwrap();
        assert_eq!(result.modified.len(), 1);
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "First line\n");
    }

    // 16. Multiple update chunks in same file
    #[tokio::test]
    async fn apply_patch_multiple_chunks() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("multi.txt");
        std::fs::write(&file, "line 1\nline 2\nline 3\nline 4\n").unwrap();
        let patch = format!(
            "*** Begin Patch\n*** Update File: {}\n@@\n line 1\n-line 2\n+LINE 2\n@@\n line 3\n-line 4\n+LINE 4\n*** End Patch",
            file.display()
        );
        let result = apply_patch(&patch).await.unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "line 1\nLINE 2\nline 3\nLINE 4\n");
    }

    // Error cases
    // 17. Update non-existent file throws
    #[tokio::test]
    async fn apply_patch_nonexistent_update_fails() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("does-not-exist.txt");
        let patch = format!("*** Begin Patch\n*** Update File: {}\n@@\n-old\n+new\n*** End Patch", file.display());
        assert!(apply_patch(&patch).await.is_err());
    }

    // 18. Delete non-existent file throws
    #[tokio::test]
    async fn apply_patch_nonexistent_delete_fails() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("does-not-exist.txt");
        let patch = format!("*** Begin Patch\n*** Delete File: {}\n*** End Patch", file.display());
        assert!(apply_patch(&patch).await.is_err());
    }

    // Fuzzy matching
    // 19. Trailing whitespace difference matched on pass 2
    #[test]
    fn fuzzy_match_trailing_whitespace() {
        let lines = ["line 1   ", "line 2", "line 3"];
        let pattern = ["line 1", "line 2"];
        let result = seek_sequence_test(&lines, &pattern, 0, false);
        assert_eq!(result, Some(0));
    }

    // 20. Unicode quote difference matched on pass 4
    #[test]
    fn fuzzy_match_unicode_quotes() {
        let lines = ["\u{201C}hello\u{201D}"];
        let pattern = ["\"hello\""];
        let result = seek_sequence_test(&lines, &pattern, 0, false);
        assert_eq!(result, Some(0));
    }
}
```

### Integration Test Mapping (from patch.test.ts)

| TS Test | Rust Test |
|---------|-----------|
| `parsePatch: should parse simple add file patch` | `parse_add_file_patch` |
| `parsePatch: should parse delete file patch` | `parse_delete_file_patch` |
| `parsePatch: should parse patch with multiple hunks` | `parse_multiple_hunks` |
| `parsePatch: should parse file move operation` | `parse_file_move` |
| `parsePatch: should throw error for invalid patch format` | `parse_invalid_patch_errors` |
| `maybeParseApplyPatch: should parse direct apply_patch command` | `detect_apply_patch_command` |
| `maybeParseApplyPatch: should parse applypatch command` | `detect_applypatch_alias` |
| `maybeParseApplyPatch: should handle bash heredoc format` | `detect_bash_heredoc` |
| `maybeParseApplyPatch: should return NotApplyPatch for non-patch` | `non_patch_command_returns_not_apply_patch` |
| `applyPatch: should add a new file` | `apply_patch_adds_new_file` |
| `applyPatch: should delete an existing file` | `apply_patch_deletes_file` |
| `applyPatch: should update an existing file` | `apply_patch_updates_file` |
| `applyPatch: should move and update a file` | `apply_patch_moves_file` |
| `applyPatch: should handle multiple operations in one patch` | `apply_patch_multiple_operations` |
| `applyPatch: should create parent directories when adding files` | `apply_patch_creates_parent_dirs` |
| `error handling: should throw error when updating non-existent file` | `apply_patch_nonexistent_update_fails` |
| `error handling: should throw error when deleting non-existent file` | `apply_patch_nonexistent_delete_fails` |
| `edge cases: should handle empty files` | `apply_patch_empty_file` |
| `edge cases: should handle files with no trailing newline` | `apply_patch_no_trailing_newline` |
| `edge cases: should handle multiple update chunks in single file` | `apply_patch_multiple_chunks` |

---
*Source: `packages/opencode/src/patch/index.ts`*
*Tests: `packages/opencode/test/patch/patch.test.ts`*
