# Test: Feature 024 — Session Touch via Isolated Subprocess

### Scope

- **Purpose**: Test cases for session touch via isolated subprocess.
- **Source**: `docs/feature/024_session_touch.md`
- **Covers**: AC-01 through AC-22

Feature behavioral requirement test cases for `docs/feature/024_session_touch.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/034_touch.md](../cli/param/34_touch.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/09_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::0` — no subprocess; inactive-timer accounts not activated when suppressed | AC-01 | Integration |
| FT-02 | `touch::1` invokes subprocess for accounts with any quota timer absent | AC-02 | Integration (lim_it) |
| FT-03 | After touch, table shows concrete `5h Reset` value (was `—`) | AC-03 | Integration (lim_it) |
| FT-04 | Errored accounts are never touched | AC-04 | Integration |
| FT-05 | When both `refresh::1` and `touch::1`, refresh runs first | AC-05 | Structural |
| FT-06 | apply_touch does not call switch_account; `_active` marker unchanged throughout cycle | AC-06 | BUG-211 MRE |
| FT-07 | Touch failure is non-aborting; row shows original data | AC-07 | Structural |
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
| FT-25 | Touched row with absent `resets_at` renders `~in Xh Ym`, never the literal `(touched)` | AC-20 | BUG-551 MRE |
| FT-26 | Projected window end floors the touch instant to a 10-minute boundary before adding 5h | AC-20 | BUG-551 Arithmetic |
| FT-27 | Touch refuted by a later window-less fetch yields no corroboration (`None`) | AC-21 | BUG-552 MRE |
| FT-28 | Display and re-touch skip guard both call `corroborated_touch_at` (single predicate) | AC-21 | BUG-552 Structural |
| FT-29 | `apply_post_switch_touch` gates `mark_touched` on its own refresh result | AC-22 | BUG-552 MRE |
| — | AC-20's projection reaches `format::tsv` and `format::json`, not only the text table and `get::5h_reset` | AC-20 | Cross-feature: Feature 033 FT-19 (BUG-553) |

**Total:** 24 FT cases

---

### FT-01: `touch::0` — no subprocess spawned; idle accounts not activated when suppressed

- **Given:** One account with valid quota data and `five_hour.resets_at` absent (idle — no active 5h window; would be touched with `touch::1`).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0. No subprocess spawned for touch. Account row shows `5h Reset = —` unchanged (still idle).
- **Exit:** 0
- **Source fn:** `it109_lim_it_touch_0_no_subprocess_idle_account_unchanged` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../docs/feature/024_session_touch.md)

---

### FT-02: `touch::1` invokes `refresh_account_token()` for accounts with `resets_at` absent (idle)

- **Given:** One account with valid quota data (result=Ok) and `five_hour.resets_at` absent (idle — no active 5h session).
- **When:** `clp .usage touch::1`
- **Then:** `refresh_account_token()` is called for that account (observable via `trace::1` output showing subprocess lifecycle). Accounts with `resets_at` present (already active) are not touched.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** `it110_lim_it_touch_1_subprocess_spawned_for_idle_account` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../docs/feature/024_session_touch.md)

---

### FT-03: After successful touch, table shows concrete `5h Reset` value (was `—`)

- **Given:** One account with valid quota data and `five_hour.resets_at` absent (idle — `5h Reset = —`); touch subprocess succeeds and re-fetch returns `resets_at` set to ~5h from current time.
- **When:** `clp .usage touch::1`
- **Then:** Account row shows a `5h Reset` value of ~5h (e.g., "in 4h 59m") — transitioned from `—` (idle) to a concrete countdown (active).
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** `it111_lim_it_touch_1_5h_reset_changes_from_dash_to_time` (in `usage_touch_test.rs`)
- **Note:** Two outcomes both satisfy this case, distinguished by the `~` prefix. When the post-touch re-fetch returns `resets_at`, the countdown is exact (`in 4h 59m`, no `~`). When the server has not yet propagated the new window, AC-20's projection renders instead (`~in 4h 5xm`) — still a countdown, still not `—`. The assertion is "no longer the idle em-dash", not "no `~`"; treating the projected form as a failure would make this test flaky against ordinary propagation lag.
- **Source:** [feature/024_session_touch.md AC-03, AC-20](../../../docs/feature/024_session_touch.md)

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

- **Given:** Source code of `src/usage/api.rs`, read via `include_str!`.
- **When:** The byte offsets of the `apply_refresh( &mut accounts, &credential_store` call site and the `apply_touch( aq, &credential_store` call site are located and compared.
- **Then:** The `apply_refresh` call site's byte offset is less than the `apply_touch` call site's byte offset — refresh is wired to run before touch in source order. This is a structural ordering guard, not a live execution trace.
- **Exit:** N/A (structural source-inspection test — no exit code)
- **Source fn:** `it112_structural_refresh_before_touch_ordering_in_source` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-05](../../../docs/feature/024_session_touch.md)

