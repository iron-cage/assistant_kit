# Testing

### Scope

- **Purpose**: Document integration and edge case test plans for all clr commands and parameters.
- **Responsibility**: Index of per-command, per-parameter, and per-group test case planning files.
- **In Scope**: All 2 clr commands, all 18 parameters, and all 3 parameter groups.
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
| Commands | 2 | ≥8 IT each |
| Parameters | 18 | ≥6 EC each |
| Parameter groups | 3 | ≥4 IT each |

### Navigation

#### Commands
- [`run`](command/01_run.md)
- [`help`](command/02_help.md)

#### Parameters
- [`[MESSAGE]`](param/01_message.md)
- [`--print`](param/02_print.md)
- [`--model`](param/03_model.md)
- [`--verbose`](param/04_verbose.md)
- [`--no-skip-permissions`](param/05_no_skip_permissions.md)
- [`--interactive`](param/06_interactive.md)
- [`--new-session`](param/07_new_session.md)
- [`--dir`](param/08_dir.md)
- [`--max-tokens`](param/09_max_tokens.md)
- [`--session-dir`](param/10_session_dir.md)
- [`--dry-run`](param/11_dry_run.md)
- [`--verbosity`](param/12_verbosity.md)
- [`--trace`](param/13_trace.md)
- [`--no-ultrathink`](param/14_no_ultrathink.md)
- [`--system-prompt`](param/15_system_prompt.md)
- [`--append-system-prompt`](param/16_append_system_prompt.md)
- [`--effort`](param/17_effort.md)
- [`--no-effort-max`](param/18_no_effort_max.md)

#### Parameter Groups
- [Claude-Native Flags](param_group/01_claude_native_flags.md)
- [Runner Control](param_group/02_runner_control.md)
- [System Prompt](param_group/03_system_prompt.md)
