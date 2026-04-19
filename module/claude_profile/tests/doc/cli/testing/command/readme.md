# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clp command and binary meta-flags.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: All 12 clp command test files plus binary meta-flag tests (`--version`/`-V`).
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 00_version.md | Test cases for `--version` / `-V` meta-flags |
| 01_dot.md | Test cases for `.` command (dot shorthand) |
| 02_help.md | Test cases for `.help` command |
| 03_account_list.md | Test cases for `.account.list` command |
| 04_account_status.md | Test cases for `.account.status` command |
| 05_account_save.md | Test cases for `.account.save` command |
| 06_account_switch.md | Test cases for `.account.switch` command |
| 07_account_delete.md | Test cases for `.account.delete` command |
| 08_token_status.md | Test cases for `.token.status` command |
| 09_paths.md | Test cases for `.paths` command |
| 10_usage.md | Test cases for `.usage` command |
| 11_credentials_status.md | Test cases for `.credentials.status` command |
| 12_account_limits.md | Test cases for `.account.limits` command |
