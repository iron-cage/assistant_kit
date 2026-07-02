# Feature: Settings Management

### Scope

- **Purpose**: Document the settings read, get, and set commands for `~/.claude/settings.json`.
- **Responsibility**: Describe settings JSON format, hand-rolled parser, atomic write, and nested object preservation.
- **In Scope**: `.settings.show`, `.settings.get`, `.settings.set`, JSON format, atomic temp-file write, nested object preservation.
- **Out of Scope**: Type inference for value:: (→ `algorithm/001_settings_type_inference.md`), dry-run mode (→ `feature/004_dry_run.md`).

> **Deprecation notice:** `.settings.show`, `.settings.get`, and `.settings.set` are deprecated in favor of the unified `.config` command (→ `feature/006_config_command.md`). They remain functional but will be removed in a future version.

### Design

**Settings format:** `~/.claude/settings.json` is a flat `{ "k1": v1, ... }` JSON object. Values may be booleans, integers, floats, strings, or nested objects.

**Nested object preservation:** `settings.json` may contain nested objects (e.g. `"env"`, `"enabledPlugins"`). The settings I/O layer preserves all nested structures during read/modify/write cycles. Nested objects are captured as raw JSON strings and written back verbatim. Only the `"env"` sub-object is actively manipulated (by version lock operations); other nested objects pass through untouched.

**Hand-rolled parser:** Read and write are implemented without serde. This avoids external dependencies and gives precise control over the flat+nested format.

**Atomic write:** `.settings.set` writes via atomic temp-file rename: writes to `{path}.json.tmp`, then renames to `{path}`. This prevents partial writes on crash.

**Commands:**
- `.settings.show` — prints all key-value pairs from `settings.json`; supports `format::json` for structured output
- `.settings.get key::K` — prints the value for key `K`; exits 1 if `key::` is missing or empty
- `.settings.set key::K value::V` — sets key `K` to value `V` (type-inferred); exits 1 if either parameter is missing or empty

**Required parameters:** `key::` and `value::` are semantically required by their respective commands. Absent or empty values produce exit 1 with `"{param} is required"` or `"{param} value cannot be empty"`.

**HOME dependency:** `ClaudePaths::new()` returns `None` when `HOME` is unset. Every settings handler must treat `None` as an error → exit 2.

### Algorithms

| File | Relationship |
|------|-------------|
| [algorithm/001_settings_type_inference.md](../algorithm/001_settings_type_inference.md) | Type inference rules for value:: |

### Features

| File | Relationship |
|------|-------------|
| [feature/004_dry_run.md](004_dry_run.md) | dry::1 preview mode for .settings.set |
| [feature/005_cli_design.md](005_cli_design.md) | CLI routing and required parameter validation |
| [feature/006_config_command.md](006_config_command.md) | Unified `.config` command replacing `.settings.*` |
| [feature/007_params_command.md](007_params_command.md) | `.params` reads settings.json config values for display (read-only) |

### Sources

| File | Relationship |
|------|-------------|
| `../../src/settings_io.rs` | Settings JSON read/write implementation |

### Provenance

| Source | Notes |
|--------|-------|
| `spec.md` (deleted) | FR-06, FR-15a, Command Inventory (commands 9-11), Parameter Inventory |

### Tests

| File | Relationship |
|------|-------------|
| [tests/docs/feature/003_settings_management.md](../../tests/docs/feature/003_settings_management.md) | Feature test spec |
