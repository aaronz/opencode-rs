# CLI-First Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a real CLI-first parity slice for `rust-opencode-port` by making `run`, `models`, and `providers` behave off actual config/model data instead of placeholders.

**Architecture:** Keep the current Clap-based command tree in `crates/cli` and upgrade only the approved slice. Load config once in the CLI layer, route `models` and `providers` through `opencode_llm::ModelRegistry` plus `opencode_core::Config`, and make `run` support a minimal non-interactive prompt path while preserving the existing TUI fallback.

**Tech Stack:** Rust, clap, serde_json, toml, tempfile, cargo test

---

## File Structure

```text
rust-opencode-port/crates/cli/src/
├── main.rs                     # modify: load config path and pass command context
└── cmd/
    ├── models.rs              # modify: replace stub output with ModelRegistry-backed output
    ├── providers.rs           # modify: replace static output with config/model-backed output
    └── run.rs                 # modify: add minimal non-interactive run flow for prompt mode

rust-opencode-port/crates/core/src/
└── config.rs                  # modify only if small helper accessors are needed

rust-opencode-port/crates/cli/tests/
├── e2e_model_workflows.rs     # modify: tighten list assertions to real fields
└── e2e_run_command.rs         # create: regression tests for prompt-mode run
```

---

## Task 1: Make `providers` return real provider data

**Files:**
- Modify: `rust-opencode-port/crates/cli/src/cmd/providers.rs`
- Modify: `rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs`

- [ ] **Step 1: Write the failing provider-list test**

Add this test block to `rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs`:

```rust
#[test]
fn test_provider_list_contains_real_status_fields() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["providers"]);

    assert_eq!(result["action"], "list");
    let providers = result["providers"].as_array().unwrap();
    assert!(!providers.is_empty(), "Should have at least one provider");

    let openai = providers.iter().find(|provider| provider["id"] == "openai").unwrap();
    assert_eq!(openai["name"], "OpenAI");
    assert!(openai.get("status").is_some());
    assert!(openai.get("enabled").is_some());
}
```

- [ ] **Step 2: Run the provider test to verify it fails**

Run: `cargo test -p opencode-cli --test e2e_model_workflows test_provider_list_contains_real_status_fields -- --exact`

Expected: FAIL because current `providers` output does not include `status` / `enabled` consistently beyond the hard-coded stub shape.

- [ ] **Step 3: Implement dynamic provider listing**

Replace the body of `rust-opencode-port/crates/cli/src/cmd/providers.rs` with:

```rust
use clap::Args;
use opencode_core::Config;
use opencode_llm::ModelRegistry;
use serde::Serialize;
use serde_json::json;

#[derive(Args, Debug)]
pub struct ProvidersArgs {
    #[arg(short, long)]
    pub json: bool,
}

#[derive(Serialize)]
struct ProviderRow {
    id: String,
    name: String,
    enabled: bool,
    status: String,
    model_count: usize,
}

fn title_case(id: &str) -> String {
    match id {
        "openai" => "OpenAI".to_string(),
        "anthropic" => "Anthropic".to_string(),
        "ollama" => "Ollama".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
}

pub fn run(args: ProvidersArgs) {
    let config = load_config();
    let registry = ModelRegistry::default();
    let provider_ids = ["openai", "anthropic", "ollama"];

    let providers: Vec<ProviderRow> = provider_ids
        .into_iter()
        .map(|id| {
            let enabled = config
                .enabled_providers
                .as_ref()
                .map(|enabled| enabled.iter().any(|value| value == id))
                .unwrap_or(true)
                && !config
                    .disabled_providers
                    .as_ref()
                    .map(|disabled| disabled.iter().any(|value| value == id))
                    .unwrap_or(false);

            ProviderRow {
                id: id.to_string(),
                name: title_case(id),
                enabled,
                status: if enabled { "available".to_string() } else { "disabled".to_string() },
                model_count: registry.list_by_provider(id).len(),
            }
        })
        .collect();

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "action": "list",
                "providers": providers,
            }))
            .unwrap()
        );
        return;
    }

    for provider in providers {
        println!("{}\t{}\t{}", provider.id, provider.status, provider.model_count);
    }
}
```

- [ ] **Step 4: Run the provider test to verify it passes**

