# Invariant: Dependency Constraints

### Scope

- **Purpose**: Document the structural constraints on claude_runner's dependencies and module organization.
- **Responsibility**: State the zero consumer workspace dependency rule, binary dependency gating requirement, and prohibited file structure patterns.
- **In Scope**: Zero consumer workspace dep rule, `enabled` feature gating for binary deps, no routines.rs, no build.rs.
- **Out of Scope**: Default flag behavior (→ `invariant/001_default_flags.md`), API contracts (→ `api/001_public_api.md`).

### Invariant Statement

claude_runner must satisfy all of the following structural constraints simultaneously:

#### Zero Consumer Workspace Dependencies

The library surface of `claude_runner` must not depend on any consumer workspace crate — this includes `wplan`, `wplan_core`, and any `dream` variant. This constraint applies to the library target (`src/lib.rs`), not just `Cargo.toml` dependencies.

**Rationale:** YAML consumers (e.g. `dream`'s `build.rs`) aggregate `COMMANDS_YAML` at compile time. If the library itself depended on consumer workspace crates, aggregating consumers would pull in the entire consumer workspace dependency tree.

#### Binary Deps Gated by `enabled`

The binary dependencies — `claude_runner_core`, `error_tools`, and `unilang` — must be optional and gated behind the `enabled` feature in `Cargo.toml`. Library consumers that only need `COMMANDS_YAML` must not pull in these heavier dependencies.

Feature structure:
```
default = ["enabled"]
enabled = ["dep:claude_runner_core", "dep:error_tools", ..., "dep:unilang", ...]
```

#### No `routines.rs`

`routines.rs` must not exist in this crate. The pattern of a `routines.rs` top-level aggregation file is prohibited for `claude_runner`.

#### No `build.rs`

`claude_runner` does not generate a static registry at build time. It provides the YAML path (`COMMANDS_YAML`) for consumers to use in their own `build.rs`. Adding a `build.rs` to `claude_runner` itself would duplicate the aggregation responsibility that belongs to consumers.

### Enforcement Mechanism

- `Cargo.toml` structure enforces feature gating at compile time
- Absence of consumer workspace deps is verifiable by inspecting `Cargo.toml` and confirmed by `cargo +nightly udeps`
- Absence of `routines.rs` and `build.rs` is verifiable by directory inspection

### Violation Consequences

- **Consumer workspace dep added:** Library consumers acquire consumer workspace transitive deps, bloating compile times for pure YAML aggregation use cases
- **Binary deps ungated:** Library consumers that default-feature-enable get unnecessary runtime dependencies
- **routines.rs added:** Introduces an aggregation pattern inconsistent with this crate's design role
- **build.rs added:** Duplicates registration work that belongs to consumers; creates a conflicting code-gen step

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Separation of concerns (library vs binary) |
| doc | [api/001_public_api.md](../api/001_public_api.md) | COMMANDS_YAML constant that is the library's sole purpose |
| source | `../../Cargo.toml` | Dependency and feature flag definitions |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Constraints section (zero consumer workspace deps, binary deps gating, no routines.rs, no build.rs) |
