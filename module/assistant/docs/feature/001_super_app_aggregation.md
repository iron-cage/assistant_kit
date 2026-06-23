# Feature: Super-App Aggregation

### Scope

- **Purpose**: Document how assistant aggregates all Layer 2 Claude CLI crates into a single `ast` binary via programmatic command registration.
- **Responsibility**: Describe the `build_registry()` assembly sequence, static YAML aggregation via `build.rs`, feature-gated compilation, and the adapter reuse contract.
- **In Scope**: `build_registry()` registration order and precedence, `register_commands()` pattern across Layer 2 crates, static YAML inclusion for YAML-backed commands, `claude_stub_routine`, exit codes, feature gate.
- **Out of Scope**: Individual command behavior in each Layer 2 crate, unilang 5-phase pipeline internals, `cla` standalone binary behavior.

### Design

**Architecture:** assistant is the Layer 3 super-app. It owns no domain logic of its own — its sole responsibility is to compose commands from five Layer 2 crates (`claude_assets`, `claude_version`, `claude_profile`, `claude_runner`, `claude_storage`) into a single `CommandRegistry` and run the shared unilang pipeline.

**Registration sequence:** `build_registry()` calls each Layer 2 crate's `register_commands()` in a fixed order that determines first-wins precedence for any command name collision:

```
claude_assets::register_commands(&mut registry)    // .list, .install, .uninstall, .kinds
claude_version::register_commands(&mut registry)   // .status, .version.*, .processes.*, .settings.*, .config
claude_profile::register_commands(&mut registry)   // .accounts, .account.*, .credentials.status, .model, .token.status, .paths, .usage
claude_runner::register_commands(&mut registry)    // runner programmatic commands
claude_storage::register_commands(&mut registry)   // .status (skipped — already registered by version)
register_static_commands(&mut registry)            // YAML-backed: .claude/.claude.help (stub), 11 storage commands
```

Duplicate registrations via `command_add_runtime` are silently skipped — the first registration wins. This means `claude_version`'s `.status` takes precedence over `claude_storage`'s `.status`.

**Static YAML aggregation:** `build.rs` concatenates the `unilang.commands.yaml` files from `claude_runner` and `claude_storage` into a compile-time-generated `static_commands.rs` (written to `OUT_DIR`). `register_static_commands()` maps each YAML-declared command name to a concrete routine function using a `phf::phf_map!` lookup table.

**Help rendering:** When `needs_help` is true (empty argv, `.help`, `--help`, `-h`), `print_usage()` renders grouped command output via `cli_fmt::CliHelpTemplate` to stdout and exits 0. Help is intercepted before the unilang pipeline. Commands are displayed in 8 groups: "Asset Management" (from cla), "Version Management" / "Settings & Config" / "Process Lifecycle" (from clv), "Account Management" / "Token & Model" (from clp), "Storage Query" / "System" (from YAML-backed static commands). Binary name is extracted via `std::env::args().next()`, not from `argv`, because `run_cli()` already applies `skip(1)` before passing `argv` to `cli::run()`.

**Adapter reuse:** `src/lib.rs` calls `claude_version::adapter::argv_to_unilang_tokens()` for argv preprocessing. assistant does not implement its own adapter — it delegates to the manager's adapter, which covers all commands in the registry.

**`.claude` stub:** In standalone `clr` context, `.claude` routes to Claude Code execution. In `ast` context, `.claude` and `.claude.help` route to `claude_stub_routine`, which prints a message directing the user to `clr`. This prevents `ast` from competing with `clr` as the execution entry point.

**Feature gate:** All Layer 2 dependencies are behind the `enabled` feature. Building without `--features enabled` produces an empty library shell — the intended behavior for library crates in this workspace. The `ast` binary target requires `enabled` via `required-features`.

### Features

| File | Relationship |
|------|--------------|
| [../../../claude_assets/docs/feature/001_asset_cli.md](../../../claude_assets/docs/feature/001_asset_cli.md) | register_commands() contract for claude_assets |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/001_aggregation_completeness.md](../invariant/001_aggregation_completeness.md) | Every Layer 2 crate must expose register_commands() |

### Sources

| File | Relationship |
|------|--------------|
| [../../src/lib.rs](../../src/lib.rs) | build_registry(), register_static_commands(), run(), feature gate |
| [../../src/main.rs](../../src/main.rs) | main() entry point — delegates to run_cli() |
| [../../build.rs](../../build.rs) | YAML aggregation that generates static_commands.rs |
| [../../Cargo.toml](../../Cargo.toml) | Layer 2 dependency declarations with feature gating |

### Tests

| File | Relationship |
|------|--------------|
| [../../tests/cli_sanity.rs](../../tests/cli_sanity.rs) | Compile and link sanity checks for ast binary against all Layer 2 crates |
| [../../tests/aggregation.rs](../../tests/aggregation.rs) | Super-app aggregation feature tests (FT-1..4) |
