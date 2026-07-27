# API: TOML I/O

### Scope

- **Purpose**: Document the programmatic interface of the claude_core `toml_io` module.
- **Responsibility**: Specify the tiered (project + user) flat-TOML key-value read/write contract.
- **In Scope**: `get_tiered`, `set_user_tier`.
- **Out of Scope**: Path resolution (→ `paths` module, undocumented at this level), process utilities (→ `process` module, undocumented at this level), flat-JSON KV I/O (→ `settings_io` module, `001_settings_io.md`).

### Abstract

`claude_core::toml_io` is the shared Layer 0 primitive for reading and writing flat-TOML config files (`~/.clr/config.toml`, project-level `.clr.toml`, and similar) across a 2-tier project/user hierarchy. It hand-rolls its own line-oriented TOML parsing to avoid extra dependencies, interprets only plain double-quoted string values, and preserves every other key's raw (unparsed) value text verbatim through read-modify-write — this is what prevents a targeted single-key write from silently deleting sibling settings of a type this module doesn't interpret (numbers, bools, arrays, inline tables). Writes are atomic (temp file + rename). Added for `claude_runner_core::resolve_isolated_default_model()` to resolve a `model` preference across `.clr.toml`/`config.toml` tiers, mirroring `settings_io`'s existing JSON engine's design.

### Operations

#### `get_tiered(project_path: Option<&Path>, user_path: &Path, key: &str) -> Option<String>`

Returns a key's string value, checking `project_path` first (if given) then `user_path`. Missing files are treated as absent, not an error. Returns `None` if the key is absent from both tiers, or if its value is present but not a plain double-quoted string (numbers, bools, arrays, and inline tables are treated as "no preference" rather than coerced to a string).

#### `set_user_tier(user_path: &Path, key: &str, value: &str) -> Result<(), io::Error>`

Writes (or updates) a single key in the user-tier file, creating the file if absent. Every other key's raw value text is preserved unchanged, regardless of its type. Atomic write.

### Error Handling

`get_tiered` never returns an `Err` — any read failure (missing file, permission error, malformed line) is treated as "no value found" for that tier, consistent with the tiered-lookup contract's `Option<String>` return type. `set_user_tier` returns `Result<(), io::Error>`; a missing file is treated as an empty starting state rather than erroring, and the write itself fails loudly if the atomic rename cannot complete.

### Compatibility Guarantees

- `get_tiered` always prefers `project_path` over `user_path` when both provide a value for the same key.
- `set_user_tier` never alters any key other than the one targeted — sibling keys' raw text (including comments-adjacent formatting quirks aside) survive read-modify-write byte-for-byte.
- Only plain double-quoted string values are ever returned by `get_tiered`; every other TOML value type is preserved on write but never surfaced as a string.

### Sources

| File | Relationship |
|------|--------------|
| `../../src/toml_io.rs` | `get_tiered`, `set_user_tier`, hand-rolled flat-TOML parser/serializer |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/toml_io_test.rs` | Tier-merge precedence, type-rejection, absent-file, and sibling-key-preservation coverage |
