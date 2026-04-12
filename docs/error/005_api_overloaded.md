# Error: API Overloaded

### Scope

- **Purpose**: Document the HTTP 529 overloaded error that Claude Code emits when the Anthropic API is temporarily at capacity.
- **Responsibility**: Distinguish this error from rate-limit and timeout errors, describe its trigger conditions, and provide recovery steps.
- **In Scope**: HTTP 529 `overloaded_error`, conditions that trigger server-side overload, recovery options during Anthropic infrastructure incidents.
- **Out of Scope**: Rate-limit errors HTTP 429 (→ `error/001_rate_limit_reached.md`); request-timeout errors (→ `error/004_request_timed_out.md`).

### Abstract

Claude Code emits this error when the Anthropic API returns HTTP 529 — indicating the API cluster is temporarily at capacity and cannot accept new requests. Unlike the rate-limit error (which is per-key quota exhaustion), overloaded errors are infrastructure-level and affect all users equally during peak demand or incident windows.

**Terminal output:**
```
API Error: 529 {"type":"error","error":{"type":"overloaded_error","message":"Overloaded"},"request_id":"req_011..."}
```

Exit code: non-zero. Claude Code does not auto-retry for overloaded errors (unlike request-timeout errors which retry up to 10 times).

### Trigger Conditions

- **Peak demand**: The Anthropic API cluster is serving more requests than its current capacity; new requests are shed with 529 until load falls.
- **Planned or unplanned maintenance**: A rolling restart, failover, or capacity event temporarily reduces available capacity.
- **Geographic concentration**: A specific API region is overloaded while other regions remain healthy; the request is routed to the saturated region.
- **Burst after outage**: A large wave of retries from many clients simultaneously after a resolved incident can cause a secondary overload.

### Recovery

1. **Wait and retry manually**: Unlike rate-limit errors, there is no per-key quota counter to wait for — the server is simply busy. Wait 30–120 seconds and reissue the command.
2. **Check `status.anthropic.com`**: If the error persists, an active incident is likely listed. The status page shows per-region and per-model health.
3. **Retry with exponential backoff** (for automated `clr` usage): Space retries at 30 s, 60 s, 120 s intervals. Flooding retries worsens the overload condition for all users.
4. **Use account rotation**: `clp account auto-rotate` switches to a different account; if the overload is per-account or the accounts are on different API credentials, it may land on a less-loaded path.
5. **Defer non-urgent work**: If the task is not time-sensitive, schedule it for off-peak hours (early morning UTC tends to have lower API load).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| error | [error/001_rate_limit_reached.md](001_rate_limit_reached.md) | Rate-limit error (429) — per-key quota, distinct from capacity overload |
| error | [error/004_request_timed_out.md](004_request_timed_out.md) | Timeout error — connection-level failure vs server-side capacity rejection |
| source | `../../module/claude_profile/src/commands.rs` | `account auto-rotate` command for switching accounts under error conditions |
