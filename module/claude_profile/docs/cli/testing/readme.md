# Testing

Test case planning for clp CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

## Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| command/ | Per-command integration test case indices (IT-N entries) |
| param/ | Per-parameter edge case indices (EC-N entries) |
| param_group/ | Per-parameter-group interaction test indices |

## Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands | 9 | >=8 IT each |
| Parameters | 5 | >=6 EC each |
| Parameter groups | 1 | >=4 IT each |

## Navigation

### Commands
- [`.` (help alias)](command/dot.md)
- [`.help`](command/help.md)
- [`.account.list`](command/account_list.md)
- [`.account.status`](command/account_status.md)
- [`.account.save`](command/account_save.md)
- [`.account.switch`](command/account_switch.md)
- [`.account.delete`](command/account_delete.md)
- [`.token.status`](command/token_status.md)
- [`.paths`](command/paths.md)

### Parameters
- [`name::`](param/name.md)
- [`verbosity::` / `v::`](param/verbosity.md)
- [`format::`](param/format.md)
- [`threshold::`](param/threshold.md)
- [`dry::`](param/dry.md)

### Parameter Groups
- [Output Control](param_group/output_control.md)
