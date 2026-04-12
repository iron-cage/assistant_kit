# permission_mode

Sets fine-grained permission behaviour for tool invocations during the session.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--permission-mode <mode>` |
| Env Var | — |
| Config Key | — |

### Type

enum — `default` `acceptEdits` `bypassPermissions` `dontAsk` `plan` `auto`

### Default

`default`

### Description

Sets fine-grained permission behaviour for the session. `default` prompts for each tool use. `acceptEdits` auto-accepts file edits but prompts for bash. `bypassPermissions` skips all checks (equivalent to `--dangerously-skip-permissions`). `dontAsk` suppresses permission prompts. `plan` enters read-only planning mode. `auto` lets Claude decide autonomously.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |