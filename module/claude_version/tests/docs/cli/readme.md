# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all cm commands, parameters, types, and parameter groups.
- **Responsibility**: Index of per-command, per-parameter, per-type, and per-group test case planning files.
- **In Scope**: All 13 cm commands, all 12 parameters, all 7 types, all 4 parameter groups, and all 5 user stories.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

4-tier testing organization for `claude_version` CLI, providing distinct audience focus at each level.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Integration test cases per command |
| `param/` | Edge case tests per parameter |
| `type/` | Type validation test cases per semantic type |
| `param_group/` | Interaction tests per parameter group |
| `user_story/` | User story acceptance tests per scenario |
| `procedure.md` | Workflow for creating and updating CLI test tiers |

### Overview

| Tier | Location | Purpose | Audience | Test Types |
|------|----------|---------|----------|-----------|
| Parameter | `param/*.md` | Validate individual parameter parsing and constraints | Developers | Unit tests, edge cases |
| Type | `type/*.md` | Validate semantic type parsing, ranges, and inference | Developers | Type validation, boundary |
| Group | `param_group/*.md` | Test parameter interactions within groups | Developers | Corner cases, dependencies |
| Command | `command/*.md` | End-to-end command integration | QA / Users | Integration tests, workflows |
| User Story | `user_story/*.md` | End-to-end workflow acceptance | QA / Users | Acceptance tests, scenarios |

### Navigation

- [Command Tests](command/) — Integration tests per command
- [Parameter Tests](param/) — Edge case tests per parameter
- [Type Tests](type/) — Type validation tests per semantic type
- [Parameter Group Tests](param_group/) — Interaction tests per group
- [User Story Tests](user_story/) — Workflow acceptance tests per scenario

### Commands
- [`.help`](command/001_help.md)
- [`.status`](command/002_status.md)
- [`.version.show`](command/003_version_show.md)
- [`.version.install`](command/004_version_install.md)
- [`.version.guard`](command/005_version_guard.md)
- [`.version.list`](command/006_version_list.md)
- [`.processes`](command/007_processes.md)
- [`.processes.kill`](command/008_processes_kill.md)
- [`.settings.show`](command/009_settings_show.md)
- [`.settings.get`](command/010_settings_get.md)
- [`.settings.set`](command/011_settings_set.md)
- [`.version.history`](command/012_version_history.md)
- [`.config`](command/013_config.md)

### Parameters
- [`version::`](param/001_version.md)
- [`dry::`](param/002_dry.md)
- [`force::`](param/003_force.md)
- [`v::` / `verbosity::`](param/004_verbosity.md)
- [`format::`](param/005_format.md)
- [`key::`](param/006_key.md)
- [`value::`](param/007_value.md)
- [`interval::`](param/008_interval.md)
- [`count::`](param/009_count.md)
- [`.help`](param/010_help_param.md)
- [`scope::`](param/011_scope.md)
- [`unset::`](param/012_unset.md)

### Types
- [`VerbosityLevel`](type/001_verbosity_level.md)
- [`OutputFormat`](type/002_output_format.md)
- [`VersionSpec`](type/003_version_spec.md)
- [`SettingsKey`](type/004_settings_key.md)
- [`SettingsValue`](type/005_settings_value.md)
- [`ConfigScope`](type/006_config_scope.md)
- [`ConfigKey`](type/007_config_key.md)

### Parameter Groups
- [Output Control](param_group/001_output_control.md)
- [Execution Control](param_group/002_execution_control.md)
- [Settings Identity](param_group/003_settings_identity.md)
- [Config Identity](param_group/004_config_identity.md)

### User Stories
- [Environment Check](user_story/001_environment_check.md)
- [Version Upgrade](user_story/002_version_upgrade.md)
- [Process Lifecycle](user_story/003_process_lifecycle.md)
- [Settings Management](user_story/004_settings_management.md)
- [Version Pinning](user_story/005_version_pinning.md)
