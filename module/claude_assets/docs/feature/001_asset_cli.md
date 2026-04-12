# Feature: Asset CLI

### Scope

- **Purpose**: Document the four CLI commands of the `cla`/`claude_assets` binary that manage Claude Code artifact installations via symlink.
- **Responsibility**: Describe command signatures, argument semantics, adapter preprocessing, and the 5-phase unilang pipeline that connects argv to domain logic.
- **In Scope**: `.list`, `.install`, `.uninstall`, `.kinds` commands; `kind::` and `name::` arguments; `v::` alias; `installed::` bool normalisation; adapter dot-prefix enforcement; unilang pipeline; exit codes.
- **Out of Scope**: Domain install/uninstall semantics (→ `claude_assets_core/docs/feature/001_artifact_installer.md`), source root resolution constraint (→ `invariant/001_source_root_resolution.md`).

### Design

**Commands:** claude_assets exposes four commands registered via `register_commands()` into a unilang `CommandRegistry`:

| Command | Purpose | Required args | Optional args |
|---------|---------|---------------|---------------|
| `.list` | Survey available and installed artifacts | — | `kind::`, `installed::`, `verbosity::` |
| `.install` | Create symlink from `$PRO_CLAUDE/<kind>/<name>` into `.claude/<kind>/` | `kind::`, `name::` | — |
| `.uninstall` | Remove installed symlink from `.claude/<kind>/` | `kind::`, `name::` | — |
| `.kinds` | Print all artifact kinds with their source and target path mappings | — | — |

**`.list` behavior:** Without `kind::`, all six artifact kinds are shown. With `kind::rule` (or any valid kind name), only that kind is shown. With `installed::1`, only installed artifacts are shown (marked `●`); uninstalled artifacts are shown as `○`. Artifacts where neither source nor target directory is readable are silently skipped.

**`.install` behavior:** Both `kind::` and `name::` are required. The command calls `claude_assets_core::install::install()` and reports `Installed`, `Reinstalled`, or propagates an error. Install is idempotent — re-installing an already-installed artifact updates the symlink and reports `Reinstalled`.

**`.uninstall` behavior:** Both `kind::` and `name::` are required. Reports `Uninstalled` on success, or `Not installed: kind/name` when no symlink exists (not an error).

**`.kinds` behavior:** Iterates all six `ArtifactKind` variants and prints one line per kind showing the source path (`$PRO_CLAUDE/<subdir>/`) and target path (`.claude/<subdir>/`). Degrades gracefully when `$PRO_CLAUDE` is unset — prints the literal string `$PRO_CLAUDE` as the source root.

**Adapter preprocessing:** `argv_to_unilang_tokens()` in `src/adapter.rs` transforms raw argv into unilang token strings before the parser runs:
- Empty argv → `.help` (shows usage)
- First arg not starting with `.` → error (commands must start with dot)
- `v::N` → `verbosity::N` (alias expansion); values outside `[0, 2]` → error
- `installed::` values other than `0` or `1` → error (bool normalisation)
- Args missing `::` separator → error
- `-` prefixed flags → error (use `param::value` syntax instead)

**Pipeline:** The 5-phase unilang pipeline (argv → tokens → parse → semantic analysis → interpret) runs in `run_cli()`. Exit codes: 0 = success, 1 = usage/input error (`ArgumentMissing`, `ArgumentTypeMismatch`), 2 = runtime error (`InternalError`).

**Dual binary:** Both `claude_assets` and the `cla` alias binary call `run_cli()` from `lib.rs` — the pipeline is compiled once, not twice.

**Registry aggregation:** `register_commands()` is exported from `lib.rs` and called by `claude_tools::build_registry()` to include all four commands in the `clt` super-app without running a separate process.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lib.rs` | register_commands(), run_cli(), COMMANDS_YAML constant |
| source | `src/commands.rs` | list_routine, install_routine, uninstall_routine, kinds_routine |
| source | `src/adapter.rs` | argv_to_unilang_tokens(): alias expansion and validation |
| source | `unilang.commands.yaml` | CLI command metadata (names, arguments, examples) |
| invariant | [invariant/001_source_root_resolution.md](../invariant/001_source_root_resolution.md) | Rule: $PRO_CLAUDE must resolve before any install/list |
| feature | [claude_assets_core/docs/feature/001_artifact_installer.md](../../claude_assets_core/docs/feature/001_artifact_installer.md) | Domain install/uninstall semantics called by the routines |
| feature | [claude_tools/docs/feature/001_super_app_aggregation.md](../../claude_tools/docs/feature/001_super_app_aggregation.md) | How register_commands() is consumed by clt |
