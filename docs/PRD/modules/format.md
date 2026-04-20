# Module PRD: format (Code Formatter Service)

## 1. Module Overview

| Field | Value |
|-------|-------|
| **Module Name** | format |
| **Source Path** | `packages/opencode/src/format/` |
| **Type** | Service / Effect Layer |
| **Rust Crate** | `opencode-format` (or within `opencode-tools`) |
| **Purpose** | Provides automatic code formatting after file edits by detecting and running the appropriate formatter for each file extension. Supports 25+ formatters across many languages. Formatters are discovered at runtime based on availability and project configuration. |

## 2. Functionality

The format module provides:

1. **Formatter Registry** (`formatter.ts`): Static definitions of 25+ formatters, each describing their name, supported file extensions, and an async `enabled()` check that returns either a command array or `false`. The `$FILE` placeholder in commands is substituted with the actual file path at runtime.

2. **Format Service** (`index.ts`): An Effect-based service that:
   - Initializes formatter state per project instance (directory-scoped)
   - Reads config to determine which formatters are active
   - Runs matching formatters for a given file path, in sequence per file
   - Checks formatter availability in parallel across different formatters
   - Spawns child processes to execute formatters

3. **Config Integration**: Supports three config modes:
   - `formatter: false` → no formatters loaded
   - `formatter: true` → all built-in formatters loaded
   - `formatter: { <name>: { disabled?, command?, extensions? } }` → per-formatter override

### Built-in Formatters (25+)

| Formatter | Extensions | Detection |
|-----------|-----------|-----------|
| `gofmt` | `.go` | `which gofmt` |
| `mix` | `.ex,.exs,.eex,.heex` | `which mix` |
| `prettier` | `.js,.ts,.html,.css,.json,.yaml,.md,...` | `package.json` has `prettier` dep |
| `oxfmt` | `.js,.ts,...` | `OPENCODE_EXPERIMENTAL_OXFMT` flag + `package.json` |
| `biome` | `.js,.ts,.html,.css,...` | `biome.json` config file + `@biomejs/biome` bin |
| `zig` | `.zig,.zon` | `which zig` |
| `clang-format` | `.c,.cc,.cpp,.h,...` | `.clang-format` config file |
| `ktlint` | `.kt,.kts` | `which ktlint` |
| `ruff` | `.py,.pyi` | `which ruff` + ruff config |
| `uv` | `.py,.pyi` | `which uv` (fallback when ruff absent) |
| `air` | `.R` | `which air` + validates it's R formatter |
| `rubocop` | `.rb,.rake,...` | `which rubocop` |
| `standardrb` | `.rb,...` | `which standardrb` |
| `htmlbeautifier` | `.erb,.html.erb` | `which htmlbeautifier` |
| `dart` | `.dart` | `which dart` |
| `ocamlformat` | `.ml,.mli` | `.ocamlformat` config file |
| `terraform` | `.tf,.tfvars` | `which terraform` |
| `latexindent` | `.tex` | `which latexindent` |
| `gleam` | `.gleam` | `which gleam` |
| `shfmt` | `.sh,.bash` | `which shfmt` |
| `nixfmt` | `.nix` | `which nixfmt` |
| `rustfmt` | `.rs` | `which rustfmt` |
| `pint` | `.php` | `composer.json` has `laravel/pint` |
| `ormolu` | `.hs` | `which ormolu` |
| `cljfmt` | `.clj,.cljs,...` | `which cljfmt` |
| `dfmt` | `.d` | `which dfmt` |

### Ruff/UV Linked Disabling
Disabling either `ruff` OR `uv` removes BOTH from the active formatters (they share the same backend binary).

## 3. API Surface

### formatter.ts

```typescript
export interface Context extends Pick<InstanceContext, "directory" | "worktree"> {}

export interface Info {
  name: string
  environment?: Record<string, string>  // env vars for the process
  extensions: string[]
  enabled(context: Context): Promise<string[] | false>  // returns command or false
}

// Named formatter exports (one per formatter):
export const gofmt: Info
export const mix: Info
export const prettier: Info
export const oxfmt: Info
export const biome: Info
export const zig: Info
export const clang: Info
export const ktlint: Info
export const ruff: Info
export const rlang: Info
export const uvformat: Info
export const rubocop: Info
export const standardrb: Info
export const htmlbeautifier: Info
export const dart: Info
export const ocamlformat: Info
export const terraform: Info
export const latexindent: Info
export const gleam: Info
export const shfmt: Info
export const nixfmt: Info
export const rustfmt: Info
export const pint: Info
export const ormolu: Info
export const cljfmt: Info
export const dfmt: Info
```

### index.ts (Format Service)

```typescript
export const Status = z.object({
  name: z.string(),
  extensions: z.string().array(),
  enabled: z.boolean(),
})
export type Status = { name: string; extensions: string[]; enabled: boolean }

export interface Interface {
  readonly init: () => Effect.Effect<void>
  readonly status: () => Effect.Effect<Status[]>
  readonly file: (filepath: string) => Effect.Effect<void>
}

export class Service extends Context.Service<Service, Interface>()("@opencode/Format") {}
export const layer: Layer.Layer<Service, never, Config.Service | ChildProcessSpawner>
export const defaultLayer: Layer.Layer<Service>
```

## 4. Data Structures

### Internal State (per directory)

```typescript
{
  formatters: Record<string, Formatter.Info>    // active formatters by name
  commands: Record<string, string[] | false>    // cached enabled() results
  isEnabled: (item: Formatter.Info) => Promise<boolean>
  formatFile: (filepath: string) => Effect.Effect<void>
}
```

### Config Shape

```typescript
// From Config module:
type FormatterConfig =
  | false                           // disable all
  | true                            // enable all built-ins
  | Record<string, {
      disabled?: boolean            // exclude this formatter
      command?: string[]            // override command
      extensions?: string[]         // override extensions
    }>
```

## 5. Dependencies

| Dependency | Purpose |
|------------|---------|
| `effect` (Effect, Layer, Context) | Service/layer injection, async composition |
| `effect/unstable/process` (ChildProcess, ChildProcessSpawner) | Spawning formatter child processes |
| `@/effect/cross-spawn-spawner` | Cross-platform process spawning |
| `@/effect` (InstanceState) | Per-directory state scoping |
| `@/config` (Config.Service) | Reading formatter configuration |
| `@/npm` (Npm) | Resolving npm-local formatter binaries |
| `@/util` (Filesystem, Process, which) | File system and process utilities |
| `@/flag` (Flag) | Feature flag for experimental formatters |
| `remeda` (mergeDeep) | Deep merging built-in formatter with user config |
| `zod` | Status schema validation |

## 6. Acceptance Criteria

- [ ] `Format.Service` is available via Effect dependency injection
- [ ] `status()` returns empty list when `formatter: false` in config
- [ ] `status()` returns all 25+ built-in formatters when `formatter: true`
- [ ] `status()` only lists formatters not marked `disabled: true`
- [ ] Disabling `ruff` also removes `uv` and vice versa
- [ ] `file(path)` runs ALL matching formatters for the file's extension, in sequence
- [ ] Multiple formatters matching the same extension run sequentially (not concurrently), preserving order
- [ ] Availability checks (`enabled()`) for different formatters run in parallel
- [ ] Custom `command` in config overrides the built-in command
- [ ] Custom `extensions` in config replaces the built-in extension list
- [ ] Formatter environment variables are passed to child processes
- [ ] `$FILE` placeholder in commands is substituted with the actual file path
- [ ] Failed formatter process logs error but does NOT throw (best-effort formatting)
- [ ] Formatter state is scoped per project directory (InstanceState)

## 7. Rust Implementation Guidance

### Crate
`opencode-format` crate in `opencode-rust/crates/format/`

### Key Crates
- `tokio` — async runtime for spawning child processes
- `tokio::process::Command` — spawning formatter subprocesses
- `serde` / `serde_json` — config deserialization
- `which` crate — finding binaries in PATH
- `tracing` — logging formatter activity

### Recommended Approach

