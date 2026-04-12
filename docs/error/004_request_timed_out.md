# Error: Request Timed Out

### Scope

- **Purpose**: Document the request-timeout error that Claude Code emits and its built-in exponential-backoff retry sequence.
- **Responsibility**: Describe the exact terminal output format, retry schedule, failure condition after 10 attempts, and recovery options.
- **In Scope**: Server-side timeout (no HTTP response within window), the 10-attempt retry loop with printed backoff delays, the unresponsive-session outcome on total failure.
- **Out of Scope**: Network reset errors before any HTTP response (→ future `error/` instance for ECONNRESET); API overloaded errors (→ `error/005_api_overloaded.md`).

### Abstract

When the Anthropic API does not return a response within its server-side timeout window, Claude Code prints a retry notice and re-issues the request using exponential backoff. The terminal shows:

```
⎿  API Error (Request timed out.) · Retrying in X seconds… (attempt Y/10)
```

Where `X` follows the backoff schedule and `Y` increments from 1 to 10. After attempt 10 fails Claude Code stops retrying and the session becomes unresponsive — it neither prints a final error nor exits cleanly.

**Observed backoff schedule (seconds):** 1, 1, 2, 4, 5, 9, 17, 36, 40, … (approximately exponential with jitter).

### Trigger Conditions

- **Slow large requests**: Requests with a very high `max_tokens` budget or long context take longer to generate; the server-side generation window may expire before the full response is streamed.
- **Degraded API infrastructure**: Anthropic infrastructure latency spikes under load cause generation to stall mid-stream, triggering a timeout on the connection.
- **Network path instability**: An intermediate hop drops packets after the HTTP connection is established but before the response completes — the server sees a hung connection and times out.
- **Streaming stall**: In streaming mode, a gap in token delivery exceeds the client's read timeout; the connection is considered timed out even if the server is still generating.

### Recovery

**During automatic retry (attempts 1–9):**
- No action needed. Claude Code is retrying automatically; leave the session running.
- Avoid sending additional input or Ctrl-C during the retry loop.

**If retries are exhausting (approaching attempt 10):**
1. Check network stability — a flaky VPN or Wi-Fi drop is the most common cause.
2. Reduce `--max-tokens` (or the `clr` equivalent) to shorten generation time and reduce timeout exposure.
3. Break the prompt into smaller subtasks so each request completes faster.

**After attempt 10 (session unresponsive):**
1. Send Ctrl-C to terminate the hung session.
2. Start a new session: `clr --new-session "..."` or `clr` (continues from last clean checkpoint).
3. If the task was partially complete, review the last committed file changes before re-issuing the prompt.

**Systemic / recurring timeouts:**
- Check `status.anthropic.com` for API incidents.
- Avoid peak hours if timeouts correlate with time of day.
- Consider setting `ANTHROPIC_BASE_URL` to an alternate regional endpoint if available.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| error | [error/005_api_overloaded.md](005_api_overloaded.md) | Overloaded error (HTTP 529) — server busy, distinct from timeout |
| error | [error/003_context_limit_reached.md](003_context_limit_reached.md) | Context-limit error — large context increases timeout risk |
| source | `../../module/claude_runner/src/main.rs` | `--max-tokens` flag and print-mode execution that captures exit code |
