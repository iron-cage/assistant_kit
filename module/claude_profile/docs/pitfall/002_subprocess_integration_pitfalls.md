# Pitfall: Subprocess Integration Pitfalls

### Pattern

`run_isolated()` has several non-obvious constraints: the exact args required, model capability asymmetry for session windows, and output capture behavior on timeout.

### Pitfall 1 — `["--print", "."]` is the ONLY valid credential-refresh invocation

Three arg combinations exist; two are broken:

| Args | Behavior | Status |
|------|----------|--------|
| `["--print", "."]` | Claude performs OAuth startup refresh; credentials updated | Correct |
| `[]` (no args) | Claude in non-TTY detects nothing to do; exits immediately without OAuth refresh; `credentials = None` always | BUG-169 |
| `["--print", ".", "--max-tokens", "1"]` | `--max-tokens 1` triggers API rejection before OAuth exchange; credentials never written | TSK-151 |

**Rule:** Always use `["--print", "."]` for credential refresh subprocess invocations.

### Pitfall 2 — Haiku cannot activate the 7d-Sonnet session window

The `seven_day_sonnet.resets_at` timer is set only by Sonnet-family API calls. A Haiku subprocess touch leaves `seven_day_sonnet.resets_at = None` after the re-fetch. The next `.usage` invocation sees an idle 7d-Sonnet window and fires another touch subprocess — infinite per-call no-op loop (BUG-289).

**Fix:** `resolve_model(Auto)` selects Sonnet whenever `son_idle = resets_at.is_none()`. A single Sonnet touch opens all idle dimensions simultaneously.

**Rule:** When the goal is to activate ALL quota windows, the touch subprocess MUST use Sonnet (or a Sonnet-family model).

### Pitfall 3 — Over-constrained model gate causes two-touch warm-up

BUG-290 introduced an over-constrained gate requiring `five_h_running AND d7_running AND son_idle` for Sonnet selection. A cold account (5h absent, 7d absent, 7d-Son absent) had `five_h_running=false` and `d7_running=false`, so the gate never fired — Haiku was selected. The Haiku touch started the 5h and 7d windows but not 7d-Sonnet. The second `.usage` call saw 5h+7d running, 7d-Sonnet still idle, but gate still didn't fire (7d_running now true). The trigger never fired at all.

**Fix:** Simplify gate to `son_idle` alone — remove `five_h_running` and `d7_running` conditions.

**Rule:** Sonnet selection must trigger on `son_idle` independently, regardless of other window states.

### Pitfall 4 — Timeout kills can discard output (fixed by BUG-243)

The old `Timeout` variant in `RunnerError` discarded all buffered subprocess output. Root cause: the thread/channel approach lost the `Child` handle on timeout, making `wait_with_output()` unreachable.

**Fix:** Use `spawn_piped()` + polling so the `Child` handle stays in scope through timeout; then `child.kill()` + `child.wait_with_output()` recovers partial data. The `TimeoutWithOutput { partial_stdout }` variant captures what was emitted.

**Rule:** Always use `spawn_piped()` + polling, never thread+channel, for subprocess management needing timeout recovery.

### Pitfall 5 — Approaching-expiry arm in `should_refresh` is permanently forbidden (BUG-323)

Adding a proactive arm to `should_refresh()` that triggers on valid-but-expiring tokens appears to solve the "token expires during polling gap" problem. It does not — it is a silent no-op.

**Why:** `run_isolated(["--print", "."])` uses the access token as-is when it is still valid. Claude Code only performs OAuth refresh when `expiresAt` is in the past (or forced to `"1"` via `refresh_account_token()`). With a genuinely valid AT, the subprocess exits without writing new credentials → returns `credentials=None`. The approaching-expiry arm calls `refresh_account_token()`, gets `credentials=None`, and silently does nothing — while paying 35 seconds of subprocess wait per account per poll.

**Spec prohibition:** `feature/017` line 8 explicitly marks "proactive expiry detection before any API call" as **Out of Scope**.

**History:** This arm was proposed during BUG-323 investigation (2026-06-29). The subprocess constraint was not checked before proposing. SR-11 in `refresh_predicate.rs` exists specifically to block future re-proposals from passing tests without resolving the subprocess constraint first.

**Rule:** Never add a `should_refresh` arm for approaching-expiry without first confirming that `run_isolated` supports proactive AT rotation with a valid token. Until that capability exists, the arm cannot be made functional.

### Cross-References

| File | Relationship |
|------|-------------|
| [subprocess/001](../subprocess/001_run_isolated_contract.md) | `run_isolated()` API and required args |
| [subprocess/003](../subprocess/003_token_refresh_invocation.md) | Token refresh invocation; Architectural Constraint section |
| [algorithm/001](../algorithm/001_touch_model_selection.md) | Model selection algorithm |
| [state_machine/003](../state_machine/003_session_window_lifecycle.md) | Session window model constraints |
| [feature/017](../feature/017_token_refresh.md) | Out of Scope: proactive expiry detection (line 8) |
| [invariant/008](../invariant/008_single_token_refresh_entry.md) | `expiresAt=1` mechanism — why valid-AT subprocess call is a no-op |
