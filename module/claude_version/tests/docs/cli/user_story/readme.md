# User Story Tests

### Scope

- **Purpose**: Acceptance test cases for all 6 cm user story scenarios.
- **Responsibility**: Index of per-user-story test files covering end-to-end workflow acceptance.
- **In Scope**: All 6 cm user stories: Environment Check, Version Upgrade, Process Lifecycle, Settings Management, Version Pinning, Config Management.
- **Out of Scope**: Command-level tests (-> `../command/`), parameter edge cases (-> `../param/`).

### Overview Table

| Name | Purpose | Status |
|------|---------|--------|
| 01_environment_check.md | US- acceptance tests for environment verification via `.status` | ✅ |
| 02_version_upgrade.md | US- acceptance tests for version upgrade workflow | ✅ |
| 03_process_lifecycle.md | US- acceptance tests for process inspection and termination | ✅ |
| 04_settings_management.md | US- acceptance tests for settings read/write workflow | ✅ |
| 05_version_pinning.md | US- acceptance tests for team-wide version pinning | ✅ |
| 06_config_management.md | AT- acceptance tests for config inspection/modification via `.config` | ✅ |
| procedure.md | Workflow for creating and updating user story test specs | ✅ |

### Navigation

- [Environment Check](01_environment_check.md) -- `.status` verification
- [Version Upgrade](02_version_upgrade.md) -- install/guard/history workflow
- [Process Lifecycle](03_process_lifecycle.md) -- `.processes` / `.processes.kill`
- [Settings Management](04_settings_management.md) -- `.settings.*` read/write
- [Version Pinning](05_version_pinning.md) -- team version alignment
- [Config Management](06_config_management.md) -- `.config` read/write workflow

### See Also

- [Source user stories](../../../../docs/cli/user_story/) -- authoritative user story definitions
- [Command Tests](../command/) -- per-command integration tests
