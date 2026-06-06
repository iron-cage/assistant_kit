# Error: Rate Limit Reached

### Scope

- **Purpose**: Document the transient HTTP 429 rate-limit error that Claude Code emits when the Anthropic API rejects a request due to per-minute request volume or concurrent-session pressure.
- **Responsibility**: Describe the error's trigger conditions, user-visible output text, and recovery steps.
- **In Scope**: Per-minute/per-second request-volume limits (HTTP 429), concurrent-session burst pressure.
- **Out of Scope**: Quota/budget exhaustion for billing period (→ `006_quota_exhausted.md`); other API errors (→ other `error/` doc instances); Anthropic subscription tier pricing (→ external Anthropic documentation).

### Abstract

Claude Code prints `API Error: Rate limit reached` when the Anthropic API returns HTTP 429 — meaning the current API key has exceeded its allowed request volume for the active time window. This is a transient condition: the limit resets within seconds to minutes. The message is emitted to stderr and Claude Code exits with a non-zero status. No retry is attempted automatically.

### Trigger Conditions

- **Per-minute request volume**: More requests issued in one minute than the model tier permits; the API responds with HTTP 429.
- **Concurrent session pressure**: Multiple simultaneous `claude` processes sharing the same API key collectively burst past the per-minute rate limit.

### Recovery

1. **Wait and retry**: Pause 30–60 seconds then re-issue the command. The limit resets within the current time window.
2. **Reduce concurrency**: Serialize `clr` / `claude` invocations instead of running them in parallel to stay within per-minute limits.

### CLR Detection

When `clr` invokes `claude --print` and a rate-limit condition occurs, the subprocess exits with code 2.

- **Primary signal — exit code 2**: the Claude CLI uses exit 2 specifically for rate-limit rejections; no output scanning is required
- **CLR stderr output**: `Error: rate limit (exit 2)`

Downstream scripts can detect this reliably:

```bash
clr run "..." 2>err.txt; code=$?
if [ "$code" = "2" ]; then
  sleep 60 && retry
fi
```

`ExecutionOutput::classify_error()` returns `Some(ErrorKind::RateLimit)` for exit code 2, enabling programmatic branching without string-parsing CLR output.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| related | [006_quota_exhausted.md](006_quota_exhausted.md) | Period quota exhaustion — distinct from transient rate limit |
| source | `../../module/claude_runner_core/src/types.rs` | `ErrorKind::RateLimit` variant and `classify_error()` on `ExecutionOutput` |