Run: `cargo test -p opencode-cli --test e2e_model_workflows test_provider_list_contains_real_status_fields -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust-opencode-port/crates/cli/src/cmd/providers.rs rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs
git commit -m "feat(cli): back providers command with real registry data"
```

---

## Task 2: Make `models` return registry-backed model rows

**Files:**
- Modify: `rust-opencode-port/crates/cli/src/cmd/models.rs`
- Modify: `rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs`

- [ ] **Step 1: Write the failing models test**

Replace the first test in `rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs` with:

```rust
#[test]
fn test_models_list_contains_registry_fields() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["models"]);

    let models = result["models"].as_array().unwrap();
    assert!(!models.is_empty(), "Should have at least one model");

    let first = &models[0];
    assert!(first.get("id").is_some());
    assert!(first.get("name").is_some());
    assert!(first.get("provider").is_some());
    assert!(first.get("supports_streaming").is_some());
    assert!(first.get("max_input_tokens").is_some());
}
```

- [ ] **Step 2: Run the models test to verify it fails**

Run: `cargo test -p opencode-cli --test e2e_model_workflows test_models_list_contains_registry_fields -- --exact`

Expected: FAIL because current `models` returns placeholder rows without registry-backed fields.

- [ ] **Step 3: Implement registry-backed models listing**

Replace the body of `rust-opencode-port/crates/cli/src/cmd/models.rs` with:

```rust
use clap::{Args, Subcommand};
use opencode_llm::ModelRegistry;
use serde::Serialize;
use serde_json::json;

#[derive(Args, Debug)]
pub struct ModelsArgs {
    #[arg(short, long)]
    pub provider: Option<String>,

    #[arg(short, long)]
    pub json: bool,

    #[arg(short, long)]
    pub visibility: Option<String>,

    #[command(subcommand)]
    pub action: Option<ModelsAction>,
}

#[derive(Subcommand, Debug)]
pub enum ModelsAction {
    Visibility {
        #[arg(short, long)]
        hide: Option<String>,

        #[arg(short, long)]
        show: Option<String>,

        #[arg(long)]
        list_hidden: bool,
    },
}

#[derive(Serialize)]
struct ModelRow {
    id: String,
    name: String,
    provider: String,
    supports_streaming: bool,
    supports_functions: bool,
    max_input_tokens: u32,
}

pub fn run(args: ModelsArgs) {
    if let Some(ModelsAction::Visibility { .. }) = args.action {
        eprintln!("model visibility mutation is not part of this parity slice yet");
        std::process::exit(1);
    }

    let registry = ModelRegistry::default();
    let model_infos = match args.provider.as_deref() {
        Some(provider) => registry.list_by_provider(provider),
        None => registry.list(),
    };

    let mut models: Vec<ModelRow> = model_infos
        .into_iter()
        .map(|model| ModelRow {
            id: model.name.clone(),
            name: model.name.clone(),
            provider: model.provider.clone(),
            supports_streaming: model.supports_streaming,
            supports_functions: model.supports_functions,
            max_input_tokens: model.max_input_tokens,
        })
        .collect();

    models.sort_by(|left, right| left.id.cmp(&right.id));

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "action": "list",
                "models": models,
            }))
            .unwrap()
        );
        return;
    }

    for model in models {
        println!("{}\t{}\t{}", model.provider, model.id, model.max_input_tokens);
    }
}
```

- [ ] **Step 4: Run the models test to verify it passes**

Run: `cargo test -p opencode-cli --test e2e_model_workflows test_models_list_contains_registry_fields -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust-opencode-port/crates/cli/src/cmd/models.rs rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs
git commit -m "feat(cli): back models command with model registry"
```

---

## Task 3: Make `run` support minimal non-interactive prompt execution

**Files:**
- Create: `rust-opencode-port/crates/cli/tests/e2e_run_command.rs`
- Modify: `rust-opencode-port/crates/cli/src/cmd/run.rs`

- [ ] **Step 1: Write the failing run-command test**

Create `rust-opencode-port/crates/cli/tests/e2e_run_command.rs` with:

```rust
mod common;

use common::TestHarness;

#[test]
fn test_run_prompt_mode_returns_structured_output() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["run", "--prompt", "hello parity", "--model", "gpt-4o"]);

    assert!(output.status.success(), "run should succeed in prompt mode");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Prompt:"), "stdout should include prompt echo");
    assert!(stdout.contains("Model: gpt-4o"), "stdout should include selected model");
    assert!(stdout.contains("Mode: non-interactive"), "stdout should include execution mode");
}
```

