# CLI Parameter: --timeout

Maximum seconds to wait for the subprocess to complete.
If the subprocess exceeds this limit and did not refresh credentials,
`clr` exits with code 2. If credentials were refreshed during the
timeout window, the updated file is written back and exit code is 0.

- **Type:** [`TimeoutSecs`](../type/09_timeout_secs.md)
- **Default:** 30 (`isolated`), 45 (`refresh`)
- **Command:** [`isolated`](../command/03_isolated.md), [`refresh`](../command/04_refresh.md)
- **JSON Key:** `"timeout"`

```sh
clr isolated --creds creds.json --timeout 60 "Explain closures"
clr isolated --creds creds.json --timeout 5 -- --version   # fast check
clr refresh --creds creds.json --timeout 90                # slow network
clr isolated --creds creds.json --timeout 0 "test"         # unlimited (no watchdog)
```

**Note:** Default differs by command: `isolated` defaults to 30s (general task
execution), `refresh` defaults to 45s (allows headroom for slow networks and
API rate limiting during OAuth token exchange).

**Note:** On timeout, any partial stdout accumulated by the subprocess before
the timeout fires is preserved in the error output, so diagnostic context is
not discarded.

**Note:** A timeout of `0` disables the watchdog entirely (unlimited runtime),
matching `run`/`ask` semantics. The subprocess runs until it exits naturally
with no deadline enforced.

<!-- BUG-445 (task/claude_runner/bug/unverified/445_clr_timeout_flag_no_gate_wait_protection_without_remaining_timeout_secs.md) —
     isolated's --timeout does not bound its --max-sessions gate-wait phase, by design; this
     doc did not cross-reference that boundary despite 036_timeout.md documenting the parallel
     run/ask case. See 033_max_sessions.md and 036_timeout.md. -->

**Note — does not bound gate-wait (`isolated` only):** For `isolated`, `--timeout` governs
only the subprocess-execution phase, AFTER an invocation has already been admitted past the
`--max-sessions` concurrency gate — `isolated` contends for a slot the same as `run`/`ask`.
If the invocation is still queued waiting for a session slot, `--timeout` has no effect:
gate-wait is bounded independently by `CLR_GATE_POLL_SECS` x `CLR_GATE_MAX_ATTEMPTS` (default
30s x 1000 = ~8.3h) unless the caller separately sets `CLR_REMAINING_TIMEOUT_SECS`. Total
wall-clock exposure for a queued-then-executing `isolated` invocation is gate-wait time PLUS
`--timeout` — no single flag bounds the sum. Passing `--trace` surfaces this when it applies
(a stderr note fires when `--timeout` is finite and `CLR_REMAINING_TIMEOUT_SECS` is unset).
See [033_max_sessions.md](033_max_sessions.md) and [036_timeout.md](036_timeout.md) for
gate-wait mechanics. **`refresh` is not affected** — `dispatch_refresh()` never calls the
concurrency gate, so `refresh`'s `--timeout` has no gate-wait interaction to document.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [`TimeoutSecs`](../type/09_timeout_secs.md) | Semantic | unsigned 64-bit integer | non-negative integer |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | `--creds`, `--trace` |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 2 | [`isolated`](../command/03_isolated.md) | 30 | 30s for general task execution |
| 3 | [`refresh`](../command/04_refresh.md) | 45 | 45s for slow OAuth token exchange |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |
