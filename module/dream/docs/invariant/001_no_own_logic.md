# Invariant: Zero Own Logic

### Scope

- **Purpose**: Enforce that `dream` contains no own type definitions, keeping it a pure facade.
- **Responsibility**: State the invariants, define the enforcement mechanism, and document violation consequences.
- **In Scope**: Forbidden definition forms in `src/` (INV-1), re-export purity (INV-2), layer dep restriction (INV-3).
- **Out of Scope**: Feature-gate activation behavior (→ `feature/001_aggregation.md`).

### Invariant Statement

| ID | Invariant |
|----|-----------|
| INV-1 | `src/` contains no `pub struct`, `pub fn`, `pub trait`, `pub enum`, or `pub type` definitions |
| INV-2 | All public items exported by `dream` originate from a core crate |
| INV-3 | `dream` has no dependency on any Layer 2 or Layer 3 crate |

### Enforcement Mechanism

**INV-1** is enforced by code review and the grep acceptance criterion:

```bash
grep -rn "^pub struct\|^pub fn\|^pub trait\|^pub enum\|^pub type" module/dream/src/
# Expected: empty output
```

**INV-2** is enforced structurally: `src/lib.rs` contains only `pub use crate_x::*` statements
inside `#[cfg(feature)]`-gated `pub mod` blocks. No item can be exported without originating
from a dep crate.

**INV-3** is enforced by the `Cargo.toml` `[dependencies]` section: only Layer 0
(`claude_core`), Layer 1 (`claude_profile_core`, `claude_runner_core`, `claude_version_core`, `claude_assets_core`),
and the out-of-hierarchy primitive (`claude_storage_core`) are listed. No Layer 2 CLI crate
(`claude_profile`, `claude_runner`, `claude_version`, `claude_storage`) or Layer 3
(`assistant`) may appear.

### Violation Consequences

- **INV-1 violated:** Own types in `dream` create a coupling point; consumers that only
  activate one feature now pull in types from unrelated domains. Breaks the zero-overhead
  facade promise.
- **INV-2 violated:** Items from unknown origin cannot be version-tracked against a specific
  core crate; breaks the single-source-of-truth model.
- **INV-3 violated:** Depending on a CLI crate pulls `unilang` and other CLI machinery into
  every library consumer's dep tree, negating the purpose of the Layer 0–1 core crate split.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/001_aggregation.md](../feature/001_aggregation.md) | Feature spec that this invariant constrains |
| doc | workspace `docs/pattern/001_crate_layering.md` | Layer definitions governing INV-3 |
| source | `../../src/lib.rs` | Implementation that must satisfy INV-1 and INV-2 |
| source | `../../Cargo.toml` | Dep declarations that must satisfy INV-3 |
