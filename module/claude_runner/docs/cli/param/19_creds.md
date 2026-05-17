# Parameter :: 19. `--creds`

Path to a JSON credentials file written to the isolated subprocess's
`~/.claude/.credentials.json`. Required for the `isolated` command.
If Claude refreshes its OAuth token during the run, the credentials
file is updated in-place with the new token before `clr isolated` exits.

- **Type:** [`CredentialsFilePath`](../type.md#type--8-credentialsfilepath)
- **Default:** — (required)
- **Command:** [`isolated`](../command.md#command--2-isolated)

```sh
clr isolated --creds ~/.claude/.credentials.json "Fix bug"
clr isolated --creds /tmp/test_creds.json --timeout 10 "hi"
```

**Note:** The file must exist before invocation.
The path is resolved against the caller's working directory; relative paths
are NOT resolved against the temp HOME created for the subprocess.

**Note:** `--creds` is positional-invariant — it may appear before or after
`[MESSAGE]` and `--timeout` with identical behavior.
