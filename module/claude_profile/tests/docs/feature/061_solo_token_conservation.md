# FT — Feature 061: Solo Token Conservation Mode

### Scope

- **Purpose**: Test cases for the solo gate predicate, `approximate_quota()` invocation, composition behaviors, and trace output.
- **Source**: `docs/feature/061_solo_token_conservation.md`
- **Covers**: AC-01 through AC-12

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `solo::1` current+owned gets live fetch; non-current owned gets `approximate_quota()` | ✅ `it258_solo_current_live_noncurrent_approx` |
| FT-02 | AC-02 | Refresh gate skips non-current under `solo::1` | ✅ `ec7_solo_gate_skips_non_current_with_trace` |
| FT-03 | AC-03 | Touch gate skips non-current under `solo::1` | ✅ `ec8_solo_gate_skips_non_current_with_trace` |
| FT-04 | AC-04 | `solo::0` (default) — all owned accounts live-fetched; exits 0 | ✅ `it257_solo_default_off_exits_0` |
| FT-05 | AC-05 | Current account not owned — all accounts approximated; zero HTTP calls | ✅ `it259_solo_current_not_owned_no_http` |
| FT-06 | AC-06 | No active marker — all accounts approximated; zero HTTP calls | ✅ `it260_solo_no_active_marker_all_approx` |
| FT-07 | AC-07 | `solo::1 rotate::1` mutual exclusion — exits 1 before fetch | ✅ `it261_solo_rotate_mutual_exclusion_exit_1` |
| FT-08 | AC-08 | `solo::1 live::1` allowed — loop runs, only current+owned fetched per cycle | ✅ `it262_solo_live_composition_allowed` |
| FT-09 | AC-09 | `solo::1 refresh::1` allowed — refresh only for current+owned | ✅ `it263_solo_refresh_composition_allowed` |
| FT-10 | AC-10 | `solo::1 touch::1` allowed — touch only for current+owned | ✅ `it264_solo_touch_composition_allowed` |
| FT-11 | AC-11 | `solo::1 only_active::1` — fetch gate and display filter compose independently | ✅ `it265_solo_only_active_composition_allowed` |
| FT-12 | AC-12 | `solo::1 trace::1` — trace lines show `solo-skip` for non-current accounts | ✅ `it266_solo_trace_shows_solo_skip` |

### Notes

- FT-01, FT-04, FT-05, FT-06, FT-07, FT-08, FT-09, FT-10, FT-11, FT-12 are integration tests in `tests/cli/usage_test.rs`.
- FT-02 is a unit test in `tests/usage/refresh_tests_b.rs` (solo gate isolation at the refresh site).
- FT-03 is a unit test in `tests/usage/touch_tests_b.rs` (solo gate isolation at the touch site).
- Parameter-level unit tests (`ec5_solo_and_rotate_mutual_exclusion`, `ec12_solo_rejects_integer_two`) in `src/usage/params.rs` complement FT-07 at the unit level.
- `approximate_quota()` behavior is validated indirectly through FT-01, FT-05, FT-06 — all verify that approximated data (not raw cache) is displayed when the solo gate fires.

---

### FT-01: Current+owned gets live fetch; non-current gets approximate_quota()

