# CLI Type: CredentialsFilePath

Filesystem path to an existing JSON file containing Claude OAuth credentials.
The file is read before subprocess launch and written back in-place if Claude
refreshes its OAuth token during the run.

The `--creds` parameter is optional; when omitted and `CLR_CREDS` is unset,
the resolved path defaults to `$HOME/.claude/.credentials.json`.

- **Purpose:** Path to an existing credentials JSON file
- **Fundamental Type:** String
- **Constants:** —
- **Constraints:** file must exist and be readable at invocation time (whether explicit or defaulted)
- **Parsing:** consumed as the next token after `--creds`; path resolved
  against the caller's working directory, not the isolated temp `HOME`
- **Methods:** —

```sh
clr isolated "Fix bug"                                    # uses $HOME/.claude/.credentials.json
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /tmp/test_creds.json --timeout 10 "hi"
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`isolated`](../command/03_isolated.md) | `--creds` |
| 3 | [`refresh`](../command/04_refresh.md) | `--creds` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 19 | [`--creds`](../param/019_creds.md) | 2 |
