# CLI Type: FilePath

Filesystem path to a readable file whose content is piped as standard input
to the `claude` subprocess. The file is opened by the runner at spawn time;
if it cannot be read, `execute()` returns an error.

- **Purpose:** Filesystem path to a readable file piped as subprocess stdin
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** file must exist and be readable at invocation time
- **Parsing:** consumed as the next token after `--file`; path resolved
  against the caller's working directory
- **Methods:** —

```sh
clr --file notes.md "Summarise the above"
clr --file /tmp/diff.txt -p "Review this diff"
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`run`](../command/01_run.md) | `--file` |
| 5 | [`ask`](../command/05_ask.md) | `--file` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 25 | [`--file`](../param/025_file.md) | 2 |
