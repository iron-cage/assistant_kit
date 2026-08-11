# Test: `touch::` Parameter

Edge case coverage for the `touch::` parameter on `.usage`. For `.account.use` touch behavior, see [command/005_account_use.md](../command/05_account_use.md) (IT-17 through IT-20). See [param/034_touch.md](../../../../docs/cli/param/034_touch.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `touch::0` accepted with empty credential store (default) | Valid Value |
| EC-2 | `touch::1` accepted with empty credential store | Valid Value |
| EC-3 | `touch::true` accepted with empty credential store | Valid Value |
| EC-4 | `touch::bogus` exits 1 (invalid value) | Invalid Value |
| EC-5 | `touch::1` with errored-quota account — errored accounts are never touched | Trigger Guard |
| EC-6 | `touch::1 format::json` — empty-store output identical (not a schema-level field check) | JSON No-op |
| EC-7 | `touch::0` with idle account — no subprocess spawned, `—` unchanged | Behavioral Divergence |
| EC-8 | `touch::1` with idle account — subprocess spawned; reset-column transition proven by a companion test | Behavioral Divergence |

---

### EC-1: `touch::0` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage touch::0`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it106_touch_0_accepted_empty_store_exits_0` (in `usage_touch_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-2: `touch::1` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned (no accounts to touch).
- **Exit:** 0
- **Source fn:** `it097_touch_1_empty_store_exits_0` (in `usage_touch_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-3: `touch::true` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage touch::true`
- **Then:** Exits 0 with "(no accounts configured)". `true` is accepted as equivalent to `1`.
- **Exit:** 0
- **Source fn:** `it107_touch_true_accepted_empty_store_exits_0` (in `usage_touch_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-4: `touch::bogus` exits 1 (invalid value)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage touch::bogus`
- **Then:** Exits 1. Stderr indicates invalid value for `touch::`.
- **Exit:** 1
- **Source fn:** `it108_touch_bogus_exits_1` (in `usage_touch_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-5: `touch::1` with errored-quota account — errored accounts are never touched

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails with Err).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned for the errored account. Account row shows original error state unchanged. Touch trigger requires `result = Ok(...)` — Err accounts are never touched.
- **Exit:** 0
- **Source fn:** `it098_touch_1_errored_account_skipped` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-04](../../../../docs/feature/024_session_touch.md)

---

### EC-6: `touch::1 format::json` — empty-store output identical (not a schema-level field check)

- **Given:** Empty credential store (directory created, but no account files written — no saved account at all).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage format::json touch::1`
- **Then-A and Then-B:** Both exit 0. stdout is asserted byte-for-byte identical between the two runs (`assert_eq!`) — with an empty store this is `[]` in both cases. Because no account exists, this test never produces a JSON object with populated fields, so it does NOT verify that `touch::` leaves per-account field schema unchanged; it only verifies `touch::1` doesn't change the empty-store output.
- **Exit:** 0 both cases
- **Source fn:** `it100_touch_json_format_unaffected` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-08](../../../../docs/feature/024_session_touch.md)

---

### EC-7: `touch::0` with idle account — no subprocess spawned (Behavioral Divergence A)

- **Given:** One saved account with valid token and quota data where `five_hour.resets_at` is absent (idle — no active 5h window; would be touched with `touch::1`).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0. No subprocess spawned. The 5h Reset column shows `—` unchanged (still idle). `touch::0` disables the touch trigger regardless of account state.
- **Exit:** 0
- **Live:** yes (requires live quota data with idle account)
- **Source fn:** `it109_lim_it_touch_0_no_subprocess_idle_account_unchanged` (in `usage_touch_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../../docs/feature/024_session_touch.md)

---

### EC-8: `touch::1` with idle account — subprocess spawned; reset-column transition proven by a companion test

- **Given:** Same account as EC-7: valid token, `five_hour.resets_at` absent (idle, confirmed via a `.usage get::5h_reset` pre-check). Requires a real Anthropic OAuth token (skips if unavailable).
- **When:** `clp .usage touch::1 trace::1`
- **Then:** Exits 0. stderr contains `switch_account`, proving a subprocess IS spawned for the idle account. This test (`it110`) does NOT re-query `.usage` after the touch and does not itself assert the 5h Reset column's post-touch value. The reset-column transition (`—` → concrete countdown, e.g. `"in Xh Ym"`) is proven by a separate, previously-uncited test in the same file — `it111_lim_it_touch_1_5h_reset_changes_from_dash_to_time` — which independently pre-checks idle state via a bare `.usage` run, then runs `.usage touch::1` and asserts stdout contains `"in "` (countdown text).
- **Exit:** 0
- **Live:** yes (requires live quota data with idle account)
- **Source fn:** `it110_lim_it_touch_1_subprocess_spawned_for_idle_account` (subprocess-spawn evidence); `it111_lim_it_touch_1_5h_reset_changes_from_dash_to_time` (reset-column-transition evidence) — both in `usage_touch_test.rs`
- **Source:** [feature/024_session_touch.md AC-01, AC-03](../../../../docs/feature/024_session_touch.md)
