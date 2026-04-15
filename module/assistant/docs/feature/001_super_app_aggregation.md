# Feature: Super-App Aggregation

### Scope

- **Purpose**: Document how assistant aggregates all Layer 2 Claude CLI crates into a single `clt` binary via programmatic command registration.
- **Responsibility**: Describe the `build_registry()` assembly sequence, static YAML aggregation via `build.rs`, feature-gated compilation, and the adapter reuse contract.
- **In Scope**: `build_registry()` registration order and precedence, `register_commands()` pattern across Layer 2 crates, static YAML inclusion for YAML-backed commands, `claude_stub_routine`, exit codes, feature gate.
- **Out of Scope**: Individual command behavior in each Layer 2 crate, unilang 5-phase pipeline internals, `cla` standalone binary behavior.

### Design

**Architecture:** assistant is the Layer 3 super-app. It owns no domain logic of its own — its sole responsibility is to compose commands from five Layer 2 crates (`claude_assets`, `claude_version`, `claude_profile`, `claude_runner`, `claude_storage`) into a single `CommandRegistry` and run the shared unilang pipeline.

**Registration sequence:** `build_registry()` calls each Layer 2 crate's `register_commands()` in a fixed order that determines first-wins precedence for any command name collision:

```
claude_assets::register_commands(&mut registry)    // .list, .install, .uninstall, .kinds
claude_version::register_commands(&mut registry)   // .status, .version.*, .processes.*, .settings.*
claude_profile::register_commands(&mut registry)   // .account.*, .token.status, .paths, .usage
claude_runner::register_commands(&mut registry)    // .claude, .claude.help (stub in clt context)
claude_storage::register_commands(&mut registry)   // .status (skipped — already registered by manager)
register_static_commands(&mut registry)            // YAML-backed runner + storage commands
```

Duplicate registrations via `command_add_runtime` are silently skipped — the first registration wins. This means `claude_version`'s `.status` takes precedence over `claude_storage`'s `.status`.

**Static YAML aggregation:** `build.rs` concatenates the `unilang.commands.yaml` files from `claude_runner` and `claude_storage` into a compile-time-generated `static_commands.rs` (written to `OUT_DIR`). `register_static_commands()` maps each YAML-declared command name to a concrete routine function using a `phf::phf_map!` lookup table.

**Adapter reuse:** `src/main.rs` calls `claude_version::adapter::argv_to_unilang_tokens()` for argv preprocessing. assistant does not implement its own adapter — it delegates to the manager's adapter, which covers all commands in the registry.

**`.claude` stub:** In standalone `clr` context, `.claude` routes to Claude Code execution. In `clt` context, `.claude` and `.claude.help` route to `claude_stub_routine`, which prints a message directing the user to `clr`. This prevents `clt` from competing with `clr` as the execution entry point.

**Feature gate:** All Layer 2 dependencies are behind the `enabled` feature. Building without `--features enabled` produces an empty library shell — the intended behavior for library crates in this workspace. The `clt` binary target requires `enabled` via `required-features`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/main.rs` | build_registry(), register_static_commands(), main() |
| source | `src/lib.rs` | Feature gate and crate-level doc comment |
| source | `build.rs` | YAML aggregation that generates static_commands.rs |
| source | `Cargo.toml` | Layer 2 dependency declarations with feature gating |
| invariant | [invariant/001_aggregation_completeness.md](../invariant/001_aggregation_completeness.md) | Rule: every Layer 2 crate must expose register_commands() |
| feature | [claude_assets/docs/feature/001_asset_cli.md](../../claude_assets/docs/feature/001_asset_cli.md) | register_commands() contract for claude_assets |
