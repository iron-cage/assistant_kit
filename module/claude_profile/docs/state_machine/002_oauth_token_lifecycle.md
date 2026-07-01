# State Machine: OAuth Token Lifecycle

### States

| State | `expiresAt` | `refreshToken` | API calls succeed? |
|-------|-------------|---------------|-------------------|
| `valid` | > `now_ms` | present | Yes |
| `at_expired` | ≤ `now_ms` | present | No (401/403) — RT can refresh |
| `rt_expired` | ≤ `now_ms` | expired server-side | No — requires browser relogin |
| `refreshed` | new value (from JWT `exp`) | new RT (rotated) | Yes |

### Transitions

```
[valid]      --time passes--> [at_expired]
[at_expired] --refresh_account_token()--> [refreshed]   (AT+RT pair rotated)
[refreshed]  --time passes--> [at_expired]              (new AT will eventually expire)
[at_expired] --refresh_account_token() with expired RT--> [rt_expired]
[rt_expired] --account.relogin browser flow--> [valid]  (new AT+RT from OAuth server)
```

### Detection

| Condition | How detected |
|-----------|-------------|
| `at_expired` | `fetch_oauth_usage()` returns HTTP 401 or 403 |
| `at_expired` (local) | `expiresAt_ms / 1000 ≤ now_secs` (from stored credential file) |
| `rt_expired` | `refresh_account_token()` returns `None` (`run_isolated` exits without credential update) |

### `expiresAt` Accuracy Warning

The `expiresAt` field in `{name}.credentials.json` is NOT updated by `run_isolated()` (BUG-162). After refresh, the new expiry must be derived from the JWT `exp` claim of the new `accessToken`, falling back to the `expiresAt` field in the response JSON. See [subprocess/002](../subprocess/002_credential_writeback.md).

### Forced Expiry for Refresh

`refresh_account_token()` sets `expiresAt: "1"` in the in-memory credential copy before calling `run_isolated` — forcing Claude CLI to treat the AT as expired, regardless of its actual validity. This rotates the RT on every call (preventing silent RT decay). The stored credential file is NOT modified.

### No `[valid]→[refreshed]` Transition — Proactive Refresh Is Out of Scope

There is no direct transition from `[valid]` to `[refreshed]`. Calling `run_isolated(["--print", "."])` while the AT is still valid causes Claude Code to use the AT as-is and exit without performing OAuth refresh → `credentials=None`. The `expiresAt=1` trick in `refresh_account_token()` only works because it forces the CLI to classify the AT as expired before the subprocess runs.

**Consequence:** Any approach that attempts to refresh a token before it expires (proactive / approaching-expiry refresh) cannot work through the `run_isolated` interface. `feature/017` line 8 explicitly marks this as **Out of Scope**. Do not add detection logic for the `[valid]→[approaching expiry]` state — the transition to `[refreshed]` from `[valid]` does not exist in this system. See BUG-323 and `pitfall/002 Pitfall 5`.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/006_token_status.md](../feature/006_token_status.md) | Token status classification (Valid, ExpiringSoon, Expired) |
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | Full refresh lifecycle |
| [feature/019_account_relogin.md](../feature/019_account_relogin.md) | RT-expired recovery |
| [subprocess/002](../subprocess/002_credential_writeback.md) | Credential write-back protocol |
| [schema/001](../schema/001_credentials_json.md) | Credential file fields |
