# Command Tests

### Scope

- **Purpose**: Document integration test cases for each clr command.
- **Responsibility**: Index of per-command integration test case files covering command-level behavior.
- **In Scope**: `run` command tests, `help` command tests, `isolated` command tests, `refresh` command tests.
- **Out of Scope**: Per-parameter edge cases (→ `param/`), parameter group interactions (→ `param_group/`).

Per-command integration test case indices for `clr`. See [001_command.md](../../../../docs/cli/001_command.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_run.md | Integration tests for the `run` command (default) |
| 002_help.md | Integration tests for the `help` command (`--help` / `-h`) |
| 003_isolated.md | Integration tests for the `isolated` command |
| 004_refresh.md | Integration tests for the `refresh` command |

### Index

| Command | File | Tests |
|---------|------|-------|
| `run` (default) | [001_run.md](001_run.md) | 16 TC |
| `help` | [002_help.md](002_help.md) | 8 TC |
| `isolated` | [003_isolated.md](003_isolated.md) | 9 TC |
| `refresh` | [004_refresh.md](004_refresh.md) | 8 TC |
