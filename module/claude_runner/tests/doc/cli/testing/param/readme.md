# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: `--no-ultrathink`, `--system-prompt`, `--append-system-prompt`, `--effort`, `--no-effort-max` edge cases.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

Per-parameter edge case indices for `clr`. See [params.md](../../../../../docs/cli/params.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 14_no_ultrathink.md | Edge cases for `--no-ultrathink` flag |
| 15_system_prompt.md | Edge cases for `--system-prompt` flag |
| 16_append_system_prompt.md | Edge cases for `--append-system-prompt` flag |
| 17_effort.md | Edge cases for `--effort` flag |
| 18_no_effort_max.md | Edge cases for `--no-effort-max` flag |

### Index

| Parameter | File | Tests |
|-----------|------|-------|
| `--no-ultrathink` | [14_no_ultrathink.md](14_no_ultrathink.md) | 5 TC |
| `--system-prompt` | [15_system_prompt.md](15_system_prompt.md) | 5 TC |
| `--append-system-prompt` | [16_append_system_prompt.md](16_append_system_prompt.md) | 5 TC |
| `--effort` | [17_effort.md](17_effort.md) | 8 TC |
| `--no-effort-max` | [18_no_effort_max.md](18_no_effort_max.md) | 5 TC |
