# src/

Source code for the `claude_version_core` crate.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; `CoreError` enum and module declarations |
| `config_catalog.rs` | Settings catalog: known config keys, env var mappings, catalog defaults |
| `config_resolve.rs` | 4-layer config resolution: env var → project config → user config → catalog default |
| `params_catalog.rs` | Claude Code parameter catalog: CLI/env/config forms plus catalog defaults |
| `paths.rs` | `ClaudeVersionPaths` struct — composed path resolution for clv-known filesystem locations |
| `settings_io.rs` | Re-exports `claude_core::settings_io` (relocated shared implementation) |
| `version.rs` | Version detection, alias resolution, install, and spec validation |
