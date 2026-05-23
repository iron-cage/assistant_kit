# skip_dangerous_mode_permission_prompt

Suppresses the interactive confirmation prompt when launching with dangerous mode active.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `skipDangerousModePermissionPrompt` |

### Type

bool

### Default

`false`

### Description

When `true`, suppresses the interactive "Are you sure?" confirmation dialog shown when `--dangerously-skip-permissions` is active. Useful in CI/CD environments and automated pipelines where no TTY is available to answer the prompt. Setting this in `~/.claude/settings.json` makes dangerous mode fully non-interactive across all sessions. Has no effect unless dangerous mode is active.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [dangerously_skip_permissions.md](dangerously_skip_permissions.md) | `--dangerously-skip-permissions` flag |
| doc | [allow_dangerously_skip_permissions.md](allow_dangerously_skip_permissions.md) | `--allow-dangerously-skip-permissions` flag |
