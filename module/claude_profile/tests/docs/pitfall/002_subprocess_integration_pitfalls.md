# Pitfall Tests: Subprocess Integration Pitfalls

Test cases verifying that each guard documented in `docs/pitfall/002_subprocess_integration_pitfalls.md`
is in place and prevents the described subprocess integration failure mode.

**Source:** [docs/pitfall/002_subprocess_integration_pitfalls.md](../../../docs/pitfall/002_subprocess_integration_pitfalls.md)
**Case prefix:** `PP-` (Pitfall Protection)

### Pitfall Guard Index

| ID | Pitfall | Bug | Guard Verified By |
|----|---------|-----|-------------------|
| PP-1 | `["--print", "."]` is the ONLY valid credential-refresh invocation | BUG-169 | `test_apply_refresh_lifecycle_l10_trace_run_isolated_invoked_no_panic` |
| PP-2 | Haiku cannot activate the 7d-Sonnet session window | BUG-289 | `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call`, `it_imodel_auto_selects_sonnet_when_son_idle` |
| PP-3 | Touch subprocess must use Sonnet (or Sonnet-family) to open all quota windows | BUG-289 | `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call`, `it_imodel_auto_selects_sonnet_when_son_idle` |
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

- **Given:** An account with `seven_day_sonnet.resets_at = None` (`son_idle=true`), 5h and 7d
  both running.
- **When:** `touch_skip_reason(&aq, store, false)` — the pure decision oracle `apply_touch`
  consults first — is evaluated twice on fresh fixtures with identical `son_idle=true` state
  (no subprocess is launched or re-queried in this test). Separately, `resolve_model(&aq,
  SubprocessModel::Auto)` is evaluated on the same `son_idle=true` state.
- **Then:** Both `touch_skip_reason` calls return `None` (no guard skips) — the trigger fires
  on every call given identical `son_idle=true` state, proving the pre-fix infinite-loop
  precondition: a Haiku touch cannot set the Sonnet window timer, so `resets_at` would stay
  `None` forever and every subsequent `.usage` call would re-fire the trigger.
  `resolve_model(Auto)` returns `"claude-sonnet-5"`, not Haiku — the actual BUG-289 fix that
  breaks the loop by activating the window on the first call. Fix BUG-289.
- **Rule:** When the goal is to activate ALL quota windows simultaneously, the touch
  subprocess MUST use a Sonnet-family model. `resolve_model(Auto)` selects Sonnet when
  `seven_day_sonnet.resets_at = None` for exactly this reason.
- **Source fn:** `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` in
  `tests/usage/touch_tests_b.rs` (proves the trigger persists across calls — the loop
  precondition); `it_imodel_auto_selects_sonnet_when_son_idle` in
  `tests/usage/subprocess_tests.rs` (proves `resolve_model(Auto)` actually selects Sonnet —
  the fix itself)
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
