# CLI Parameter: --timeout

Maximum seconds to wait for the subprocess to complete.
If the subprocess exceeds this limit and did not refresh credentials,
`clr` exits with code 2. If credentials were refreshed during the
timeout window, the updated file is written back and exit code is 0.

- **Type:** [`TimeoutSecs`](../type/09_timeout_secs.md)
- **Default:** 30 (`isolated`), 45 (`refresh`)
- **Command:** [`isolated`](../command/02_isolated.md), [`refresh`](../command/03_refresh.md)

```sh
clr isolated --creds creds.json --timeout 60 "Explain closures"
clr isolated --creds creds.json --timeout 5 -- --version   # fast check
clr refresh --creds creds.json --timeout 90                # slow network
clr isolated --creds creds.json --timeout 0 "test"         # immediate timeout
```

**Note:** Default differs by command: `isolated` defaults to 30s (general task
execution), `refresh` defaults to 45s (allows headroom for slow networks and
API rate limiting during OAuth token exchange).

**Note:** On timeout, any partial stdout accumulated by the subprocess before
the timeout fires is preserved in the error output, so diagnostic context is
not discarded.

**Note:** A timeout of `0` causes immediate expiry — useful for testing the
credential-refresh path (OAuth token written at startup before subprocess
blocks on input).

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`TimeoutSecs`](../type/09_timeout_secs.md) | Semantic | unsigned 64-bit integer | non-negative integer |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | `--creds`, `--trace` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`isolated`](../command/02_isolated.md) | 30 | 30s for general task execution |
| 3 | [`refresh`](../command/03_refresh.md) | 45 | 45s for slow OAuth token exchange |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |
