# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clp command and binary meta-flags.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: All 13 clp command test files plus binary meta-flag tests (`--version`/`-V`).
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 000_version.md | Test cases for `--version` / `-V` meta-flags |
| 001_dot.md | Test cases for `.` command (dot shorthand) |
| 002_help.md | Test cases for `.help` command |
| 003_accounts.md | Test cases for `.accounts` command |
| 004_account_save.md | Test cases for `.account.save` command |
| 005_account_use.md | Test cases for `.account.use` command |
| 006_account_delete.md | Test cases for `.account.delete` command |
| 007_token_status.md | Test cases for `.token.status` command |
| 008_paths.md | Test cases for `.paths` command |
| 009_usage.md | Test cases for `.usage` command |
| 010_credentials_status.md | Test cases for `.credentials.status` command |
| 011_account_limits.md | Test cases for `.account.limits` command |
| 012_account_relogin.md | Test cases for `.account.relogin` command |
| 013_account_rotate.md | Test cases for `.account.rotate` command |
