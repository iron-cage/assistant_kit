# Runbox Plugins

One file per swappable behavioral slot in the runbox infrastructure.
Plugins have lifecycle management — their absence changes the execution shape, not just a value.

### Scope

- **Purpose:** Document every swappable behavioral slot in the runbox infrastructure.
- **Responsibility:** Per-plugin reference covering status, what it controls, mechanism, and notes.
- **In Scope:** Configurable plugin slots and hardcoded-but-swappable infrastructure components.
- **Out of Scope:** Scalar parameters (→ `parameter/`); plugin configuration values.

### Overview Table

| ID | Plugin | Status | Category |
|----|--------|--------|----------|
| [001](001_bin_plugin.md) | `bin_plugin` | ⚠️ | Binary Injection |
| [002](002_plugin_mount.md) | `plugin_mount` | ⚠️ | Data Mount |
| [003](003_dep_cache.md) | Dep cache | 🔒 | Build Infrastructure |
| [004](004_build_cache_persistence.md) | Build cache persistence | 🔒 | Build Infrastructure |
| [005](005_test_lister.md) | Test lister | 🔒 | Test Runner |
| [006](006_offline_runner.md) | Offline runner | 🔒 | Test Runner |
