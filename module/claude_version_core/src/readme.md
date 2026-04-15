# src/

Source code for the `claude_version_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; `CoreError` enum and module declarations |
| `settings_io.rs` | Atomic read/write of `settings.json`; type inference for plain values |
| `version.rs` | Version detection, alias resolution, install, and spec validation |
