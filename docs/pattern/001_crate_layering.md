# Pattern: Crate Layering

### Scope

- **Purpose**: Document the four-layer crate dependency hierarchy governing the assistant workspace.
- **Responsibility**: Describe the layer definitions, Layer Invariant, permitted dep directions, and crate-to-layer assignments.
- **In Scope**: Layer 0–3 definitions, Layer Invariant (no cross-layer-N deps), dependency table, Layer * position (claude_storage_core, claude_auth, claude_quota — outside hierarchy).
- **Out of Scope**: Cross-workspace integration (→ `integration/001_consumer_integration.md`), privacy invariant (→ `invariant/001_privacy_invariant.md`).

### Problem

A workspace with 15 crates that have varying responsibilities risks uncontrolled dependency graphs — any crate can depend on any other, creating cycles and tight coupling. Without explicit layer rules, adding a dependency that "just works" today can create a cycle that prevents future refactoring or publishing.

### Solution

Strict four-layer hierarchy with one rule: **dependencies flow downward only**. No Layer N crate may depend on another Layer N crate.

```
Layer 3: assistant                                                   (cli — not claude_-prefixed by design)
             ↓
Layer 2: dream                                                      (lib — not claude_-prefixed by design)
         claude_assets · claude_version · claude_runner · claude_profile · claude_storage  (cli)
             ↓
Layer 1: claude_assets_core · claude_profile_core · claude_version_core · claude_runner_core
             ↓
Layer 0: claude_core                                                  (zero workspace deps — ClaudePaths + process utilities)
*        claude_storage_core                                            (zero-dep JSONL parser — no claude_core dep)
*        claude_auth                                                    (zero workspace deps — OAuth token refresh transport)
*        claude_quota                                                   (zero workspace deps — API rate-limit HTTP transport)
```

**Dependencies per crate:**

| Layer | Crate | Kind | Binaries |
|-------|-------|------|----------|
| 0 | `claude_core` | lib | — |
| * | `claude_storage_core` | lib | — |
| * | `claude_auth` | lib | — |
| * | `claude_quota` | lib | — |
| 1 | `claude_assets_core` | lib | — |
| 1 | `claude_profile_core` | lib | — |
| 1 | `claude_version_core` | lib | — |
| 1 | `claude_runner_core` | lib | — |
| 2 | `dream` | lib | — |
| 2 | `claude_assets` | cli | `claude_assets`, `cla` |
| 2 | `claude_profile` | cli | `clp`, `claude_profile` |
| 2 | `claude_storage` | cli | `clg`, `claude_storage` |
| 2 | `claude_runner` | cli | `clr`, `claude_runner` |
| 2 | `claude_version` | cli | `clv`, `claude_version` |
| 3 | `assistant` | cli | `ast`, `assistant` |

`*` = outside layer hierarchy.

**Layer `*` position:** Three crates sit outside the numbered layer hierarchy. They have no workspace dependencies (only an optional `ureq` or no external dep):
- `claude_storage_core` — zero-dep JSONL parsing primitive; uses env-var paths, not `ClaudePaths`; wrapped by Layer 2's `claude_storage`
- `claude_auth` — OAuth token refresh transport; standalone primitive usable without any workspace dep
- `claude_quota` — API rate-limit HTTP transport; standalone primitive usable without any workspace dep

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
| invariant | [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | Privacy constraint: no private consumer workspace deps |
| source | `../../Cargo.toml` | Workspace manifest enforcing member deps |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Four-Layer Crate Architecture section |
