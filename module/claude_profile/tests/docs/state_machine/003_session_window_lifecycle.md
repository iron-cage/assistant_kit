# State Machine 003: Session Window Lifecycle

AC test cases for `docs/state_machine/003_session_window_lifecycle.md`. Tests the
`idle/active/exhausted` state transitions for the 5h, 7d, and 7d-Sonnet quota windows,
including model-capability constraints and touch trigger conditions.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | `idle` (resets_at=None) — touch trigger fires for 7d absent window | Transition | ✅ |
| AC-2 | `idle` (resets_at=None) — touch trigger confirmed by predicate | Transition | ✅ |
| AC-3 | `active` (resets_at=Some) — all windows running, touch skips | Transition | ✅ |
| AC-4 | `exhausted` (7d) — account skipped from touch | Boundary | ✅ |
| AC-5 | Model-capability constraint — Haiku cannot activate 7d-Sonnet window (BUG-289) | Invariant | ✅ |
| AC-6 | `exhausted` (5h) — account skipped from touch, full exhaustion only (TSK-418) | Boundary | ✅ |

---

### AC-1: `idle` — touch fires when 7d window absent (BUG-215 regression)

- **Given:** An account with `seven_day.resets_at = None` (7d window not started). 5h window
  is also idle. Account is owned and not occupied elsewhere.
- **When:** `apply_touch()` runs.
- **Then:** Touch subprocess fires. The 7d-absent window is the trigger condition (BUG-215
  regression: 7d timer absence must trigger touch, not just 5h absence).
- **Source fn:** `test_mre_bug215_apply_touch_fires_when_7d_timer_absent` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/003_session_window_lifecycle.md](../../../docs/state_machine/003_session_window_lifecycle.md)

---

### AC-2: `idle` — touch trigger predicate: resets_at=None → touch fires

- **Given:** An account where at least one quota window has `resets_at = None`.
- **When:** `apply_touch()` evaluates the idle-window predicate.
- **Then:** Touch fires (subprocess called). The predicate `resets_at = None` is the canonical
  indicator of `idle` state — the window has not been started and requires a touch subprocess
  to transition to `active`.
- **Source fn:** `it_apply_touch_trigger_fires_resets_at_none` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/003_session_window_lifecycle.md](../../../docs/state_machine/003_session_window_lifecycle.md)

---

### AC-3: `active` — all windows running, touch trigger skips

- **Given:** An account where all quota windows have `resets_at = Some(timestamp)` (all windows
  in `active` state, none idle).
- **When:** `apply_touch()` evaluates the idle-window predicate.
- **Then:** Touch does NOT fire. The `reason: already active` skip-reason is emitted in trace mode.
  All windows are `active` — no window needs to be opened.
- **Source fn:** `it_apply_touch_trigger_skips_resets_at_some` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/003_session_window_lifecycle.md](../../../docs/state_machine/003_session_window_lifecycle.md)

---

### AC-4: `exhausted` (7d) — account skipped from touch

- **Given:** An account with `seven_day_left ≤ 0.0%` (7d window exhausted). Even if the 5h
  window is idle, the account has negligible remaining capacity.
- **When:** `apply_touch()` evaluates the weekly-exhausted gate.
- **Then:** Account is skipped. Touch does NOT fire. `reason: 7d-exhausted` is emitted.
  Opening a new session window on an exhausted account would consume remaining headroom
  without benefit.
- **Source fn:** `test_mre_bug214_apply_touch_skips_7d_exhausted_account` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/003_session_window_lifecycle.md](../../../docs/state_machine/003_session_window_lifecycle.md)

---

### AC-5: Model-capability constraint — Haiku cannot activate 7d-Sonnet window (BUG-289)

- **Given:** An account with `seven_day_sonnet.resets_at = None` (Sonnet window `idle`).
  A touch subprocess is launched using Haiku model (`IsolatedModel::Haiku`).
- **When:** Touch completes and quota is re-fetched.
- **Then:** `seven_day_sonnet.resets_at` remains `None` — Haiku API calls cannot open the
  7d-Sonnet window. The window stays `idle`. The next `.usage` call detects the idle Sonnet
  window again and fires another Haiku touch — creating an infinite no-op loop (BUG-289).
  Fix: `resolve_model(Auto)` selects Sonnet when `son_idle=true`. Sonnet-family API calls
  are the ONLY mechanism that can transition the 7d-Sonnet window from `idle` to `active`.
- **Source fn:** `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/003_session_window_lifecycle.md](../../../docs/state_machine/003_session_window_lifecycle.md)

---

### AC-6: `exhausted` (5h) — account skipped from touch, full exhaustion only (TSK-418)

- **Given:** An account with `five_hour_left ≤ 0.0%` (5h window fully exhausted — utilization
  100%). Even if the 5h `resets_at` is absent (idle) or another window is idle, full 5h
  exhaustion overrides.
- **When:** `apply_touch()` evaluates the h-exhausted gate.
- **Then:** Account is skipped. Touch does NOT fire. `reason: h-exhausted` is emitted. The
  threshold is full/literal exhaustion (`≤ 0.0%`) — NOT the 15%-remaining
  `H_EXHAUSTED_THRESHOLD` used for display/sort classification (TSK-190). A partially-exhausted
  account (e.g. 11% remaining) is NOT skipped and still qualifies for touch; TSK-418 corrects
  BUG-178/TSK-196's original over-broad reuse of the display threshold.
- **Source fn:** `test_tsk418_apply_touch_fires_at_partial_exhaustion_skips_at_full_exhaustion` in
  `tests/usage/touch_tests.rs`
- **Source:** [state_machine/003_session_window_lifecycle.md](../../../docs/state_machine/003_session_window_lifecycle.md)
