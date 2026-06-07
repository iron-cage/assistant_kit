# Parameter :: 23. `trace::`

When enabled, writes `[trace]` diagnostic lines to stderr for internal operations performed by any `clp` command: file reads, API calls, subprocess lifecycle steps, and multi-step operation outcomes.

- **Type:** `bool`
- **Default:** `0` (off — no diagnostic output)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; ignored in live monitor mode (`live::1`)
- **Commands:** [`.credentials.status`](../command/002_credentials.md), [`.accounts`](../command/001_account.md), [`.account.limits`](../command/001_account.md), [`.account.save`](../command/001_account.md), [`.account.use`](../command/001_account.md#command--5-accountuse), [`.account.delete`](../command/001_account.md), [`.account.relogin`](../command/001_account.md), [`.account.rotate`](../command/001_account.md), [`.account.inspect`](../command/001_account.md#command--15-accountinspect), [`.token.status`](../command/005_token.md), [`.paths`](../command/004_paths.md), [`.usage`](../command/006_usage.md#command--9-usage)
- **Purpose:** Exposes internal mechanics so failures can be diagnosed without guessing. On `.usage`: shows credential reads, API calls (URL + token prefix), API results, and every lifecycle step of the `refresh::1` retry and `touch::1` subprocess paths. On `.account.use`: shows credential read, quota fetch, idle/active determination, model/effort resolution, and subprocess dispatch decision (only when `touch::1`). On `.account.inspect`: shows per-endpoint call with URL and HTTP status for endpoints 001, 002, and 005. On other commands: shows file-read and write steps.
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
  [trace] refresh  alice@example.com  read credentials: OK
  [trace] refresh  alice@example.com  run_isolated: invoking claude  args=["--print", "."]  timeout=35s
  [trace] refresh  alice@example.com  run_isolated: OK credentials=Some
  [trace] refresh  alice@example.com  write credentials: OK
  [trace] refresh  alice@example.com  save: OK
  [trace] refresh  alice@example.com  token refreshed, retrying quota fetch
  [trace] refresh  alice@example.com  retry OK
  [trace] refresh  alice@example.com  restore switch_account: OK
  ```
- For a rate-limited account with a non-expired token (refresh not triggered):
  ```
  [trace] alice@example.com  GET https://api.anthropic.com/api/oauth/usage  token=sk-ant-oA3Txy6P1w...  exp=valid(1h 22m left)
  [trace] alice@example.com  result: Err(HTTP transport error: HTTP 429)
  [trace] refresh  alice@example.com  should_retry=false (reason: HTTP transport error: HTTP 429)
  ```
- Full `.account.use` trace for an idle account (subprocess spawned):
  ```
  [trace] account.use  alice@home.com  reading /home/user/.pro/.../alice@home.com.credentials.json
  [trace] account.use  alice@home.com  reading: OK
  [trace] account.use  alice@home.com  quota fetch: OK
  [trace] account.use  alice@home.com  idle check: resets_at=absent → idle
  [trace] account.use  alice@home.com  model: claude-opus-4-6  effort: low
  [trace] account.use  alice@home.com  subprocess: spawned
  ```
- `.account.use` trace for an already-active account (subprocess skipped):
  ```
  [trace] account.use  alice@home.com  reading /home/user/.pro/.../alice@home.com.credentials.json
  [trace] account.use  alice@home.com  reading: OK
  [trace] account.use  alice@home.com  quota fetch: OK
  [trace] account.use  alice@home.com  idle check: resets_at=present → already active
  [trace] account.use  alice@home.com  subprocess: skipped (reason: already active)
  ```
