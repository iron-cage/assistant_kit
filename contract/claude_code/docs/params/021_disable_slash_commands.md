# disable_slash_commands

Disables all slash command skills for the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--disable-slash-commands` |
| Env Var | — |
| Config Key | — |

### Type

bool

### Default

`off`

### Description

Disables all slash command skills for the session. Slash commands (e.g. `/commit`, `/review-pr`) are user-invocable skills defined in command files. Disabling them prevents accidental skill invocation in contexts where raw Claude behaviour without skill augmentation is required. Has no effect on tool availability — only on slash-command parsing.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |