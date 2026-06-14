# Test: Feature 024 — Session Touch via Isolated Subprocess

Feature behavioral requirement test cases for `docs/feature/024_session_touch.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/034_touch.md](../cli/param/34_touch.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::0` — no subprocess; inactive-timer accounts not activated when suppressed | AC-01 | Integration |
| FT-02 | `touch::1` invokes subprocess for accounts with any quota timer absent | AC-02 | Integration (lim_it) |
| FT-03 | After touch, table shows concrete `5h Reset` value (was `—`) | AC-03 | Integration (lim_it) |
| FT-04 | Errored accounts are never touched | AC-04 | Integration |
| FT-05 | When both `refresh::1` and `touch::1`, refresh runs first | AC-05 | Integration |
| FT-06 | apply_touch does not call switch_account; `_active` marker unchanged throughout cycle | AC-06 | BUG-211 MRE |
| FT-07 | Touch failure is non-aborting; row shows original data | AC-07 | Integration |
| FT-08 | `touch::` does not affect `format::json` output structure | AC-08 | Integration |
| FT-09 | `trace=true` emits `[trace]` lines for touch subprocess lifecycle | AC-09 | Integration (lim_it) |
| FT-10 | `touch::` appears in `.usage.help` with default `1` | AC-10 | Integration |
| FT-11 | Account with any timer absent IS touched (positive trigger) | AC-02 | Trigger |
| FT-12 | In `live::1` mode, touch fires each cycle when any timer absent | AC-11 | Live Mode |
| FT-13 | All three timers running → account NOT touched ("already active") | AC-02, AC-12 | Trigger Guard |
| FT-14 | Skip trace line emitted for each account not qualifying for touch | AC-09, AC-12 | Trace |
| FT-15 | no switch_account called in apply_touch; `_active` unchanged confirms no restore occurred | AC-13 | BUG-211 MRE |
| FT-16 | 7d-exhausted account (7d Left = 0%, 5h idle) is NOT touched — 7d guard fires | AC-14 | BUG-214 MRE |
| FT-17 | 5h timer running but 7d or 7d-Sonnet timer absent → touch fires (3-timer trigger) | AC-15 | BUG-215 MRE |
| FT-18 | After `apply_post_switch_touch` re-fetches quota (BUG-288 fix), `apply_touch` skips account as already-active; no second subprocess | AC-03 | BUG-288 Cross-Feature |
| FT-19 | Account with `touch_idle=false` in quota cache skipped before `all_running` check; no subprocess spawned (BUG-288 Fix B defense-in-depth) | AC-16 | BUG-288 Fix B MRE |
| FT-20 | `son_running=false` (5h+7d running, Sonnet 7d absent) + `imodel::auto` (Haiku) → touch fires both calls; Sonnet window unchanged; touch re-fires on second call (BUG-289 infinite loop MRE) | AC-02, AC-15 | BUG-289 MRE |
| FT-21 | Non-owned account (`aq.is_owned == false`) skipped by `apply_touch`; trace line emitted when `trace::1` | AC-17 | G4 Ownership Gate |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | touch::0 no subprocess | AC-01 | Default Behavior |
| FT-02 | touch::1 subprocess for accounts with resets_at absent (idle) | AC-02 | Trigger |
| FT-03 | After touch concrete 5h Reset shown (was —) | AC-03 | Re-fetch |
| FT-04 | Errored accounts not touched | AC-04 | Trigger Guard |
| FT-05 | refresh before touch ordering | AC-05 | Ordering |
| FT-06 | apply_touch does not call switch_account; _active unchanged | AC-06 | BUG-211 MRE |
| FT-07 | Touch failure non-aborting | AC-07 | Failure Handling |
| FT-08 | JSON unaffected by touch | AC-08 | JSON No-op |
| FT-09 | Trace shows touch lifecycle | AC-09 | Trace |
| FT-10 | touch:: in help with default 1 | AC-10 | Help Output |
| FT-11 | Valid account with resets_at absent IS touched | AC-02 | Trigger |
| FT-12 | live::1 touch fires each cycle when resets_at absent | AC-11 | Live Mode |
| FT-13 | All three timers running → account NOT touched (already active) | AC-02, AC-12 | Trigger Guard |
| FT-14 | Skip trace line emitted for each non-qualifying account | AC-09, AC-12 | Trace |
| FT-15 | no switch_account called in apply_touch; _active unchanged confirms no restore | AC-13 | BUG-211 MRE |
| FT-16 | 7d-exhausted account (7d Left = 0%, 5h idle) NOT touched — 7d guard fires | AC-14 | BUG-214 MRE |
| FT-17 | 5h timer running but 7d or 7d-Sonnet timer absent → touch fires (3-timer trigger) | AC-15 | BUG-215 MRE |
| FT-18 | apply_post_switch_touch quota re-fetch prevents double subprocess in apply_touch | AC-03 | BUG-288 Cross-Feature |
| FT-19 | account with touch_idle=false in cache skipped before all_running check — no subprocess (BUG-288 Fix B) | AC-16 | BUG-288 Fix B MRE |
| FT-20 | son_running=false + imodel::auto (Haiku) fires touch both calls; Sonnet window unchanged; re-fires on second call (BUG-289 MRE) | AC-02, AC-15 | BUG-289 MRE |
| FT-21 | Non-owned account skipped by apply_touch; trace line emitted (G4 ownership gate) | AC-17 | G4 Ownership Gate |