- **Given:** Two owned accounts. Account A is `is_current && is_owned`. Account B is owned but not current. Both have `cache.history[]` entries.
- **When:** `clp .usage solo::1`
- **Then:** Account A shows live quota data (HTTP fetch performed). Account B shows approximated data returned by `approximate_quota()` — no HTTP call to Account B. Both rows appear in the table.
- **Exit:** 0
- **Source fn:** `it258_solo_current_live_noncurrent_approx` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-01](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-02: Refresh gate skips non-current under solo::1

- **Given:** Account B is owned but not current, with auth error (401) and expired token (`expires_at_ms = 0`). Empty credential store (no cred files). `solo=true`.
- **When:** `apply_refresh()` is called with `solo=true`.
- **Then:** The solo gate fires before G2. Account B's result is unchanged (original 401 error preserved). Without solo, the same account would be refreshed — `should_refresh` returns true (401+expired), `refresh_account_token` returns None (empty store), result becomes `Err("refresh token expired")`. Converted from gag-based trace capture to behavioral test.
- **Exit:** result still contains "401"
- **Source fn:** `ec7_solo_gate_skips_non_current_with_trace` (in `tests/usage/refresh_tests_b.rs`)
- **Source:** [061_solo_token_conservation.md AC-02](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-03: Touch gate skips non-current under solo::1

- **Given:** Account A is current+owned; Account B is owned but not current. Both have idle 5h windows. `solo=true`, `touch=true`.
- **When:** `apply_touch()` is called with `solo=true` for Account B.
- **Then:** The solo gate fires before G4. `touch_skip_reason()` returns `Some("solo-skip")` for Account B — the same reason string `apply_touch()`'s trace would emit. No touch subprocess is spawned for Account B. Returns `Ok(())`. Converted from gag-based trace capture to a direct oracle call.
- **Exit:** Ok(())
- **Source fn:** `ec8_solo_gate_skips_non_current_with_trace` (in `tests/usage/touch_tests_b.rs`)
- **Source:** [061_solo_token_conservation.md AC-03](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-04: solo::0 (default) — all owned accounts live-fetched

- **Given:** A credential store with two owned accounts.
- **When:** `clp .usage solo::0`
- **Then:** Exits 0. Both accounts show live quota data. Behavior identical to omitting `solo::`.
- **Exit:** 0
- **Source fn:** `it257_solo_default_off_exits_0` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-04](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-05: Current account not owned — all accounts approximated

- **Given:** Two accounts. The current account has `owner` set to a foreign identity (`is_owned == false`). The second account is owned but not current.
- **When:** `clp .usage solo::1`
- **Then:** Exits 0. No account passes `is_current && is_owned`. All rows show approximated data. Zero HTTP calls made.
- **Exit:** 0
- **Source fn:** `it259_solo_current_not_owned_no_http` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-05](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-06: No active marker — all accounts approximated

- **Given:** Two owned accounts; no `_active_*` marker file exists (no current account on this machine).
- **When:** `clp .usage solo::1`
- **Then:** Exits 0. `is_current == false` for all accounts. All rows show approximated data. Zero HTTP calls.
- **Exit:** 0
- **Source fn:** `it260_solo_no_active_marker_all_approx` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-06](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-07: solo::1 + rotate::1 mutual exclusion — exits 1 before fetch

- **Given:** Any environment.
- **When:** `clp .usage solo::1 rotate::1`
- **Then:** Exits 1. Error message references both `"solo"` and `"rotate"`. No HTTP calls made (exits before fetch).
- **Exit:** 1
- **Source fn:** `it261_solo_rotate_mutual_exclusion_exit_1` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-07](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-08: solo::1 + live::1 — allowed composition

- **Given:** Two owned accounts. Account A is current+owned.
- **When:** `clp .usage solo::1 live::1 interval::30`
- **Then:** Live monitor loop starts. Each cycle: Account A gets live HTTP fetch; Account B shows approximated data. Both rows appear each cycle.
- **Exit:** 0 (after signal)
- **Source fn:** `it262_solo_live_composition_allowed` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-08](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-09: solo::1 + refresh::1 — allowed; refresh solo gate fires for non-current

- **Given:** Alice is current+owned (live token match). Bob is not current. `solo::1 refresh::1 trace::1`.
- **When:** `clp .usage solo::1 refresh::1 trace::1`
- **Then:** Exits 0. Refresh solo gate fires for Bob — stderr contains `"solo-skip"` in a refresh trace line. Alice passes the solo gate; no subprocess fires (no 401 from HTTP).
- **Exit:** 0
- **Source fn:** `it263_solo_refresh_composition_allowed` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-09](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-10: solo::1 + touch::1 — allowed; touch solo gate fires for non-current

- **Given:** Alice is current+owned (live token match). Bob is not current. `solo::1 touch::1 trace::1`.
- **When:** `clp .usage solo::1 touch::1 trace::1`
- **Then:** Exits 0. Touch solo gate fires for Bob — stderr contains `"solo-skip"` in a touch trace line. Alice passes the solo gate; no subprocess fires (no active idle window without real quota data).
- **Exit:** 0
- **Source fn:** `it264_solo_touch_composition_allowed` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-10](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-11: Display filters independent of solo

- **Given:** Two owned accounts. Account A is current+owned+active. Account B is owned but not current.
- **When:** `clp .usage solo::1 only_active::1`
- **Then:** Exits 0. Only Account A's row appears (display filter). Account A has live data (solo allows). The two params compose independently — solo controls fetch, `only_active::` controls display.
- **Exit:** 0
- **Source fn:** `it265_solo_only_active_composition_allowed` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-11](../../../docs/feature/061_solo_token_conservation.md)

---

### FT-12: Trace output for solo-skipped accounts — all three gate sites

- **Given:** Alice is current+owned (live token match). Bob is not current. `solo::1 refresh::1 touch::1 trace::1`.
- **When:** `clp .usage solo::1 refresh::1 touch::1 trace::1`
- **Then:** Stderr contains `solo-skip: approximated` in Bob's fetch trace line; `solo-skip` in Bob's refresh trace line; `solo-skip` in Bob's touch trace line. Alice's fetch trace shows normal live fetch (no `solo-skip`).
- **Exit:** 0
- **Source fn:** `it266_solo_trace_shows_solo_skip` (in `tests/cli/usage_test.rs`)
- **Source:** [061_solo_token_conservation.md AC-12](../../../docs/feature/061_solo_token_conservation.md)
