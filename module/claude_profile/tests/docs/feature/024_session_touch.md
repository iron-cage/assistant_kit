# Test: Feature 024 — Session Touch via Isolated Subprocess

Feature behavioral requirement test cases for `docs/feature/024_session_touch.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/034_touch.md](../cli/param/034_touch.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::0` — no subprocess; idle accounts not activated when suppressed | AC-01 | Integration |
| FT-02 | `touch::1` invokes subprocess for accounts with `resets_at` absent (idle) | AC-02 | Integration (lim_it) |
| FT-03 | After touch, table shows concrete `5h Reset` value (was `—`) | AC-03 | Integration (lim_it) |
| FT-04 | Errored accounts are never touched | AC-04 | Integration |
| FT-05 | When both `refresh::1` and `touch::1`, refresh runs first | AC-05 | Integration |
| FT-06 | Original active account restored after all touch operations | AC-06 | Integration (lim_it) |
| FT-07 | Touch failure is non-aborting; row shows original data | AC-07 | Integration |
| FT-08 | `touch::` does not affect `format::json` output structure | AC-08 | Integration |
| FT-09 | `trace=true` emits `[trace]` lines for touch subprocess lifecycle | AC-09 | Integration (lim_it) |
| FT-10 | `touch::` appears in `.usage.help` with default `1` | AC-10 | Integration |
| FT-11 | Valid account with `resets_at` absent IS touched (positive trigger) | AC-02 | Trigger |
| FT-12 | In `live::1` mode, touch fires each cycle for accounts with `resets_at` absent | AC-11 | Live Mode |
| FT-13 | Account with `resets_at` present (already active) is NOT touched | AC-02, AC-12 | Trigger Guard |
| FT-14 | Skip trace line emitted for each account not qualifying for touch | AC-09, AC-12 | Trace |
| FT-15 | `trace::1` emits `restore switch_account` line after all touch operations; failure always logged | AC-13 | Restore Trace |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | touch::0 no subprocess | AC-01 | Default Behavior |
| FT-02 | touch::1 subprocess for accounts with resets_at absent (idle) | AC-02 | Trigger |
| FT-03 | After touch concrete 5h Reset shown (was —) | AC-03 | Re-fetch |
| FT-04 | Errored accounts not touched | AC-04 | Trigger Guard |
| FT-05 | refresh before touch ordering | AC-05 | Ordering |
| FT-06 | Active account restored after touch | AC-06 | Restore |
| FT-07 | Touch failure non-aborting | AC-07 | Failure Handling |
| FT-08 | JSON unaffected by touch | AC-08 | JSON No-op |
| FT-09 | Trace shows touch lifecycle | AC-09 | Trace |
| FT-10 | touch:: in help with default 1 | AC-10 | Help Output |
| FT-11 | Valid account with resets_at absent IS touched | AC-02 | Trigger |
| FT-12 | live::1 touch fires each cycle when resets_at absent | AC-11 | Live Mode |
| FT-13 | Account with resets_at present (active) NOT touched | AC-02, AC-12 | Trigger Guard |
| FT-14 | Skip trace line emitted for each non-qualifying account | AC-09, AC-12 | Trace |
| FT-15 | `trace::1` emits `restore switch_account` line after all touch operations; failure always logged | AC-13 | Restore Trace |

**Total:** 15 FT cases

---

### FT-01: `touch::0` — no subprocess spawned; idle accounts not activated when suppressed

- **Given:** One account with valid quota data and `five_hour.resets_at` absent (idle — no active 5h window; would be touched with `touch::1`).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0. No subprocess spawned for touch. Account row shows `5h Reset = —` unchanged (still idle).
- **Exit:** 0
- **Source fn:** `it099_lim_it_touch_0_no_subprocess_idle_account_unchanged` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../../docs/feature/024_session_touch.md)

---

### FT-02: `touch::1` invokes `refresh_account_token()` for accounts with `resets_at` absent (idle)

- **Given:** One account with valid quota data (result=Ok) and `five_hour.resets_at` absent (idle — no active 5h session).
- **When:** `clp .usage touch::1`
- **Then:** `refresh_account_token()` is called for that account (observable via `trace::1` output showing subprocess lifecycle). Accounts with `resets_at` present (already active) are not touched.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** `it100_lim_it_touch_1_subprocess_spawned_for_idle_account` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../../docs/feature/024_session_touch.md)

---

### FT-03: After successful touch, table shows concrete `5h Reset` value (was `—`)

- **Given:** One account with valid quota data and `five_hour.resets_at` absent (idle — `5h Reset = —`); touch subprocess succeeds and re-fetch returns `resets_at` set to ~5h from current time.
- **When:** `clp .usage touch::1`
- **Then:** Account row shows a `5h Reset` value of ~5h (e.g., "in 4h 59m") — transitioned from `—` (idle) to a concrete countdown (active).
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

- **Given:** One account with expired token (quota fetch would fail with 401).
- **When:** `clp .usage refresh::1 touch::1 trace::1`
- **Then:** Stderr `[trace]` lines show refresh lifecycle before any touch lifecycle. Touch runs on post-refresh results — if refresh started a session (making `resets_at` present), that account is skipped by touch (already activated by refresh). If the post-refresh result still has `resets_at` absent, touch fires.
- **Exit:** 0
- **Live:** yes (lim_it — requires expired token + active 5h window)
- **Source fn:** `it102_structural_refresh_before_touch_ordering_in_source` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-05](../../../../docs/feature/024_session_touch.md)

---

### FT-06: Original `_active` account restored after all touch operations complete

