# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all cm commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 12 cm commands, all 9 parameters, and all 2 parameter groups.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

3-tier testing organization for `claude_version` CLI, providing distinct audience focus at each level.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `command/` | Integration test cases per command |
| `param/` | Edge case tests per parameter |
| `param_group/` | Interaction tests per parameter group |

### Overview

| Tier | Location | Purpose | Audience | Test Types |
|------|----------|---------|----------|-----------|
| Parameter | `testing/param/*.md` | Validate individual parameter parsing and constraints | Developers | Unit tests, edge cases |
| Group | `testing/param_group/*.md` | Test parameter interactions within groups | Developers | Corner cases, dependencies |
| Command | `testing/command/*.md` | End-to-end command integration | QA / Users | Integration tests, workflows |

### Navigation

- [Command Tests](command/) — Integration tests per command
- [Parameter Tests](param/) — Edge case tests per parameter
- [Parameter Group Tests](param_group/) — Interaction tests per group

### Commands
- [`.help`](command/01_help.md)
- [`.status`](command/02_status.md)
- [`.version.show`](command/03_version_show.md)
- [`.version.install`](command/04_version_install.md)
- [`.version.guard`](command/05_version_guard.md)
- [`.version.list`](command/06_version_list.md)
- [`.processes`](command/07_processes.md)
- [`.processes.kill`](command/08_processes_kill.md)
- [`.settings.show`](command/09_settings_show.md)
- [`.settings.get`](command/10_settings_get.md)
- [`.settings.set`](command/11_settings_set.md)
- [`.version.history`](command/12_version_history.md)

### Parameters
- [`version::`](param/01_version.md)
- [`dry::`](param/02_dry.md)
- [`force::`](param/03_force.md)
- [`v::` / `verbosity::`](param/04_verbosity.md)
- [`format::`](param/05_format.md)
- [`key::`](param/06_key.md)
- [`value::`](param/07_value.md)
- [`interval::`](param/08_interval.md)
- [`count::`](param/09_count.md)

### Parameter Groups
- [Output Control](param_group/01_output_control.md)
- [Execution Control](param_group/02_execution_control.md)
