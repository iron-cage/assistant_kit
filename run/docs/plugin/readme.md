# Runbox Plugins

One file per swappable behavioral slot in the runbox infrastructure.
Plugins have lifecycle management — their absence changes the execution shape, not just a value.

### Scope

- **Purpose:** Document every swappable behavioral slot in the runbox infrastructure.
- **Responsibility:** Per-plugin reference covering status, what it controls, mechanism, and notes.
- **In Scope:** Configurable plugin slots and hardcoded-but-swappable infrastructure components.
- **Out of Scope:** Scalar parameters (→ `parameter/`); plugin configuration values.

### Status Legend

✅ = fully extracted to `run/plugins.sh`; configured via `runbox.yml`; remove `plugins.sh` to disable
🔒 = hardcoded in `runbox-run` or `run/runbox.dockerfile`; requires code changes to swap

### Overview Table

| ID | Plugin | Status | Category | Controls | Parameterizable | Affects |
|----|--------|--------|----------|----------|-----------------|---------|
| [001](001_bin_plugin.md) | `bin_plugin` | ✅ | Binary Injection | Host binary + working volume injected into container | `bin_plugin`, `bin_plugin_volume` | `.test` `.shell` `.build` |
| [002](002_plugin_mount.md) | `plugin_mount` | ✅ | Data Mount | Host dir mounted `rw` for `.test`, `ro` for `.shell` | `plugin_mount` | `.test` `.shell` |
| [003](003_dep_cache.md) | Dep cache | 🔒 | Build Infrastructure | External dep pre-compilation (cargo-chef 4-stage) | — | `.build` |
| [004](004_build_cache_persistence.md) | Build cache persistence | 🔒 | Build Infrastructure | Volume seeded on `.build`; mounted on `.test.offline` | — | `.build` `.test.offline` |
| [005](005_test_lister.md) | Test lister | 🔒 | Test Runner | `cargo nextest list` hardcoded; flags follow `cargo_features` | `cargo_features` | `.list` |
| [006](006_offline_runner.md) | Offline runner | 🔒 | Test Runner | Default `CMD` baked at build; scope/features/filter all tunable | `CMD_SCOPE` `CARGO_FEATURES` `CMD_FILTER` | `.test.offline` |
