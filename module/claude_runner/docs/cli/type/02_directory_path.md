# CLI Type: DirectoryPath

Filesystem path to a directory. Passed as-is to the subprocess working
directory or session storage environment variable.

- **Purpose:** Filesystem path to a directory
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** any valid filesystem path
- **Parsing:** consumed as the next token after `--dir` or `--session-dir`
- **Methods:** —

```sh
clr --dir /home/user/project "Fix bug"
clr --session-dir /tmp/sessions "test"
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--dir`, `--session-dir` |
| 5 | [`ask`](../command/05_ask.md) | `--dir`, `--session-dir` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 8 | [`--dir`](../param/008_dir.md) | 2 |
| 10 | [`--session-dir`](../param/010_session_dir.md) | 2 |
