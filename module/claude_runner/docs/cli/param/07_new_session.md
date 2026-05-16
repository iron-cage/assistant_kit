# Parameter :: 7. `--new-session`

Disable the default session continuation. Normally `clr` passes
`-c` to claude on every invocation, resuming the most recent conversation.
`--new-session` suppresses that flag, starting a genuinely fresh session
with no prior context.

- **Type:** bool (standalone flag)
- **Default:** false (continuation is automatic)
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)

```sh
clr --new-session "Analyse this new codebase from scratch"
clr --new-session "Review this PR fresh" --model opus
```

**Note:** Use when switching to a genuinely unrelated task where prior
conversation context would be misleading or harmful.
