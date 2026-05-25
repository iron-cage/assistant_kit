# Test: Feature 024 — Session Touch via Isolated Subprocess

Feature behavioral requirement test cases for `docs/feature/024_session_touch.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/034_touch.md](../cli/param/034_touch.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::0` — no subprocess; active accounts not touched when suppressed | AC-01 | Integration |
| FT-02 | `touch::1` invokes subprocess for accounts with `resets_at` present | AC-02 | Integration (lim_it) |
| FT-03 | After touch, table shows concrete `5h Reset` value | AC-03 | Integration (lim_it) |
| FT-04 | Errored accounts are never touched | AC-04 | Integration |
| FT-05 | When both `refresh::1` and `touch::1`, refresh runs first | AC-05 | Integration |
| FT-06 | Original active account restored after all touch operations | AC-06 | Integration (lim_it) |
| FT-07 | Touch failure is non-aborting; row shows original data | AC-07 | Integration |
| FT-08 | `touch::` does not affect `format::json` output structure | AC-08 | Integration |
| FT-09 | `trace=true` emits `[trace]` lines for touch subprocess lifecycle | AC-09 | Integration (lim_it) |
| FT-10 | `touch::` appears in `.usage.help` with default `1` | AC-10 | Integration |
| FT-11 | Valid account with `resets_at` present IS touched (positive trigger) | AC-02 | Trigger |
| FT-12 | In `live::1` mode, touch fires each cycle for accounts with `resets_at` present | AC-11 | Live Mode |
| FT-13 | H-exhausted account with `resets_at` present is NOT touched | AC-02, AC-12 | Trigger Guard |
| FT-14 | Skip trace line emitted for each account not qualifying for touch | AC-09, AC-12 | Trace |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | touch::0 no subprocess | AC-01 | Default Behavior |
| FT-02 | touch::1 subprocess for accounts with resets_at present | AC-02 | Trigger |
| FT-03 | After touch concrete 5h Reset shown | AC-03 | Re-fetch |
| FT-04 | Errored accounts not touched | AC-04 | Trigger Guard |
| FT-05 | refresh before touch ordering | AC-05 | Ordering |
| FT-06 | Active account restored after touch | AC-06 | Restore |
| FT-07 | Touch failure non-aborting | AC-07 | Failure Handling |
| FT-08 | JSON unaffected by touch | AC-08 | JSON No-op |
| FT-09 | Trace shows touch lifecycle | AC-09 | Trace |
| FT-10 | touch:: in help with default 1 | AC-10 | Help Output |
| FT-11 | Valid account with resets_at present IS touched | AC-02 | Trigger |
| FT-12 | live::1 touch fires each cycle when resets_at present | AC-11 | Live Mode |
| FT-13 | H-exhausted account with resets_at present NOT touched | AC-02, AC-12 | Trigger Guard |
| FT-14 | Skip trace line emitted for each non-qualifying account | AC-09, AC-12 | Trace |

**Total:** 14 FT cases

---

### FT-01: `touch::0` — no subprocess spawned; active accounts suppressed

- **Given:** One account with valid quota data and `five_hour.resets_at` present (active 5h window — would be touched with `touch::1`).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0. No subprocess spawned for touch. Account row shows original `5h Reset` countdown value unchanged.
- **Exit:** 0
- **Source fn:** `it099_lim_it_touch_0_no_subprocess_active_account_unchanged` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../../docs/feature/024_session_touch.md)

---

### FT-02: `touch::1` invokes `refresh_account_token()` for accounts with `resets_at` present

- **Given:** One account with valid quota data (result=Ok) and `five_hour.resets_at` present (5h window active).
- **When:** `clp .usage touch::1`
- **Then:** `refresh_account_token()` is called for that account (observable via `trace::1` output showing subprocess lifecycle). Accounts with `resets_at` absent are not touched.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** `it100_lim_it_touch_1_subprocess_spawned_for_active_account` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../../docs/feature/024_session_touch.md)

---

### FT-03: After successful touch, table shows `5h Reset` value extended ~5h forward

- **Given:** One account with valid quota data and `five_hour.resets_at` present (e.g., showing ~2h remaining); touch subprocess succeeds and re-fetch returns `resets_at` extended ~5h from current time.
- **When:** `clp .usage touch::1`
- **Then:** Account row shows a `5h Reset` value extended ~5h forward (e.g., "in 4h 59m") — visibly farther than the pre-touch value.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** `it101_lim_it_touch_1_5h_reset_changes_from_dash_to_time` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-03](../../../../docs/feature/024_session_touch.md)

---

### FT-04: Errored accounts (quota fetch failed) are never touched

- **Given:** One account whose credential file has no `accessToken` (quota fetch returns Err — not a successful result with valid quota data).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned. Account row shows original error state unchanged. Touch trigger requires `result = Ok(...)`.
- **Exit:** 0
- **Source fn:** `it088_touch_1_errored_account_skipped`
- **Source:** [feature/024_session_touch.md AC-04](../../../../docs/feature/024_session_touch.md)

---

### FT-05: When both `refresh::1` and `touch::1` active, refresh runs first

