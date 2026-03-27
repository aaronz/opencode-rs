## ADDED Requirements

### Requirement: agent-triage
GitHub issue triaging agent with automatic labeling and assignment.

#### Scenario: Triage GitHub issues
- **WHEN** issue needs triage
- **THEN** system uses github-triage tool to triage issues
- **AND** applies labels: windows, perf, desktop, nix, zen, core, acp, docs, opentui
- **AND** assigns to appropriate team member based on issue category
- **AND** follows determinism rules for label assignment

#### Scenario: Label windows issues
- **WHEN** issue mentions "Windows" or "WSL"
- **THEN** system adds "windows" label
- **AND** assigns to Hona

#### Scenario: Label TUI issues
- **WHEN** issue is about keybindings, scroll speed, flickering, crashes with opentui
- **THEN** system adds "opentui" label
- **AND** assigns to kommander or rekram1-node

#### Scenario: Label core issues
- **WHEN** issue is about LSP server, harness, agent context, API, provider integration
- **THEN** system adds "core" label
- **AND** assigns to thdxr or rekram1-node

#### Scenario: Label desktop issues
- **WHEN** issue is about desktop app or "opencode web" command
- **THEN** system adds "desktop" label
- **AND** assigns randomly to Desktop/Web team

#### Scenario: Determinism rules
- **WHEN** applying labels
- **THEN** system follows rules:
  - No "zen" in title+body → no zen label
  - "nix" label added but no nix/nixos in title+body → drop "nix"
  - "nix/nixos" in title+body → add nix label and assign to rekram1-node
  - "desktop" label added → override assignee randomly
