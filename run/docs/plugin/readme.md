# Runbox Plugins

One file per swappable behavioral slot in the runbox infrastructure.
Plugins have lifecycle management — their absence changes the execution shape, not just a value.

### Scope

- **Purpose:** Document every swappable behavioral slot in the runbox infrastructure.
- **Responsibility:** Per-plugin reference covering status, what it controls, mechanism, and notes.
- **In Scope:** All swappable behavioral slots — hook-based (✅ `plugins.sh`) and param-based (🔧 `runbox.yml`).
- **Out of Scope:** Scalar parameters (→ `parameter/`); plugin configuration values.

### Status Legend

✅ = logic lives in `run/plugins.sh`; swap by overriding the hook; `plugins.sh` absent = slot inactive
🔧 = param read by `runbox-run` core via `cfg_or`; swap by setting `runbox.yml` key; always has a default
🔒 = requires code changes to swap (reserved for future hardcoded slots)

### Overview Table

| ID | Plugin | Status | Category | Controls | Default | Change via | Rebuild | Affects |
|----|--------|--------|----------|----------|---------|------------|---------|---------|
| [001](001_bin_plugin.md) | `bin_plugin` | ✅ | Binary Injection | Host binary + working volume injected into container | inactive | `bin_plugin`, `bin_plugin_volume` in `runbox.yml` | no | `.test` `.shell` `.build` |
| [002](002_plugin_mount.md) | `plugin_mount` | ✅ | Data Mount | Host dir mounted `rw` for `.test`, `ro` for `.shell` | inactive | `plugin_mount` in `runbox.yml` | no | `.test` `.shell` |
| [003](003_dep_cache.md) | Dep cache | 🔧 | Build Infrastructure | External dep pre-compilation strategy | cargo-chef 4-stage | `dockerfile:` in `runbox.yml` | yes | `.build` |
| [004](004_build_cache_persistence.md) | Build cache persistence | 🔧 | Build Infrastructure | Named volume seeded on `.build`; mounted for `.test.offline` and `.shell` | `target` dir | `cache_dir:` in `runbox.yml` | no | `.build` `.test.offline` `.shell` |
| [005](005_test_lister.md) | Test lister | ✅ | Test Runner | List command for `.list` sub-command | `cargo nextest list` | `_plugin_list_cmd()` in `plugins.sh` | no | `.list` |
| [006](006_offline_runner.md) | Offline runner | 🔧 | Test Runner | Baked image `CMD` executed by `.test.offline` | `cargo nextest run` | `dockerfile:` in `runbox.yml` | yes | `.test.offline` |