- **Given:** One account with expired token (quota fetch would fail with 401) and `five_hour.resets_at` present.
- **When:** `clp .usage refresh::1 touch::1 trace::1`
- **Then:** Stderr `[trace]` lines show refresh lifecycle before any touch lifecycle. Touch runs on post-refresh results (if refresh succeeded and resulting quota has `resets_at` present).
- **Exit:** 0
- **Live:** yes (lim_it — requires expired token + active 5h window)
- **Source fn:** `it102_structural_refresh_before_touch_ordering_in_source` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-05](../../../../docs/feature/024_session_touch.md)

---

### FT-06: Original `_active` account restored after all touch operations complete

- **Given:** Two accounts: `alice@test.com` (stored as `_active`) and `active@test.com` (valid quota, `resets_at` present). `touch::1`.
- **When:** `clp .usage touch::1`
- **Then:** After touch subprocess for `active@test.com` completes, `_active` file points back to `alice@test.com` (original active account restored).
- **Exit:** 0
- **Live:** yes (lim_it)
- **Source fn:** `it103_lim_it_active_account_restored_after_touch` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-06](../../../../docs/feature/024_session_touch.md)

---

### FT-07: Touch subprocess failure is non-aborting; account row shows original data unchanged

- **Given:** One account with valid quota data and `resets_at` present; touch subprocess fails (returns non-zero exit or timeout).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Account row shows original quota data (not a hard error). Table still renders. Touch failure does not abort the command.
- **Exit:** 0
- **Source fn:** `it104_structural_touch_failure_non_aborting_guard_exists` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-07](../../../../docs/feature/024_session_touch.md)

---

### FT-08: `touch::` does not affect `format::json` output structure

- **Given:** One account with valid quota data and `resets_at` present.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage touch::1 format::json`
- **Then-A and Then-B:** JSON arrays have identical schema. `touch::` does not add or remove fields. Touched accounts appear as normal data objects with their re-fetched quota values.
- **Exit:** 0 both cases
- **Source fn:** `it090_touch_json_format_unaffected`
- **Source:** [feature/024_session_touch.md AC-08](../../../../docs/feature/024_session_touch.md)

---

### FT-09: `trace=true` emits `[trace]` lines for touch subprocess lifecycle

- **Given:** One account with valid quota data and `resets_at` present; `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains `[trace]` lines showing the touch subprocess lifecycle with per-step elapsed time (switch_account duration, run_isolated duration). Lines include account name and subprocess status.
- **Exit:** 0
- **Live:** yes (lim_it — requires account with active 5h window for subprocess to be triggered)
- **Source fn:** `it105_lim_it_trace_1_shows_touch_lifecycle` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09](../../../../docs/feature/024_session_touch.md)

---

### FT-10: `touch::` appears in `.usage.help` output with default value `1`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains "touch". Output shows the default value as `1` (on).
- **Exit:** 0
- **Source fn:** `it091_usage_help_shows_touch_param`
- **Source:** [feature/024_session_touch.md AC-10](../../../../docs/feature/024_session_touch.md)

---

### FT-11: Valid account with `resets_at` present IS touched (positive trigger case)

- **Given:** One account with valid quota data (`result=Ok`) where `five_hour.resets_at` is present (not None) — meaning the 5h window is active.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Subprocess is spawned for this account. After touch, the `5h Reset` column shows a refreshed countdown value extended ~5h forward. The trigger condition `resets_at.is_some()` fires for this account.
- **Exit:** 0
- **Live:** yes (lim_it — requires account with active 5h window)
- **Source fn:** `it106_lim_it_account_with_resets_at_present_is_touched` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../../docs/feature/024_session_touch.md)

---

### FT-12: In `live::1` mode, touch fires each cycle for accounts with `resets_at` present

- **Given:** One account with an active 5h window (`resets_at` present); `live::1 touch::1`.
- **When:** `clp .usage live::1 touch::1` (observed over two cycles via trace output or structural assertion)
- **Then:** On each cycle where `resets_at` is present, the touch trigger fires (subprocess spawned) and the 5h window is extended. The trigger does not fire for accounts with `resets_at` absent.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window + two live::1 cycles)
- **Source fn:** `it110_lim_it_ft12_touch_trigger_fires_per_active_window_cycle` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-11](../../../../docs/feature/024_session_touch.md)

---

### FT-13: H-exhausted account with `resets_at` present is NOT touched (trigger guard)

- **Given:** One account with valid quota data, `five_hour.resets_at` present (active 5h window), and `five_hour_left ≤ 15%` (h-exhausted — nearly fully consumed 5h session).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned for that account. The trigger guard rejects accounts with `five_hour_left ≤ 15%` even when `resets_at` is present — only accounts with `five_hour_left > 15%` qualify. Account row shows original quota data unchanged.
- **Exit:** 0
- **Source fn:** ⏳ `it130_touch_1_h_exhausted_account_with_resets_at_not_touched` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02, AC-12](../../../../docs/feature/024_session_touch.md)

---

### FT-14: Skip trace line emitted for each account not qualifying for touch

- **Given:** Two accounts: one with `resets_at` absent (no active 5h window); one with `resets_at` present but `five_hour_left ≤ 15%` (h-exhausted). `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains two `[trace] touch  <name>  skipped (reason: ...)` lines — one for each non-qualifying account. The `resets_at = None` case and the `five_hour_left ≤ 15%` case each produce a diagnostically distinct skip-reason line. No subprocess spawned for either account.
- **Exit:** 0
- **Source fn:** ⏳ `it131_trace_skip_lines_emitted_for_non_qualifying_accounts` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09, AC-12](../../../../docs/feature/024_session_touch.md)
