# Feature Doc Entity

### Scope

- **Purpose**: Document test case planning for feature doc instances in `docs/feature/`.
- **Responsibility**: Index of per-feature-doc test case spec files.
- **In Scope**: Feature doc instances 001–005. Instances 006 and 007 have no FT spec — see the Responsibility Table for each one's reason.
- **Out of Scope**: CLI parameter tests (→ `../cli/`), invariant tests (→ `../invariant/`).

Per-feature-doc test case indices for `claude_runner`. See [feature/readme.md](../../../docs/feature/readme.md) for the feature doc instances.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| [001_runner_tool.md](001_runner_tool.md) | Test cases for the runner tool feature doc instance | ✅ |
| [002_journaling_integration.md](002_journaling_integration.md) | Test cases for the journaling integration feature doc instance | ✅ |
| [003_retry_hierarchy.md](003_retry_hierarchy.md) | Test cases for the retry hierarchy feature doc instance | ✅ |
| [004_json_config.md](004_json_config.md) | Test cases for the JSON config loading feature doc instance | ✅ |
| [005_session_path_resolution.md](005_session_path_resolution.md) | Test cases for the session path resolution feature doc instance | ✅ |
| — (no FT spec) | [`docs/feature/006_cli_design.md`](../../../docs/feature/006_cli_design.md) documents `--flag value` syntax rationale and parser design decisions — rationale for choices already exercised by the `../cli/` parameter specs, with no separate behavioral surface of its own | ➖ |
| — (no FT spec) | [`docs/feature/007_yaml_global_config.md`](../../../docs/feature/007_yaml_global_config.md) is 🔄 Planned — a design proposal only, with no implementation to test (`serde_yaml` wiring, `YamlConfig`, `apply_yaml_config()` all explicitly out of its own scope) | 🔄 |
