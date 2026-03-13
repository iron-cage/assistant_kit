# Testing

Test case planning for `clr` CLI. Each file contains a Test Case Index with coverage summary. Detailed test sections (executable specs) are added at L5.

## Responsibility Table

| Directory | Responsibility |
|-----------|----------------|
| command/ | Per-command integration test case indices (TC-N entries) |
| param/ | Per-parameter edge case indices (TC-N entries) |
| param_group/ | Per-parameter-group interaction test indices |

## Coverage Summary

| Scope | Files | Min Tests |
|-------|-------|-----------|
| Commands | 2 | ≥4 TC each |
| Parameters | 2 | ≥4 TC each |
| Parameter groups | 1 | ≥4 TC each |

## Navigation

### Commands
- [`run`](command/run.md)
- [`help`](command/help.md)

### Parameters
- [`--system-prompt`](param/system_prompt.md)
- [`--append-system-prompt`](param/append_system_prompt.md)

### Parameter Groups
- [System Prompt](param_group/system_prompt.md)
