# Parameter Group Tests

### Scope

- **Purpose**: Document interaction tests for clr parameter groups.
- **Responsibility**: Index of per-parameter-group interaction test files covering group-level behavior.
- **In Scope**: All 4 clr parameter group test files.
- **Out of Scope**: Per-command tests (→ `command/`), per-parameter edge cases (→ `param/`).

Per-group interaction test indices for `clr`. See [004_param_group.md](../../../../docs/cli/004_param_group.md) for specification.

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_claude_native_flags.md | Interaction tests for Group 1 (Claude-Native Flags) |
| 002_runner_control.md | Interaction tests for Group 2 (Runner Control) |
| 003_system_prompt.md | Interaction tests for Group 3 (System Prompt) |
| 004_credential_operations.md | Interaction tests for Group 4 (Credential Operations) |

### Index

| Group | File | Tests |
|-------|------|-------|
| Claude-Native Flags (Group 1) | [001_claude_native_flags.md](001_claude_native_flags.md) | 4 CC |
| Runner Control (Group 2) | [002_runner_control.md](002_runner_control.md) | 4 CC |
| System Prompt (Group 3) | [003_system_prompt.md](003_system_prompt.md) | 4 CC |
| Credential Operations (Group 4) | [004_credential_operations.md](004_credential_operations.md) | 6 CC |