```rust
// Formatter trait (equivalent to Info interface)
#[async_trait]
pub trait Formatter: Send + Sync {
    fn name(&self) -> &str;
    fn extensions(&self) -> &[&str];
    fn environment(&self) -> Option<&HashMap<String, String>> { None }
    async fn enabled(&self, ctx: &FormatterContext) -> Option<Vec<String>>;
    // Returns Some(command) if enabled, None if disabled
}

// Context passed to enabled() check
pub struct FormatterContext {
    pub directory: PathBuf,
    pub worktree: PathBuf,
}

// Status returned to callers
#[derive(Serialize)]
pub struct FormatterStatus {
    pub name: String,
    pub extensions: Vec<String>,
    pub enabled: bool,
}

// Format Service
pub struct FormatService {
    formatters: HashMap<String, Box<dyn Formatter>>,
    commands: Mutex<HashMap<String, Option<Vec<String>>>>,
}

impl FormatService {
    pub async fn new(config: &FormatterConfig) -> Self { ... }

    pub async fn status(&self, ctx: &FormatterContext) -> Vec<FormatterStatus> { ... }

    pub async fn file(&self, filepath: &Path, ctx: &FormatterContext) -> Result<()> {
        let ext = filepath.extension().and_then(|e| e.to_str()).unwrap_or("");
        let matching = self.get_formatters_for_ext(ext, ctx).await;
        for (cmd, formatter) in matching {
            let replaced: Vec<String> = cmd.iter()
                .map(|s| s.replace("$FILE", filepath.to_str().unwrap()))
                .collect();
            let mut child = tokio::process::Command::new(&replaced[0])
                .args(&replaced[1..])
                .envs(formatter.environment().cloned().unwrap_or_default())
                .current_dir(ctx.directory.as_path())
                .spawn()?;
            let status = child.wait().await?;
            if !status.success() {
                tracing::error!("Formatter {} failed", formatter.name());
            }
        }
        Ok(())
    }
}

// Built-in formatter implementations
pub struct Gofmt;
#[async_trait]
impl Formatter for Gofmt {
    fn name(&self) -> &str { "gofmt" }
    fn extensions(&self) -> &[&str] { &[".go"] }
    async fn enabled(&self, _ctx: &FormatterContext) -> Option<Vec<String>> {
        which::which("gofmt").ok().map(|p| vec![
            p.to_string_lossy().into_owned(),
            "-w".into(),
            "$FILE".into(),
        ])
    }
}
```

### Config Design

```rust
#[derive(Deserialize)]
#[serde(untagged)]
pub enum FormatterConfig {
    Disabled(bool),              // false = disable all, true = enable all
    Custom(HashMap<String, FormatterOverride>),
}

#[derive(Deserialize, Default)]
pub struct FormatterOverride {
    pub disabled: Option<bool>,
    pub command: Option<Vec<String>>,
    pub extensions: Option<Vec<String>>,
}
```

## 8. Test Design

