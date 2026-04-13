# Constitution Updates - Iteration 15

**Generated:** 2026-04-13
**Based on Gap Analysis:** `iteration-15/gap-analysis.md`
**Previous Constitution:** `iteration-11/constitution_updates.md` (v2.7)
**Status:** Amendment Proposal

---

## Executive Summary

Iteration 15 gap analysis reveals **3 new P0 blockers** in the Tools System that were not adequately covered by the existing Constitution:

1. **Custom tool discovery format mismatch** — Implementation scans `TOOL.md` but PRD requires TypeScript/JavaScript
2. **Custom tools not registered with ToolRegistry** — Discovered tools never added to execution pipeline
3. **Plugin tool registration not integrated** — `ToolProvider` trait exists but is not connected to `ToolRegistry`

**Assessment:** Constitution v2.7 is **INADEQUATE** for current P0 gaps. New constitutional amendments required in Article III (Tools System).

**Net new constitutional content required:** 1 new amendment (P) + 3 sections under existing Article III

---

## Article I: Gap Analysis Summary (Iteration 15)

### P0 Blockers Identified

| Gap ID | Description | Module | Severity | Constitutional Coverage |
|--------|-------------|--------|----------|------------------------|
| P0-15-1 | Custom tool discovery scans TOOL.md, not .ts/.js | tools | **P0** | ❌ Not covered |
| P0-15-2 | Custom tools discovered but not registered | tools | **P0** | ❌ Not covered |
| P0-15-3 | Plugin tool registration not integrated | plugin | **P0** | ⚠️ Partial (trait exists, not used) |

### Critical Discrepancy

The Constitution Art III §3.2 (from Iteration 4) was marked as "Verified" since Iteration 6, but the Iteration 15 gap analysis reveals:

- `PluginToolAdapter` exists and implements `Tool` trait
- `ToolProvider` trait with `get_tools()` exists in `crates/plugin/src/lib.rs:80`
- **BUT** `ToolProvider` is never used — no implementation found
- `PluginManager` does not expose `register_tools()` to `ToolRegistry`

This represents a **constitutional verification failure** — the check was marked complete without actual implementation verification.

---

## Article II: Constitutional Coverage Analysis

### Constitution v2.7 Coverage for Iteration 15 P0s

| Constitution Reference | Mandate | Iteration 15 Status |
|------------------------|---------|---------------------|
| Art II §2.1 | Primary agent invariant | ✅ Verified |
| Art II §2.2 | Subagent lifecycle | ✅ Verified |
| Art II §2.3 | Task/delegation schema | ✅ Verified |
| Art III §3.1 | Deterministic hook order | ⚠️ **BROKEN** (IndexMap used but order depends on discovery) |
| Art III §3.2 | Plugin tool registration | ❌ **NOT INTEGRATED** |
| Art III §3.3 | Config ownership boundary | ✅ Verified |
| Art IV §4.1 | MCP transport | ✅ Verified |
| Art IV §4.2 | LSP diagnostics pipeline | ✅ Verified |
| Art V §5.1–5.3 | Server API hardening | ✅ Verified |
| Art VI §6.1 | Desktop WebView | ✅ Verified |
| Art VI §6.2 | ACP HTTP+SSE transport | ✅ Verified |
| Amend A §A.1 | Build integrity gate | ✅ Verified |
| Amend J §J.1 | Clippy linting gate | ✅ Verified |
| Amend K §K.1 | CLI test quality gate | ✅ Verified |
| Amend O §O.1 | CI Gate Enforcement | ✅ Verified |
| **NEW: Amend P** | **Custom tool discovery** | **❌ NOT COVERED** |
| **NEW: Art III §3.4** | **Custom tool registration** | **❌ NOT COVERED** |

---

## Article III: New P0 Requirements

### Section 3.4 - Custom Tool Discovery and Registration

**Requirement:** Custom tools MUST be discovered from TypeScript/JavaScript files and registered with the ToolRegistry:

```typescript
// File: .opencode/tools/my-tool/index.ts
// Format: ES module with export default

import { tool } from '@opencodehq/sdk';

export default tool({
  name: 'my-tool',
  description: 'A custom tool',
  input_schema: {
    type: 'object',
    properties: {
      arg1: { type: 'string' }
    }
  },
  async execute(args) {
    // Tool implementation
    return { result: args.arg1 };
  }
});
```

**CONSTRAINT:** Directory scanner MUST:
1. Scan `.opencode/tools/*/{index,.}*.{ts,js,mjs}` files
2. Parse ES module syntax to extract tool definition
3. Support dynamic import for execution (via V8/WASM runtime or exec)