---

### FT-06: apply_touch does not call switch_account; live session credentials file untouched by touch cycle

- **Given:** `apply_touch` is called with one qualifying account (valid quota, `resets_at` absent — idle). The credential-store `_active` marker IS pre-written (content `"test@example.com"`) before the call — its content is not the object under test here. The live session credentials file (`paths.credentials_file()`) does NOT exist before the call.
- **When:** `apply_touch` processes the qualifying account and completes.
- **Then:** The live session credentials file still does NOT exist after the call — no `switch_account`-style write to the live session file occurred. `apply_touch` does not restore via `switch_account`; the refresh path never invokes it.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `it_apply_touch_trigger_fires_resets_at_none` (in `touch_tests_b.rs`)
- **Note:** BUG-211 regression guard — verifies snapshot+restore was not re-introduced in `apply_touch`. The `_active` marker's own unchanged-content assertion is covered by FT-15's companion test. Symmetric to FT-13/BUG-211 guard in `017_token_refresh` test spec.
- **Source:** [feature/024_session_touch.md AC-06](../../../docs/feature/024_session_touch.md)

---

### FT-07: Touch subprocess failure is non-aborting; account row shows original data unchanged

- **Given:** Source code of `src/usage/touch.rs`, read via `include_str!`.
- **When:** The source text is searched for the failure-tolerant guard pattern around the subprocess result.
- **Then:** The source contains the literal guard `if let Some( ref creds ) = new_creds` — the refreshed-credentials branch is conditional, so a subprocess failure (leaving `new_creds` as `None`) falls through without aborting; the account keeps its original quota data and the table still renders.
- **Exit:** N/A (structural source-inspection test — no exit code)
- **Source fn:** `it114_structural_touch_failure_non_aborting_guard_exists` (in `usage_touch_test.rs`)
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
- **Source fn:** `it115_lim_it_trace_1_shows_touch_lifecycle` (in `usage_touch_test.rs`)
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
- **Source fn:** `it116_lim_it_account_with_resets_at_absent_is_touched` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../docs/feature/024_session_touch.md)

---

### FT-12: In `live::1` mode, touch fires each cycle for accounts with `resets_at` absent

- **Given:** One account that becomes idle between cycles (`resets_at` becomes absent after session expires); `live::1 touch::1`.
- **When:** `clp .usage live::1 touch::1` (observed over two cycles via trace output or structural assertion)
- **Then:** On each cycle where `resets_at` is absent (account became idle), the touch trigger fires (subprocess spawned) and a new 5h session is started. The trigger does not fire for accounts with `resets_at` present (still active).
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window + two live::1 cycles)
- **Source fn:** `it120_lim_it_ft12_touch_trigger_fires_per_idle_account_cycle` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-11](../../../docs/feature/024_session_touch.md)

---

### FT-13: All three quota timers running → account NOT touched ("already active")

- **Given:** `apply_touch` is called directly (not via CLI) with one account from `mk_aq_with_resets_at(Some("2099-01-01T00:00:00Z"))`: `five_hour.resets_at` is `Some(...)` (active), and `seven_day`/`seven_day_sonnet` are both `None` (absent entirely — treated as "running" by the `map_or(true, ...)` default, not because either field is itself "present"). `trace=false`.
- **When:** `apply_touch(&mut aq, &store, Some(&paths), false, SubprocessModel::Auto, SubprocessEffort::Auto, false)` processes the account.
- **Then:** `apply_touch` returns early without calling `refresh_account_token` — the live session credentials file (`.claude/.credentials.json`) is NOT written. No subprocess is spawned.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `it_apply_touch_trigger_skips_resets_at_some` (in `touch_tests_b.rs`)
- **Source:** [feature/024_session_touch.md AC-02, AC-12](../../../docs/feature/024_session_touch.md)

---

### FT-14: Skip trace line emitted for each account not qualifying for touch

- **Given:** One account (`err@x.com`) with no `accessToken` — quota fetch fails with an `Err` result (error account; skip reason: "error account"). `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Exits 0. Stderr contains the line `· touch  err@x.com  skipped (reason: error account)`. No subprocess spawned for the errored account.
- **Exit:** 0
- **Source fn:** `it141_trace_skip_lines_emitted_for_non_qualifying_accounts` (in `usage_model_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09, AC-12](../../../docs/feature/024_session_touch.md)

---

### FT-15: No switch_account called in apply_touch; `_active` unchanged confirms no restore occurred

