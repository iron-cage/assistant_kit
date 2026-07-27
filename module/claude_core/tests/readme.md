# tests/

Unit tests for the `claude_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `paths_test.rs` | `ClaudePaths` path construction and `HOME`-absent guard |
| `settings_io_test.rs` | `get_string_setting` type-rejection/malformed-value behavior; `json_parse_flat_object` success/empty/error paths |
| `toml_io_test.rs` | `get_tiered` tier-merge/type-rejection behavior; `set_user_tier` round-trip and sibling-key preservation |
