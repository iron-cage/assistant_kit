# Test: `solo::` Parameter

Edge case coverage for the `solo::` bool param on `.usage`.
See [param/060_solo.md](../../../../docs/cli/param/060_solo.md) for specification.

`solo::1` restricts all credential-consuming operations to the current+owned account. All other accounts display approximated historical data via `approximate_quota()`.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `solo::0` (default) — all owned accounts live-fetched; exits 0 | Default |
| EC-2 | `solo::1` current+owned account gets live data; non-current owned account gets approximated data | Core Gate |
| EC-3 | `solo::1` current account NOT owned — all accounts approximated; zero HTTP calls | Edge: No Qualifying Account |
| EC-4 | `solo::1` no active marker — all accounts approximated; zero HTTP calls | Edge: No Current |
| EC-5 | `solo::1 rotate::1` — mutual exclusion; exits 1 before fetch | Mutual Exclusion |
| EC-6 | `solo::1 live::1` — allowed; loop runs; only current+owned live-fetched per cycle | Composition |
| EC-7 | `solo::1 refresh::1` — refresh fires only for current+owned; other errors preserved | Composition |
| EC-8 | `solo::1 touch::1` — touch fires only for current+owned; other idle accounts untouched | Composition |
| EC-9 | `solo::1 only_active::1` — orthogonal; fetch gate and display filter compose independently | Composition |
| EC-10 | `solo::1 trace::1` — trace lines show `solo-skip: approximated` for non-current accounts | Trace |
| EC-11 | `solo::true` rejected — non-integer value on `Kind::Integer` param; exits 1 | Type Rejection |
| EC-12 | `solo::2` rejected — integer outside {0, 1}; exits 1 | Out-of-Range |

---

### EC-1: `solo::0` (default) — all owned accounts live-fetched; exits 0