- **Given:** `apply_touch` is called with `trace=true`. The account being touched (`test@example.com`, `resets_at` absent — qualifies) has NO credential file in the store, so the refresh path cannot find a token. The `_active` marker IS pre-written in the credential store but holds a DIFFERENT account's name (`"alice@example.com"`) — a credential file for `alice@example.com` also exists in the store but must never be copied to the live session file.
- **When:** `apply_touch` processes the qualifying account and completes.
- **Then:** The live session credentials file (`paths.credentials_file()`) does NOT exist after the call — no `switch_account` write occurred during touch cycling. The `_active` marker file is read back and is UNCHANGED — still `"alice@example.com"` — confirming touch never overwrites or restores the marker.
- **Source fn:** `test_apply_touch_mre_bug208_restore_trace_emitted` (in `tests/usage/touch_tests.rs`)
- **Note:** BUG-211 MRE — function name preserved from BUG-208 era; now asserts absence of restore side-effects rather than presence of restore trace. Symmetric to FT-17 in `017_token_refresh` test spec.
- **Source:** [feature/024_session_touch.md AC-13](../../../docs/feature/024_session_touch.md)

---

### FT-16: 7d-exhausted account (7d Left = 0%, 5h idle) is NOT touched — 7d guard fires

- **Given:** `apply_touch` is called with one account whose `AccountQuota` has: `result = Ok(data)` with `seven_day_left = 0.0` (weekly quota fully exhausted, via `seven_day.utilization = 100.0`), `five_hour_left = 50.0` (5h budget half-consumed but non-zero — not h-exhausted), and `five_hour.resets_at = None` (idle — no active 5h session). The 7d guard is present in `apply_touch`.
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
- **Source fn:** `mre_bug288_post_switch_touch_refetch_updates_quota` (in `tests/usage/api_tests_b.rs`, structural block — asserts `write_quota_cache` is called in `apply_post_switch_touch` fn body) + `it_apply_touch_trigger_skips_resets_at_some` (in `tests/usage/touch_tests_b.rs` — corrected file path; asserts `apply_touch` skips when `resets_at = Some`).
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
- **Source fn:** `mre_bug309_apply_touch_refetch_writes_cache_and_clears_cached_flag` (in `touch_tests_b.rs`)
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

---

### FT-25: Touched row with absent `resets_at` renders a projected countdown, never the literal `(touched)`

- **Given:** One account `touched@x.com` whose quota result carries no `five_hour.resets_at`, with `touched_at_secs = Some(now - 600)` — a corroborated touch record 10 minutes old.
- **When:** `render_text` produces the table.
- **Then:** The row's `5h Reset` cell contains no occurrence of the substring `(touched)`, contains `~in `, and specifically contains `~in 4h`. The last assertion pins the projection arithmetic end to end: flooring a 10-minute-old touch leaves between 4h40m and 4h50m, so the countdown must still name hours — a `~in 4h` that became `~in 0h` or a bare minute value would signal an imminent reset that is not happening.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug551_touched_row_renders_projected_countdown_not_placeholder` (in `render_tests_b.rs`)
- **Companion:** `test_bug551_get_field_and_table_agree_on_touched_row` (same file) uses the same fixture and asserts twice — `extract_get_field( .., FiveHourReset, now )` starts with `~in `, and the rendered table row carries the same projection. The table half brackets `render_text`'s clock (`before`/`after` reads around the call) and accepts either resulting label, because `render_text` samples its own clock with no injection point: comparing the table against a single `now`-derived string races the countdown's minute tick and fails intermittently. `projected_reset_label` is monotonic in `now_secs`, so the cell must equal the label at some instant in `[before, after]` — the two candidates collapse to one string in the common case and straddle exactly one boundary otherwise, which is what makes the equality exact rather than approximate.
- **Note:** BUG-551 MRE. RED→GREEN verified: with the `(touched)` literal temporarily restored in `render.rs`, this test fails with the exact reported symptom (`(touched)` in the `5h Reset` column). The companion `get::` test exists because BUG-551's original defect was surface-divergent — the placeholder appeared only in the text table, while `get::`, JSON, and TSV silently emitted the plain absent form.
- **Source:** [feature/024_session_touch.md AC-20](../../../docs/feature/024_session_touch.md)

---

### FT-26: Projected window end floors the touch instant to a 10-minute boundary before adding 5h

