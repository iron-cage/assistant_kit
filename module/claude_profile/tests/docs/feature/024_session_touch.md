# Test: Feature 024 — Session Touch via Isolated Subprocess

### Scope

- **Purpose**: Test cases for session touch via isolated subprocess.
- **Source**: `docs/feature/024_session_touch.md`
- **Covers**: AC-01 through AC-19

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
| FT-09 | `trace=true` emits timestamped diagnostic lines for touch subprocess lifecycle | AC-09 | Integration (lim_it) |
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
| FT-22 | Owned account with `is_occupied_elsewhere == true` skipped by `apply_touch`; trace line emitted when `trace::1` | AC-17 | G4 Occupancy Guard |
| FT-23 | `apply_touch` re-fetch block writes cache and clears cached metadata (BUG-309 MRE) | AC-18 | BUG-309 MRE |
| FT-24 | 5h-exhaustion skip guard fires only at full exhaustion (`five_hour_left <= 0.0%`); partial exhaustion (11%) fires touch, not skipped | AC-19 | TSK-418 MRE |

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
| FT-22 | Owned account occupied elsewhere skipped by apply_touch; trace line emitted (G4 occupancy guard) | AC-17 | G4 Occupancy Guard |
| FT-23 | apply_touch re-fetch writes cache + clears cached flag (BUG-309 structural) | AC-18 | BUG-309 MRE |
| FT-24 | h-exhausted guard threshold is 0.0% (full exhaustion), not 15%; 11%-remaining account fires touch | AC-19 | TSK-418 MRE |

**Total:** 24 FT cases

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
- **Then:** Stderr timestamped diagnostic lines show refresh lifecycle before any touch lifecycle. Touch runs on post-refresh results — if refresh started a session (making `resets_at` present), that account is skipped by touch (already activated by refresh). If the post-refresh result still has `resets_at` absent, touch fires.
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
- **Source fn:** `it_apply_touch_trigger_fires_resets_at_none` (in `tests/usage/touch_tests.rs`)
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

### FT-09: `trace=true` emits timestamped diagnostic lines for touch subprocess lifecycle

- **Given:** One account with valid quota data and `resets_at` absent (idle — qualifies for touch); `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains timestamped diagnostic lines showing the touch subprocess lifecycle steps (`read credentials`, `run_isolated` with elapsed time, `write credentials`, `save`). Lines include account name and subprocess status.
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
- **Source fn:** `it_apply_touch_trigger_skips_resets_at_some` (in `tests/usage/touch_tests.rs`)
- **Source:** [feature/024_session_touch.md AC-02, AC-12](../../../docs/feature/024_session_touch.md)

---

### FT-14: Skip trace line emitted for each account not qualifying for touch

- **Given:** Two accounts: one with `resets_at` present (already active — skip reason: "already active"); one with errored quota (no valid data — skip reason: error account). `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains timestamped `... · touch  <name>  skipped (reason: ...)` lines for each non-qualifying account. The `resets_at` present case and the errored case each produce a diagnostically distinct skip-reason line. No subprocess spawned for either account.
- **Exit:** 0
- **Source fn:** `it141_trace_skip_lines_emitted_for_non_qualifying_accounts` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09, AC-12](../../../docs/feature/024_session_touch.md)

---

### FT-15: No switch_account called in apply_touch; `_active` unchanged confirms no restore occurred

- **Given:** `apply_touch` is called with `trace=true`; one qualifying account with `resets_at` absent; the `_active` marker is NOT present in the credential store before the call.
- **When:** `apply_touch` processes the qualifying account and completes.
- **Then:** The credential-store `_active` file does NOT exist after the call — no `switch_account` write occurred during touch cycling, confirming `update_marker=false` suppresses all `_active` writes.
- **Source fn:** `test_apply_touch_mre_bug208_restore_trace_emitted` (in `tests/usage/touch_tests.rs`)
- **Note:** BUG-211 MRE — function name preserved from BUG-208 era; now asserts absence of restore side-effects rather than presence of restore trace. Symmetric to FT-17 in `017_token_refresh` test spec.
- **Source:** [feature/024_session_touch.md AC-13](../../../docs/feature/024_session_touch.md)

---

### FT-16: 7d-exhausted account (7d Left = 0%, 5h idle) is NOT touched — 7d guard fires

- **Given:** `apply_touch` is called with one account whose `AccountQuota` has: `result = Ok(data)` with `seven_day_left = 0.0` (weekly quota fully exhausted), `five_hour_left = 100.0` (5h budget non-zero), and `five_hour.resets_at = None` (idle — no active 5h session). The 7d guard is present in `apply_touch`.
- **When:** `apply_touch` processes this account.
- **Then:** `touch_skip_reason(&aq, credential_store, false)` returns `Some("skipped (reason: 7d-exhausted)")` — the same reason string `apply_touch()`'s trace line would emit. `apply_touch` only reaches subprocess-spawning logic when this oracle returns `None`, so the `Some(..)` result structurally proves `run_isolated` is never called.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug214_apply_touch_skips_7d_exhausted_account` (in `tests/usage/touch_tests.rs`)
- **Note:** BUG-214 MRE. Mirrors FT-13 (which tests the all-timers-running guard) and the h-exhausted guard test (BUG-178). The account passes the error guard and the 5h-idle guard but must be caught by the new 7d guard. Converted from gag-based stderr capture to a direct `touch_skip_reason()` oracle call.
- **Source:** [feature/024_session_touch.md AC-14](../../../docs/feature/024_session_touch.md)

---

### FT-17: 5h timer running but 7d or 7d-Sonnet timer absent → touch fires (3-timer trigger)

- **Given:** `apply_touch` is called with one account whose `AccountQuota` has: `result = Ok(data)` with `five_hour.resets_at = Some(...)` (5h session active), `five_hour_left > 15.0` (not h-exhausted), `seven_day_left > 0.0` (not 7d-exhausted), and `seven_day.resets_at = None` (7d window timer absent — period exists but no active countdown). The 3-timer trigger is implemented.
- **When:** `apply_touch` processes this account.
- **Then:** `touch_skip_reason(&aq, credential_store, false)` returns `None` — the trigger would fire (`refresh_account_token` would be called). The account is NOT skipped as "already active" because not all three timers are running.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug215_apply_touch_fires_when_7d_timer_absent` (in `tests/usage/touch_tests.rs`)
- **Note:** BUG-215 MRE. The scenario where the 5h session is active but the 7d window was just reset (no `resets_at`) was incorrectly skipped as "already active" before the 3-timer fix. This test verifies the fix: the oracle returns `None` whenever any timer is absent, not only when the 5h timer is absent. Converted from gag-based stderr capture to a direct `touch_skip_reason()` oracle call.
- **Source:** [feature/024_session_touch.md AC-15](../../../docs/feature/024_session_touch.md)

---

### FT-18: After `apply_post_switch_touch` quota re-fetch, `apply_touch` skips account as already-active (BUG-288 cross-feature guard)

- **Given:** `apply_post_switch_touch` has executed its post-subprocess `write_quota_cache` call (Fix(BUG-288)). The cache for the target account now records `five_hour.resets_at = Some(...)` — the subprocess activated the 5h session window and the re-fetch persisted it to disk. This is the state verified by Feature 027 FT-21 (`mre_bug288_post_switch_touch_refetch_updates_quota`, structural block).
- **When:** `apply_touch` is subsequently called for that account and evaluates the trigger condition.
- **Then:** `all_running = true` (5h `resets_at` is Some — active session); `apply_touch` skips the account with reason `already active` and does NOT spawn a second subprocess. Behavior is identical to FT-13 (`it_apply_touch_trigger_skips_resets_at_some`).
- **Exit:** N/A (structural cross-reference — no separate test code; covered compositionally by FT-21 × FT-13)
- **Source fn:** `mre_bug288_post_switch_touch_refetch_updates_quota` (in `tests/usage/api_tests_b.rs`, structural block — asserts `write_quota_cache` is called in `apply_post_switch_touch` fn body) + `it_apply_touch_trigger_skips_resets_at_some` (in `tests/usage/touch_tests.rs` — asserts `apply_touch` skips when `resets_at = Some`).
- **Note:** BUG-288 cross-feature interaction. AC-03 re-fetch requirement applies to ALL touch paths — both `apply_touch` (this feature) and `apply_post_switch_touch` (Feature 027 AC-21). Before the fix, `apply_post_switch_touch` omitted the re-fetch: the on-disk cache still showed `resets_at = None`, so `apply_touch` saw a qualifying idle account and spawned a redundant second subprocess. After Fix A: `apply_post_switch_touch` writes updated quota (including `resets_at = Some`) to the cache, and `apply_touch` skips the account. End-to-end live coverage is provided by Feature 027 FT-01 (live integration test, marked `lim_it`).
- **Source:** [feature/024_session_touch.md AC-03](../../../docs/feature/024_session_touch.md)

---

### FT-19: Account with `touch_idle=false` in quota cache is skipped before `all_running` check (BUG-288 Fix B MRE)

- **Given:** `apply_touch` is called with one account whose quota cache entry has `touch_idle = Some(false)` — written by `apply_post_switch_touch` at `api.rs:330-332` after its subprocess activated the account. The account's quota data shows `five_hour.resets_at = None` (would qualify for touch by timer state alone — the API has not yet propagated the new session's `resets_at` to the quota endpoint).
- **When:** `apply_touch` evaluates skip conditions for that account (with `trace=true`).
- **Then:** `touch_skip_reason` reads `touch_idle = Some(false)` from the quota cache and returns `Some("skipped (reason: touch_idle=false)")` before the `all_running` check — the same reason string `apply_touch()`'s trace line would emit.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug288_apply_touch_skips_touch_idle_false` (in `tests/usage/touch_tests.rs`) — behavioral: writes `touch_idle=Some(false)` to quota cache for an idle account (`resets_at=None`), asserts `touch_skip_reason(&aq, store.path(), false)` returns `Some("skipped (reason: touch_idle=false)")` (guard fires before `all_running` check).
- **Note:** BUG-288 Fix B MRE (TSK-291). Before Fix B, `api.rs:330-332` wrote `touch_idle=false` with zero read sites — dead write. Fix B adds the read site at `touch.rs:59-66`. Defense-in-depth for API propagation lag: when the Anthropic API hasn't reflected the new session's `resets_at` at the quota endpoint by the time `.usage` runs (even after Fix A's re-fetch), the local `touch_idle=false` flag prevents a redundant subprocess. Converted from gag-based stderr capture to a direct `touch_skip_reason()` oracle call.
- **Source:** [feature/024_session_touch.md AC-16](../../../docs/feature/024_session_touch.md)

---

### FT-20: `son_running=false` (5h+7d running, Sonnet 7d absent) + `imodel::auto` (Haiku) → touch fires on both calls; Sonnet window unchanged; re-fires on second call (BUG-289 MRE)

- **Given (two-call design, two separate stores):** Two `TempDir` stores (`store_a`, `store_b`), each with its own fresh `mk_aq_with_son_idle()` account overridden with `seven_day=Some(PeriodUsage{utilization:0.0, resets_at:Some(...)})` — explicit `d7_running=true` (not the `map_or(true)` default path). Both accounts have `five_h_running=true`, `d7_running=true`, `son_running=false` (Sonnet 7d field present, `resets_at=None`).
- **Call A (store_a):** `touch_skip_reason(&aq_a, store_a.path(), false)`.
- **Then A:** Returns `None` — touch would fire (no guard skips). Ensures the `son_running=false` trigger is non-vacuous: this call proves no guard fires for the given account state.
- **Call B (store_b):** `touch_skip_reason(&aq_b, store_b.path(), false)` — fresh store and fresh `AccountQuota`, identical state to Call A.
- **Then B:** Returns `None` AGAIN for the identical account state. This proves the infinite loop: nothing in `touch_skip_reason`'s inputs changes between calls unless a live Sonnet-family API call activates the 7d-Sonnet window (which a Haiku subprocess cannot do) — so the trigger fires on every invocation.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug289_son_running_false_haiku_touch_fires_on_every_call` (in `tests/usage/touch_tests_b.rs`)
- **Note:** BUG-289 MRE (two-call non-vacuous pattern). Call A anchors non-vacuity (oracle returns `None` for `son_running=false`). Call B proves persistence (trigger not cleared — infinite loop). Uses separate stores to avoid state leakage. Converted from gag-based stderr capture (matching `"run_isolated: invoking"` in captured output) to direct two-call `touch_skip_reason()` oracle assertions — no credential store or subprocess needed, since the oracle is the pure decision function `apply_touch` calls first. Companion positive test: FT-22 in [tests/docs/feature/026_subprocess_model_effort.md](026_subprocess_model_effort.md) — `it_imodel_auto_selects_sonnet_when_son_idle` asserts `resolve_model` returns Sonnet when `son_idle=true` (Fix BUG-289, BUG-290, TSK-292). ✅ Passing.
- **Source:** [feature/024_session_touch.md AC-02, AC-15](../../../docs/feature/024_session_touch.md)

---

### FT-21: Non-owned account (`aq.is_owned == false`) skipped by `apply_touch`; trace line emitted when `trace::1`

- **Given:** `apply_touch` is called with one account (`alice`) whose `AccountQuota` has `is_owned = false` (set by G1 during fetch — `alice.json` contains `"owner": "other@remote"`). `trace::1` is enabled.
- **When:** `apply_touch` processes the account list containing `alice`.
- **Then:** `touch_skip_reason(&aq, credential_store, false)` returns `Some("skipped (reason: not owned)")` — the same reason string `apply_touch()`'s trace line would emit. The skip fires before any timer checks — `is_owned` is evaluated as the first guard after the error-account check.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `ft07_touch_skips_non_owned_with_trace` (in `tests/usage/touch_tests_b.rs`)
- **Note:** G4 ownership gate from Feature 036 AC-07 / Feature 024 AC-17. Shared with Feature 036 FT-07 — same test function, both specs reference it. Trace format matches other touch skip traces (`skipped (reason: not owned)` — see AC-12 for full list of skip reasons). Converted from gag-based stderr capture to a direct `touch_skip_reason()` oracle call.
- **Source:** [feature/024_session_touch.md AC-17](../../../docs/feature/024_session_touch.md)

---

### FT-22: Owned account with `is_occupied_elsewhere == true` skipped by `apply_touch`; trace line emitted when `trace::1`

- **Given:** `apply_touch` is called with one account (`bob`) whose `AccountQuota` has `is_owned = true` (this machine is the credential owner) AND `is_occupied_elsewhere = true` (another machine's `_active_*` marker file names this account). `trace::1` is enabled.
- **When:** `apply_touch` processes the account list containing `bob`.
- **Then:** `touch_skip_reason(&aq, credential_store, false)` returns `Some("skipped (reason: occupied elsewhere)")` — the same reason string `apply_touch()`'s trace line would emit. The skip fires immediately after the `is_owned` check — `is_occupied_elsewhere` is evaluated as a second gate before any timer checks.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `ft_touch_skips_occupied_elsewhere_with_trace` (in `tests/usage/touch_tests_b.rs`)
- **Note:** BUG-302 MRE. Complements FT-21 (non-owned account skip); this tests the occupancy case where ownership is confirmed but concurrent use by another machine prevents the touch subprocess. The two guards are independent: G4 (`!is_owned` → skip) and occupancy guard (`is_occupied_elsewhere` → skip). Reason string `"occupied elsewhere"` distinguishes this from the `"not owned"` reason of FT-21. Converted from gag-based stderr capture to a direct `touch_skip_reason()` oracle call.
- **Source:** [feature/024_session_touch.md AC-17](../../../docs/feature/024_session_touch.md)

---

### FT-23: `apply_touch` re-fetch block writes cache and clears cached metadata (BUG-309 MRE)

- **Given:** `apply_touch` source code at `src/usage/touch.rs`.
- **When:** The `if let Ok( new_data ) = claude_quota::fetch_oauth_usage(...)` re-fetch block is inspected.
- **Then:** The block contains all three required mutations: (1) `write_quota_cache(...)` is called, (2) `aq.cached = false` is set, (3) `aq.cache_age_secs = None` is set. Additionally, `write_quota_cache` appears BEFORE `aq.result = Ok( new_data )` — enforcing the borrow-before-move ordering constraint.
- **Exit:** N/A (structural source-inspection test — no exit code)
- **Source fn:** `mre_bug309_apply_touch_refetch_writes_cache_and_clears_cached_flag` (in `tests/usage/touch_tests.rs`)
- **Note:** BUG-309 MRE. Structural guard ensuring the three post-fetch mutations are never accidentally dropped by a refactor or merge conflict. Mirrors `mre_bug256_retry_ok_stale_cached_metadata` in `refresh_tests.rs` for the `apply_touch` code path.
- **Source:** [feature/024_session_touch.md AC-18](../../../docs/feature/024_session_touch.md)

---

### FT-24: 5h-exhaustion skip guard fires only at full exhaustion (`five_hour_left <= 0.0%`), not partial exhaustion

- **Given:** Two accounts, both idle (`five_hour.resets_at = None` — qualifies for touch by timer state). Account A: `five_hour.utilization = 89.0` (`five_hour_left = 11.0`, matching the real-world i16@wbox.pro scenario). Account B: `five_hour.utilization = 100.0` (`five_hour_left = 0.0`, fully exhausted).
- **When:** `touch_skip_reason(&aq, store.path(), false)` is evaluated for each account.
- **Then:** Account A (11% remaining) returns `None` — touch fires; a partially-exhausted account still benefits from a touch subprocess and is not skipped. Account B (0% remaining) returns `Some("skipped (reason: h-exhausted)")` — a fully-exhausted account gains nothing from a subprocess spawn and is skipped. `H_EXHAUSTED_THRESHOLD = 15.0` (the human-facing display/sort classification constant, TSK-190) is NOT referenced by this guard.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_tsk418_apply_touch_fires_at_partial_exhaustion_skips_at_full_exhaustion` (in `tests/usage/touch_tests.rs`)
- **Note:** TSK-418 corrective MRE. BUG-178/TSK-196 originally added the h-exhausted guard by reusing `H_EXHAUSTED_THRESHOLD = 15.0`, over-broadly skipping touch for any account ≤15% remaining rather than only fully-exhausted (0%) ones — never covered by a dedicated FT (see `touch_tests.rs` BUG-214 MRE doc comment: "the h-exhausted guard was added in isolation without extending the test surface"). This test closes that gap.
- **Source:** [feature/024_session_touch.md AC-19](../../../docs/feature/024_session_touch.md)
