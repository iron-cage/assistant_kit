# Invariant: Zero Own Logic

### Scope

- **Purpose**: Enforce that `assistant_kit` contains no own type definitions, keeping it a pure facade.
- **Responsibility**: State the invariants, define the enforcement mechanism, and document violation consequences.
- **In Scope**: Forbidden definition forms in `src/` (INV-1), re-export purity (INV-2), dependency-set restriction (INV-3), no own CLI framework deps (INV-4).
- **Out of Scope**: Feature-gate activation behavior (→ `feature/001_aggregation.md`).

### Invariant Statement

| ID | Invariant |
|----|-----------|
| INV-1 | `src/` contains no `pub struct`, `pub fn`, `pub trait`, `pub enum`, or `pub type` definitions |
| INV-2 | All public items exported by `assistant_kit` originate from a Layer 2 crate |
| INV-3 | `assistant_kit` has no dependency on any Layer 3 crate |
| INV-4 | `assistant_kit` declares no direct dependency on a CLI framework crate (`unilang`, `error_tools`) |

**Note on INV-3 vs the sibling facade.** `dream` forbids depending on Layer 2 because it
re-exports `*_core` crates only. `assistant_kit` *requires* Layer 2 deps — that is its whole
purpose — and instead forbids Layer 3. The two invariants read alike but are not the same
constraint; do not copy one crate's `Cargo.toml` rule onto the other.

### Enforcement Mechanism

**INV-1** is enforced by code review and the grep acceptance criterion:

```bash
grep -rnE '^pub (struct|fn|trait|enum|type)' module/assistant_kit/src/
# Expected: empty output
```

**INV-2** is enforced structurally: `src/lib.rs` contains only `pub use crate_x::*` statements
inside `#[cfg(feature)]`-gated `pub mod` blocks. No item can be exported without originating
from a dep crate.

**INV-3** and **INV-4** are enforced by the `Cargo.toml` `[dependencies]` section, which lists
exactly the five Layer 2 crates and nothing else:

```bash
grep -nE 'unilang|error_tools|^assistant\b' module/assistant_kit/Cargo.toml
# Expected: empty output
```

### Violation Consequences

- **INV-1 violated:** Own types in `assistant_kit` create a coupling point; consumers that only
  activate one feature now pull in types from unrelated domains. Breaks the zero-overhead
  facade promise.
- **INV-2 violated:** Items from unknown origin cannot be version-tracked against a specific
  Layer 2 crate; breaks the single-source-of-truth model.
- **INV-3 violated:** Depending on `assistant` (the Layer 3 super-app binary) inverts the
  layering — the facade would depend on its own consumer, creating a dependency cycle at the
  workspace level.
- **INV-4 violated:** A direct CLI framework dep would be linked even when no domain feature is
  active, breaking FR-6's zero-dependency default build. The framework must arrive only
  transitively, through whichever Layer 2 crate a consumer actually opted into.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/001_aggregation.md](../feature/001_aggregation.md) | Feature spec that this invariant constrains |
| doc | workspace `docs/pattern/001_crate_layering.md` | Layer definitions governing INV-3 |
| doc | `../../../dream/docs/invariant/001_no_own_logic.md` | Layer 2 sibling's corresponding — and deliberately different — dep restriction |
| source | `../../src/lib.rs` | Implementation that must satisfy INV-1 and INV-2 |
| source | `../../Cargo.toml` | Dep declarations that must satisfy INV-3 and INV-4 |
