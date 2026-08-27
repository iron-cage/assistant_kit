# Algorithm Doc Entity

### Scope

- **Purpose**: Document algorithms implemented in the `claude_version_core` crate.
- **Responsibility**: Index of algorithm doc instances covering settings config resolution.
- **In Scope**: 4-layer config resolution algorithm (env → project → user → catalog default), implemented by `config_resolve.rs` over the catalog in `config_catalog.rs`.
- **Out of Scope**: CLI command surface and output formatting that consume this algorithm (→ `claude_version/docs/feature/006_config_command.md`, `claude_version/docs/algorithm/002_config_resolution.md`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index and navigation for algorithm instances |
| 002_config_resolution.md | 4-layer resolution: env → project → user → catalog default |

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 002 | [Config Resolution](002_config_resolution.md) | 4-layer resolution: env → project → user → catalog default | ✅ |
