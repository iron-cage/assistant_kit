# CLI Type: TimeoutSecs

Unsigned integer representing seconds to wait for the isolated Claude
subprocess to complete. Zero causes immediate expiry — useful for testing
the credential-refresh path without waiting for Claude to start.

- **Purpose:** Subprocess wait limit in seconds
- **Fundamental Type:** unsigned 64-bit integer
- **Constants:** —
- **Constraints:** non-negative integer; no upper bound enforced by clr; default 30
- **Parsing:** integer parse; rejects negative, float, non-numeric
- **Methods:** —

```sh
clr isolated --creds creds.json --timeout 0 "test"    # immediate timeout
clr isolated --creds creds.json --timeout 30 "test"   # default (same as omitting)
clr isolated --creds creds.json --timeout 120 "test"  # 2-minute window
clr isolated --creds creds.json --timeout -1 "test"   # error: negative
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`isolated`](../command/02_isolated.md) | `--timeout` |
| 3 | [`refresh`](../command/03_refresh.md) | `--timeout` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 20 | [`--timeout`](../param/020_timeout.md) | 2 |
