# Pattern: Crate Layering

### Scope

- **Purpose**: Document the four-layer crate dependency hierarchy governing the assistant workspace.
- **Responsibility**: Describe the layer definitions, Layer Invariant, permitted dep directions, and crate-to-layer assignments.
- **In Scope**: Layer 0–3 definitions, Layer Invariant (no cross-layer-N deps) and its known deviations, dependency table, Layer * position (claude_storage_core, claude_auth, claude_quota, claude_journal, svg_chart, json_redact, claude_pty_core — outside hierarchy).
- **Out of Scope**: Cross-workspace integration (→ `integration/001_consumer_integration.md`), privacy invariant (→ `invariant/001_privacy_invariant.md`).

### Problem

A workspace with 24 crates that have varying responsibilities risks uncontrolled dependency graphs — any crate can depend on any other, creating cycles and tight coupling. Without explicit layer rules, adding a dependency that "just works" today can create a cycle that prevents future refactoring or publishing.

### Solution

Strict four-layer hierarchy with one rule: **dependencies flow downward only**. No Layer N crate may depend on another Layer N crate.

```
Layer 3: assistant · assistant_kit                                   (cli + lib — not claude_-prefixed by design)
             ↓
Layer 2: dream                                                      (lib — not claude_-prefixed by design)
         claude_assets · claude_version · claude_runner · claude_profile · claude_storage · claude_journal_viewer  (cli)
             ↓
Layer 1: claude_assets_core · claude_profile_core · claude_version_core · claude_runner_core · claude_journal_charts
         claude_session_core · claude_daemon_core †
             ↓
Layer 0: claude_core                                                  (zero workspace deps — ClaudePaths + process utilities)
*        claude_storage_core                                            (zero-dep JSONL parser — no claude_core dep)
*        claude_auth                                                    (zero workspace deps — OAuth token refresh transport)
*        claude_quota                                                   (zero workspace deps — API rate-limit HTTP transport)
*        claude_journal                                                  (zero workspace deps — append-only event journal library)
*        svg_chart                                                       (zero workspace deps — SVG line/bar chart renderer)
*        json_redact                                                     (zero workspace deps — sensitive-value redaction)
*        claude_pty_core                                                 (zero workspace deps — pseudo-terminal session mechanics)
```

`†` participates in a known Layer Invariant deviation — see **Layer Invariant Deviations** below.

**Dependencies per crate:**

| Layer | Crate | Kind | Binaries |
|-------|-------|------|----------|
| 0 | `claude_core` | lib | — |
| * | `claude_storage_core` | lib | — |
| * | `claude_auth` | lib | — |
| * | `claude_quota` | lib | — |
| * | `claude_journal` | lib | — |
| * | `svg_chart` | lib | — |
| * | `json_redact` | lib | — |
| * | `claude_pty_core` | lib | — |
| 1 | `claude_assets_core` | lib | — |
| 1 | `claude_profile_core` † | lib | — |
| 1 | `claude_version_core` | lib | — |
| 1 | `claude_runner_core` | lib | — |
| 1 | `claude_journal_charts` | lib | — |
| 1 | `claude_session_core` | lib | — |
| 1 | `claude_daemon_core` † | lib | — |
| 2 | `dream` | lib | — |
| 2 | `claude_assets` | cli | `claude_assets`, `cla` |
| 2 | `claude_profile` | cli | `clp`, `claude_profile` |
| 2 | `claude_storage` | cli | `clg`, `claude_storage` |
| 2 | `claude_runner` | cli | `clr`, `c`, `claude_runner` |
| 2 | `claude_version` | cli | `clv`, `claude_version` |
| 2 | `claude_journal_viewer` | cli | `clj` |
| 3 | `assistant` | cli | `ast`, `assistant` |
| 3 | `assistant_kit` | lib | — |

`*` = outside layer hierarchy.

**Layer `*` position:** Seven crates sit outside the numbered layer hierarchy. They have no workspace dependencies (only external crate deps):
- `claude_storage_core` — zero-dep JSONL parsing primitive; uses env-var paths, not `ClaudePaths`; wrapped by Layer 2's `claude_storage`
- `claude_auth` — OAuth token refresh transport; standalone primitive usable without any workspace dep
- `claude_quota` — API rate-limit HTTP transport; standalone primitive usable without any workspace dep
- `claude_journal` — append-only event journal library; zero workspace deps; wrapped by Layer 2's `claude_journal_viewer`
- `svg_chart` — SVG line/bar chart renderer wrapping `plotters`; zero workspace deps; wrapped by Layer 1's `claude_journal_charts`
- `json_redact` — domain-agnostic sensitive-value redaction; zero workspace deps; consumed by Layer 2's `claude_profile`
- `claude_pty_core` — pseudo-terminal session mechanics; zero workspace deps; consumed by Layer 1's `claude_daemon_core` and Layer 2's `claude_runner`

### Layer Invariant Deviations

Two dependency edges in the current workspace violate the Layer Invariant stated above. Both are **default-on** (not opt-in), both are recorded here rather than silently tolerated, and neither has an agreed resolution yet:

| # | Edge | Both at | Conditionality | Status |
|---|------|---------|----------------|--------|
| D1 | `claude_profile_core` → `claude_runner_core` | Layer 1 | `optional = true`, but reached via `default = [ "enabled" ]` → active in a default build | Unresolved |
| D2 | `claude_daemon_core` → `claude_session_core` | Layer 1 | Unconditional | Unresolved |

**D1** predates the daemon stack. **D2** arrived with it: `claude_daemon_core` composes `claude_session_core` (session observation) with `claude_pty_core` (terminal mechanics) and `claude_core`, and is itself consumed by Layer 2's `claude_runner` — so it cannot move up to Layer 2 without creating a Layer 2 → Layer 2 edge instead.

The three resolutions the Consequences section already anticipates apply here: move the shared code down a layer, introduce an intermediate layer, or amend the Layer Invariant to permit an explicit, documented intra-layer ordering. Choosing among them is an open architectural decision, not a documentation fix — this section records the deviation so the diagram and table above are not read as claiming an invariant that the manifests do not currently satisfy.

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

### Features

| File | Relationship |
|------|--------------|
| [feature/001_workspace_design.md](../feature/001_workspace_design.md) | Crate inventory that follows this pattern |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_privacy_invariant.md](../invariant/001_privacy_invariant.md) | Privacy constraint: no private consumer workspace deps |

### Sources

| File | Relationship |
|------|--------------|
| `../../Cargo.toml` | Workspace manifest enforcing member deps |

### Provenance

| File | Relationship |
|------|--------------|
| `spec.md` (deleted — migrated here) | Four-Layer Crate Architecture section |
