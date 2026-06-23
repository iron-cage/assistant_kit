# Subprocess: Token Refresh Invocation

### Purpose

Document when `.usage refresh::` triggers `refresh_account_token()` and the exact predicate controlling that decision.

### Trigger Predicate (`should_refresh()`)

An account is submitted for refresh when ALL of the following are true:

| Condition | Rationale |
|-----------|-----------|
| `aq.is_owned == true` AND `aq.is_occupied_elsewhere == false` | G2 gate — non-owned or occupied accounts are skipped (BUG-303 fix) |
| NOT solo gate (solo::0 or is_current) | Solo gate skips non-current accounts when `solo::1` |
| `result` is auth error (`"401"` or `"403"`) | Direct auth failure |
| OR `result` is 429 AND `expires_at_ms / 1000 ≤ now_secs` | 429 with locally-expired token (stale per-account copy) |

Source: `src/usage/refresh_predicate.rs`

### Invocation

```rust
account::refresh_account_token(
    name             : &str,
    credential_store : &Path,
    claude_paths     : Option<&ClaudePaths>,  // Some = check live creds for race recovery
    imodel           : IsolatedModel,
    effort_opt       : Option<&str>,
    trace            : bool,
)
```

Internally runs `run_isolated(["--print", "."], model, timeout_secs=35)`.

### Post-Refresh Actions

After `refresh_account_token()` returns `Some(new_json)`:
1. Derive `expires_at_ms` (JWT `exp` → `expiresAt` fallback — see [subprocess/002](002_credential_writeback.md))
2. Retry `fetch_oauth_usage(new_token)` — one retry per account per invocation
3. On successful retry: `fetch_oauth_account(new_token)` to repopulate `~Renews`/`Sub` columns (Fix BUG-171)

After `refresh_account_token()` returns `None`:
- Set `aq.result = Err("refresh token expired")` (Fix BUG-297 — prevent downstream `apply_touch` from firing on unrecoverable account)

### Default Behavior

`refresh::1` is the default. Every `clp .usage` call automatically retries 401/403. Pass `refresh::0` to disable.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/017_token_refresh.md](../feature/017_token_refresh.md) | Full feature spec, all acceptance criteria |
| [subprocess/001](001_run_isolated_contract.md) | `run_isolated()` contract |
| [subprocess/002](002_credential_writeback.md) | Credential write-back protocol |
| [subprocess/004](004_session_touch_invocation.md) | Touch invocation (same `refresh_account_token()` call) |
