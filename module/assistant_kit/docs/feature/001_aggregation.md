# Feature: Facade Aggregation

### Scope

- **Purpose**: Specify the feature-gate re-export behavior of the `assistant_kit` facade crate.
- **Responsibility**: Define which Cargo features activate which domain modules, how each feature activates its dependency's own CLI surface, and zero-dep behavior.
- **In Scope**: Feature-to-module mapping (FR-1–FR-5), zero-dep compile (FR-6), bundle features (FR-7, FR-10), transitive activation (FR-8), independent activatability (FR-9).
- **Out of Scope**: Own type definitions (→ `invariant/001_no_own_logic.md`), workspace layering (→ workspace `docs/pattern/001_crate_layering.md`), the behavior of the re-exported crates themselves (→ each Layer 2 crate's own `docs/`).

### Design

**Feature-to-module mapping:**

| ID | Requirement |
|----|-------------|
| FR-1 | When feature `profile` is enabled, `assistant_kit::profile` re-exports all public items from `claude_profile` |
| FR-2 | When feature `runner` is enabled, `assistant_kit::runner` re-exports all public items from `claude_runner` |
| FR-3 | When feature `version` is enabled, `assistant_kit::version` re-exports all public items from `claude_version` |
| FR-4 | When feature `assets` is enabled, `assistant_kit::assets` re-exports all public items from `claude_assets` |
| FR-5 | When feature `storage` is enabled, `assistant_kit::storage` re-exports all public items from `claude_storage` |

**Compilation and activation behavior:**

| ID | Requirement |
|----|-------------|
| FR-6 | With no features enabled, the crate compiles with zero runtime dependencies |
| FR-7 | Feature `full` enables all five domain modules simultaneously |
| FR-8 | Each domain feature additionally activates its dependency's own CLI-surface feature, so the re-exported command surface is present rather than compiled out |
| FR-9 | Each feature is independently activatable without enabling unrelated features |
| FR-10 | Feature `enabled` is an alias for `full`, matching the workspace-wide conventional activation name |

**FR-8 in detail — the activation feature is not uniform across the five deps:**

| Feature | Optional dep | Dependency feature activated |
|---------|--------------|------------------------------|
| `profile` | `claude_profile` | `claude_profile/enabled` |
| `runner` | `claude_runner` | `claude_runner/enabled` |
| `version` | `claude_version` | `claude_version/enabled` |
| `assets` | `claude_assets` | `claude_assets/enabled` |
| `storage` | `claude_storage` | `claude_storage/cli` |

`claude_storage` is the one asymmetry: its CLI surface is gated behind `cli`, not `enabled`.
Changing that gate name in `claude_storage` silently reduces `assistant_kit::storage` to the
library-only surface, so the mapping above is the authoritative record of the coupling.

**Feature graph (authoritative):**

```toml
[features]
default = []
profile = [ "dep:claude_profile", "claude_profile/enabled" ]
runner  = [ "dep:claude_runner",  "claude_runner/enabled"  ]
version = [ "dep:claude_version", "claude_version/enabled" ]
assets  = [ "dep:claude_assets",  "claude_assets/enabled"  ]
storage = [ "dep:claude_storage", "claude_storage/cli"     ]
full    = [ "profile", "runner", "version", "assets", "storage" ]
enabled = [ "full" ]

[dependencies]
claude_profile = { workspace = true, optional = true }
claude_runner  = { workspace = true, optional = true }
claude_version = { workspace = true, optional = true }
claude_assets  = { workspace = true, optional = true }
claude_storage = { workspace = true, optional = true }
```

**Re-export module pattern:**

Each domain module uses `#[cfg(feature = "X")]` gating on the `pub mod` declaration (not on
`pub use`) so that the module path does not exist unless the feature is active:

```rust
#[ cfg( feature = "profile" ) ]
pub mod profile
{
  //! Re-exports from [`claude_profile`].
  pub use claude_profile::*;
}
```

### Acceptance Criteria

FR-1–FR-5 and FR-9 are each verified by one `#[cfg(feature)]`-gated smoke test in
`tests/integration/facade_test.rs`, compiled and run per-feature so that a test only proves
its own feature's path:

```bash
cargo test -p assistant_kit --no-default-features --features profile --test integration
cargo test -p assistant_kit --no-default-features --features runner  --test integration
cargo test -p assistant_kit --no-default-features --features version --test integration
cargo test -p assistant_kit --no-default-features --features assets  --test integration
cargo test -p assistant_kit --no-default-features --features storage --test integration
cargo test -p assistant_kit --no-default-features --features full    --test integration
```

FR-6 is verified by a bare default-feature build producing no runtime dependency edges:

```bash
cargo build -p assistant_kit --no-default-features
cargo tree -p assistant_kit --no-default-features --edges normal
# Expected: assistant_kit alone, no dependency lines
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [invariant/001_no_own_logic.md](../invariant/001_no_own_logic.md) | Zero-own-logic constraint complementing these feature specs |
| doc | workspace `docs/pattern/001_crate_layering.md` | Layer 3 dep rules governing `assistant_kit`'s dependency set |
| doc | `../../../dream/docs/feature/001_aggregation.md` | The Layer 2 sibling facade — same pattern over `*_core` crates |
| source | `../../Cargo.toml` | Authoritative feature graph declaration |
| source | `../../src/lib.rs` | Implementation of the five re-export modules |
| test | `../../tests/integration/facade_test.rs` | Per-feature re-export path smoke tests |
