# User Story Doc Entity

### Scope

- **Purpose**: Document persona-goal scenarios for the clv CLI.
- **Responsibility**: User story catalog with acceptance criteria for each scenario.
- **In Scope**: Developer and team lead personas; version, process, settings, and environment workflows.
- **Out of Scope**: Command reference (-> `../command/`), behavioral contracts (-> `../../feature/`).

### Overview Table

| File | ID | Title | Persona | Primary Commands |
|------|----|-------|---------|-----------------|
| [001_environment_check.md](001_environment_check.md) | 001 | Environment Check | developer | `.status` |
| [002_version_upgrade.md](002_version_upgrade.md) | 002 | Version Upgrade | developer | `.version.show`, `.version.install`, `.version.guard`, `.version.history` |
| [003_process_lifecycle.md](003_process_lifecycle.md) | 003 | Process Lifecycle | developer | `.processes`, `.processes.kill` |
| [004_settings_management.md](004_settings_management.md) | 004 | Settings Management | developer | `.settings.show`, `.settings.get`, `.settings.set` |
| [005_version_pinning.md](005_version_pinning.md) | 005 | Version Pinning | team lead | `.version.list`, `.version.install`, `.version.show` |
