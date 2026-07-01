# Parameter: 23. `trace::`

When enabled, writes timestamped diagnostic lines to stderr (prefix format: `YYYY-MM-DD · HH:MM:SS · `, UTC) for internal operations performed by any `clp` command: file reads, API calls, subprocess lifecycle steps, and multi-step operation outcomes.

- **Default:** `0` (off — no diagnostic output)
- **Constraints:** Accepted values: `0`, `1`, `false`, `true`; ignored in live monitor mode (`live::1`)
- **Purpose:** Exposes internal mechanics so failures can be diagnosed without guessing. On `.usage`: shows credential reads, API calls (URL + token prefix), API results, and every lifecycle step of the `refresh::1` retry and `touch::1` subprocess paths. On `.account.use`: shows credential read, quota fetch, idle/active determination, model/effort resolution, and subprocess dispatch decision (only when `touch::1`). On `.account.inspect`: shows per-endpoint call with URL and HTTP status for endpoints 001, 002, and 005. On other commands: shows file-read and write steps.

**Examples:**

```text
trace::0   → no diagnostic output (default)
trace::1   → print timestamped diagnostic lines to stderr; stdout output unchanged
```

**Notes:**
- Output goes to stderr so it does not interfere with `format::json` parsing on stdout.
- Each diagnostic line is prefixed with a UTC timestamp in `YYYY-MM-DD · HH:MM:SS · ` format, enabling correlation with watchdog and other time-stamped output.
- Token values in GET lines are truncated to the first 20 characters followed by `...` (`sk-ant-oA3Txy6P1wRmV2...`).
- The fetch phase emits one `reading` line, one `GET` line (with token prefix and expiry status), and one `result` line per account. The refresh phase emits one `should_retry` line per account, then detailed lifecycle step lines from `refresh_account_token` for accounts where a refresh is attempted.
- Full trace output for an expired account whose OAuth refresh succeeds:
  ```
  2026-06-25 · 16:40:04 · alice@example.com  reading /home/user/.pro/.../alice@example.com.credentials.json
  2026-06-25 · 16:40:04 · alice@example.com  GET https://api.anthropic.com/api/oauth/usage  token=sk-ant-oA3Txy6P1w...  exp=expired(2d 3h ago)
  2026-06-25 · 16:40:04 · alice@example.com  result: Err(HTTP transport error: HTTP 401)
  2026-06-25 · 16:40:04 · refresh  alice@example.com  should_retry=true (reason: HTTP transport error: HTTP 401)
  2026-06-25 · 16:40:04 · refresh  alice@example.com  attempting token refresh
  2026-06-25 · 16:40:04 · refresh  alice@example.com  read credentials: OK
  2026-06-25 · 16:40:04 · refresh  alice@example.com  run_isolated: invoking claude  args=["--print", "."]  timeout=35s
  2026-06-25 · 16:40:04 · refresh  alice@example.com  run_isolated: OK credentials=Some
  2026-06-25 · 16:40:04 · refresh  alice@example.com  write credentials: OK
  2026-06-25 · 16:40:04 · refresh  alice@example.com  save: OK
  2026-06-25 · 16:40:04 · refresh  alice@example.com  token refreshed, retrying quota fetch
  2026-06-25 · 16:40:04 · refresh  alice@example.com  retry OK
  2026-06-25 · 16:40:04 · refresh  alice@example.com  restore switch_account: OK
  ```
- For a rate-limited account with a non-expired token (refresh not triggered):
  ```
  2026-06-25 · 16:40:04 · alice@example.com  GET https://api.anthropic.com/api/oauth/usage  token=sk-ant-oA3Txy6P1w...  exp=valid(1h 22m left)
  2026-06-25 · 16:40:04 · alice@example.com  result: Err(HTTP transport error: HTTP 429)
  2026-06-25 · 16:40:04 · refresh  alice@example.com  should_retry=false (reason: HTTP transport error: HTTP 429)
  ```
- Full `.account.use` trace when quota fetch succeeds (subprocess always dispatched; Fix(BUG-285)):
  ```
  2026-06-25 · 16:40:04 · account.use  alice@home.com  reading /home/user/.pro/.../alice@home.com.credentials.json
  2026-06-25 · 16:40:04 · account.use  alice@home.com  reading: OK
  2026-06-25 · 16:40:04 · account.use  alice@home.com  quota fetch: OK
  2026-06-25 · 16:40:04 · account.use  alice@home.com  subprocess: scheduled (idle check removed)
  2026-06-25 · 16:40:04 · account.use  alice@home.com  model: claude-opus-4-6  effort: low
  2026-06-25 · 16:40:04 · account.use  alice@home.com  subprocess: spawned
  ```

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 1 | [Fetch Behavior](../param_group/003_fetch_behavior.md) | Member parameter |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.credentials.status`](../command/002_credentials.md#command--10-credentialsstatus) | File-read and credential diagnostic traces |
| 2 | [`.accounts`](../command/001_account.md#command--3-accounts) | Per-account read traces |
| 3 | [`.account.limits`](../command/001_account.md#command--11-accountlimits) | File-read and write step traces |
| 4 | [`.account.save`](../command/001_account.md#command--4-accountsave) | Credential save step traces |
| 5 | [`.account.use`](../command/001_account.md#command--5-accountuse) | Quota fetch, idle check, subprocess dispatch traces |
| 6 | [`.account.delete`](../command/001_account.md#command--6-accountdelete) | Credential removal step traces |
| 7 | [`.account.relogin`](../command/001_account.md#command--12-accountrelogin) | Re-authentication step traces |
| 8 | [`.account.rotate`](../command/001_account.md#command--13-accountrotate) | Token rotation step traces |
| 9 | [`.account.inspect`](../command/001_account.md#command--15-accountinspect) | Per-endpoint call and HTTP status traces |
| 10 | [`.token.status`](../command/005_token.md#command--7-tokenstatus) | Token classification step traces |
| 11 | [`.paths`](../command/004_paths.md#command--8-paths) | Path resolution step traces |
| 12 | [`.usage`](../command/006_usage.md#command--9-usage) | Credential reads, API calls, refresh retry traces |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Credential Diagnostics](../user_story/005_credential_diagnostics.md) | Expose internal mechanics for failure diagnosis |
