# Test: `touch::` Parameter

Edge case coverage for the `touch::` parameter on `.usage`. See [param/034_touch.md](../../../../docs/cli/param/034_touch.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `touch::0` accepted with empty credential store (default) | Valid Value |
| EC-2 | `touch::1` accepted with empty credential store | Valid Value |
| EC-3 | `touch::true` accepted with empty credential store | Valid Value |
| EC-4 | `touch::bogus` exits 1 (invalid value) | Invalid Value |
| EC-5 | `touch::1` with errored-quota account — errored accounts are never touched | Trigger Guard |
| EC-6 | `touch::1 format::json` — `touch::` does not affect JSON output structure | JSON No-op |
| EC-7 | `touch::0` with idle account — no subprocess spawned, 5h Reset stays `—` | Behavioral Divergence |
| EC-8 | `touch::1` with idle account — subprocess spawned, 5h Reset changes (Behavioral Divergence B) | Behavioral Divergence |

---

### EC-1: `touch::0` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage touch::0`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned.
- **Exit:** 0
- **Source fn:** `it096_touch_0_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-2: `touch::1` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter. No subprocess spawned (no accounts to touch).
- **Exit:** 0
- **Source fn:** `it087_touch_1_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-3: `touch::true` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage touch::true`
- **Then:** Exits 0 with "(no accounts configured)". `true` is accepted as equivalent to `1`.
- **Exit:** 0
- **Source fn:** `it097_touch_true_accepted_empty_store_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-4: `touch::bogus` exits 1 (invalid value)

- **Given:** Any environment (empty credential store).
- **When:** `clp .usage touch::bogus`
- **Then:** Exits 1. Stderr indicates invalid value for `touch::`.
- **Exit:** 1
- **Source fn:** `it098_touch_bogus_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/034_touch.md](../../../../docs/cli/param/034_touch.md)

---

### EC-5: `touch::1` with errored-quota account — errored accounts are never touched

- **Given:** One saved account whose credential file has no `accessToken` (quota fetch fails with Err — not a successful result with missing `resets_at`).
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. No subprocess spawned for the errored account. Account row shows original error state unchanged. Touch trigger requires `result = Ok(...)` — Err accounts are never touched.
- **Exit:** 0
- **Source fn:** `it088_touch_1_errored_account_skipped` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-04](../../../../docs/feature/024_session_touch.md)

---

### EC-6: `touch::1 format::json` — `touch::` does not affect JSON output structure

- **Given:** One saved account with valid token and quota data (any `five_hour.resets_at` state).
- **When-A:** `clp .usage format::json`
- **When-B:** `clp .usage touch::1 format::json`
- **Then-A and Then-B:** Both produce JSON arrays with identical schema. `touch::` does not add or remove fields from JSON objects.
- **Exit:** 0 both cases
- **Source fn:** `it090_touch_json_format_unaffected` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-08](../../../../docs/feature/024_session_touch.md)

---

### EC-7: `touch::0` with idle account — no subprocess spawned (Behavioral Divergence A)

- **Given:** One saved account with valid token and quota data where `five_hour.resets_at` is `None` (idle state — 5h window has not started).
- **When:** `clp .usage touch::0`
- **Then:** Exits 0. No subprocess spawned. The 5h Reset column shows `—` (unchanged idle state). `touch::0` disables the touch trigger regardless of account state.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source fn:** `it099_lim_it_touch_0_no_subprocess_idle_account_unchanged` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01](../../../../docs/feature/024_session_touch.md)

---

### EC-8: `touch::1` with idle account — subprocess spawned (Behavioral Divergence B)

- **Given:** Same account as EC-7: valid token, `five_hour.resets_at = None` (idle state). Neither is current.
- **When:** `clp .usage touch::1`
- **Then:** Exits 0. A subprocess IS spawned for the idle account (touch trigger fires: `result = Ok(...)` AND `resets_at = None`). After the subprocess, quota is re-fetched; the 5h Reset column changes from `—` to a concrete countdown. Divergence from EC-7: the SAME idle account produces DIFFERENT output under `touch::0` vs `touch::1`, proving the parameter governs subprocess dispatch.
- **Exit:** 0
- **Live:** yes (requires live quota data)
- **Source fn:** `it100_lim_it_touch_1_subprocess_spawned_for_idle_account` (in `tests/cli/usage_test.rs`)
- **Source:** [feature/024_session_touch.md AC-01, AC-03](../../../../docs/feature/024_session_touch.md)
