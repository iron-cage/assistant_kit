# allow_dangerously_skip_permissions

Makes permission bypass available as a session option without enabling it unconditionally.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--allow-dangerously-skip-permissions` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Since

pre-v1.0 (unverified)

### Description

Makes skip-permissions available as an option during the session without enabling it by default. Differs from `--dangerously-skip-permissions` in that permission bypass is not automatic — Claude may still prompt in some cases. Recommended for sandboxes where the operator wants to allow unattended operation but not force it unconditionally.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [018_dangerously_skip_permissions.md](018_dangerously_skip_permissions.md) | Unconditional bypass (stronger form) |
| doc | [061_skip_dangerous_mode_permission_prompt.md](061_skip_dangerous_mode_permission_prompt.md) | Suppress dangerous-mode confirmation prompt |