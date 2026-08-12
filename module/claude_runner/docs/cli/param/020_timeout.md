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

<!-- BUG-445 (task/claude_runner/bug/executing/445_clr_timeout_flag_no_gate_wait_protection_without_remaining_timeout_secs.md) —
     originally: isolated's --timeout did not bound its --max-sessions gate-wait phase.
     Fix Location #2 changed this: an EXPRESSED --timeout now defaults the gate budget.
     See 033_max_sessions.md and 036_timeout.md. -->

**Note — gate-wait defaulting (`isolated` only, BUG-445):** `isolated` contends for a
`--max-sessions` slot the same as `run`/`ask`. An *expressed* `--timeout N` (the flag, or an
applied `CLR_TIMEOUT`) also defaults the gate-wait budget to `N` seconds when
`CLR_REMAINING_TIMEOUT_SECS` is absent or unparseable; a parseable
`CLR_REMAINING_TIMEOUT_SECS` always wins. The built-in 30 s default is NOT expressed — a
plain `clr isolated ... "msg"` keeps the gate's own ceiling (`CLR_GATE_POLL_SECS` x
`CLR_GATE_MAX_ATTEMPTS`, default 30s x 1000 = ~8.3h), so default invocations queue patiently
instead of failing fast at 30 s, and only in that unexpressed case is total wall-clock
exposure gate-wait time PLUS `--timeout`. An explicit `--timeout 0` is an unlimited opt-out
(no execution bound, no gate budget). With `--trace` and NO expressed timeout, a stderr note
names the unbounded exposure when `CLR_REMAINING_TIMEOUT_SECS` is unusable. Verify:
`clr isolated --creds <f> --max-sessions 1 --timeout 5 x` under a held slot reports
`gate-deadline  engaged (5s from --timeout ...)` on stderr. See
[033_max_sessions.md](033_max_sessions.md) and [036_timeout.md](036_timeout.md) for
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