**Total:** 21 FT cases

---

### FT-01: `touch::0` — no subprocess spawned; idle accounts not activated when suppressed

- **Given:** One account with valid quota data and `five_hour.resets_at` absent (idle — no active 5h window; would be touched with `touch::1`).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0. No subprocess spawned for touch. Account row shows `5h Reset = —` unchanged (still idle).
- **Exit:** 0
- **Source fn:** `it109_lim_it_touch_0_no_subprocess_idle_account_unchanged` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../docs/feature/024_session_touch.md)

---

### FT-02: `touch::1` invokes `refresh_account_token()` for accounts with `resets_at` absent (idle)

- **Given:** One account with valid quota data (result=Ok) and `five_hour.resets_at` absent (idle — no active 5h session).
- **When:** `clp .usage touch::1`
- **Then:** `refresh_account_token()` is called for that account (observable via `trace::1` output showing subprocess lifecycle). Accounts with `resets_at` present (already active) are not touched.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** `it110_lim_it_touch_1_subprocess_spawned_for_idle_account` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../docs/feature/024_session_touch.md)

---

### FT-03: After successful touch, table shows concrete `5h Reset` value (was `—`)

- **Given:** One account with valid quota data and `five_hour.resets_at` absent (idle — `5h Reset = —`); touch subprocess succeeds and re-fetch returns `resets_at` set to ~5h from current time.
- **When:** `clp .usage touch::1`
- **Then:** Account row shows a `5h Reset` value of ~5h (e.g., "in 4h 59m") — transitioned from `—` (idle) to a concrete countdown (active).
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** `it111_lim_it_touch_1_5h_reset_changes_from_dash_to_time` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-03](../../../docs/feature/024_session_touch.md)

---

### FT-04: Errored accounts (quota fetch failed) are never touched

- **Given:** One account whose credential file has no `accessToken` (quota fetch returns Err — not a successful result with valid quota data).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned. Account row shows original error state unchanged. Touch trigger requires `result = Ok(...)`.
- **Exit:** 0
- **Source fn:** `it098_touch_1_errored_account_skipped`
- **Source:** [feature/024_session_touch.md AC-04](../../../docs/feature/024_session_touch.md)

---

### FT-05: When both `refresh::1` and `touch::1` active, refresh runs first

- **Given:** One account with expired token (quota fetch would fail with 401).
- **When:** `clp .usage refresh::1 touch::1 trace::1`
- **Then:** Stderr `[trace]` lines show refresh lifecycle before any touch lifecycle. Touch runs on post-refresh results — if refresh started a session (making `resets_at` present), that account is skipped by touch (already activated by refresh). If the post-refresh result still has `resets_at` absent, touch fires.
- **Exit:** 0
- **Live:** yes (lim_it — requires expired token + active 5h window)
- **Source fn:** `it112_structural_refresh_before_touch_ordering_in_source` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-05](../../../docs/feature/024_session_touch.md)

