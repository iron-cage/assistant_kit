# Feature: Facade Aggregation

### Scope

- **Purpose**: Specify the feature-gate re-export behavior of the `dream` facade crate.
- **Responsibility**: Define which Cargo features activate which domain modules, isolation rules, and zero-dep behavior.
- **In Scope**: Feature-to-module mapping (FR-1–FR-5), zero-dep compile (FR-6), bundle features (FR-7), storage isolation (FR-8), independent activatability (FR-9).
- **Out of Scope**: Own type definitions (→ `invariant/001_no_own_logic.md`), workspace layering (→ workspace `docs/pattern/001_crate_layering.md`).

### Design

**Feature-to-module mapping:**

| ID | Requirement |
|----|-------------|
| FR-1 | When feature `common` is enabled, `dream::common` re-exports all public items from `claude_core` |
| FR-2 | When feature `storage` is enabled, `dream::storage` re-exports all public items from `claude_storage_core` |
| FR-3 | When feature `profile` is enabled, `dream::profile` re-exports all public items from `claude_profile_core` |
| FR-4 | When feature `runner` is enabled, `dream::runner` re-exports all public items from `claude_runner_core` |
| FR-5 | When feature `manager` is enabled, `dream::manager` re-exports all public items from `claude_version_core` |

**Compilation and isolation behavior:**

| ID | Requirement |
|----|-------------|
| FR-6 | With no features enabled, the crate compiles with zero runtime dependencies |
| FR-7 | Feature `full` enables all five domain modules simultaneously |
| FR-8 | Enabling `storage` does NOT activate `claude_core` as a runtime dependency |
| FR-9 | Each feature is independently activatable without enabling unrelated features |

**Feature graph (authoritative):**

```toml
[features]
default = []
common  = [ "dep:claude_core" ]
storage = [ "dep:claude_storage_core" ]
profile = [ "dep:claude_profile_core" ]
runner  = [ "dep:claude_runner_core" ]
manager = [ "dep:claude_version_core" ]
full    = [ "common", "storage", "profile", "runner", "manager" ]
enabled = [ "full" ]
```

**Re-export module pattern:**

Each domain module uses `#[cfg(feature = "X")]` gating on the `pub mod` declaration (not on
`pub use`) so that the module path does not exist unless the feature is active:

```rust
#[cfg(feature = "common")]
pub mod common {
  //! Re-exports from [`claude_core`].
  pub use claude_core::*;
}
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| invariant | [invariant/001_no_own_logic.md](../invariant/001_no_own_logic.md) | Zero-own-logic constraint complementing these feature specs |
| pattern | workspace `docs/pattern/001_crate_layering.md` | Layer 2 dep rules governing `dream`'s dependency set |
| source | `../../Cargo.toml` | Authoritative feature graph declaration |
