# Invariant: No Param Mutation

### Scope

- **Purpose**: Guarantee that `.param.*` commands never write, modify, or delete any Claude Code settings, ensuring the subject remains strictly a read-only inspection surface.
- **Governs**: The `.param.list` and `.param.show` command handlers.
- **In Scope**: Every code path reachable from the `.param.*` command handlers.
- **Out of Scope**: `.patch.*` commands, which DO mutate state (install/uninstall/pin/unpin) by design (→ `feature/001_patch_cli.md`).

### Invariant Statement

`.param.*` command handlers MUST NOT call any settings-mutating function (e.g. `claude_core::settings_io::set_setting`, `remove_setting`) or any `claude_patch_core` install/uninstall/pin/unpin operation. They may only call read/resolve functions.

### Enforcement Mechanism

`.param.*` handlers are restricted, by construction, to importing only read-path functions from `claude_version_core` (`config_resolve::{resolve, resolve_all}`, `params_catalog::{lookup, params_catalog}`) plus environment and file reads. No handler imports `claude_core::settings_io::{set_setting, remove_setting}` or any `claude_patch_core` mutating operation. A code review or lint rule checking `.param.*` handler imports against this allow-list is the intended enforcement point once implementation begins.

### Violation Consequences

If a `.param.*` command silently mutated a setting (e.g. auto-persisting a discovered default), a user running `.param.show` purely to inspect provenance would unknowingly change Claude Code's actual configuration — violating the basic expectation that inspection commands are safe to run at any time, including from scripts or automation that assume read-only side effects.

### Features

| File | Relationship |
|------|--------------|
| [feature/002_param_cli.md](../feature/002_param_cli.md) | Full `.param.*` design this invariant constrains |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/param.rs` (to create) | `.param.*` command handlers — must satisfy the read-only import allow-list |

### Tests

| File | Relationship |
|------|--------------|
| `tests/param_cli.rs` (to create) | Asserts no settings.json mtime change after any `.param.*` invocation |
