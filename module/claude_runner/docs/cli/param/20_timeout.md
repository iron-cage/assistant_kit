# Parameter :: 20. `--timeout`

Maximum seconds to wait for the isolated Claude subprocess to complete.
If the subprocess exceeds this limit and did not refresh credentials,
`clr isolated` exits with code 2. If credentials were refreshed during
the timeout window, the updated file is written back and exit code is 0.

- **Type:** [`TimeoutSecs`](../type.md#type--9-timeoutsecs)
- **Default:** 30
- **Command:** [`isolated`](../command.md#command--2-isolated)

```sh
clr isolated --creds creds.json --timeout 60 "Explain closures"
clr isolated --creds creds.json --timeout 5 -- --version   # fast check
clr isolated --creds creds.json --timeout 0 "test"         # immediate timeout
```

**Note:** A timeout of `0` causes immediate expiry — useful for testing the
credential-refresh path (OAuth token written at startup before subprocess
blocks on input).

**Note:** `--timeout` and `--creds` are the only parameters exclusive to
`isolated`. All other Claude behavior is controlled by `[MESSAGE]` or flags
forwarded via `--` separator.
