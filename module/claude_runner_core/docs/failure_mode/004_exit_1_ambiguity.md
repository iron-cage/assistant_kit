# Failure Mode: Exit Code 1 Is Ambiguous

### Scope

- **Purpose**: Document that `claude` uses exit code 1 for multiple distinct failure modes, making exit-code-only error handling incorrect.
- **Responsibility**: List all failure modes that produce exit 1, explain pattern-priority ordering, and provide correct detection guidance.
- **In Scope**: Quota exhaustion via text pattern (exit 1), auth errors (exit 1), API errors (exit 1), unknown errors (exit 1).
- **Out of Scope**: Exit code 2 rate-limit sentinel (→ `failure_mode/001_rate_limit_exit_2.md`), signal exits (exit > 128).

### Behavior

The `claude` binary exits with code **1** for all of the following:

| Failure Mode | Example Text | `ErrorKind` |
|--------------|--------------|-------------|
| Quota exhausted | `"You've hit your limit"` in stdout or stderr | `QuotaExhausted` |
| Auth / org access denied | `"Your organization does not have access to Claude"` | `AuthError` |
| API error (4xx/5xx) | `"API Error: 529 overloaded"` | `ApiError` |
| Unknown / unrecognized | no matching text | `Unknown` |

A caller that maps `exit_code != 0` → generic error will conflate these four distinct failure modes, making error reporting and retry logic incorrect:
- Rate-limit should trigger a back-off wait
- AuthError should alert the operator (wrong API key or org)
- ApiError may be transient (retry with exponential back-off)
- Unknown requires investigation

### Detection Rule

Never branch on exit code 1 alone. Always inspect output first:

```
1. scan stdout + stderr for QuotaExhausted pattern → ErrorKind::QuotaExhausted
2. scan stdout + stderr for AuthError pattern      → ErrorKind::AuthError
3. scan stdout + stderr for ApiError pattern       → ErrorKind::ApiError
4. if exit_code == 2                               → ErrorKind::RateLimit   (silent sentinel)
5. if exit_code > 128                              → ErrorKind::Signal
6. any other non-zero exit                         → ErrorKind::Unknown
```

QuotaExhausted is priority 1 (distinct pattern; never co-occurs with AuthError text).
AuthError is checked before ApiError because 401 responses can contain both `"Your organization does not have access to Claude"` and `"API Error: "` simultaneously — AuthError wins.

### Anti-Pattern

```rust
// Wrong: treats all non-zero exits as the same failure
if output.exit_code != 0 {
    return Err(Error::msg("claude failed"));
}

// Correct: classify before acting
match output.classify_error() {
    Some(ErrorKind::RateLimit)       => back_off_and_retry(),
    Some(ErrorKind::QuotaExhausted)  => switch_account_or_wait(),
    Some(ErrorKind::AuthError)       => alert_operator(),
    Some(ErrorKind::ApiError)        => retry_with_backoff(),
    Some(ErrorKind::Signal)          => log_signal_kill(),
    Some(ErrorKind::Unknown)         => log_and_investigate(),
    None                             => { /* success */ }
}
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [failure_mode/001_rate_limit_exit_2.md](001_rate_limit_exit_2.md) | The one truly unambiguous exit code: 2 = rate-limit (no text) |
| doc | [failure_mode/002_diagnostic_on_stdout.md](002_diagnostic_on_stdout.md) | Why stdout must be scanned — patterns appear on both channels |
| source | `../../src/types.rs` | `ErrorKind` enum, `classify_error()` priority implementation |
| test | `../../tests/classify_error_test.rs` | T03–T08, T11–T12, priority test; full matrix |
| bug | BUG-037 | Root cause: no `ErrorKind` enum; all non-zero exits emitted the same generic message |

### Sources

| File | Notes |
|------|-------|
| `tests/classify_error_test.rs` | Full test matrix covering all six outcomes; BUG-037 root-cause comment |
| BUG-037 | Fix: added `ErrorKind` enum and `classify_error()` with priority-ordered pattern scan |
