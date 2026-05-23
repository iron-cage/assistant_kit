# Parameter :: 20. `--timeout`

Maximum seconds to wait for the subprocess to complete.
If the subprocess exceeds this limit and did not refresh credentials,
`clr` exits with code 2. If credentials were refreshed during the
timeout window, the updated file is written back and exit code is 0.

- **Type:** [`TimeoutSecs`](../type.md#type--9-timeoutsecs)
- **Default:** 30 (`isolated`), 45 (`refresh`)
- **Command:** [`isolated`](../command.md#command--2-isolated), [`refresh`](../command.md#command--3-refresh)

```sh
clr isolated --creds creds.json --timeout 60 "Explain closures"
clr isolated --creds creds.json --timeout 5 -- --version   # fast check
clr refresh --creds creds.json --timeout 90                # slow network
clr isolated --creds creds.json --timeout 0 "test"         # immediate timeout
```

**Note:** Default differs by command: `isolated` defaults to 30s (general task
execution), `refresh` defaults to 45s (allows headroom for slow networks and
API rate limiting during OAuth token exchange).

**Note:** A timeout of `0` causes immediate expiry — useful for testing the
credential-refresh path (OAuth token written at startup before subprocess
blocks on input).
