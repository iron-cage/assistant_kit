# Test: Feature 024 — Session Touch via Isolated Subprocess

Feature behavioral requirement test cases for `docs/feature/024_session_touch.md`. Each FT case maps to one acceptance criterion. Parameter edge cases are in [cli/param/034_touch.md](../cli/param/034_touch.md). Command-level tests (IT-N) are in [cli/command/009_usage.md](../cli/command/009_usage.md).

### AC Coverage Index

| FT | Criterion | AC | Notes |
|----|-----------|-----|-------|
| FT-01 | `touch::0` — no subprocess; idle accounts appear as-is | AC-01 | Integration |
| FT-02 | `touch::1` invokes subprocess for accounts with missing `resets_at` | AC-02 | Integration (lim_it) |
| FT-03 | After touch, table shows concrete `5h Reset` value | AC-03 | Integration (lim_it) |
| FT-04 | Errored accounts are never touched | AC-04 | Integration |
| FT-05 | When both `refresh::1` and `touch::1`, refresh runs first | AC-05 | Integration |
| FT-06 | Original active account restored after all touch operations | AC-06 | Integration (lim_it) |
| FT-07 | Touch failure is non-aborting; row shows original data | AC-07 | Integration |
| FT-08 | `touch::` does not affect `format::json` output structure | AC-08 | Integration |
| FT-09 | `trace=true` emits `[trace]` lines for touch subprocess lifecycle | AC-09 | Integration (lim_it) |
| FT-10 | `touch::` appears in `.usage.help` with default `0` | AC-10 | Integration |
| FT-11 | Valid account with `resets_at` present is NOT touched | AC-02 | Trigger Guard |
| FT-12 | In `live::1` mode, touch runs every cycle but idle trigger fires only when `resets_at` absent | AC-11 | Live Mode |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | touch::0 no subprocess | AC-01 | Default Behavior |
| FT-02 | touch::1 subprocess for idle accounts | AC-02 | Trigger |
| FT-03 | After touch concrete 5h Reset shown | AC-03 | Re-fetch |
| FT-04 | Errored accounts not touched | AC-04 | Trigger Guard |
| FT-05 | refresh before touch ordering | AC-05 | Ordering |
| FT-06 | Active account restored after touch | AC-06 | Restore |
| FT-07 | Touch failure non-aborting | AC-07 | Failure Handling |
| FT-08 | JSON unaffected by touch | AC-08 | JSON No-op |
| FT-09 | Trace shows touch lifecycle | AC-09 | Trace |
| FT-10 | touch:: in help with default 0 | AC-10 | Help Output |
| FT-11 | Valid account with resets_at present not touched | AC-02 | Trigger Guard |
| FT-12 | live::1 touch runs every cycle, idle trigger only when resets_at absent | AC-11 | Live Mode |

**Total:** 12 FT cases

---

### FT-01: `touch::0` — no subprocess spawned; idle accounts appear as-is

- **Given:** One account with valid quota data but `five_hour.resets_at` absent (`5h Reset = —`).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0. No subprocess spawned for touch. Account row shows `—` in `5h Reset` column (unchanged from quota fetch).
- **Exit:** 0
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../../docs/feature/024_session_touch.md)

---

### FT-02: `touch::1` invokes `refresh_account_token()` for accounts with missing `resets_at`

- **Given:** One account with valid quota data (result=Ok) but `five_hour.resets_at` absent.
- **When:** `clp .usage touch::1`
- **Then:** `refresh_account_token()` is called for that account (observable via `trace::1` output showing subprocess lifecycle). Accounts with `resets_at` present are not touched.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../../docs/feature/024_session_touch.md)

---

### FT-03: After successful touch, table shows concrete `5h Reset` value instead of `—`

- **Given:** One account with valid quota data but `five_hour.resets_at` absent; touch subprocess succeeds and re-fetch returns active `resets_at`.
- **When:** `clp .usage touch::1`
- **Then:** Account row shows a concrete `5h Reset` value (e.g., "in 5h 0m") instead of `—`.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window)
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-03](../../../../docs/feature/024_session_touch.md)

---

### FT-04: Errored accounts (quota fetch failed) are never touched

- **Given:** One account whose credential file has no `accessToken` (quota fetch returns Err — not a successful result with missing `resets_at`).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned. Account row shows original error state unchanged. Touch trigger requires `result = Ok(...)`.
- **Exit:** 0
- **Source fn:** `it088_touch_1_errored_account_skipped`
- **Source:** [feature/024_session_touch.md AC-04](../../../../docs/feature/024_session_touch.md)

---

### FT-05: When both `refresh::1` and `touch::1` active, refresh runs first

- **Given:** One account with expired token (quota fetch would fail with 401) and `five_hour.resets_at` absent.
- **When:** `clp .usage refresh::1 touch::1 trace::1`
- **Then:** Stderr `[trace]` lines show refresh lifecycle before any touch lifecycle. Touch runs on post-refresh results (if refresh succeeded and resulting quota still has missing `resets_at`).
- **Exit:** 0
- **Live:** yes (lim_it — requires expired token + idle 5h window)
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-05](../../../../docs/feature/024_session_touch.md)

---

### FT-06: Original `_active` account restored after all touch operations complete

- **Given:** Two accounts: `alice@test.com` (stored as `_active`) and `idle@test.com` (valid quota, `resets_at` absent). `touch::1`.
- **When:** `clp .usage touch::1`
- **Then:** After touch subprocess for `idle@test.com` completes, `_active` file points back to `alice@test.com` (original active account restored).
- **Exit:** 0
- **Live:** yes (lim_it)
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-06](../../../../docs/feature/024_session_touch.md)

---

### FT-07: Touch subprocess failure is non-aborting; account row shows original data unchanged

- **Given:** One account with valid quota data and `resets_at` absent; touch subprocess fails (returns non-zero exit or timeout).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. Account row shows original quota data (not a hard error). Table still renders. Touch failure does not abort the command.
- **Exit:** 0
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-07](../../../../docs/feature/024_session_touch.md)

---

### FT-08: `touch::` does not affect `format::json` output structure

- **Given:** One account with valid quota data and `resets_at` absent.
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage touch::1 format::json`
- **Then-A and Then-B:** JSON arrays have identical schema. `touch::` does not add or remove fields. Touched accounts appear as normal data objects with their re-fetched quota values.
- **Exit:** 0 both cases
- **Source fn:** `it090_touch_json_format_unaffected`
- **Source:** [feature/024_session_touch.md AC-08](../../../../docs/feature/024_session_touch.md)

---

### FT-09: `trace=true` emits `[trace]` lines for touch subprocess lifecycle

- **Given:** One account with valid quota data and `resets_at` absent; `touch::1 trace::1`.
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Stderr contains `[trace]` lines showing the touch subprocess lifecycle (same format as refresh trace output). Lines include account name and subprocess status.
- **Exit:** 0
- **Live:** yes (lim_it — requires idle account for subprocess to be triggered)
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-09](../../../../docs/feature/024_session_touch.md)

---

### FT-10: `touch::` appears in `.usage.help` output with default value `0`

- **Given:** Standard environment.
- **When:** `clp .usage.help`
- **Then:** Exits 0. Stdout contains "touch". Output shows the default value as `0` (off).
- **Exit:** 0
- **Source fn:** `it091_usage_help_shows_touch_param`
- **Source:** [feature/024_session_touch.md AC-10](../../../../docs/feature/024_session_touch.md)

---

### FT-11: Valid account with `resets_at` present is NOT touched

- **Given:** One account with valid quota data (`result=Ok`) where `five_hour.resets_at` is present (not None) — meaning the 5h window is already active.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned for this account. The `5h Reset` column shows the existing countdown value (e.g., "in 3h 12m") — unchanged. The trigger guard `resets_at.is_some() → skip` fires before any subprocess is invoked.
- **Exit:** 0
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-02](../../../../docs/feature/024_session_touch.md)

---

### FT-12: In `live::1` mode, touch runs every cycle but idle trigger fires only when `resets_at` absent

- **Given:** One account; `live::1 touch::1`. On first cycle the account is idle (`resets_at` absent). After touch succeeds, subsequent cycles see `resets_at` present.
- **When:** `clp .usage live::1 touch::1` (observed over two cycles via trace output or structural assertion)
- **Then:** On the first cycle the touch trigger fires (subprocess spawned). On subsequent cycles, since `resets_at` is now present, the trigger does not fire for that account. The 5h window must fully expire (account goes idle again) before the trigger re-arms.
- **Exit:** 0
- **Live:** yes (lim_it — requires live credential + idle 5h window + two live::1 cycles)
- **Source fn:** ⏳ TBD (integration test in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-11](../../../../docs/feature/024_session_touch.md)
