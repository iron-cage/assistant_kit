# Test: `rotate::` Parameter

Edge case coverage for the `rotate::` bool param on `.usage`.
See [param/059_rotate.md](../../../../docs/cli/param/059_rotate.md) for specification.

`rotate::1` executes an account switch to the `→` winner after the quota table is rendered.
The switch uses the same account selected by `find_next_for_strategy()`.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `rotate::0` (default) — display-only; no switch; exits 0 | Default |
| EC-2 | `rotate::1` empty store → param accepted; exits 1 "no eligible" | Valid Value |
| EC-3 | `rotate::1 live::1` → mutual exclusion; exits 1 before table | Mutual Exclusion |
| EC-4 | `rotate::1 dry::1` → preview message; credentials unchanged; exits 0 | Dry-run Interaction |
| EC-5 | `rotate::true` rejected — non-integer value on `Kind::Integer` param; exits 1 | Type Rejection |
| EC-6 | `rotate::false` rejected — non-integer value on `Kind::Integer` param; exits 1 | Type Rejection |
| EC-7 | `rotate::2` rejected — integer outside {0, 1}; exits 1 | Out-of-Range |

---

### EC-1: `rotate::0` (default) — display-only; no switch; exits 0

- **Given:** A credential store with one account.
- **When:** `clp .usage rotate::0`
- **Then:** Exits 0. Quota table rendered normally. Credentials unchanged. No "switched to" message.
- **Exit:** 0
- **Note:** Equivalent to omitting `rotate::` entirely. `rotate::0` is the default and must be accepted without error.
- **Source:** [param/059_rotate.md](../../../../docs/cli/param/059_rotate.md)

---

### EC-2: `rotate::1` with empty credential store → param accepted; exits 1 "no eligible"

- **Given:** Empty credential store (no accounts saved).
- **When:** `clp .usage rotate::1`
- **Then:** Exits 1. Table rendered (empty). Output contains `"no eligible account"` or equivalent. No crash, no "unrecognized parameter" error.
- **Exit:** 1
- **Source fn:** `ft03_no_eligible_account_exits_1` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [param/059_rotate.md](../../../../docs/cli/param/059_rotate.md)

---

### EC-3: `rotate::1 live::1` → mutual exclusion; exits 1 before table

- **Given:** Any environment (empty credential store acceptable).
- **When:** `clp .usage rotate::1 live::1`
- **Then:** Exits 1. Stderr (or combined output) references both `"rotate"` and `"live"` in the error message. No table rendered (exits before fetch).
- **Exit:** 1
- **Source fn:** `ft04_rotate_live_mutual_exclusion` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-04](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### EC-4: `rotate::1 dry::1` → preview message; credentials unchanged; exits 0

- **Given:** Two owned accounts; one is the `sort::renew` winner. Live API credentials.
- **When:** `clp .usage rotate::1 dry::1`
- **Then:** Exits 0 (when an eligible account exists). Output contains `[dry-run] would switch to '{name}'`. Credentials file unchanged (mtime/content identical before and after).
- **Exit:** 0
- **Live:** yes (requires real quota data for `→` winner selection)
- **Source fn:** `ft02_lim_it_dry_run_no_switch` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [feature/038_usage_strategy_rotate.md AC-02](../../../../docs/feature/038_usage_strategy_rotate.md)

---

### EC-5: `rotate::true` rejected — non-integer value on `Kind::Integer` param

- **Given:** Any environment.
- **When:** `clp .usage rotate::true`
- **Then:** Exits 1. `rotate::` is registered as `Kind::Integer`; `"true"` is not a valid integer literal, so the framework rejects it before the routine runs.
- **Exit:** 1
- **Note:** The production spec (`059_rotate.md`) defines valid values as `0` and `1`. `true`/`false` are not accepted because the param uses `Kind::Integer`, not `Kind::Boolean`. Compare with `bfd()` params (e.g., `dry::`) which use `Kind::Boolean` and accept both forms.
- **Source fn:** `ec05_rotate_true_rejected_not_integer` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [param/059_rotate.md](../../../../docs/cli/param/059_rotate.md)

---

### EC-6: `rotate::false` rejected — non-integer value on `Kind::Integer` param

- **Given:** Any environment.
- **When:** `clp .usage rotate::false`
- **Then:** Exits 1. Same reason as EC-5: `"false"` is not a valid integer literal.
- **Exit:** 1
- **Source fn:** `ec06_rotate_false_rejected_not_integer` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [param/059_rotate.md](../../../../docs/cli/param/059_rotate.md)

---

### EC-7: `rotate::2` rejected — integer outside {0, 1}; exits 1

- **Given:** Any environment.
- **When:** `clp .usage rotate::2`
- **Then:** Exits 1. The framework accepts `2` as a valid integer (`Kind::Integer`), but `parse_int_flag` rejects integers outside `{0, 1}`. Error message: `"rotate:: must be 0, 1, false, or true"`.
- **Exit:** 1
- **Source fn:** `ec07_rotate_2_rejected_out_of_range` (in `tests/cli/usage_rotate_test.rs`)
- **Source:** [param/059_rotate.md](../../../../docs/cli/param/059_rotate.md)
