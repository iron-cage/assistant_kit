# Feature: Expired Token Refresh via Isolated Subprocess

### Scope

- **Purpose**: Allow `.usage` to silently refresh expired OAuth tokens before fetching quota, so users see current quota data rather than per-account auth error rows.
- **Responsibility**: Documents the `refresh::` parameter, its retry-on-auth-error trigger, the `claude_runner_core::run_isolated()` invocation, and credential write-back to disk.
- **In Scope**: `refresh::` parameter semantics; HTTP auth error detection from `fetch_oauth_usage`; `run_isolated()` call chain from `usage.rs`; credential write-back; one-retry-per-account semantics; non-aborting error handling.
- **Out of Scope**: `run_isolated()` internals (→ `claude_runner_core/docs/feature/004_run_isolated.md`); live monitor mode (→ `018_live_monitor.md`); `fetch_oauth_usage` implementation (→ `claude_quota`); proactive expiry detection before any API call.

### Design

The `refresh::` parameter takes `0` (default, off) or `1` (on). When `0`, `.usage` behaves identically to the baseline — auth errors appear as error rows in the table and no subprocess is spawned.

When `refresh::1`, the command wraps `fetch_oauth_usage` with a retry layer: on an HTTP authentication error (401 or 403), it calls `claude_runner_core::run_isolated()` with that account's stored credentials JSON, then retries the quota fetch if updated credentials are returned.

**Trigger condition:** Only HTTP auth errors trigger a refresh attempt. Network failures, timeouts, and non-auth HTTP errors are passed through as-is. This prevents unnecessary subprocess launches on transient connectivity issues.

**Algorithm:**

```
for each account:
    token  = read_token(account)
    result = fetch_oauth_usage(token)

    if refresh_param == 1 AND is_auth_error(result):
        creds_json  = read_credentials_file(account)
        run_result  = run_isolated(creds_json, refresh_trigger_args, timeout_secs=120)

        if run_result is Ok(r) AND r.credentials is Some(new_json):
            write new_json to account credential file on disk
            new_token = extract_token(new_json)
            result    = fetch_oauth_usage(new_token)   // one retry, result is final

    push (account, result) to table
```

Where `is_auth_error(e)` matches `QuotaError::HttpTransport(msg)` when `msg` contains "401" or "403".

**Retry semantics:** Exactly one retry per account per invocation. If the retried `fetch_oauth_usage` also fails, the final error is shown in the account's row — the table continues processing remaining accounts (non-aborting).

**Credential write-back:** When `run_isolated` returns `credentials: Some(new_json)`, the account's credential file at `{credential_store}/{name}.credentials.json` is overwritten with `new_json` before the retry. This ensures future invocations use the refreshed token without requiring another subprocess launch.

**Subprocess trigger args:** `run_isolated` is called with minimal args (e.g., `["--print", ".", "--output-format", "text", "--max-tokens", "1"]`) to trigger claude's internal OAuth refresh with minimal API cost. The args produce a trivial API call; the important side-effect is that claude reads the expired credentials, contacts Anthropic's auth server with the `refresh_token`, and writes the updated `access_token` back to `HOME/.claude.json`.

**Feature gate:** The retry logic is compiled only under `#[cfg(feature = "enabled")]`, matching `fetch_oauth_usage`. When `enabled` is absent, `refresh::1` is accepted as a parameter but no refresh attempt is made (offline builds cannot spawn subprocesses).

**No behavioral change at default:** `refresh::0` introduces no new overhead, no subprocess spawn, and no credential file writes. Existing tests are unaffected.

**Output format:** When refresh succeeds, the account's row shows normal quota data — the refresh is invisible to the user. When refresh fails (subprocess error or second fetch also fails), the error reason appears in the account's row exactly as it would without `refresh::`.

### Acceptance Criteria

- **AC-18**: `refresh::0` (default) produces no calls to `run_isolated`; `.usage` behavior is unchanged from the baseline.
- **AC-19**: `refresh::1` invokes `claude_runner_core::run_isolated()` for any account whose `fetch_oauth_usage` returns an HTTP auth error (401 or 403).
- **AC-20**: When `run_isolated` returns `credentials: Some(new_json)`, the credential file for that account is updated on disk before the retry fetch.
- **AC-21**: If the refresh attempt fails (subprocess error, or retried fetch still fails), the account's row shows the final error; the remaining accounts are still processed and the table is still rendered.
- **AC-22**: `refresh::` does not affect `format::json` output structure — refreshed accounts appear as normal data objects, failed-refresh accounts appear as error objects.
- **AC-23**: The `refresh::` parameter appears in `.usage --help` output with its default value (`0`).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/usage.rs` | `refresh::` param read; retry-on-auth-error logic; credential write-back |
| source | `src/lib.rs` | `refresh::` parameter registration via `register_commands()` |
| dep | `claude_runner_core` | `run_isolated()` — isolated subprocess; `IsolatedRunResult`, `RunnerError` types |
| dep | `claude_quota` | `fetch_oauth_usage()` — quota HTTP transport; `QuotaError::HttpTransport` |
| task | `task/claude_runner_core/136_run_isolated_subprocess.md` | Prerequisite: implement `run_isolated()` |
| task | `task/claude_profile/137_usage_refresh_param.md` | Implementation task for this feature |
| doc | [009_token_usage.md](009_token_usage.md) | Baseline `.usage` algorithm that this extends |
| doc | `claude_runner_core/docs/feature/004_run_isolated.md` | `run_isolated()` API contract |
| doc | [cli/commands.md](../cli/commands.md#command--9-usage) | `.usage` CLI command specification |
| doc | [cli/params.md](../cli/params.md#parameter--19-refresh) | `refresh::` parameter specification |
