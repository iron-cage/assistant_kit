# CLI Parameter: --creds

Path to a JSON credentials file written to the isolated subprocess's
`~/.claude/.credentials.json`. Required for the `isolated` and `refresh` commands.
If Claude refreshes its OAuth token during the run, the credentials
file is updated in-place with the new token before `clr` exits.

- **Type:** [`CredentialsFilePath`](../type/08_credentials_file_path.md)
- **Default:** — (required)
- **Command:** [`isolated`](../command/02_isolated.md), [`refresh`](../command/03_refresh.md)

```sh
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /tmp/test_creds.json --timeout 10 "hi"
clr refresh --creds ~/.claude/.credentials.json
```

**Note:** The file must exist before invocation.
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
| 2 | [`isolated`](../command/02_isolated.md) | — | Required parameter |
| 3 | [`refresh`](../command/03_refresh.md) | — | Required parameter |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |
