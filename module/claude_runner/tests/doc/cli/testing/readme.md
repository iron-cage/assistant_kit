# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all clr commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 2 clr commands, all 5 parameters, and all 1 parameter groups.
- **Out of Scope**: Automated test implementations (→ `tests/` in crate), spec documentation (→ `docs/feature/`).

Test case planning for `clr` CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

### Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| command/ | Per-command integration test case indices |
| param/ | Per-parameter edge case indices |
| param_group/ | Per-parameter-group interaction test indices |

### Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands | 2 | ≥4 TC each |
| Parameters | 5 | ≥4 TC each |
| Parameter groups | 1 | ≥4 TC each |

### Navigation

#### Commands
- [`run`](command/01_run.md)
- [`help`](command/02_help.md)

#### Parameters
- [`--no-ultrathink`](param/14_no_ultrathink.md)
- [`--system-prompt`](param/15_system_prompt.md)
- [`--append-system-prompt`](param/16_append_system_prompt.md)
- [`--effort`](param/17_effort.md)
- [`--no-effort-max`](param/18_no_effort_max.md)

#### Parameter Groups
- [System Prompt](param_group/03_system_prompt.md)
