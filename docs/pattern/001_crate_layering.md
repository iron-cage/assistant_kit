# Pattern: Crate Layering

### Scope

- **Purpose**: Document the four-layer crate dependency hierarchy governing the agent_kit workspace.
- **Responsibility**: Describe the layer definitions, Layer Invariant, permitted dep directions, and crate-to-layer assignments.
- **In Scope**: Layer 0–3 definitions, Layer Invariant (no cross-layer-N deps), dependency table, claude_storage_core position outside hierarchy.
- **Out of Scope**: Cross-workspace integration (→ `integration/001_willbe_integration.md`), privacy invariant (→ `invariant/001_privacy_invariant.md`).

### Problem

A workspace with 11 crates that have varying responsibilities risks uncontrolled dependency graphs — any crate can depend on any other, creating cycles and tight coupling. Without explicit layer rules, adding a dependency that "just works" today can create a cycle that prevents future refactoring or publishing.

### Solution

Strict four-layer hierarchy with one rule: **dependencies flow downward only**. No Layer N crate may depend on another Layer N crate.

```
Layer 3: claude_tools           (super-app aggregator — clt binary)
             ↓
Layer 2: agent_kit · claude_manager · claude_runner · claude_profile · claude_storage
             ↓
Layer 1: claude_profile_core · claude_manager_core · claude_runner_core
             ↓
Layer 0: claude_common          (zero workspace deps — ClaudePaths + process utilities)
```

**Permitted dependencies per layer:**

| Layer | Crate | Permitted workspace deps |
|-------|-------|--------------------------|
| 0 | `claude_common` | stdlib only (zero workspace deps) |
| 1 | `claude_profile_core` | `claude_common` + `error_tools` |
| 1 | `claude_manager_core` | `claude_common` + `error_tools` |
| 1 | `claude_runner_core` | `claude_common` + `error_tools` |
| 2 | `agent_kit` | Layer 0, 1, `claude_storage_core` — all optional via feature gates |
| 2 | `claude_profile` | Layer 0 + Layer 1 + `unilang` |
| 2 | `claude_manager` | Layer 0 + Layer 1 + `unilang` |
| 2 | `claude_runner` | Layer 0 + Layer 1 + `unilang` |
| 2 | `claude_storage` | `claude_storage_core` + `unilang` |
| 3 | `claude_tools` | any Layer 2 crate |

**`claude_storage_core` position:** Sits outside the layer hierarchy. It has no `claude_common` dependency (uses env-var paths, not `ClaudePaths`) and is a zero-dep JSONL parsing primitive. Layer 2's `claude_storage` wraps it.

### Applicability

This pattern applies when:
- Multiple related crates need clear ownership boundaries
- Higher-layer crates must be able to compose lower-layer crates without circular deps
- Individual crates in lower layers must be publishable and usable independently

### Consequences

**Benefits:**
- Layer Invariant prevents circular dependencies at compile time
- Lower-layer crates (Layer 0, 1) are publishable and usable without upper-layer overhead
- Adding a new crate only requires deciding which layer it belongs to

**Costs:**
- Breaking Layer Invariant requires refactoring to introduce a new layer or extract shared code
- Same-layer crates that need to share logic must move shared code down a layer

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| feature | [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Crate inventory that follows this pattern |
| invariant | [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | Privacy constraint on upstream deps (no willbe) |
| source | `../../Cargo.toml` | Workspace manifest enforcing member deps |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Four-Layer Crate Architecture section |
