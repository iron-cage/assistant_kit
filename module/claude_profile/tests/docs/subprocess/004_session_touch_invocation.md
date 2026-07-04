# Subprocess 004: Session Touch Invocation

AC test cases for `docs/subprocess/004_session_touch_invocation.md`. Tests the
`apply_touch()` predicate — idle-window detection, G4 gate enforcement, cache guard,
post-touch cache write, refresh-before-touch ordering, and Sonnet model requirement.

### AC Case Index

| AC | Short Name | Category | Status |
|----|------------|----------|--------|
| AC-1 | Idle 7d window (`resets_at=None`) — touch fires | Predicate | ✅ |
| AC-2 | All windows active (`resets_at=Some`) — touch skips | Predicate | ✅ |
| AC-3 | Error account skipped before `touch_idle` guard | Predicate | ✅ |
| AC-4 | G4 gate — non-owned account skipped | Gate | ✅ |
| AC-5 | G4 gate — occupied-elsewhere account skipped | Gate | ✅ |
| AC-6 | `touch_idle=false` cache flag prevents re-touch | Cache | ✅ |
| AC-7 | Post-touch refetch writes quota cache (BUG-309) | Post-touch | ✅ |
| AC-8 | `refresh::` ordering — runs before `touch::` (structural) | Ordering | ✅ |
| AC-9 | Sonnet model required when `son_idle=true` (BUG-289 fix) | Model | ✅ |

---

### AC-1: Idle 7d window (`resets_at=None`) — touch fires

- **Given:** An account with `seven_day.resets_at = None` (7d window not started). Account is
  owned, not occupied elsewhere, not error, `five_hour_left > 15%`, `seven_day_left > 0%`.
- **When:** `apply_touch()` evaluates the idle-window predicate.
- **Then:** Touch fires. `refresh_account_token()` is called to submit a `["--print", "."]`
  subprocess, transitioning the 7d window from `idle` to `active`.
- **Source fn:** `test_mre_bug215_apply_touch_fires_when_7d_timer_absent` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-2: All windows active (`resets_at=Some`) — touch skips

- **Given:** An account where ALL three quota windows (5h, 7d, 7d-Sonnet) have
  `resets_at = Some(timestamp)` (all windows in `active` state).
- **When:** `apply_touch()` evaluates the idle-window predicate.
- **Then:** Touch does NOT fire. `reason: all_running` skip-reason emitted. No window needs
  to be opened — the touch invocation serves no purpose.
- **Source fn:** `it_apply_touch_trigger_skips_resets_at_some` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-3: Error account skipped before `touch_idle` guard

- **Given:** An account where `aq.result` is `Err(...)` (quota fetch failed or refresh token
  expired).
- **When:** `apply_touch()` evaluates the error-early-exit predicate.
- **Then:** Touch is skipped immediately, BEFORE the `touch_idle` cache guard is checked.
  `reason: Err` is emitted. Only accounts with `Ok(data)` quota results are eligible for touch.
- **Source fn:** `test_apply_touch_error_account_skips_before_touch_idle_guard` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-4: G4 gate — non-owned account skipped

- **Given:** An account with `is_owned=false` (owned by another machine). All other touch
  conditions are met (idle window, valid quota, etc.).
- **When:** `apply_touch()` evaluates the G4 ownership gate.
- **Then:** Touch is skipped. `reason: not owned` is emitted in trace mode. The G4 gate
  prevents touch from writing a new token to a foreign-owned account, which would invalidate
  the owning machine's active session.
- **Source fn:** `ft07_touch_skips_non_owned_with_trace` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-5: G4 gate — occupied-elsewhere account skipped

- **Given:** An account with `is_occupied_elsewhere=true` (currently active on another machine).
  All other touch conditions are met.
- **When:** `apply_touch()` evaluates the G4 gate.
- **Then:** Touch is skipped. `reason: occupied elsewhere` is emitted. The G4 gate blocks
  BOTH non-owned AND occupied-elsewhere accounts — either condition independently prevents touch.
  Fix BUG-302.
- **Source fn:** `ft_touch_skips_occupied_elsewhere_with_trace` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-6: `touch_idle=false` cache flag prevents re-touch

- **Given:** An account where the quota cache contains `touch_idle=false` (indicates a touch
  subprocess was already invoked in this cycle and the idle status was set to false).
- **When:** `apply_touch()` checks the cache guard.
- **Then:** Touch is skipped. `reason: touch_idle=false` is emitted. This defense-in-depth
  guard prevents API-lag re-triggers where a just-touched account still shows idle windows
  due to propagation delay from the quota server. Fix BUG-288.
- **Source fn:** `test_mre_bug288_apply_touch_skips_touch_idle_false` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-7: Post-touch refetch writes quota cache (BUG-309)

- **Given:** `apply_touch()` calls `refresh_account_token()` which returns (with or without
  new credentials).
- **When:** Post-touch actions run unconditionally.
- **Then:** Quota is re-fetched via `fetch_oauth_usage(new_token)`. The quota cache is written
  with `touch_idle=false` flag (via `write_quota_cache()`). Fix BUG-309: without this cache
  write, the next `.usage` call would re-detect idle windows and fire another redundant touch.
- **Source fn:** `mre_bug309_apply_touch_refetch_writes_cache_and_clears_cached_flag` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-8: `refresh::` ordering — runs before `touch::` (structural enforcement)

- **Given:** A `.usage` call with both `refresh::1` and `touch::1` active.
- **When:** The source code of the dispatch routine is inspected.
- **Then:** `apply_refresh()` appears before `apply_touch()` in the source. This ordering
  guarantees that auth errors are retried and fresh tokens are acquired before touch
  subprocesses are fired. A structural test verifies this ordering in the source code.
- **Source fn:** `it112_structural_refresh_before_touch_ordering_in_source` in
  `tests/cli/usage_touch_test.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)

---

### AC-9: Sonnet model required when `son_idle=true` (BUG-289 fix)

- **Given:** An account with `seven_day_sonnet.resets_at = None` (`son_idle=true`).
- **When:** `resolve_model(Auto)` is called to select the touch subprocess model.
- **Then:** Sonnet (not Haiku) is selected. A Haiku touch subprocess cannot open the 7d-Sonnet
  window — it remains idle forever, causing an infinite per-call touch loop (BUG-289). The
  `resolve_model(Auto)` logic selects Sonnet specifically when `son_idle=true` to ensure ALL
  quota windows can be activated in a single touch invocation.
- **Source fn:** `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` in
  `tests/usage/touch_tests.rs`
- **Source:** [subprocess/004_session_touch_invocation.md](../../../docs/subprocess/004_session_touch_invocation.md)
