# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clr command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: `run` command tests, `help` command tests.
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

Per-command integration test case indices for `clr`. See [command.md](../../../../docs/cli/command.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_run.md | Integration tests for the `run` command (default) |
| 02_help.md | Integration tests for the `help` command (`--help` / `-h`) |

### Index

| Command | File | Tests |
|---------|------|-------|
| `run` (default) | [01_run.md](01_run.md) | 16 TC |
| `help` | [02_help.md](02_help.md) | 8 TC |
