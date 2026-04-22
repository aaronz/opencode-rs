# PRD: Provider and Model System

## Overview

This document describes the provider/model abstraction used by OpenCode, including provider registration concepts, model selection, reasoning/variant behavior, and authentication patterns.

It is **not** the canonical config schema reference. For config ownership and exact config shape, see [06-configuration-system.md](./06-configuration-system.md).

---

## Provider Abstraction

### Representative Built-in Providers

OpenCode supports a broad set of hosted and local model providers. Representative examples include:
- OpenAI
- Anthropic
- Google
- Amazon Bedrock
- Azure OpenAI
- Ollama
- LM Studio
- Local models (via OpenAI-compatible APIs)

### Configuration Reference

Custom provider configuration is owned by the main config. See [Configuration System](./06-configuration-system.md) for the canonical schema and precedence model.

---

## Model Selection

### Format

Models use `provider/model-id` format:
- `openai/gpt-4o`
- `anthropic/claude-sonnet-4-5`
- `opencode/gpt-5.1-codex`

### Setting Default

The default model is set via the `model` key in `opencode.json`. See [Configuration System](./06-configuration-system.md) for schema details.

### Per-Agent Override

Agent-specific model overrides are configured via the `agent` section of main config. See [Configuration System](./06-configuration-system.md) for the canonical schema and [Agent System](./02-agent-system.md) for the execution model.

---

## Model Variants

Many models support variants with different configurations:

### Built-in Variants

**Anthropic:**
- `high` - High thinking budget (default)
- `max` - Maximum thinking

**OpenAI:**
- `none`, `minimal`, `low`, `medium`, `high`, `xhigh`

**Google:**
- `low`, `high`

### Custom Variants

Model variants are configured via the `provider` key in `opencode.json`. See [Configuration System](./06-configuration-system.md) for the canonical schema.

---

## Provider-Specific Options

### Amazon Bedrock

Provider options for Amazon Bedrock are configured via the `provider` key in `opencode.json`.

**Auth Priority:**
1. Bearer token (`AWS_BEARER_TOKEN_BEDROCK`)
2. AWS credentials chain (profile, keys, IRSA)

See [Configuration System](./06-configuration-system.md) for schema details.

### Azure OpenAI

Requires `AZURE_RESOURCE_NAME` environment variable.

### Local Models

Local model providers (Ollama, LM Studio, llama.cpp) are configured via the `provider` key in `opencode.json`. See [Configuration System](./06-configuration-system.md) for the canonical schema.

---

## Model Loading Priority

1. `--model` / `-m` CLI flag
2. `model` in opencode.json
3. Last used model
4. First available by internal priority

---

## Curated / Managed Offerings

Some OpenCode distributions may ship curated model offerings or managed endpoints with pre-tested defaults. These should be treated as product-layer conveniences on top of the provider/model abstraction, not a separate architecture.

### Available Models

| Model | Format | Pricing |
|-------|--------|---------|
| GPT 5.4 | openai/gpt-5.4 | Tiered |
| Claude Sonnet 4.6 | anthropic/claude-sonnet-4-6 | Tiered |
| Gemini 3 Pro | google/gemini-3-pro | Tiered |
| MiniMax M2.5 | opencode/minimax-m2.5 | $0.30/1M in |

### API Endpoints

```
https://opencode.ai/zen/v1/chat/completions  (OpenAI-compatible)
https://opencode.ai/zen/v1/messages         (Anthropic-compatible)
```

---

## Authentication

### Via /connect

Interactive auth flow in TUI.

### Via Environment

```bash
ANTHROPIC_API_KEY=sk-... opencode
OPENAI_API_KEY=sk-... opencode
```

### Via Config

Provider credentials can be configured via the `provider` key in `opencode.json`. See [Configuration System](./06-configuration-system.md) for the canonical schema.

---

## Model Limits

Model limits and provider-specific ceilings are configured through the canonical config system. See [Configuration System](./06-configuration-system.md).

---

## Thinking / Reasoning Budget

For models supporting extended reasoning or thinking controls, configuration belongs to the canonical config system. This document describes the concept only; see [Configuration System](./06-configuration-system.md) for ownership.

---

## Cross-References

| Document | Topic |
|----------|-------|
| [06-configuration-system.md](./06-configuration-system.md) | Canonical config schema and provider config ownership |
| [02-agent-system.md](./02-agent-system.md) | Agent-level model selection behavior |
| [07-server-api.md](./07-server-api.md) | Provider/model-related HTTP resource groups |