**CONSTRAINT:** Discovered tools MUST be registered with ToolRegistry:
```rust
// Discovery → Registration flow
fn discover_and_register_custom_tools(base_path: &Path) -> Result<usize, Error> {
    let scanner = DirectoryScanner::new();
    let discovered = scanner.scan_tools(base_path);  // Now scans .ts/.js
    
    let mut registry = ToolRegistry::current();
    let mut count = 0;
    
    for tool_info in discovered {
        let tool = load_tool_from_ts(&tool_info.path)?;
        registry.register(tool)?;  // Actually register!
        count += 1;
    }
    Ok(count)
}
```

**P0 Gap Addressed:**
- "Custom tool discovery scans TOOL.md instead of .ts/.js"
- "Custom tools discovered but not registered with ToolRegistry"

### Section 3.5 - Plugin Tool Registration Integration

**Requirement:** Plugin tools MUST be integrated with ToolRegistry via ToolProvider:

```rust
// Plugin trait already has get_tools() via ToolProvider
// Missing: Integration with ToolRegistry

pub struct PluginManager {
    // ... existing fields ...
    tool_registry: Option<Arc<RwLock<ToolRegistry>>>,  // NEW
}

impl PluginManager {
    /// Connect to the global ToolRegistry
    pub fn connect_to_registry(&mut self, registry: Arc<RwLock<ToolRegistry>>) {
        self.tool_registry = Some(registry);
    }
    
    /// Register all plugin tools with the registry
    pub fn register_plugin_tools(&self) -> Result<(), PluginError> {
        let registry = self.tool_registry
            .ok_or_else(|| PluginError::NotConnected)?;
            
        for (name, plugin) in &self.plugins {
            if let Some(provider) = plugin.as_any().downcast_ref::<dyn ToolProvider>() {
                let tools = runtime::block_on(provider.get_tools());
                let mut reg = registry.write().unwrap();
                for tool_def in tools {
                    let adapter = PluginToolAdapter::new(tool_def, name.clone());
                    reg.register(adapter)?;  // Register via Tool trait
                }
            }
        }
        Ok(())
    }
}
```

**CONSTRAINT:** Tool collision priority (reinforced):
```
Priority: Built-in > Plugin > CustomProject > CustomGlobal
On collision: Warn in logs, first registered wins
```

**CONSTRAINT:** Plugin tool execution MUST go through permission gate:
```rust
// All plugin tools MUST be permission-checked before execution
async fn execute_plugin_tool(
    tool_name: &str,
    args: Value,
    context: &ExecutionContext,
) -> Result<ToolResult, ToolError> {
    // 1. Permission check
    context.permission_scope.check_tool(tool_name)?;
    // 2. Execute via PluginToolAdapter
    let tool = get_plugin_tool(tool_name)?;
    tool.execute(args).await
}
```

**P0 Gap Addressed:**
- "Plugin tool registration not implemented"
- "ToolProvider trait exists but not integrated"

### Section 3.6 - Hook Execution Determinism (Reinforcement)

**Issue:** Art III §3.1 was marked verified but hook execution order depends on plugin discovery order, not explicit priority.

**Requirement:** Hooks MUST execute in deterministic priority order:

```rust
pub trait Plugin: Send + Sync {
    // ... existing methods ...
    
    /// Hook execution priority (lower = earlier)
    /// Default: 0, System hooks: -100, User hooks: +100
    fn hook_priority(&self) -> i32 {
        0
    }
}

// Hook execution MUST be sorted
fn execute_hooks(hooks: &[&dyn Plugin], event: &str) {
    let mut sorted: Vec<_> = hooks.iter().collect();
    sorted.sort_by_key(|p| p.hook_priority());  // Deterministic!
    for hook in sorted {
        hook.on_hook(event)?;
    }
}
```

**CONSTRAINT:** IndexMap alone is NOT sufficient — insertion order depends on discovery order. Explicit priority sorting is REQUIRED.

**P0 Gap Addressed:**
- "Non-deterministic hook execution order"

---

## Article IV: Constitutional Verification Failure Report

### Verification Failure: Art III §3.2

| Iteration | Verification Report | Actual Status |
|-----------|---------------------|---------------|
| Iteration 6 | "Plugin tool registration ✅ Verified" | ❌ Not integrated |
| Iteration 7 | "Plugin tool registration ✅ Verified" | ❌ Not integrated |
| Iteration 8 | "Plugin tool registration ✅ Verified" | ❌ Not integrated |
| Iteration 9 | "Plugin tool registration ✅ Verified" | ❌ Not integrated |
| Iteration 10 | "Plugin tool registration ✅ Verified" | ❌ Not integrated |
| Iteration 11 | "Plugin tool registration ✅ Verified" | ❌ Not integrated |

**Root Cause:** Verification was performed on `crates/plugin/src/lib.rs` existence of `PluginToolAdapter` and `ToolProvider` trait, but never verified:
1. `ToolProvider::get_tools()` is called anywhere
2. `PluginManager::register_plugin_tools()` exists
3. Plugin tools appear in `ToolRegistry`

