# src/account/

The `account` module — split by cluster; `mod.rs` glob-re-exports every submodule,
so the public path of every item stays `claude_profile_core::account::X`.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Module doc, submodule declarations, public re-exports |
| `types.rs` | `AccountBackend` and `Account` domain types |
| `store.rs` | Store CRUD: list, mutation lock, save, delete |
| `validate.rs` | Account-name validation and credential-filename mapping |
| `switch.rs` | Active-account switching and live-state patching |
| `session_settings.rs` | Live settings.json model/effort mutations |
| `refresh.rs` | OAuth token refresh and credential usability guards |
| `ownership.rs` | Machine identity, active markers, owner/claim fields |
| `tags.rs` | Tag normalization, set mutation ops, lazy role migration |
| `filter.rs` | Per-identity tag filter file IO and eligibility predicate |
| `renewal.rs` | Billing renewal override and timestamp helpers |
| `json_field.rs` | Dependency-free flat-JSON field extraction |
| `quota_cache.rs` | Per-host quota cache tree writes and merged reads |
| `history.rs` | Quota measurement history ring buffer |
