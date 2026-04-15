# Testing

Test case planning for cm CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

## Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| command/ | Per-command integration test case indices (IT-N entries) |
| param/ | Per-parameter edge case indices (EC-N entries) |
| param_group/ | Per-parameter-group interaction test indices |

## Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands | 12 | ≥6 IT each |
| Parameters | 9 | ≥6 EC each |
| Parameter groups | 2 | ≥4 IT each |

## Navigation

### Commands
- [`.help` / `.` alias / empty argv](command/help.md)
- [`.status`](command/status.md)
- [`.version.show`](command/version_show.md)
- [`.version.install`](command/version_install.md)
- [`.version.list`](command/version_list.md)
- [`.version.guard`](command/version_guard.md)
- [`.version.history`](command/version_history.md)
- [`.processes`](command/processes.md)
- [`.processes.kill`](command/processes_kill.md)
- [`.settings.show`](command/settings_show.md)
- [`.settings.get`](command/settings_get.md)
- [`.settings.set`](command/settings_set.md)
### Parameters
- [`version::`](param/version.md)
- [`dry::`](param/dry.md)
- [`force::`](param/force.md)
- [`verbosity::` / `v::`](param/verbosity.md)
- [`format::`](param/format.md)
- [`key::`](param/key.md)
- [`value::`](param/value.md)
- [`interval::`](param/interval.md)
- [`count::`](param/count.md)

### Parameter Groups
- [Execution Control](param_group/execution_control.md)
- [Output Control](param_group/output_control.md)
