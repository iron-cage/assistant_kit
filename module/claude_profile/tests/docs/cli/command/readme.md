# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clp command and binary meta-flags.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: All 11 clp command test files plus binary meta-flag tests (`--version`/`-V`).
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 00_version.md | Test cases for `--version` / `-V` meta-flags |
| 01_dot.md | Test cases for `.` command (dot shorthand) |
| 02_help.md | Test cases for `.help` command |
| 03_accounts.md | Test cases for `.accounts` command |
| 04_account_save.md | Test cases for `.account.save` command |
| 05_account_use.md | Test cases for `.account.use` command |
| 06_account_delete.md | Test cases for `.account.delete` command |
| 07_token_status.md | Test cases for `.token.status` command |
| 08_paths.md | Test cases for `.paths` command |
| 09_usage.md | Test cases for `.usage` command |
| 10_credentials_status.md | Test cases for `.credentials.status` command |
| 11_account_limits.md | Test cases for `.account.limits` command |