- **Given:** Three touch instants: one at `:30:54` past the hour, one at `:27:41`, and one already exactly on a 10-minute boundary.
- **When:** `projected_window_end_secs( touch_secs )` is evaluated for each.
- **Then:** The `:30:54` instant projects to `+5h` from `:30:00` (seconds discarded); the `:27:41` instant floors *back* to `:20:00`, not forward to `:30:00`; the already-aligned instant is unchanged by the floor. Each result equals `floor_to_10_minutes(touch) + WINDOW_5H_S`.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_bug551_projected_window_end_floors_to_ten_minute_boundary` (in `format_tests.rs`)
- **Note:** Pins the arithmetic AC-20's ±60s accuracy claim depends on. The floor direction matters: rounding to the *nearest* boundary rather than flooring would push a `:27:41` touch's projected end 10 minutes late, outside the tolerance. Fixture values are taken from live accounts i13 (`:30:54`) and i2 (`:27:41`), the two whose real `resets_at` most directly discriminate floor-vs-round.
- **Source:** [feature/024_session_touch.md AC-20](../../../docs/feature/024_session_touch.md)

---

### FT-27: A touch refuted by a later window-less fetch yields no corroboration

- **Given:** Five deterministic cache fixtures over a fixed clock, all sharing `last_touch_at = 2026-08-22T17:30:00Z`: (A) quota fetched 1s after the touch, no 5h window; (B) fetched 22 minutes after, no window; (C) fetched 22 minutes after, window present; (D) fetched exactly 300s after, no window; (E) same as A but evaluated after `TOUCH_GRACE_SECS` has elapsed.
- **When:** `corroborated_touch_at( &cache, now_secs )` is evaluated for each.
- **Then:** A returns `Some(touch_at)` — a fetch 1s later is uninformative, so the touch is not refuted. B returns `None` — a fetch 22 minutes later still showing no window refutes the touch (this is the live i15 state). C returns `Some(touch_at)` — the window's presence corroborates regardless of fetch timing. D returns `Some(touch_at)` — the boundary is exclusive, exactly `TOUCH_PROPAGATION_SECS` does not refute. E returns `None` — the grace window governs independently of refutation.
- **Exit:** N/A (unit test — no exit code)
- **Source fn:** `test_mre_bug552_refuted_touch_yields_no_corroboration` (in `touch_tests_b.rs`)
- **Note:** BUG-552 MRE. Scenario D pins the boundary as `>` not `>=`, so the constant's stated value is testable rather than incidental. Fixtures use literal ISO-8601 strings and a fixed `now_secs` rather than deriving timestamps, keeping the test independent of both wall clock and any date-formatting helper.
- **Source:** [feature/024_session_touch.md AC-21](../../../docs/feature/024_session_touch.md)

---

### FT-28: Display and re-touch skip guard both call the same corroboration predicate

- **Given:** `src/usage/touch.rs` source.
- **When:** The file is scanned for corroboration call sites.
- **Then:** `corroborated_touch_at( &cache` appears exactly twice — once in `derive_touched_recently` (the display path) and once in `touch_skip_reason` (the re-touch skip guard) — and the superseded bare-grace call the skip guard previously used is absent.
- **Exit:** N/A (structural source-inspection test — no exit code)
- **Source fn:** `test_bug552_both_consumers_share_the_corroboration_predicate` (in `touch_tests_b.rs`)
- **Note:** BUG-552 structural guard. The bug's severity came from the two consumers agreeing on a *wrong* answer: the same unverified flag both fabricated a window on screen and suppressed the re-touch that would have corrected it. Keeping them on one predicate is what makes AC-21 hold; a refactor that reintroduces a second, laxer check would restore the self-sustaining failure, so the count is asserted rather than the mere presence of a call.
- **Source:** [feature/024_session_touch.md AC-21](../../../docs/feature/024_session_touch.md)

---

### FT-29: The post-switch touch writer gates its stamp on the refresh result

- **Given:** `src/usage/api_switch.rs` source.
- **When:** `apply_post_switch_touch` is scanned for its `mark_touched` call site.
- **Then:** The call sits inside an `if refreshed.is_some()` block — the guard both precedes it and has not closed before it — and appears exactly once.
- **Exit:** N/A (structural source-inspection test — no exit code)
- **Source fn:** `mre_bug552_post_switch_touch_gates_mark_touched_on_refresh` (in `touch_tests_b.rs`), `bug_reproducer(BUG-552)`
- **Note:** Structural rather than behavioural by necessity: `apply_post_switch_touch` spawns a real `claude` subprocess and performs live quota fetches, and the project forbids mocking either away — the same constraint that makes FT-28 structural. Two sibling tests cover the observable consequence from the other side: `mre_bug288_post_switch_touch_refetch_updates_quota` and `it_apply_post_switch_touch_cred_file_absent_skips_refetch` (in `api_tests_b.rs`) both run the real function with a refresh that cannot succeed and assert no touch flags are written. Both previously asserted the opposite — that the flags were written *unconditionally* — which is how the ungated stamp survived: the tests had pinned the defect as the design.
- **Source:** [feature/024_session_touch.md AC-22](../../../docs/feature/024_session_touch.md)
