# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clr parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: All 18 clr parameter edge case files.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

Per-parameter edge case indices for `clr`. See [param/readme.md](../../../../docs/cli/param/readme.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_message.md | Edge cases for `[MESSAGE]` positional parameter |
| 02_print.md | Edge cases for `--print` / `-p` flag |
| 03_model.md | Edge cases for `--model` flag |
| 04_verbose.md | Edge cases for `--verbose` flag |
| 05_no_skip_permissions.md | Edge cases for `--no-skip-permissions` flag |
| 06_interactive.md | Edge cases for `--interactive` flag |
| 07_new_session.md | Edge cases for `--new-session` flag |
| 08_dir.md | Edge cases for `--dir` flag |
| 09_max_tokens.md | Edge cases for `--max-tokens` flag |
| 10_session_dir.md | Edge cases for `--session-dir` flag |
| 11_dry_run.md | Edge cases for `--dry-run` flag |
| 12_verbosity.md | Edge cases for `--verbosity` flag |
| 13_trace.md | Edge cases for `--trace` flag |
| 14_no_ultrathink.md | Edge cases for `--no-ultrathink` flag |
| 15_system_prompt.md | Edge cases for `--system-prompt` flag |
| 16_append_system_prompt.md | Edge cases for `--append-system-prompt` flag |
| 17_effort.md | Edge cases for `--effort` flag |
| 18_no_effort_max.md | Edge cases for `--no-effort-max` flag |

### Index

| Parameter | File | Tests |
|-----------|------|-------|
| `[MESSAGE]` | [01_message.md](01_message.md) | 6 EC |
| `--print` | [02_print.md](02_print.md) | 6 EC |
| `--model` | [03_model.md](03_model.md) | 6 EC |
| `--verbose` | [04_verbose.md](04_verbose.md) | 6 EC |
| `--no-skip-permissions` | [05_no_skip_permissions.md](05_no_skip_permissions.md) | 6 EC |
| `--interactive` | [06_interactive.md](06_interactive.md) | 6 EC |
| `--new-session` | [07_new_session.md](07_new_session.md) | 6 EC |
| `--dir` | [08_dir.md](08_dir.md) | 6 EC |
| `--max-tokens` | [09_max_tokens.md](09_max_tokens.md) | 6 EC |
| `--session-dir` | [10_session_dir.md](10_session_dir.md) | 6 EC |
| `--dry-run` | [11_dry_run.md](11_dry_run.md) | 6 EC |
| `--verbosity` | [12_verbosity.md](12_verbosity.md) | 6 EC |
| `--trace` | [13_trace.md](13_trace.md) | 6 EC |
| `--no-ultrathink` | [14_no_ultrathink.md](14_no_ultrathink.md) | 6 EC |
| `--system-prompt` | [15_system_prompt.md](15_system_prompt.md) | 6 EC |
| `--append-system-prompt` | [16_append_system_prompt.md](16_append_system_prompt.md) | 6 EC |
| `--effort` | [17_effort.md](17_effort.md) | 8 EC |
| `--no-effort-max` | [18_no_effort_max.md](18_no_effort_max.md) | 6 EC |
