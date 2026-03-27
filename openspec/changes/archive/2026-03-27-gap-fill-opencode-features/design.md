## Context

The mycode project replicates opencode's .opencode configuration. Current state:
- mycode has 4 openspec commands (opsx-propose, opsx-apply, opsx-explore, opsx-archive) 
- mycode has 4 skills (openspec-propose, openspec-apply-change, openspec-explore, openspec-archive-change)
- Target opencode has 7 commands + 4 agents

## Goals / Non-Goals

**Goals:**
1. Add 7 missing commands to match target opencode's command set
2. Add 4 missing agents to match target opencode's agent set
3. Set up test infrastructure to validate implementations

**Non-Goals:**
- Modifying existing openspec commands/skills (they already work)
- Implementing the full test suite (target has 100+ test files)
- Replicating themes or glossary (non-essential for functionality)

## Decisions

### Decision 1: Command Format
Use YAML frontmatter with standard fields (description, model, subtask, etc.) matching target opencode exactly.

### Decision 2: Agent Format
Use YAML frontmatter with fields: mode, hidden, model, color, tools. Each agent uses specific tools.

### Decision 3: Test Infrastructure
Set up basic test structure matching target project's test organization. Focus on tool tests first.

## Risks / Trade-offs

- **Risk**: Some target commands may require external tools (gh cli) not available locally
  - **Mitigation**: Document tool requirements in command descriptions
  
- **Risk**: Test infrastructure requires understanding target's test framework
  - **Mitigation**: Study target's test/preload.ts and test/lib/ for setup patterns
