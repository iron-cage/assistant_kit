# Testing

Test case planning for clp CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

### Scope

- **Purpose**: Document integration and edge case test plans for all clp commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 11 clp commands plus binary meta-flags (`--version`/`-V`), all 18 parameters, and all 2 parameter groups.
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
| Commands + meta-flags | 12 | >=8 IT each |
| Parameters | 18 | >=6 EC each |
| Parameter groups | 2 | >=4 IT each |

### Navigation

**Commands:**
- [`--version` / `-V`](command/00_version.md)
- [`.` (help alias)](command/01_dot.md)
- [`.help`](command/02_help.md)
- [`.accounts`](command/03_accounts.md)
- [`.account.save`](command/04_account_save.md)
- [`.account.switch`](command/05_account_switch.md)
- [`.account.delete`](command/06_account_delete.md)
- [`.token.status`](command/07_token_status.md)
- [`.paths`](command/08_paths.md)
- [`.usage`](command/09_usage.md)
- [`.credentials.status`](command/10_credentials_status.md)
- [`.account.limits`](command/11_account_limits.md)

**Parameters:**
- [`name::`](param/01_name.md)
- [`verbosity::` / `v::`](param/02_verbosity.md)
- [`format::` / `fmt::`](param/03_format.md)
- [`threshold::`](param/04_threshold.md)
- [`dry::`](param/05_dry.md)
- [`account::`](param/06_account.md)
- [`sub::`](param/07_sub.md)
- [`tier::`](param/08_tier.md)
- [`token::`](param/09_token.md)
- [`expires::`](param/10_expires.md)
- [`email::`](param/11_email.md)
- [`file::`](param/12_file.md)
- [`saved::`](param/13_saved.md)
- [`active::`](param/14_active.md)
- [`display_name::`](param/15_display_name.md)
- [`role::`](param/16_role.md)
- [`billing::`](param/17_billing.md)
- [`model::`](param/18_model.md)

**Parameter Groups:**
- [Output Control](param_group/01_output_control.md)
- [Field Presence](param_group/02_field_presence.md)
