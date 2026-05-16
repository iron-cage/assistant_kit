# Parameter :: 8. `--dir`

Set the working directory for the Claude Code subprocess. The runner
changes to this directory before invoking claude.

- **Type:** [`DirectoryPath`](../type.md#type--2-directorypath)
- **Default:** current working directory
- **Command:** [`run`](../command.md#command--1-run)
- **Group:** [Runner Control](../param_group.md#group--2-runner-control)
- **Validation:** requires a value; `--dir` at end of argv → error

```sh
clr "Fix bug" --dir /path/to/project
```

**Note:** When `--dir` appears multiple times, the last value wins.
