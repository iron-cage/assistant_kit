# User Stories

### Scope

- **Purpose**: Document persona-goal scenarios for the clv CLI.
- **Responsibility**: User story catalog with acceptance criteria for each scenario.
- **In Scope**: Developer and team lead personas; version, process, settings, config, and environment workflows.
- **Out of Scope**: Command reference (-> `../command/`), behavioral contracts (-> `../../feature/`).

### Responsibility Table

| File | Responsibility |
|------|---------------|
| readme.md | Index, scope, and Overview Table for user story instances |
| procedure.md | Steps for adding, updating, or removing user story instances |
| 001_environment_check.md | Verify environment state at a glance |
| 002_version_upgrade.md | Upgrade installed Claude Code version |
| 003_process_lifecycle.md | Inspect and terminate running processes |
| 004_settings_management.md | Read and write settings via deprecated commands |
| 005_version_pinning.md | Pin team-wide Claude Code version |
| 006_config_management.md | Inspect and modify config via `.config` |
| 007_params_inspection.md | Discover Claude Code params and inspect current observable values |
| 008_path_discovery.md | Discover on-disk paths clv manages |

### Overview Table

| File | ID | Title | Persona | Primary Commands |
|------|----|-------|---------|-----------------|
| [001_environment_check.md](001_environment_check.md) | 001 | Environment Check | developer | `.status` |
| [002_version_upgrade.md](002_version_upgrade.md) | 002 | Version Upgrade | developer | `.version.show`, `.version.install`, `.version.guard`, `.version.history` |
| [003_process_lifecycle.md](003_process_lifecycle.md) | 003 | Process Lifecycle | developer | `.processes`, `.processes.kill` |
| [004_settings_management.md](004_settings_management.md) | 004 | Settings Management | developer | `.settings.show`, `.settings.get`, `.settings.set` |
| [005_version_pinning.md](005_version_pinning.md) | 005 | Version Pinning | team lead | `.version.list`, `.version.install`, `.version.show`, `.version.guard` |
| [006_config_management.md](006_config_management.md) | 006 | Config Management | developer | `.config` |
| [007_params_inspection.md](007_params_inspection.md) | 007 | Params Inspection | developer | `.params` |
| [008_path_discovery.md](008_path_discovery.md) | 008 | Path Discovery | developer | `.paths` |
