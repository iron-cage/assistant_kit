# src/

Source code for the `claude_version_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; `CoreError` enum and module declarations |
| `settings_io.rs` | Re-exports `claude_core::settings_io` (relocated shared implementation) |
| `version.rs` | Version detection, alias resolution, install, and spec validation |
