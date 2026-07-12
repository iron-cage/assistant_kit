# API: Settings I/O

### Scope

- **Purpose**: Document the programmatic interface of the claude_core `settings_io` module.
- **Responsibility**: Specify the atomic flat-JSON key-value read/write contract, type inference rules, and return types.
- **In Scope**: `read_all_settings`, `get_setting`, `get_string_setting`, `set_setting`, `remove_setting`, `set_env_var`, `remove_env_var`, `json_parse_flat_object`, `json_escape`, `infer_type`, `StoredAs`.
- **Out of Scope**: Path resolution (→ `paths` module, undocumented at this level), process utilities (→ `process` module, undocumented at this level).

### Abstract

`claude_core::settings_io` is the shared Layer 0 primitive for reading and writing Claude Code's flat-JSON settings files (`settings.json`, `~/.clr/prefs.json`, and similar). It hand-rolls its own JSON parsing to avoid extra dependencies, infers scalar JSON types (`Bool`/`Number`/`Str`) from raw string input, preserves nested objects/arrays verbatim as opaque raw text across round-trips, and writes atomically (temp file + rename) to prevent partial-write corruption. Originally implemented in `claude_version_core`, relocated here so `claude_profile` and `claude_runner_core` can depend on the same engine without a workspace-crate dependency.

### Operations

#### `StoredAs`

Enum describing how a raw string value is represented in the JSON file: `Bool` (`"true"`/`"false"`), `Number` (parses as `i64` or finite `f64`), `Str` (anything else), `Raw` (nested object/array, or `null` — output verbatim, never quoted).

#### `infer_type(raw: &str) -> StoredAs`

Infers the `StoredAs` variant for a raw string value. `"0"` and `"1"` are intentionally classified `Number`, not `Bool`. Non-finite floats (`NaN`, `inf`, `infinity`) are rejected as `Number` and fall through to `Str`, since they are not valid JSON number literals.

#### `read_all_settings(path: &Path) -> Result<Vec<(String, String)>, io::Error>`

Reads all key-value pairs from a JSON file. Booleans are returned as `"true"`/`"false"`, numbers as their decimal string form, strings unquoted, nested structures as raw JSON text.

#### `get_setting(path: &Path, key: &str) -> Result<Option<String>, io::Error>`

Returns the stringified value for a single key, or `None` if absent, regardless of underlying JSON type.

#### `get_string_setting(path: &Path, key: &str) -> Result<Option<String>, io::Error>`

Returns the value for a single key only if its underlying JSON type is a plain string. Numbers, bools, `null`, and nested objects/arrays are treated as `None` ("no preference") rather than coerced to a string — use `get_setting` when the type-erased stringified form is wanted instead.

#### `set_setting(path: &Path, key: &str, raw_value: &str) -> Result<StoredAs, io::Error>`

Writes (or updates) a single key, creating the file if absent. Returns the `StoredAs` variant chosen for the value. Atomic write.

#### `remove_setting(path: &Path, key: &str) -> Result<(), io::Error>`

Removes a top-level key. No-op if the key or file is absent. Atomic write.

#### `set_env_var(path: &Path, key: &str, value: &str) -> Result<(), io::Error>`

Sets a key inside the `"env"` sub-object, creating the `"env"` key if absent. Environment variable values are always stored as JSON strings. Atomic write.

#### `remove_env_var(path: &Path, key: &str) -> Result<(), io::Error>`

Removes a key from the `"env"` sub-object. No-op if the key or `"env"` block is absent. Atomic write.

#### `json_parse_flat_object(src: &str) -> Result<Vec<(String, String, StoredAs)>, io::Error>`

Parses a JSON object string into typed key-value triples. Public so callers (e.g. `claude_version_core::config_resolve::resolve()`) can parse an already-fetched nested sub-object (like the `"env"` block) without re-reading the file from disk.

#### `json_escape(s: &str) -> String`

Escapes a string for embedding inside a JSON quoted string (`"`, `\`, `\n`, `\r`, `\t`).

### Error Handling

All fallible operations return `std::io::Error`. Read operations (`read_all_settings`, `get_setting`, `get_string_setting`) return `Err(NotFound)` if the file does not exist and `Err(InvalidData)` if the file is not well-formed JSON. Write operations (`set_setting`, `set_env_var`) treat a missing file as an empty starting state rather than erroring. Removal operations (`remove_setting`, `remove_env_var`) treat a missing file or absent key as a no-op success. `json_parse_flat_object` returns `Err(InvalidData)` if `src` is not a well-formed flat JSON object (missing braces, unquoted key, unterminated string, unbalanced nested structure).

### Compatibility Guarantees

- `StoredAs`'s four variants (`Bool`, `Number`, `Str`, `Raw`) are stable; `"0"`/`"1"` classify as `Number`, never `Bool`.
- All read functions accept any well-formed flat JSON object; nested objects/arrays always round-trip verbatim (byte-for-byte within whitespace normalization), never partially interpreted.
- All write functions (`set_setting`, `remove_setting`, `set_env_var`, `remove_env_var`) are atomic — a crash mid-write never leaves a partially-written settings file.

### Sources

| File | Relationship |
|------|--------------|
| `../../src/settings_io.rs` | `StoredAs`, all operations, hand-rolled JSON parser/serializer |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/settings_io_test.rs` | `get_string_setting` type-rejection coverage, parameter-trace structural guards, `json_parse_flat_object` mixed-type/empty/malformed coverage |
