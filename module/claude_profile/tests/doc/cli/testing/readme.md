# Testing

Test case planning for clp CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

### Scope

- **Purpose**: Document integration and edge case test plans for all clp commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 12 clp commands plus binary meta-flags (`--version`/`-V`), all 5 parameters, and all 1 parameter groups.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| command/ | Per-command integration test case indices (IT-N entries) |
| param/ | Per-parameter edge case indices (EC-N entries) |
| param_group/ | Per-parameter-group interaction test indices |

### Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands + meta-flags | 13 | >=5 IT each |
| Parameters | 5 | >=6 EC each |
| Parameter groups | 1 | >=4 IT each |

### Navigation

**Commands:**
- [`--version` / `-V`](command/00_version.md)
- [`.` (help alias)](command/01_dot.md)
- [`.help`](command/02_help.md)
- [`.account.list`](command/03_account_list.md)
- [`.account.status`](command/04_account_status.md)
- [`.account.save`](command/05_account_save.md)
- [`.account.switch`](command/06_account_switch.md)
- [`.account.delete`](command/07_account_delete.md)
- [`.token.status`](command/08_token_status.md)
- [`.paths`](command/09_paths.md)
- [`.usage`](command/10_usage.md)
- [`.credentials.status`](command/11_credentials_status.md)
- [`.account.limits`](command/12_account_limits.md)

**Parameters:**
- [`name::`](param/01_name.md)
- [`verbosity::` / `v::`](param/02_verbosity.md)
- [`format::`](param/03_format.md)
- [`threshold::`](param/04_threshold.md)
- [`dry::`](param/05_dry.md)

**Parameter Groups:**
- [Output Control](param_group/01_output_control.md)
