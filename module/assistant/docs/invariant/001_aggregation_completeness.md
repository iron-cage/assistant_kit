# Invariant: Aggregation Completeness

### Scope

- **Purpose**: Guarantee that every Layer 2 crate included in the `ast` binary exposes a `register_commands()` function and that no Layer 2 CLI command exists outside the `ast` registry.
- **Responsibility**: Document the two-condition aggregation contract and the enforcement mechanism that prevents orphan commands.
- **In Scope**: All crates listed as optional dependencies in `assistant/Cargo.toml` under the `enabled` feature; any new Layer 2 crate added in future.
- **Out of Scope**: Layer 1 core crates (`claude_assets_core`, `claude_runner_core`, `claude_storage_core`) — these are domain libraries without CLI commands of their own.

### Invariant Statement

Every Layer 2 crate that exposes CLI commands MUST satisfy both conditions when included in assistant:

1. **`register_commands(fn(&mut CommandRegistry))` must exist** — the function signature must be `pub fn register_commands(registry: &mut unilang::registry::CommandRegistry)`. assistant calls this function in `build_registry()` to populate the shared registry.

2. **`COMMANDS_YAML: &str` constant must exist** — the constant must resolve to the absolute path of the crate's `unilang.commands.yaml` file. `build.rs` uses it for `rerun-if-changed` dependency tracking, ensuring the `ast` binary is rebuilt whenever any Layer 2 command definition changes.

**Current Layer 2 crates:**

| Crate | register_commands | COMMANDS_YAML | Registered in ast |
|-------|------------------|---------------|-------------------|
| claude_assets | ✅ | ✅ | ✅ |
| claude_version | ✅ | ✅ | ✅ |
| claude_profile | ✅ | ✅ | ✅ |
| claude_runner | ✅ | ✅ | ✅ |
| claude_storage | ✅ | ✅ | ✅ |

### Enforcement Mechanism

`build_registry()` in `src/lib.rs` calls each Layer 2 crate's `register_commands()` at compile time. If a crate lacks the function, compilation fails — the invariant is enforced by the type system. `build.rs` consumes `COMMANDS_YAML` paths for `cargo:rerun-if-changed` directives, ensuring the `ast` binary is rebuilt whenever any Layer 2 command definition changes. Build-time dependency tracking prevents stale command metadata from being silently ignored.

### Violation Consequences

A Layer 2 crate that declares CLI commands but is not registered in `build_registry()` creates orphan commands — commands that exist in the codebase but are never reachable from `ast`. Users lose the single entry point guarantee: CLI functionality scatters across standalone binaries with no way to discover all commands from one tool.

### Features

| File | Relationship |
|------|--------------|
| [../feature/001_super_app_aggregation.md](../feature/001_super_app_aggregation.md) | Registration sequence and precedence design |

### Sources

| File | Relationship |
|------|--------------|
| [../../src/lib.rs](../../src/lib.rs) | build_registry() — the authoritative registration call site |
| [../../build.rs](../../build.rs) | COMMANDS_YAML consumption for rerun-if-changed tracking |
| [../../Cargo.toml](../../Cargo.toml) | Layer 2 optional dependency declarations |

### Tests

| File | Relationship |
|------|--------------|
| [../../tests/cli_sanity.rs](../../tests/cli_sanity.rs) | Verifies Layer 2 crate commands are reachable through ast binary |
| [../../tests/aggregation.rs](../../tests/aggregation.rs) | Aggregation completeness invariant tests (IC-1..2) |
