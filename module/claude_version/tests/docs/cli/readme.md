# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all cm commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 12 cm commands, all 10 parameters, and all 3 parameter groups.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

3-tier testing organization for `claude_version` CLI, providing distinct audience focus at each level.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Integration test cases per command |
| `param/` | Edge case tests per parameter |
| `param_group/` | Interaction tests per parameter group |
| `procedure.md` | Workflow for creating and updating CLI test tiers |

### Overview

| Tier | Location | Purpose | Audience | Test Types |
|------|----------|---------|----------|-----------|
| Parameter | `param/*.md` | Validate individual parameter parsing and constraints | Developers | Unit tests, edge cases |
| Group | `param_group/*.md` | Test parameter interactions within groups | Developers | Corner cases, dependencies |
| Command | `command/*.md` | End-to-end command integration | QA / Users | Integration tests, workflows |

### Navigation

- [Command Tests](command/) — Integration tests per command
- [Parameter Tests](param/) — Edge case tests per parameter
- [Parameter Group Tests](param_group/) — Interaction tests per group

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

### Parameter Groups
- [Output Control](param_group/001_output_control.md)
- [Execution Control](param_group/002_execution_control.md)
- [Settings Identity](param_group/003_settings_identity.md)