- **Given:** Two accounts: `alice@test.com` (stored as `_active`) and `idle@test.com` (valid quota, `resets_at` absent — idle). `touch::1`.
- **When:** `clp .usage touch::1`
- **Then:** After touch subprocess for `idle@test.com` completes, `_active` file points back to `alice@test.com` (original active account restored).
- **Exit:** 0
- **Live:** yes (lim_it)
- **Source fn:** `it103_lim_it_active_account_restored_after_touch` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-06](../../../../docs/feature/024_session_touch.md)

---

### FT-07: Touch subprocess failure is non-aborting; account row shows original data unchanged

- **Given:** One account with valid quota data and `resets_at` absent (idle — qualifies for touch); touch subprocess fails (returns non-zero exit or timeout).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Account row shows original quota data (not a hard error). Table still renders. Touch failure does not abort the command.
- **Exit:** 0
- **Source fn:** `it104_structural_touch_failure_non_aborting_guard_exists` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-07](../../../../docs/feature/024_session_touch.md)

---

### FT-08: `touch::` does not affect `format::json` output structure

- **Given:** One account with valid quota data and `resets_at` absent (idle — qualifies for touch).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage touch::1 format::json`
- **Then-A and Then-B:** JSON arrays have identical schema. `touch::` does not add or remove fields. Touched accounts appear as normal data objects with their re-fetched quota values.
- **Exit:** 0 both cases
- **Source fn:** `it090_touch_json_format_unaffected`
- **Source:** [feature/024_session_touch.md AC-08](../../../../docs/feature/024_session_touch.md)

---

### FT-09: `trace=true` emits `[trace]` lines for touch subprocess lifecycle

- **Given:** One account with valid quota data and `resets_at` absent (idle — qualifies for touch); `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains `[trace]` lines showing the touch subprocess lifecycle steps (`read credentials`, `run_isolated` with elapsed time, `write credentials`, `save`). Lines include account name and subprocess status.
- **Exit:** 0
- **Live:** yes (lim_it — requires idle account for subprocess to be triggered)
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

### FT-11: Valid account with `resets_at` absent IS touched (positive trigger case)

- **Given:** One account with valid quota data (`result=Ok`) where `five_hour.resets_at` is absent (None) — meaning the account is idle with no active 5h session.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Subprocess is spawned for this account. After touch, the `5h Reset` column shows a concrete countdown value (~5h) — transitioned from `—` to active. The trigger condition `resets_at.is_none()` fires for this account.
- **Exit:** 0
- **Live:** yes (lim_it — requires idle account)
- **Source fn:** `it106_lim_it_account_with_resets_at_absent_is_touched` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../../docs/feature/024_session_touch.md)

---

### FT-12: In `live::1` mode, touch fires each cycle for accounts with `resets_at` absent

- **Given:** One account that becomes idle between cycles (`resets_at` becomes absent after session expires); `live::1 touch::1`.
- **When:** `clp .usage live::1 touch::1` (observed over two cycles via trace output or structural assertion)
- **Then:** On each cycle where `resets_at` is absent (account became idle), the touch trigger fires (subprocess spawned) and a new 5h session is started. The trigger does not fire for accounts with `resets_at` present (still active).
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window + two live::1 cycles)
- **Source fn:** `it110_lim_it_ft12_touch_trigger_fires_per_idle_account_cycle` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-11](../../../../docs/feature/024_session_touch.md)

---

### FT-13: Account with `resets_at` present (already active) is NOT touched (trigger guard)

- **Given:** One account with valid quota data and `five_hour.resets_at` present (already has an active 5h session window with a countdown).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned for that account. The trigger guard skips accounts with `resets_at` present — they already have active sessions and don't need activation. Account row shows original quota data unchanged.
- **Exit:** 0
- **Source fn:** `it_apply_touch_trigger_skips_resets_at_some` (in `src/usage.rs #[cfg(test)]`)
- **Source:** [feature/024_session_touch.md AC-02, AC-12](../../../../docs/feature/024_session_touch.md)

---

### FT-14: Skip trace line emitted for each account not qualifying for touch

- **Given:** Two accounts: one with `resets_at` present (already active — skip reason: "already active 5h window"); one with errored quota (no valid data — skip reason: Err). `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains `[trace] touch  <name>  skipped (reason: ...)` lines for each non-qualifying account. The `resets_at` present case and the errored case each produce a diagnostically distinct skip-reason line. No subprocess spawned for either account.
- **Exit:** 0
- **Source fn:** `it131_trace_skip_lines_emitted_for_non_qualifying_accounts` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09, AC-12](../../../../docs/feature/024_session_touch.md)

---

### FT-15: `trace::1` emits `restore switch_account` line after all touch operations; failure always logged

- **Given:** `apply_touch` is called with `trace=true`; `original_active` is set (active marker contains a non-empty account name); the original account has a credential file in the store so `switch_account` can succeed.
- **When:** `apply_touch(&mut aq, store.path(), Some(&paths), true, imodel, effort)` is called (unit test context; equivalent to `clp .usage touch::1 trace::1` with active marker present)
- **Then:** Stderr contains `[trace] touch  {original_name}  restore switch_account: OK`; the restore step is not silent under `trace::1`.
- **And:** In a separate scenario where `switch_account` fails at restore time, stderr contains the failure line unconditionally — without requiring `trace=true`.
- **Source fn:** `test_apply_touch_mre_bug208_restore_trace_emitted` (in `src/usage.rs #[cfg(test)]`)
- **Note:** Fix for BUG-208 — `apply_touch` used `let _ = switch_account(...)` at the restore site, making restore failures silent and restore trace completeness impossible. Symmetric fix to `apply_refresh` (FT-17 in 017_token_refresh test spec).
- **Source:** [feature/024_session_touch.md AC-13](../../../../docs/feature/024_session_touch.md)