---

### FT-06: apply_touch does not call switch_account; `_active` marker unchanged throughout touch cycle

- **Given:** `apply_touch` is called with one qualifying account (valid quota, `resets_at` absent — idle); the `_active` marker is NOT present in the credential store before the call.
- **When:** `apply_touch` processes the qualifying account and completes.
- **Then:** The `_active` marker file does NOT exist after the call — no `switch_account` write occurred. `apply_touch` does not restore via `switch_account`; `refresh_account_token` passes `update_marker=false` to `save()` so the marker is never written.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `it_apply_touch_trigger_fires_resets_at_none` (in `src/usage/touch_tests.rs`)
- **Note:** BUG-211 regression guard — verifies snapshot+restore was not re-introduced in `apply_touch`. Symmetric to FT-13/BUG-211 guard in `017_token_refresh` test spec.
- **Source:** [feature/024_session_touch.md AC-06](../../../docs/feature/024_session_touch.md)

---

### FT-07: Touch subprocess failure is non-aborting; account row shows original data unchanged

- **Given:** One account with valid quota data and `resets_at` absent (idle — qualifies for touch); touch subprocess fails (returns non-zero exit or timeout).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Account row shows original quota data (not a hard error). Table still renders. Touch failure does not abort the command.
- **Exit:** 0
- **Source fn:** `it114_structural_touch_failure_non_aborting_guard_exists` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-07](../../../docs/feature/024_session_touch.md)

---

### FT-08: `touch::` does not affect `format::json` output structure

- **Given:** One account with valid quota data and `resets_at` absent (idle — qualifies for touch).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage touch::1 format::json`
- **Then-A and Then-B:** JSON arrays have identical schema. `touch::` does not add or remove fields. Touched accounts appear as normal data objects with their re-fetched quota values.
- **Exit:** 0 both cases
- **Source fn:** `it100_touch_json_format_unaffected`
- **Source:** [feature/024_session_touch.md AC-08](../../../docs/feature/024_session_touch.md)

---

### FT-09: `trace=true` emits `[trace]` lines for touch subprocess lifecycle

- **Given:** One account with valid quota data and `resets_at` absent (idle — qualifies for touch); `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains `[trace]` lines showing the touch subprocess lifecycle steps (`read credentials`, `run_isolated` with elapsed time, `write credentials`, `save`). Lines include account name and subprocess status.
- **Exit:** 0
- **Live:** yes (lim_it — requires idle account for subprocess to be triggered)
- **Source fn:** `it115_lim_it_trace_1_shows_touch_lifecycle` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09](../../../docs/feature/024_session_touch.md)

---

### FT-10: `touch::` appears in `.usage.help` output with default value `1`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains "touch". Output shows the default value as `1` (on).
- **Exit:** 0
- **Source fn:** `it101_usage_help_shows_touch_param`
- **Source:** [feature/024_session_touch.md AC-10](../../../docs/feature/024_session_touch.md)

---

### FT-11: Valid account with `resets_at` absent IS touched (positive trigger case)

- **Given:** One account with valid quota data (`result=Ok`) where `five_hour.resets_at` is absent (None) — meaning the account is idle with no active 5h session.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Subprocess is spawned for this account. After touch, the `5h Reset` column shows a concrete countdown value (~5h) — transitioned from `—` to active. The trigger condition `resets_at.is_none()` fires for this account.
- **Exit:** 0
- **Live:** yes (lim_it — requires idle account)
- **Source fn:** `it116_lim_it_account_with_resets_at_absent_is_touched` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../docs/feature/024_session_touch.md)

---

### FT-12: In `live::1` mode, touch fires each cycle for accounts with `resets_at` absent

- **Given:** One account that becomes idle between cycles (`resets_at` becomes absent after session expires); `live::1 touch::1`.
- **When:** `clp .usage live::1 touch::1` (observed over two cycles via trace output or structural assertion)
- **Then:** On each cycle where `resets_at` is absent (account became idle), the touch trigger fires (subprocess spawned) and a new 5h session is started. The trigger does not fire for accounts with `resets_at` present (still active).
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window + two live::1 cycles)
- **Source fn:** `it120_lim_it_ft12_touch_trigger_fires_per_idle_account_cycle` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-11](../../../docs/feature/024_session_touch.md)

---

### FT-13: All three quota timers running → account NOT touched ("already active")

- **Given:** One account with valid quota data where all three quota timers have active `resets_at` values: `five_hour.resets_at` present AND `seven_day.resets_at` present AND `seven_day_sonnet.resets_at` present (all windows active).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned for that account. The trigger guard skips accounts where all three timers are running — all quota windows are active. Account row shows original quota data unchanged. Trace emits `skipped (reason: already active)`.
- **Exit:** 0
- **Source fn:** `it_apply_touch_trigger_skips_resets_at_some` (in `src/usage/touch_tests.rs`)
- **Source:** [feature/024_session_touch.md AC-02, AC-12](../../../docs/feature/024_session_touch.md)

---

### FT-14: Skip trace line emitted for each account not qualifying for touch

- **Given:** Two accounts: one with `resets_at` present (already active — skip reason: "already active 5h window"); one with errored quota (no valid data — skip reason: Err). `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains `[trace] touch  <name>  skipped (reason: ...)` lines for each non-qualifying account. The `resets_at` present case and the errored case each produce a diagnostically distinct skip-reason line. No subprocess spawned for either account.
- **Exit:** 0
- **Source fn:** `it141_trace_skip_lines_emitted_for_non_qualifying_accounts` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09, AC-12](../../../docs/feature/024_session_touch.md)

---

### FT-15: No switch_account called in apply_touch; `_active` unchanged confirms no restore occurred

- **Given:** `apply_touch` is called with `trace=true`; one qualifying account with `resets_at` absent; the `_active` marker is NOT present in the credential store before the call.
- **When:** `apply_touch` processes the qualifying account and completes.
- **Then:** The credential-store `_active` file does NOT exist after the call — no `switch_account` write occurred during touch cycling, confirming `update_marker=false` suppresses all `_active` writes.
- **Source fn:** `test_apply_touch_mre_bug208_restore_trace_emitted` (in `src/usage/touch_tests.rs`)
- **Note:** BUG-211 MRE — function name preserved from BUG-208 era; now asserts absence of restore side-effects rather than presence of restore trace. Symmetric to FT-17 in `017_token_refresh` test spec.
- **Source:** [feature/024_session_touch.md AC-13](../../../docs/feature/024_session_touch.md)

---

### FT-16: 7d-exhausted account (7d Left = 0%, 5h idle) is NOT touched — 7d guard fires

- **Given:** `apply_touch` is called with one account whose `AccountQuota` has: `result = Ok(data)` with `seven_day_left = 0.0` (weekly quota fully exhausted), `five_hour_left = 100.0` (5h budget non-zero), and `five_hour.resets_at = None` (idle — no active 5h session). The 7d guard is present in `apply_touch`.
- **When:** `apply_touch` processes this account.
- **Then:** No subprocess is spawned (`refresh_account_token` is NOT called). The account is skipped with a trace line `[trace] touch  <name>  skipped (reason: 7d-exhausted)` (or equivalent). `apply_touch` returns after the skip without calling `run_isolated`.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug214_apply_touch_skips_7d_exhausted_account` (in `src/usage/touch_tests.rs`)
- **Note:** BUG-214 MRE. Mirrors FT-13 (which tests the all-timers-running guard) and the h-exhausted guard test (BUG-178). The account passes the error guard and the 5h-idle guard but must be caught by the new 7d guard.
- **Source:** [feature/024_session_touch.md AC-14](../../../docs/feature/024_session_touch.md)

---

### FT-17: 5h timer running but 7d or 7d-Sonnet timer absent → touch fires (3-timer trigger)

- **Given:** `apply_touch` is called with one account whose `AccountQuota` has: `result = Ok(data)` with `five_hour.resets_at = Some(...)` (5h session active), `five_hour_left > 15.0` (not h-exhausted), `seven_day_left > 0.0` (not 7d-exhausted), and `seven_day.resets_at = None` (7d window timer absent — period exists but no active countdown). The 3-timer trigger is implemented.
- **When:** `apply_touch` processes this account.
- **Then:** The trigger fires — `refresh_account_token` IS called. The account is NOT skipped as "already active" because not all three timers are running. No `[trace] touch  <name>  skipped` line emitted.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug215_apply_touch_fires_when_7d_timer_absent` (in `src/usage/touch_tests.rs`)
- **Note:** BUG-215 MRE. The scenario where the 5h session is active but the 7d window was just reset (no `resets_at`) was incorrectly skipped as "already active" before the 3-timer fix. This test verifies the fix: touch fires whenever any timer is absent, not only when the 5h timer is absent.
- **Source:** [feature/024_session_touch.md AC-15](../../../docs/feature/024_session_touch.md)

---

### FT-18: After `apply_post_switch_touch` quota re-fetch, `apply_touch` skips account as already-active (BUG-288 cross-feature guard)

- **Given:** `apply_post_switch_touch` has executed its post-subprocess `write_quota_cache` call (Fix(BUG-288)). The cache for the target account now records `five_hour.resets_at = Some(...)` — the subprocess activated the 5h session window and the re-fetch persisted it to disk. This is the state verified by Feature 027 FT-21 (`mre_bug288_post_switch_touch_refetch_updates_quota`, structural block).
- **When:** `apply_touch` is subsequently called for that account and evaluates the trigger condition.
- **Then:** `all_running = true` (5h `resets_at` is Some — active session); `apply_touch` skips the account with reason `already active` and does NOT spawn a second subprocess. Behavior is identical to FT-13 (`it_apply_touch_trigger_skips_resets_at_some`).
- **Exit:** N/A (structural cross-reference — no separate test code; covered compositionally by FT-21 × FT-13)
- **Source fn:** `mre_bug288_post_switch_touch_refetch_updates_quota` (in `src/usage/api_tests.rs`, structural block — asserts `write_quota_cache` is called in `apply_post_switch_touch` fn body) + `it_apply_touch_trigger_skips_resets_at_some` (in `src/usage/touch_tests.rs` — asserts `apply_touch` skips when `resets_at = Some`).
- **Note:** BUG-288 cross-feature interaction. AC-03 re-fetch requirement applies to ALL touch paths — both `apply_touch` (this feature) and `apply_post_switch_touch` (Feature 027 AC-21). Before the fix, `apply_post_switch_touch` omitted the re-fetch: the on-disk cache still showed `resets_at = None`, so `apply_touch` saw a qualifying idle account and spawned a redundant second subprocess. After Fix A: `apply_post_switch_touch` writes updated quota (including `resets_at = Some`) to the cache, and `apply_touch` skips the account. End-to-end live coverage is provided by Feature 027 FT-01 (live integration test, marked `lim_it`).
- **Source:** [feature/024_session_touch.md AC-03](../../../docs/feature/024_session_touch.md)

---

### FT-19: Account with `touch_idle=false` in quota cache is skipped before `all_running` check (BUG-288 Fix B MRE)

- **Given:** `apply_touch` is called with one account whose quota cache entry has `touch_idle = Some(false)` — written by `apply_post_switch_touch` at `api.rs:330-332` after its subprocess activated the account. The account's quota data shows `five_hour.resets_at = None` (would qualify for touch by timer state alone — the API has not yet propagated the new session's `resets_at` to the quota endpoint).
- **When:** `apply_touch` evaluates skip conditions for that account (with `trace=true`).
- **Then:** `apply_touch` reads `touch_idle = Some(false)` from the quota cache; skips the account before the `all_running` check without spawning a subprocess; emits `[trace] touch  <name>  skipped (reason: touch_idle=false)`.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug288_apply_touch_skips_touch_idle_false` (in `src/usage/touch_tests.rs`) — behavioral: writes `touch_idle=Some(false)` to quota cache for an idle account (`resets_at=None`), calls `apply_touch` with `trace=true`, asserts `[trace] touch  <name>  skipped (reason: touch_idle=false)` emitted (guard fires before `all_running` check; no subprocess spawned because `claude_paths=None`).
- **Note:** BUG-288 Fix B MRE (TSK-291). Before Fix B, `api.rs:330-332` wrote `touch_idle=false` with zero read sites — dead write. Fix B adds the read site at `touch.rs:59-66`. Defense-in-depth for API propagation lag: when the Anthropic API hasn't reflected the new session's `resets_at` at the quota endpoint by the time `.usage` runs (even after Fix A's re-fetch), the local `touch_idle=false` flag prevents a redundant subprocess.
- **Source:** [feature/024_session_touch.md AC-16](../../../docs/feature/024_session_touch.md)

---

### FT-20: `son_running=false` (5h+7d running, Sonnet 7d absent) + `imodel::auto` (Haiku) → touch fires on both calls; Sonnet window unchanged; re-fires on second call (BUG-289 MRE)

- **Given (two-call design, two separate stores):** Two `TempDir` stores (`store_a`, `store_b`). Each account has `result=Ok`, `five_hour.resets_at=Some(...)` (5h running — `five_h_running=true`), `seven_day.resets_at=Some(...)` (7d running — `d7_running=true`), `seven_day_sonnet=Some(PeriodUsage { resets_at: None, ... })` (Sonnet 7d field present, no active session — `son_running=false`), `five_hour_left=50.0` (not h-exhausted), `seven_day_left=100.0` (not 7d-exhausted). No `touch_idle=false` cache entry in either store. `claude_paths=None` (subprocess returns `None` credentials — simulates Haiku completing without populating `seven_day_sonnet.resets_at`).
- **Call A (store_a):** `apply_touch` with `imodel::auto`, `trace=true`. Capture stderr to `captured_a`.
- **Then A:** `captured_a` contains `"run_isolated: invoking"` — touch fired (not skipped). Ensures the `son_running=false` trigger is non-vacuous: this call proves the guard EXISTS and fires for the given account state.
- **Call B (store_b):** `apply_touch` with the same account state (fresh from scratch, same `seven_day_sonnet.resets_at=None`), `imodel::auto`, `trace=true`. Capture stderr to `captured_b`.
- **Then B:** `captured_b` contains `"run_isolated: invoking"` — touch fires AGAIN for the identical account state. Neither call contains `"skipped (reason: already active)"`. This proves the infinite loop: `son_running=false` is never cleared by the Haiku subprocess (`resets_at` remains `None`), so the trigger fires on every invocation.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` (in `src/usage/touch_tests.rs`)
- **Note:** BUG-289 MRE (two-call non-vacuous pattern). Call A anchors non-vacuity (guard fires for `son_running=false`). Call B proves persistence (trigger not cleared — infinite loop). Uses separate stores to avoid state leakage. `claude_paths=None` keeps the test hermetic (no live API calls). Companion positive test: FT-22 in [tests/docs/feature/26_subprocess_model_effort.md](26_subprocess_model_effort.md) — `it_imodel_auto_selects_sonnet_when_son_idle` asserts `resolve_model` returns Sonnet when `son_idle=true` (Fix BUG-289, BUG-290, TSK-292). ✅ Passing.
- **Source:** [feature/024_session_touch.md AC-02, AC-15](../../../docs/feature/024_session_touch.md)

---

### FT-21: Non-owned account (`aq.is_owned == false`) skipped by `apply_touch`; trace line emitted when `trace::1`

- **Given:** `apply_touch` is called with one account (`alice`) whose `AccountQuota` has `is_owned = false` (set by G1 during fetch — `alice.json` contains `"owner": "other@remote"`). `trace::1` is enabled.
- **When:** `apply_touch` processes the account list containing `alice`.
- **Then:** No subprocess is spawned for `alice` (`refresh_account_token` is NOT called). Stderr contains `[trace] touch  alice  skipped (reason: not owned)`. The skip fires before any timer checks — `is_owned` is evaluated as the first guard after the error-account check.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `ft07_touch_skips_non_owned_with_trace` (in `src/usage/touch.rs` `#[cfg(test)]` module)
- **Note:** G4 ownership gate from Feature 036 AC-07 / Feature 024 AC-17. Shared with Feature 036 FT-07 — same test function, both specs reference it. Trace format matches other touch skip traces (`skipped (reason: not owned)` — see AC-12 for full list of skip reasons).
- **Source:** [feature/024_session_touch.md AC-17](../../../docs/feature/024_session_touch.md)
