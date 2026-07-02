# CLI Type: TimeoutSecs

Unsigned integer representing seconds to wait for the isolated Claude
subprocess to complete. Zero disables the watchdog entirely (unlimited
runtime), matching `run`/`ask` semantics.

- **Purpose:** Subprocess wait limit in seconds
- **Fundamental Type:** unsigned 64-bit integer
- **Constants:** —
- **Constraints:** non-negative integer; no upper bound enforced by clr; default 30
- **Parsing:** integer parse; rejects negative, float, non-numeric
- **Methods:** —

```sh
clr isolated --creds creds.json --timeout 0 "test"    # unlimited (no watchdog)
clr isolated --creds creds.json --timeout 30 "test"   # default (same as omitting)
clr isolated --creds creds.json --timeout 120 "test"  # 2-minute window
clr isolated --creds creds.json --timeout -1 "test"   # error: negative
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`isolated`](../command/03_isolated.md) | `--timeout` |
| 3 | [`refresh`](../command/04_refresh.md) | `--timeout` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 20 | [`--timeout`](../param/020_timeout.md) | 2 |
