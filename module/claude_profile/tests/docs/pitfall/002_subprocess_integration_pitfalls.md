# Pitfall Tests: Subprocess Integration Pitfalls

Test cases verifying that each guard documented in `docs/pitfall/002_subprocess_integration_pitfalls.md`
is in place and prevents the described subprocess integration failure mode.

**Source:** [docs/pitfall/002_subprocess_integration_pitfalls.md](../../../docs/pitfall/002_subprocess_integration_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | `["--print", "."]` is the ONLY valid credential-refresh invocation | BUG-169 | `test_apply_refresh_lifecycle_l10_trace_run_isolated_invoked_no_panic` |
| PP-2 | Haiku cannot activate the 7d-Sonnet session window | BUG-289 | `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` |
| PP-3 | Touch subprocess must use Sonnet (or Sonnet-family) to open all quota windows | BUG-289 | `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` |
| PP-4 | Refresh scope guard: non-owned and occupied accounts are skipped | BUG-295, BUG-298 | `mre_bug295_apply_refresh_trace_reason_not_owned`, `mre_bug306_refresh_trace_reason_occupied_elsewhere` |

---

### PP-1: `["--print", "."]` is the ONLY valid credential-refresh invocation

- **Given:** A subprocess is about to be launched to refresh OAuth credentials.
- **When:** `run_isolated()` is called.
- **Then:** The subprocess receives exactly `["--print", "."]` as its arguments. No other
  argument combination is used. Fix BUG-169: empty args (`[]`) causes Claude to exit
  immediately without OAuth refresh, producing `credentials = None`.
- **Rule:** Always use `["--print", "."]` for credential refresh subprocess invocations.
  Never use `[]` (no args) or `["--print", ".", "--max-tokens", "1"]` (API rejection).
- **Source fn:** `test_apply_refresh_lifecycle_l10_trace_run_isolated_invoked_no_panic` in
  `tests/usage/refresh_tests_a.rs`
- **Source:** [pitfall/002_subprocess_integration_pitfalls.md §P1](../../../docs/pitfall/002_subprocess_integration_pitfalls.md)

---

### PP-2 / PP-3: Touch subprocess model must be Sonnet to activate all quota windows

- **Given:** An account with `seven_day_sonnet.resets_at = None` (Sonnet window idle). A
  Haiku subprocess touch is launched.
- **When:** The touch subprocess completes and quota is re-fetched.
- **Then:** `seven_day_sonnet.resets_at` remains `None` — Haiku cannot set the Sonnet window
  timer. The next `.usage` call detects the idle Sonnet window again and fires another touch
  — creating an infinite no-op loop. Fix BUG-289.
- **Rule:** When the goal is to activate ALL quota windows simultaneously, the touch
  subprocess MUST use a Sonnet-family model. `resolve_model(Auto)` selects Sonnet when
  `seven_day_sonnet.resets_at = None` for exactly this reason.
- **Source fn:** `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` in
  `tests/usage/touch_tests.rs`
- **Source:** [pitfall/002_subprocess_integration_pitfalls.md §P2-P3](../../../docs/pitfall/002_subprocess_integration_pitfalls.md)

---

### PP-4: Refresh skips non-owned and occupied-elsewhere accounts

- **Given:** Account A is not owned by this machine (`is_owned = false`). Account B is owned
  but actively in use on another machine (`is_occupied_elsewhere = true`).
- **When:** `apply_refresh` runs the batch credential refresh cycle.
- **Then:** Both Account A and Account B are skipped — their credentials are NOT refreshed.
  The trace reason emitted is `"not_owned"` or `"occupied_elsewhere"` respectively.
  Fix BUG-295 (not-owned trace), Fix BUG-298 (cached-expired-occupied trace).
- **Rule:** The refresh gate must check BOTH `!is_owned` AND `is_occupied_elsewhere`. Missing
  either half refreshes credentials that would immediately invalidate another machine's active
  session.
- **Source fn:** `mre_bug295_apply_refresh_trace_reason_not_owned`,
  `mre_bug306_refresh_trace_reason_occupied_elsewhere` in `tests/usage/refresh_tests_b.rs`
- **Source:** [pitfall/002_subprocess_integration_pitfalls.md §P4](../../../docs/pitfall/002_subprocess_integration_pitfalls.md)
