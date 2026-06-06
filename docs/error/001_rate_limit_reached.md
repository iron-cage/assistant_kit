# Error: Rate Limit Reached

### Scope

- **Purpose**: Document the "API Error: Rate limit reached" message that Claude Code emits when the Anthropic API rejects a request due to exhausted rate quota.
- **Responsibility**: Describe the error's trigger conditions, user-visible output text, and recovery steps.
- **In Scope**: Per-minute request-volume limits, daily/monthly token budget exhaustion, concurrent-session pressure.
- **Out of Scope**: Other API errors (→ future `error/` doc instances); Anthropic subscription tier pricing (→ external Anthropic documentation).

### Abstract

Claude Code prints `API Error: Rate limit reached` when the Anthropic API returns HTTP 429 — meaning the current API key has exceeded its allowed request volume or token budget for the active time window. The message is emitted to stderr and Claude Code exits with a non-zero status. No retry is attempted automatically.

### Trigger Conditions

- **Per-minute request volume**: More requests issued in one minute than the model tier permits; the API responds with HTTP 429.
- **Monthly token budget exhausted**: The Claude Max subscription's monthly token allocation for the billing period is fully consumed.
- **Concurrent session pressure**: Multiple simultaneous `claude` processes sharing the same API key collectively burst past the per-minute rate limit.

### Recovery

1. **Wait and retry**: For per-minute limits, pause ~60 seconds then re-issue the command. Claude Code does not auto-retry.
2. **Check quota**: Log in to `claude.ai` → Settings → Billing to inspect remaining token quota for the current billing period.
3. **Reduce concurrency**: Serialize `clr` / `claude` invocations instead of running them in parallel to stay within per-minute limits.
4. **Rotate account**: Use `clp account auto-rotate` to switch to an account with remaining quota when multiple accounts are configured in the workspace.

### CLR Detection

When `clr` invokes `claude --print` and a rate-limit condition occurs, the subprocess exits non-zero with empty stderr — the rate-limit reason is written only to Claude's JSONL session file, not to stderr or stdout.

- **Primary signal — exit code 2**: the Claude CLI uses exit 2 specifically for rate-limit rejections; no output scanning is required
- **Secondary signal — pattern match**: if stderr or stdout contains `"You've hit your limit"`, the rate-limit is confirmed regardless of exit code
- **CLR stderr output**: `Error: rate limit (exit 2)`

Downstream scripts can detect this reliably:

```bash
clr run "..." 2>err.txt; code=$?
if grep -qF "rate limit" err.txt || [ "$code" = "2" ]; then
  sleep 60 && retry
fi
```

`ExecutionOutput::classify_error()` returns `Some(ErrorKind::RateLimit)` for this case, enabling programmatic branching without string-parsing CLR output.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../module/claude_profile/src/commands.rs` | `account auto-rotate` command — mitigates rate-limit exhaustion across accounts |
| source | `../../module/claude_runner/src/main.rs` | Entry point that invokes the `claude` binary and propagates its exit code |
| source | `../../module/claude_runner_core/src/types.rs` | `ErrorKind::RateLimit` variant and `classify_error()` on `ExecutionOutput` |