Tests are in `test/format/format.test.ts`. They use Effect layers with a tmp directory instance for isolation.

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // 1. status() returns empty list when formatter config is false
    #[tokio::test]
    async fn status_empty_when_disabled() {
        let service = FormatService::new(&FormatterConfig::Disabled(false)).await;
        let ctx = FormatterContext { directory: "/tmp".into(), worktree: "/tmp".into() };
        let statuses = service.status(&ctx).await;
        assert!(statuses.is_empty());
    }

    // 2. status() includes gofmt when formatter: true
    #[tokio::test]
    async fn status_includes_gofmt_when_all_enabled() {
        let service = FormatService::new(&FormatterConfig::Disabled(true)).await;
        let ctx = test_ctx();
        let statuses = service.status(&ctx).await;
        let gofmt = statuses.iter().find(|s| s.name == "gofmt");
        assert!(gofmt.is_some());
        assert!(gofmt.unwrap().extensions.contains(&".go".to_string()));
    }

    // 3. Disabled formatter excluded from status()
    #[tokio::test]
    async fn status_excludes_disabled_formatter() {
        let mut overrides = HashMap::new();
        overrides.insert("gofmt".to_string(), FormatterOverride { disabled: Some(true), ..Default::default() });
        let service = FormatService::new(&FormatterConfig::Custom(overrides)).await;
        let statuses = service.status(&test_ctx()).await;
        assert!(statuses.iter().find(|s| s.name == "gofmt").is_none());
    }

    // 4. Disabling ruff also removes uv
    #[tokio::test]
    async fn disabling_ruff_removes_uv() {
        let mut overrides = HashMap::new();
        overrides.insert("ruff".to_string(), FormatterOverride { disabled: Some(true), ..Default::default() });
        let service = FormatService::new(&FormatterConfig::Custom(overrides)).await;
        let statuses = service.status(&test_ctx()).await;
        assert!(statuses.iter().find(|s| s.name == "ruff").is_none());
        assert!(statuses.iter().find(|s| s.name == "uv").is_none());
    }

    // 5. Disabling uv also removes ruff
    #[tokio::test]
    async fn disabling_uv_removes_ruff() {
        let mut overrides = HashMap::new();
        overrides.insert("uv".to_string(), FormatterOverride { disabled: Some(true), ..Default::default() });
        let service = FormatService::new(&FormatterConfig::Custom(overrides)).await;
        let statuses = service.status(&test_ctx()).await;
        assert!(statuses.iter().find(|s| s.name == "ruff").is_none());
        assert!(statuses.iter().find(|s| s.name == "uv").is_none());
    }

    // 6. Two matching formatters run sequentially (ordering preserved)
    #[tokio::test]
    async fn matching_formatters_run_sequentially() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.seq");
        std::fs::write(&file, "x").unwrap();

        // Configure two custom formatters that append "A" and "B" respectively
        let mut overrides = HashMap::new();
        overrides.insert("first".to_string(), FormatterOverride {
            command: Some(vec!["sh".into(), "-c".into(), r#"v=$(cat "$1"); printf '%sA' "$v" > "$1""#.into(), "sh".into(), "$FILE".into()]),
            extensions: Some(vec![".seq".into()]),
            ..Default::default()
        });
        overrides.insert("second".to_string(), FormatterOverride {
            command: Some(vec!["sh".into(), "-c".into(), r#"v=$(cat "$1"); printf '%sB' "$v" > "$1""#.into(), "sh".into(), "$FILE".into()]),
            extensions: Some(vec![".seq".into()]),
            ..Default::default()
        });

        let service = FormatService::new(&FormatterConfig::Custom(overrides)).await;
        service.file(&file, &test_ctx()).await.unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        assert_eq!(content, "xAB");
    }

    // 7. State is isolated per project directory
    #[tokio::test]
    async fn formatter_state_isolated_per_directory() {
        let svc_disabled = FormatService::new(&FormatterConfig::Disabled(false)).await;
        let svc_enabled = FormatService::new(&FormatterConfig::Disabled(true)).await;
        let statuses_a = svc_disabled.status(&test_ctx()).await;
        let statuses_b = svc_enabled.status(&test_ctx()).await;
        assert!(statuses_a.is_empty());
        assert!(!statuses_b.is_empty());
    }

    // 8. $FILE placeholder substituted with actual path
    #[tokio::test]
    async fn file_placeholder_substituted() {
        // Test that the command runs with the real file path
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("placeholder.test");
        std::fs::write(&file, "original").unwrap();
        // Use a formatter that writes "touched" to the file
        // Verify the file was modified (meaning $FILE was correctly substituted)
        // ... implementation details depend on mock formatter setup
    }

    // 9. Failed formatter does not propagate error
    #[tokio::test]
    async fn failed_formatter_does_not_panic() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.go");
        std::fs::write(&file, "invalid go code").unwrap();
        // Even if gofmt fails, file() should not return an error
        // (best-effort formatting)
    }
}
```

### Integration Tests (from format.test.ts)

| TS Test | Rust Equivalent |
|---------|-----------------|
| `status() returns empty list when no formatters` | `status_empty_when_disabled` |
| `status() returns built-in formatters when formatter is true` | `status_includes_gofmt_when_all_enabled` |
| `status() keeps built-in formatters when config object provided` | `config_object_keeps_all_builtins` |
| `status() excludes formatters marked as disabled` | `status_excludes_disabled_formatter` |
| `status() excludes uv when ruff is disabled` | `disabling_ruff_removes_uv` |
| `status() excludes ruff when uv is disabled` | `disabling_uv_removes_ruff` |
| `service initializes without error` | `service_initializes_without_error` |
| `status() initializes formatter state per directory` | `formatter_state_isolated_per_directory` |
| `runs enabled checks for matching formatters in parallel` | `enabled_checks_run_in_parallel` |
| `runs matching formatters sequentially for the same file` | `matching_formatters_run_sequentially` |

---
*Source: `packages/opencode/src/format/` — formatter.ts, index.ts*
*Tests: `packages/opencode/test/format/format.test.ts`*