- [ ] **Step 2: Run the run-command test to verify it fails**

Run: `cargo test -p opencode-cli --test e2e_run_command test_run_prompt_mode_returns_structured_output -- --exact`

Expected: FAIL because current `run` launches the TUI instead of producing deterministic prompt-mode output.

- [ ] **Step 3: Implement minimal prompt-mode execution**

Update `rust-opencode-port/crates/cli/src/cmd/run.rs` to:

```rust
use clap::Args;
use opencode_core::Config;
use opencode_tui::App;

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(short, long)]
    pub prompt: Option<String>,

    #[arg(short, long)]
    pub agent: Option<String>,

    #[arg(short, long)]
    pub model: Option<String>,

    #[arg(short, long)]
    pub continue_session: Option<String>,

    #[arg(short, long)]
    pub attach: Option<String>,

    #[arg(short = 'y', long)]
    pub yes: bool,

    #[arg(long)]
    pub title: Option<String>,
}

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
}

pub fn run(args: RunArgs) {
    if let Some(prompt) = args.prompt.clone() {
        let config = load_config();
        let model = args.model.or(config.model).unwrap_or_else(|| "gpt-4o".to_string());
        println!("Mode: non-interactive");
        println!("Model: {}", model);
        println!("Prompt: {}", prompt);
        return;
    }

    let mut app = App::new();

    if let Some(agent) = args.agent {
        app.agent = agent;
    }

    if let Err(error) = app.run() {
        eprintln!("Error running TUI: {}", error);
        std::process::exit(1);
    }
}
```

- [ ] **Step 4: Run the run-command test to verify it passes**

Run: `cargo test -p opencode-cli --test e2e_run_command test_run_prompt_mode_returns_structured_output -- --exact`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add rust-opencode-port/crates/cli/src/cmd/run.rs rust-opencode-port/crates/cli/tests/e2e_run_command.rs
git commit -m "feat(cli): add minimal non-interactive run mode"
```

---

## Task 4: Verify the whole approved slice together

**Files:**
- Modify: `rust-opencode-port/crates/cli/tests/e2e_core_flow.rs`
- Test: `rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs`
- Test: `rust-opencode-port/crates/cli/tests/e2e_run_command.rs`

- [ ] **Step 1: Tighten the core-flow providers test to match real output**

Ensure `rust-opencode-port/crates/cli/tests/e2e_core_flow.rs` contains:

```rust
#[test]
fn test_cli_providers_list() {
    let harness = TestHarness::setup();
    let result = harness.run_cli_json(&["providers"]);

    assert_eq!(result["action"], "list");
    let providers = result["providers"].as_array().unwrap();
    assert!(providers.iter().any(|provider| provider["id"] == "openai"));
    assert!(providers.iter().any(|provider| provider["id"] == "anthropic"));
    assert!(providers.iter().all(|provider| provider.get("status").is_some()));
}
```

- [ ] **Step 2: Run the focused CLI parity suite**

Run: `cargo test -p opencode-cli --test e2e_core_flow --test e2e_model_workflows --test e2e_run_command`

Expected: PASS for the approved `run` / `models` / `providers` slice.

- [ ] **Step 3: Run package diagnostics**

Run: `cargo check -p opencode-cli`

Expected: SUCCESS

- [ ] **Step 4: Commit**

```bash
git add rust-opencode-port/crates/cli/src/cmd/run.rs rust-opencode-port/crates/cli/src/cmd/models.rs rust-opencode-port/crates/cli/src/cmd/providers.rs rust-opencode-port/crates/cli/tests/e2e_core_flow.rs rust-opencode-port/crates/cli/tests/e2e_model_workflows.rs rust-opencode-port/crates/cli/tests/e2e_run_command.rs
git commit -m "feat(cli): implement first parity slice for run models and providers"
```

---

## Self-Review

- Spec coverage: the approved scope was `run`, `models`, `providers`, plus minimal config/model plumbing. Each command now has its own task and verification path.
- Placeholder scan: no TODO/TBD placeholders remain in tasks.
- Type consistency: the plan uses `Config`, `ModelRegistry`, `ProvidersArgs`, `ModelsArgs`, and `RunArgs` consistently across all tasks.
