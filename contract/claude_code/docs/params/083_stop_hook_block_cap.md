# stop_hook_block_cap

Maximum number of consecutive tool calls a hook can block before being bypassed.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` |
| Config Key | — |

### Type

integer

### Default

Binary default (unspecified)

### Since

v2.1.147

### Description

Sets an upper limit on how many consecutive tool calls a `PreToolUse` hook can
block. After this cap is reached, subsequent tool calls bypass the hook to
prevent infinite blocking loops. Prevents runaway hook configurations from
making sessions unusable.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [031_hooks.md](031_hooks.md) | Hooks configuration |
