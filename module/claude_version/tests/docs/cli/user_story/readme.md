# User Story Tests

### Scope

- **Purpose**: Acceptance test cases for all 8 clv user story scenarios.
- **Responsibility**: Index of per-user-story test files covering end-to-end workflow acceptance.
- **In Scope**: All 8 clv user stories: Environment Check, Version Upgrade, Process Lifecycle, Settings Management, Version Pinning, Config Management, Params Inspection, Path Discovery.
- **Out of Scope**: Command-level tests (-> `../command/`), parameter edge cases (-> `../param/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 001_environment_check.md | US- acceptance tests for environment verification via `.status` | ✅ |
| 002_version_upgrade.md | US- acceptance tests for version upgrade workflow | ✅ |
| 003_process_lifecycle.md | US- acceptance tests for process inspection and termination | ✅ |
| 004_settings_management.md | US- acceptance tests for settings read/write workflow | ✅ |
| 005_version_pinning.md | US- acceptance tests for team-wide version pinning | ✅ |
| 006_config_management.md | US- acceptance tests for config inspection/modification via `.config` | ✅ |
| 007_params_inspection.md | US- acceptance tests for param inspection via `.params` | ✅ |
| 008_path_discovery.md | US- acceptance tests for path discovery via `.paths` | ✅ |
| procedure.md | Workflow for creating and updating user story test specs | ✅ |

### Navigation

- [Environment Check](001_environment_check.md) -- `.status` verification
- [Version Upgrade](002_version_upgrade.md) -- install/guard/history workflow
- [Process Lifecycle](003_process_lifecycle.md) -- `.processes` / `.processes.kill`
- [Settings Management](004_settings_management.md) -- `.settings.*` read/write
- [Version Pinning](005_version_pinning.md) -- team version alignment
- [Config Management](006_config_management.md) -- `.config` read/write workflow
- [Params Inspection](007_params_inspection.md) -- `.params` parameter catalog
- [Path Discovery](008_path_discovery.md) -- `.paths` filesystem path discovery

### See Also

- [Source user stories](../../../../docs/cli/user_story/) -- authoritative user story definitions
- [Command Tests](../command/) -- per-command integration tests
