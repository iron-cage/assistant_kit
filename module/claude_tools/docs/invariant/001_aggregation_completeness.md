# Invariant: Aggregation Completeness

### Scope

- **Purpose**: Guarantee that every Layer 2 crate included in the `clt` binary exposes a `register_commands()` function and that no Layer 2 CLI command exists outside the `clt` registry.
- **Governs**: `build_registry()` in `src/main.rs`; the `[features]` and `[dependencies]` sections of `Cargo.toml`; any future Layer 2 crate added to the workspace.
- **In Scope**: All crates listed as optional dependencies in `claude_tools/Cargo.toml` under the `enabled` feature; any new Layer 2 crate added in future.
- **Out of Scope**: Layer 1 core crates (`claude_assets_core`, `claude_runner_core`, `claude_storage_core`) — these are domain libraries without CLI commands of their own.

### Rule

Every Layer 2 crate that exposes CLI commands MUST satisfy both of the following conditions when included in claude_tools:

1. **`register_commands(fn(&mut CommandRegistry))` must exist** — the function signature must be `pub fn register_commands(registry: &mut unilang::registry::CommandRegistry)`. claude_tools calls this function in `build_registry()` to populate the shared registry.

2. **`COMMANDS_YAML: &str` constant must exist** — the constant must resolve to the absolute path of the crate's `unilang.commands.yaml` file. `build.rs` uses it for `rerun-if-changed` dependency tracking, ensuring the `clt` binary is rebuilt whenever any Layer 2 command definition changes.

**No orphan commands:** A Layer 2 crate that declares CLI commands but is not registered in `build_registry()` creates orphan commands — commands that exist in the codebase but are never reachable from `clt`. Orphan commands are a violation of this invariant.

**Rationale:** The aggregation contract is the mechanism by which `clt` achieves completeness. If any Layer 2 crate could define commands without registering them in `clt`, users would have no single entry point to all CLI functionality. The `COMMANDS_YAML` requirement ensures build-time dependency tracking prevents stale command metadata from being silently ignored.

**Current Layer 2 crates (as of TSK-089):**

| Crate | register_commands | COMMANDS_YAML | Registered in clt |
|-------|------------------|---------------|-------------------|
| claude_assets | ✅ | ✅ | ✅ |
| claude_manager | ✅ | ✅ | ✅ |
| claude_profile | ✅ | ✅ | ✅ |
| claude_runner | ✅ | ✅ | ✅ |
| claude_storage | ✅ | ✅ | ✅ |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/main.rs` | build_registry() — the authoritative registration call site |
| source | `build.rs` | COMMANDS_YAML consumption for rerun-if-changed tracking |
| source | `Cargo.toml` | Layer 2 optional dependency declarations |
| feature | [feature/001_super_app_aggregation.md](../feature/001_super_app_aggregation.md) | Registration sequence and precedence design |
