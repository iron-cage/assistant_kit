# Runbox Parameters

One file per scalar configuration slot in the runbox infrastructure.
Parameters have fixed presence — they always exist; only their value changes.

### Scope

- **Purpose:** Document every scalar configuration slot in the runbox infrastructure.
- **Responsibility:** Per-parameter reference covering status, current state, data flow, and notes.
- **In Scope:** All `runbox.yml` keys (required and optional-with-default) in the runbox infrastructure.
- **Out of Scope:** Plugin slots (→ `plugin/`); test results; implementation code.

### Overview Table

✅ = configurable in `runbox.yml` (required or optional-with-default)

| ID | Parameter | Status | Category |
|----|-----------|--------|----------|
| [001](001_image.md) | `image` | ✅ | Image Setup |
| [002](002_test_user.md) | `test_user` | ✅ | User Context |
| [003](003_cmd_scope.md) | `cmd_scope` | ✅ | Test Execution |
| [004](004_cmd_filter.md) | `cmd_filter` | ✅ | Test Execution |
| [005](005_test_script.md) | `test_script` | ✅ | Test Execution |
| [006](006_base_image.md) | `base_image` | ✅ | Image Setup |
| [007](007_rustup_components.md) | `rustup_components` | ✅ | Toolchain |
| [008](008_system_packages.md) | `system_packages` | ✅ | Toolchain |
| [009](009_cargo_features.md) | `cargo_features` | ✅ | Test Execution |
| [010](010_workspace_dir.md) | `workspace_dir` | ✅ | Infrastructure |
| [011](011_dockerfile.md) | `dockerfile` | ✅ | Infrastructure |
| [012](012_cache_dir.md) | `cache_dir` | ✅ | Infrastructure |
| [013](013_workspace_root.md) | `workspace_root` | ✅ | Infrastructure |
