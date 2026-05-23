# Parameter :: 19. `--creds`

Path to a JSON credentials file written to the isolated subprocess's
`~/.claude/.credentials.json`. Required for the `isolated` and `refresh` commands.
If Claude refreshes its OAuth token during the run, the credentials
file is updated in-place with the new token before `clr` exits.

- **Type:** [`CredentialsFilePath`](../type.md#type--8-credentialsfilepath)
- **Default:** — (required)
- **Command:** [`isolated`](../command.md#command--2-isolated), [`refresh`](../command.md#command--3-refresh)

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
