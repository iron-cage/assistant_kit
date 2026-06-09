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

**Type inference:** The `value::` parameter is type-inferred before writing, using the same algorithm as `.settings.set`. See `algorithm/001_settings_type_inference.md`.

**Known settings catalog:** Defines the `model` key with env override; see `algorithm/002_config_resolution.md` § Catalog for the full list. `.config` accepts any arbitrary key in addition to catalog keys — unknown keys have no env mapping or default.

**Deprecation:** `.settings.show`, `.settings.get`, and `.settings.set` are deprecated in favor of `.config`. They continue to operate as before (no breaking change) but will be removed in a future version.

### Acceptance Criteria

| ID | Criterion |
|----|-----------|
| AC-01 | `.config` (no params) prints all resolved settings across all 4 layers in text format; exit 0 |
| AC-02 | `.config key::K` prints the resolved effective value for key K with the source layer (env/project/user/default/absent); exit 0 |
| AC-03 | `.config key::K value::V` writes K→V to `~/.claude/settings.json` with type inference; exit 0 |
| AC-04 | `.config key::K value::V scope::project` writes K→V to `{cwd}/.claude/settings.json`; exit 0 |
| AC-05 | `.config key::K unset::1` removes key K from the scope target settings file; exit 0 |
| AC-06 | `.config format::json` outputs resolved settings as a JSON object including source metadata per key |
| AC-07 | Resolution priority: env var → project config → user config → catalog default; each layer overrides the next |
| AC-08 | `.config key::K` exits 0 and prints an absent indicator when K has no value in any layer and no catalog default |
| AC-09 | `.config key::K value::V dry::1` prints the preview without modifying any file; exit 0 |
| AC-10 | HOME unset → exit 2 for any operation requiring `~/.claude/settings.json` access |
| AC-11 | `.config key::K value::V` for a non-catalog key writes the value without error (arbitrary key support) |
| AC-12 | Known settings catalog exposes: `model` (env: `CLAUDE_MODEL`, default: `claude-sonnet-4-6`), `preferredVersionSpec` (default: `stable`), `autoUpdates` (bool, default: `true`), `theme` (default: `system`), `hasCompletedOnboarding` (bool, default: `false`), `env.DISABLE_AUTOUPDATER` (default: absent) |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/003_settings_management.md](003_settings_management.md) | Deprecated `.settings.*` commands; I/O atomics |
| doc | [algorithm/001_settings_type_inference.md](../algorithm/001_settings_type_inference.md) | Type inference rules for value:: |
| doc | [algorithm/002_config_resolution.md](../algorithm/002_config_resolution.md) | 4-layer resolution algorithm and catalog |
| doc | [feature/004_dry_run.md](004_dry_run.md) | dry::1 preview mode semantics |
| doc | [feature/005_cli_design.md](005_cli_design.md) | CLI routing and required parameter validation |
| source | `../../src/commands.rs` | `.config` command handler |
| source | `../../../claude_version_core/src/config_catalog.rs` | Known settings catalog (SettingDef registry) |
| source | `../../../claude_version_core/src/config_resolve.rs` | 4-layer resolution engine |

### Sources

| File | Notes |
|------|-------|
| User design session 2026-06-09 | `.config` replaces `.settings.*`; resolution chain; catalog; scope param; unset param |