- **Given:** A credential store with two owned accounts.
- **When:** `clp .usage solo::0`
- **Then:** Exits 0. Both accounts show live quota data (not approximated). Equivalent to omitting `solo::`.
- **Exit:** 0
- **Source fn:** `it257_solo_default_off_exits_0` (in `tests/cli/usage_test.rs`); `solo_field_default_false` (in `src/usage/params.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-2: `solo::1` current+owned account gets live data; non-current owned gets approximated

- **Given:** Two owned accounts. Account A is current+owned. Account B is owned but not current. Both have cached quota data.
- **When:** `clp .usage solo::1`
- **Then:** Exits 0. Account A row shows live quota data (fresh HTTP fetch). Account B row shows approximated data from `approximate_quota()` (cache values, no HTTP call). Both rows appear in the table.
- **Exit:** 0
- **Source fn:** `it258_solo_current_live_noncurrent_approx` (in `tests/cli/usage_test.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-3: `solo::1` current account NOT owned — all accounts approximated

- **Given:** Two accounts. Current account has `owner` field set to a different identity. Second account is owned but not current.
- **When:** `clp .usage solo::1`
- **Then:** Exits 0. No account passes both `is_current && is_owned`. All rows show approximated data. Zero HTTP calls made.
- **Exit:** 0
- **Source fn:** `it259_solo_current_not_owned_no_http` (in `tests/cli/usage_test.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-4: `solo::1` no active marker — all accounts approximated

- **Given:** Two owned accounts but no `_active_*` marker file exists (no current account).
- **When:** `clp .usage solo::1`
- **Then:** Exits 0. `is_current` is `false` for all accounts. All rows show approximated data. Zero HTTP calls.
- **Exit:** 0
- **Source fn:** `it260_solo_no_active_marker_all_approx` (in `tests/cli/usage_test.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-5: `solo::1 rotate::1` — mutual exclusion; exits 1 before fetch

- **Given:** Any environment.
- **When:** `clp .usage solo::1 rotate::1`
- **Then:** Exits 1. Error message references both `"solo"` and `"rotate"`. No table rendered (exits before fetch).
- **Exit:** 1
- **Source fn:** `it261_solo_rotate_mutual_exclusion_exit_1` (in `tests/cli/usage_test.rs`); `ec5_solo_and_rotate_mutual_exclusion` (in `src/usage/params.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-6: `solo::1 live::1` — allowed; only current+owned live-fetched per cycle

- **Given:** Two owned accounts. Account A is current+owned. Account B is owned but not current.
- **When:** `clp .usage solo::1 live::1 interval::30`
- **Then:** Loop starts. Each cycle: Account A gets live HTTP fetch; Account B shows approximated data. Both rows appear in every cycle's table. Ctrl-C exits cleanly.
- **Exit:** 0 (after Ctrl-C)
- **Source fn:** `it262_solo_live_composition_allowed` (in `tests/cli/usage_test.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-7: `solo::1 refresh::1` — refresh fires only for current+owned

- **Given:** Two owned accounts both with auth errors (401). Account A is current+owned. Account B is owned but not current.
- **When:** `clp .usage solo::1 refresh::1`
- **Then:** Refresh subprocess fires only for Account A. Account B retains its error state (no refresh attempted). Account B's error data is from `approximate_quota()`.
- **Exit:** 0
- **Source fn:** `it263_solo_refresh_composition_allowed` (in `tests/cli/usage_test.rs`); `ec7_solo_gate_skips_non_current_with_trace` (in `src/usage/refresh_tests.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-8: `solo::1 touch::1` — touch fires only for current+owned

- **Given:** Two owned accounts both with idle 5h windows. Account A is current+owned. Account B is owned but not current.
- **When:** `clp .usage solo::1 touch::1`
- **Then:** Touch subprocess fires only for Account A. Account B remains idle (no subprocess spawned).
- **Exit:** 0
- **Source fn:** `it264_solo_touch_composition_allowed` (in `tests/cli/usage_test.rs`); `ec8_solo_gate_skips_non_current_with_trace` (in `src/usage/touch_tests.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-9: `solo::1 only_active::1` — orthogonal composition

- **Given:** Two owned accounts. Account A is current+owned+active.
- **When:** `clp .usage solo::1 only_active::1`
- **Then:** Exits 0. Only Account A's row displayed (display filter). Account A has live data (solo allows). The two params compose independently — solo controls fetch, only_active controls display.
- **Exit:** 0
- **Source fn:** `it265_solo_only_active_composition_allowed` (in `tests/cli/usage_test.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-10: `solo::1 trace::1` — trace lines show `solo-skip: approximated`

- **Given:** Two owned accounts. Account A is current+owned. Account B is not current.
- **When:** `clp .usage solo::1 trace::1`
- **Then:** Stderr contains `solo-skip: approximated` for Account B's fetch trace. Account A's trace shows `live (current+owned)` or similar. Refresh and touch traces for Account B show `solo-skip`.
- **Exit:** 0
- **Source fn:** `it266_solo_trace_shows_solo_skip` (in `tests/cli/usage_test.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-11: `solo::true` rejected — non-integer value on `Kind::Integer` param

- **Given:** Any environment.
- **When:** `clp .usage solo::true`
- **Then:** Exits 1. `solo::` is registered as `Kind::Integer`; `"true"` is not a valid integer literal.
- **Exit:** 1
- **Source fn:** `it267_solo_true_rejected_type_error` (in `tests/cli/usage_test.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)

---

### EC-12: `solo::2` rejected — integer outside {0, 1}; exits 1

- **Given:** Any environment.
- **When:** `clp .usage solo::2`
- **Then:** Exits 1. `parse_int_flag` rejects integers outside `{0, 1}`.
- **Exit:** 1
- **Source fn:** `it268_solo_2_rejected_out_of_range` (in `tests/cli/usage_test.rs`); `ec12_solo_rejects_integer_two` (in `src/usage/params.rs`)
- **Source:** [param/060_solo.md](../../../../docs/cli/param/060_solo.md)
