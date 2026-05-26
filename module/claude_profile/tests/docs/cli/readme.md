# Testing

Test case planning for clp CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

### Scope

- **Purpose**: Document integration and edge case test plans for all clp commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 13 clp commands plus binary meta-flags (`--version`/`-V`), all 35 parameters, and all 5 parameter groups.
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
| Commands + meta-flags | 14 | >=8 IT each |
| Parameters | 35 | >=6 EC each |
| Parameter groups | 5 | >=4 IT each |

### Navigation

**Commands:**
- [`--version` / `-V`](command/000_version.md)
- [`.` (help alias)](command/001_dot.md)
- [`.help`](command/002_help.md)
- [`.accounts`](command/003_accounts.md)
- [`.account.save`](command/004_account_save.md)
- [`.account.use`](command/005_account_use.md)
- [`.account.delete`](command/006_account_delete.md)
- [`.token.status`](command/007_token_status.md)
- [`.paths`](command/008_paths.md)
- [`.usage`](command/009_usage.md)
- [`.credentials.status`](command/010_credentials_status.md)
- [`.account.limits`](command/011_account_limits.md)
- [`.account.relogin`](command/012_account_relogin.md)
- [`.account.rotate`](command/013_account_rotate.md)

**Parameters:**
- [`name::`](param/001_name.md)
- [`format::` / `fmt::`](param/003_format.md)
- [`threshold::`](param/004_threshold.md)
- [`dry::`](param/005_dry.md)
- [`account::`](param/006_account.md)
- [`sub::`](param/007_sub.md)
- [`tier::`](param/008_tier.md)
- [`token::`](param/009_token.md)
- [`expires::`](param/010_expires.md)
- [`email::`](param/011_email.md)
- [`file::`](param/012_file.md)
- [`saved::`](param/013_saved.md)
- [`active::`](param/014_active.md)
- [`display_name::`](param/015_display_name.md)
- [`role::`](param/016_role.md)
- [`billing::`](param/017_billing.md)
- [`model::`](param/018_model.md)
- [`refresh::`](param/019_refresh.md)
- [`live::`](param/020_live.md)
- [`interval::`](param/021_interval.md)
- [`jitter::`](param/022_jitter.md)
- [`trace::`](param/023_trace.md)
- [`field::`](param/024_field.md)
- [`sort::`](param/025_sort.md)
- [`desc::`](param/026_desc.md)
- [`prefer::`](param/027_prefer.md)
- [`uuid::`](param/028_uuid.md)
- [`capabilities::`](param/029_capabilities.md)
- [`org_uuid::`](param/030_org_uuid.md)
- [`org_name::`](param/031_org_name.md)
- [`next::`](param/032_next.md)
- [`cols::`](param/033_cols.md)
- [`touch::`](param/034_touch.md)
- [`imodel::`](param/035_imodel.md)
- [`effort::`](param/036_effort.md)

**Parameter Groups:**
- [Output Control](param_group/001_output_control.md)
- [Field Presence](param_group/002_field_presence.md)
- [Fetch Behavior](param_group/003_fetch_behavior.md)
- [Sort Control](param_group/004_sort_control.md)
- [Display Control](param_group/005_display_control.md)
