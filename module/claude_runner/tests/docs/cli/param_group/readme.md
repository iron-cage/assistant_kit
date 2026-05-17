# Parameter Group Tests

### Scope

- **Purpose**: Document interaction tests for clr parameter groups.
- **Responsibility**: Index of per-parameter-group interaction test files covering group-level behavior.
- **In Scope**: All 4 clr parameter group test files.
- **Out of Scope**: Per-command tests (→ `command/`), per-parameter edge cases (→ `param/`).

Per-group interaction test indices for `clr`. See [param_group.md](../../../../docs/cli/param_group.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_claude_native_flags.md | Interaction tests for Group 1 (Claude-Native Flags) |
| 02_runner_control.md | Interaction tests for Group 2 (Runner Control) |
| 03_system_prompt.md | Interaction tests for Group 3 (System Prompt) |
| 04_isolated_subcommand.md | Interaction tests for Group 4 (Isolated Subcommand) |

### Index

| Group | File | Tests |
|-------|------|-------|
| Claude-Native Flags (Group 1) | [01_claude_native_flags.md](01_claude_native_flags.md) | 4 CC |
| Runner Control (Group 2) | [02_runner_control.md](02_runner_control.md) | 4 CC |
| System Prompt (Group 3) | [03_system_prompt.md](03_system_prompt.md) | 4 CC |
| Isolated Subcommand (Group 4) | [04_isolated_subcommand.md](04_isolated_subcommand.md) | 4 CC |
