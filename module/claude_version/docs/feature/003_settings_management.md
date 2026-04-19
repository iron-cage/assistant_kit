# Feature: Settings Management

### Scope

- **Purpose**: Document the settings read, get, and set commands for `~/.claude/settings.json`.
- **Responsibility**: Describe settings JSON format, hand-rolled parser, atomic write, and nested object preservation.
- **In Scope**: `.settings.show`, `.settings.get`, `.settings.set`, JSON format, atomic temp-file write, nested object preservation.
- **Out of Scope**: Type inference for value:: (→ `algorithm/001_settings_type_inference.md`), dry-run mode (→ `feature/004_dry_run.md`).

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

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [algorithm/001_settings_type_inference.md](../algorithm/001_settings_type_inference.md) | Type inference rules for value:: |
| doc | [feature/004_dry_run.md](004_dry_run.md) | dry::1 preview mode for .settings.set |
| doc | [feature/005_cli_design.md](005_cli_design.md) | CLI routing and required parameter validation |
| source | `../../src/settings_io.rs` | Settings JSON read/write implementation |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-06, FR-15a, Command Inventory (commands 9-11), Parameter Inventory (key::, value::), Known Limitations |