**Corrective Action:** Future verification MUST:
- Execute `cargo test -p opencode-plugin` and verify tool registration test
- Verify `ToolRegistry` contains plugin-registered tools
- Add integration test: `test_plugin_tools_available_in_registry`

---

## Article V: Updated Compliance Checklist

### Tools System (NEW — Amendment P + Art III §3.4-3.6)

- [ ] Custom tool discovery scans `.ts/.js` files, not `TOOL.md`
- [ ] Discovered custom tools registered with `ToolRegistry`
- [ ] Custom tool format follows ES module `export default tool({...})`
- [ ] `PluginManager::connect_to_registry()` implemented
- [ ] `PluginManager::register_plugin_tools()` implemented
- [ ] Plugin tools appear in `ToolRegistry::list_tools()`
- [ ] Hook execution sorted by `hook_priority()` (not just IndexMap)
- [ ] Tool collision priority enforced: Built-in > Plugin > Custom

### Build Quality Gate (Amendment A + J + M + O)
- [x] `cargo build --all` exits 0
- [x] `cargo test --all --no-run` exits 0
- [x] `cargo clippy --all --all-targets -- -D warnings` exits 0
- [ ] No P0 gaps in tools system

---

## Appendix A: Gap → Constitution Mapping (Iteration 15)

| Gap ID | Description | Constitution Reference | Status |
|--------|-------------|----------------------|--------|
| P0-15-1 | Custom tool discovery format | Amend P §P.1 | **NEW** |
| P0-15-2 | Custom tools not registered | Art III §3.4 | **NEW** |
| P0-15-3 | Plugin tool registration | Art III §3.5 | **NEW** |
| P0-15-4 | Hook execution non-deterministic | Art III §3.6 | **NEW** |
| P1-15-1 | Plugin config ownership not enforced | Art III §3.3 | ⚠️ Re-verify |
| P1-15-2 | Desktop/Web/ACP not implemented | Art VI | In progress |

---

## Appendix B: Constitution Lineage

| Version | Iteration | Articles | Key Additions |
|---------|-----------|----------|---------------|
| v1.0 | Iteration 1 | I–VI | Foundational principles |
| v2.0 | Iteration 4 | I–VII | Agent system, plugin, MCP/LSP, Server API |
| v2.1 | Iteration 5 | I–VII + A–D | Build gate, JSONC errors, slash commands |
| v2.2 | Iteration 6 | I–VII + A–F | Test code quality, ACP verification |
| v2.3 | Iteration 7 | I–VII + A–I | Desktop WebView, test enforcement |
| v2.4 | Iteration 8 | I–VII + A–L | Clippy hard gate, CLI tests |
| v2.5 | Iteration 9 | I–VII + A–N | Extended clippy coverage |
| v2.6 | Iteration 10 | I–VII + A–O | Clippy enforcement mechanism |
| v2.7 | Iteration 11 | I–VII + A–O | No changes (adequate) |
| **v2.8** | **Iteration 15** | **I–VII + A–P** | **Custom tool discovery/registration (P), Hook determinism (3.6)** |

---

## Priority Summary for Iteration 15

| Priority | Item | Action Required |
|----------|------|-----------------|
| **P0** | Custom tool discovery format | Implement TypeScript/JavaScript scanning |
| **P0** | Custom tool registration | Connect discovery to ToolRegistry |
| **P0** | Plugin tool registration | Implement ToolProvider integration |
| **P1** | Hook execution determinism | Add priority sorting |
| **P1** | Constitutional verification reform | Verify before marking complete |

**Constitutional additions in Iteration 15:** Amendment P (Custom Tools) + Art III §3.4-3.6

---

## Summary

**Overall Completion:** ~65-70% (regressed from 92-94% in Iteration 11)

**Constitutional Assessment: INADEQUATE**

The Constitution v2.7 is **inadequate** for current P0 blockers. Three new constitutional articles/sections are required:

1. **Amendment P:** Custom Tool Discovery — mandates TypeScript/JavaScript file format
2. **Article III §3.4:** Custom Tool Registration — mandates ToolRegistry integration
3. **Article III §3.5:** Plugin Tool Registration — mandates ToolProvider integration
4. **Article III §3.6:** Hook Determinism — reinforces IndexMap with priority sorting

**Critical Failure:** Art III §3.2 (Plugin tool registration) was marked "Verified" for 6 iterations without actual implementation. This represents a constitutional verification failure that must be addressed.

---

*Constitution v2.8 — Iteration 15*
*Total constitutional articles: 7 (original) + 16 amendments (A–P)*
*P0 blockers constitutionally covered: 3 new (P0-15-1, P0-15-2, P0-15-3)*
*Constitutional verification failure: Art III §3.2 marked verified without implementation*
*Report generated: 2026-04-13*