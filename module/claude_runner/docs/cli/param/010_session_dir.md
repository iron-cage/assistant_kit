# Parameter :: 10. `--session-dir`

Override the session storage directory. Passed via the
`CLAUDE_SESSION_DIR` environment variable.

- **Type:** [`DirectoryPath`](../005_type.md#type--2-directorypath)
- **Default:** — (Claude Code default)
- **Command:** [`run`](../001_command.md#command--1-run)
- **Group:** [Runner Control](../004_param_group.md#group--2-runner-control)
- **Validation:** requires a value

```sh
clr "Fix bug" --session-dir /tmp/my-sessions
```
