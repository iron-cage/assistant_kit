# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clr command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: `run` command tests, `help` command tests, `isolated` command tests, `refresh` command tests, `ask` command tests, `ps` command tests, `kill` command tests, `tools` command tests.
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

Per-command integration test case indices for `clr`. See [command/](../../../../docs/cli/command/) for specification.

### Responsibility Table

| Name | Purpose | Status |
|------|---------|--------|
| `01_run.md` | Integration tests for the `run` command (default) | ✅ |
| `02_help.md` | Integration tests for the `help` command (`--help` / `-h`) | ✅ |
| `03_isolated.md` | Integration tests for the `isolated` command | ✅ |
| `04_refresh.md` | Integration tests for the `refresh` command | ✅ |
| `05_ask.md` | Integration tests for the `ask` command | ✅ |
| `06_ps.md` | Integration tests for the `ps` command | ✅ |
| `07_kill.md` | Integration tests for the `kill` command | ✅ |
| `08_tools.md` | Integration tests for the `tools` command | ✅ |
