# CLI Parameter: --creds

Path to a JSON credentials file written to the isolated subprocess's
`~/.claude/.credentials.json`. Optional for the `isolated` and `refresh` commands;
when omitted, defaults to `~/.claude/.credentials.json` in the caller's real `HOME`
(the current Claude Code account's credentials file).
If Claude refreshes its OAuth token during the run, the credentials
file is updated in-place with the new token before `clr` exits.

- **Type:** [`CredentialsFilePath`](../type/08_credentials_file_path.md)
- **Default:** `~/.claude/.credentials.json`
- **Command:** [`isolated`](../command/03_isolated.md), [`refresh`](../command/04_refresh.md)
- **JSON Key:** `"creds"`

```sh
clr isolated "Fix bug"                                    # uses ~/.claude/.credentials.json
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /tmp/test_creds.json --timeout 10 "hi"
clr refresh                                               # uses ~/.claude/.credentials.json
clr refresh --creds ~/.claude/.credentials.json
```

**Note:** The file must exist and be readable at invocation time, whether specified
explicitly or resolved from the default. When `--creds` is omitted and `CLR_CREDS`
is unset, the default `$HOME/.claude/.credentials.json` is used; if `HOME` is not
set or the file is absent, the command exits 1 with a file-not-found error.
The path is resolved against the caller's working directory; relative paths
are NOT resolved against the temp HOME created for the subprocess.

**Note:** `--creds` is positional-invariant — it may appear before or after
`[MESSAGE]` and `--timeout` with identical behavior.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`CredentialsFilePath`](../type/08_credentials_file_path.md) | Semantic | String | file must exist and be readable |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | `--timeout`, `--trace` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`isolated`](../command/03_isolated.md) | `~/.claude/.credentials.json` | Optional; defaults to current account credentials |
| 3 | [`refresh`](../command/04_refresh.md) | `~/.claude/.credentials.json` | Optional; defaults to current account credentials |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |
