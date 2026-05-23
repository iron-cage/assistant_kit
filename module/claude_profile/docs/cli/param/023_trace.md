# Parameter :: 23. `trace::`

When enabled, writes one `[trace]` diagnostic line to stderr for each internal operation performed by `.usage`: credential file reads, API calls (URL + token prefix), API results, and every step of the `refresh::1` retry path.

- **Type:** `bool`
- **Default:** `0` (off — no diagnostic output)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; ignored in live monitor mode (`live::1`)
- **Commands:** [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Exposes the internal fetch and refresh mechanics so failures can be diagnosed without guessing. Particularly useful when `refresh::1` appears to have no effect — trace shows exactly whether the refresh was triggered, what `run_isolated` returned, and why the retry was skipped.
- **Group:** [Fetch Behavior](../param_group/003_fetch_behavior.md)

**Examples:**

```text
trace::0   → no diagnostic output (default)
trace::1   → print [trace] lines to stderr; stdout output unchanged
```

**Notes:**
- Output goes to stderr so it does not interfere with `format::json` parsing on stdout.
- Token values in GET lines are truncated to the first 20 characters followed by `...` (`sk-ant-oA3Txy6P1wRmV2...`).
- The fetch phase emits one `reading` line, one `GET` line (with token prefix and expiry status), and one `result` line per account. The refresh phase emits one `should_retry` line per account, then detailed lifecycle step lines from `refresh_account_token` for accounts where a refresh is attempted.
- Full trace output for an expired account whose OAuth refresh succeeds:
  ```
  [trace] alice@example.com  reading /home/user/.pro/.../alice@example.com.credentials.json
  [trace] alice@example.com  GET https://api.anthropic.com/api/oauth/usage  token=sk-ant-oA3Txy6P1w...  exp=expired(2d 3h ago)
  [trace] alice@example.com  result: Err(HTTP transport error: HTTP 401)
  [trace] refresh  alice@example.com  should_retry=true (reason: HTTP transport error: HTTP 401)
  [trace] refresh  alice@example.com  attempting token refresh
  [trace] refresh  alice@example.com  switch_account: OK
  [trace] refresh  alice@example.com  run_isolated: invoking claude  args=["--print", "."]  timeout=35s
  [trace] refresh  alice@example.com  run_isolated: OK credentials=Some
  [trace] refresh  alice@example.com  save: OK
  [trace] refresh  alice@example.com  token refreshed, retrying quota fetch
  [trace] refresh  alice@example.com  retry OK
  ```
- For a rate-limited account with a non-expired token (refresh not triggered):
  ```
  [trace] alice@example.com  GET https://api.anthropic.com/api/oauth/usage  token=sk-ant-oA3Txy6P1w...  exp=valid(1h 22m left)
  [trace] alice@example.com  result: Err(HTTP transport error: HTTP 429)
  [trace] refresh  alice@example.com  should_retry=false (reason: HTTP transport error: HTTP 429)
  ```
