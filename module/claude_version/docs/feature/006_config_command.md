# Feature: Config Command

### Scope

- **Purpose**: Document the unified `.config` command for reading and writing Claude Code settings with 4-layer effective-value resolution.
- **Responsibility**: Describe the four operating modes (show-all / get / set / unset), 4-layer resolution chain, known settings catalog, scope parameter, and deprecation of the `.settings.*` commands.
- **In Scope**: `.config` show-all / get / set / unset modes, 4-layer resolution, `scope::` parameter, `unset::` parameter, known settings catalog, deprecation note.
- **Out of Scope**: Settings I/O atomics and nested-object preservation (→ `feature/003_settings_management.md`), type inference algorithm (→ `algorithm/001_settings_type_inference.md`), resolution algorithm step-by-step (→ `algorithm/002_config_resolution.md`), CLI routing and pipeline (→ `feature/005_cli_design.md`).

### Design

**Operating modes:** The mode is determined by the combination of `key::`, `value::`, and `unset::` parameters:

| key:: | value:: | unset:: | Mode |
|-------|---------|---------|------|
| absent | absent | absent/false | show-all: print all resolved settings |
| present | absent | absent/false | get: print resolved value for one key |
| present | present | absent/false | set: write value to target scope |
| present | absent | true | unset: delete key from target scope |

**Resolution chain:** When showing or getting a setting, the effective value is the first non-absent value found in this priority order:

1. Environment variable (e.g., `CLAUDE_MODEL` overrides `model`)
2. Project config: `{cwd}/.claude/settings.json` or ancestor `.claude/settings.json`
3. User config: `~/.claude/settings.json`
4. Catalog default (from the known settings catalog)

If no layer supplies a value, the key is absent (not an error). See `algorithm/002_config_resolution.md` for the full resolution algorithm.

**Scope parameter:** `scope::` controls which file is the write target for set/unset operations:

- `scope::user` (default) — writes to `~/.claude/settings.json`
- `scope::project` — writes to `{cwd}/.claude/settings.json` (creates directory and file if absent)

Providing `scope::` without a write operation (without `key:: + value::` or `key:: + unset::1`) exits 1 with `"scope:: applies to write operations only (key:: + value:: or key:: + unset::1)"`. `scope::` in show-all or get mode is always an error — regardless of which scope value was provided.

**Type inference:** The `value::` parameter is type-inferred before writing, using the same algorithm as `.settings.set`. See `algorithm/001_settings_type_inference.md`.

**Known settings catalog:** Defines the `model` key with env override; see `algorithm/002_config_resolution.md` § Catalog for the full list. `.config` accepts any arbitrary key in addition to catalog keys — unknown keys have no env mapping or default.

**Deprecation:** `.settings.show`, `.settings.get`, and `.settings.set` are deprecated in favor of `.config`. They continue to operate as before (no breaking change) but will be removed in a future version.

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/001_settings_type_inference.md](../algorithm/001_settings_type_inference.md) | Type inference rules for value:: writes |
| [algorithm/002_config_resolution.md](../algorithm/002_config_resolution.md) | 4-layer resolution algorithm and catalog |

### Features

| File | Relationship |
|------|-------------|
| [feature/003_settings_management.md](003_settings_management.md) | Deprecated .settings.* commands; I/O atomics |
| [feature/004_dry_run.md](004_dry_run.md) | dry::1 preview mode semantics |
| [feature/005_cli_design.md](005_cli_design.md) | CLI routing and required parameter validation |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/commands/config.rs` | .config command handler |
| `../../../claude_version_core/src/config_catalog.rs` | Known settings catalog (SettingDef registry) |
| `../../../claude_version_core/src/config_resolve.rs` | 4-layer resolution engine |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/006_config_command.md](../../tests/docs/feature/006_config_command.md) | Feature test spec |
