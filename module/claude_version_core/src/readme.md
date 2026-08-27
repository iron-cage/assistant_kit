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
| `version.rs` | Version detection, alias resolution, install, and spec validation |

See [`docs/algorithm/002_config_resolution.md`](../docs/algorithm/002_config_resolution.md) for the `config_resolve.rs`/`config_catalog.rs` algorithm, and [`docs/pattern/002_parameter_trace.md`](../docs/pattern/002_parameter_trace.md) for the stderr trace convention applied across `version.rs`.
