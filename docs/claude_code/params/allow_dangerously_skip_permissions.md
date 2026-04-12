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

### Description

Makes skip-permissions available as an option during the session without enabling it by default. Differs from `--dangerously-skip-permissions` in that permission bypass is not automatic — Claude may still prompt in some cases. Recommended for sandboxes where the operator wants to allow unattended operation but not force it unconditionally.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |