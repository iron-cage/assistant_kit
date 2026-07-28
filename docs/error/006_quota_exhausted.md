# Error: Quota Exhausted

### Scope

- **Purpose**: Document the "You've hit your limit" message that Claude Code emits when the account's usage budget for the current billing period is fully consumed.
- **Responsibility**: Describe the error's trigger conditions, user-visible output text, reset timing, and recovery steps.
- **In Scope**: 5-hour session quota exhaustion, 7-day rolling quota exhaustion, billing-period budget depletion.
- **Out of Scope**: Transient per-minute rate limits (→ `001_rate_limit_reached.md`); subscription cancellation (→ `002_authentication_failed.md`); Anthropic pricing (→ external documentation).

### Abstract

Claude Code prints `You've hit your limit · resets [time]` when the account's usage quota for a billing period (5-hour session window or 7-day rolling window) is fully consumed. Unlike transient rate limits (HTTP 429), this is a period-boundary condition: recovery requires waiting for the quota reset or switching to a different account. The error includes a human-readable reset timestamp.

### Trigger Conditions

- **5-hour session quota exhausted**: The account has consumed 100% of its 5-hour rolling token budget. Resets when the 5-hour window rolls over.
- **7-day rolling quota exhausted**: The account has consumed 100% of its 7-day rolling token budget. Resets when the oldest usage falls outside the 7-day window.
- **Model-specific exhaustion**: The 7-day Sonnet-specific quota may exhaust independently of the general 7-day quota (visible as `7d(Son) = 0%` in `.usage` output).

### Recovery

1. **Switch account**: Use `clp .account.use <other-account>` to switch to an account with remaining quota. The `.usage` table shows per-account quota status.
2. **Wait for reset**: The error message includes the reset time. Session (5h) resets are shorter; weekly (7d) resets require hours to days.
3. **Use automatic rotation**: Run `clp .usage rotate::1` to automatically switch to the best available account based on quota strategy (default `sort::renew`; override with `sort::`).
4. **Check quota status**: Run `clp .usage trace::1` to see all accounts' remaining quota and next reset times.

### CLR Detection

When `clr` invokes `claude --print` and a quota-exhaustion condition occurs, the subprocess exits non-zero. The detailed message (including reset time) is written to Claude's JSONL session file; stderr/stdout may contain the `"You've hit your limit"` pattern.

- **Primary signal — pattern match**: stderr or stdout contains `"You've hit your limit"` — confirms quota exhaustion (not transient rate limit). Pattern match takes priority over exit code fallback.
- **CLR stderr output**: `Error: quota exhausted (exit N)` where N is the subprocess exit code (typically 1)
- **Distinction from rate limit**: pattern `"You've hit your limit"` → `QuotaExhausted` regardless of exit code; exit code 2 without this pattern → `RateLimit` (transient HTTP 429). Pattern match always wins over exit-code classification.

Downstream scripts can detect this:

```bash
clr run "..." 2>err.txt; code=$?
if grep -qF "quota exhausted" err.txt; then
  clp .usage rotate::1   # switch to available account
  retry
fi
```

`ExecutionOutput::classify_error()` returns `Some(ErrorKind::QuotaExhausted)` for this case, enabling programmatic branching. The variant is distinct from `ErrorKind::RateLimit` (transient 429).

### Errors

| File | Relationship |
|------|--------------|
| [001_rate_limit_reached.md](001_rate_limit_reached.md) | Transient HTTP 429 rate limit — distinct from period quota exhaustion |

### Sources

| File | Relationship |
|------|--------------|
| `../../module/claude_runner_core/src/types.rs` | `ErrorKind::QuotaExhausted` variant and `classify_error()` on `ExecutionOutput` |
| `../../module/claude_profile/src/usage/api.rs` | `.usage` command showing per-account quota status and reset times |
| `../../module/claude_profile/src/usage/api.rs` | `.usage rotate::1` — automatic account rotation on quota exhaustion (Feature 038) |
