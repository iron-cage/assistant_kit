# Test: `abs::` Parameter

Edge case coverage for the `abs::` parameter on `.usage`. See [param/046_abs.md](../../../../docs/cli/param/046_abs.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `abs::1` accepted with empty credential store | Guard |
| EC-2 | `abs::0` (default) shows percentage values | Behavioral Divergence |
| EC-3 | `abs::bad` exits 1 naming valid values | Invalid Value |
| EC-4 | `abs::1` shows absolute token counts instead of percentages | Behavioral Divergence |
| EC-5 | `abs::1` on error row — error message shown unchanged | Error Row |
| EC-6 | `abs::true` accepted (alias for 1) | Alias Acceptance |

---

### EC-1: `abs::1` accepted with empty credential store

- **Given:** Empty credential store.
- **When:** `clp .usage abs::1`
- **Then:** Exits 0 with "(no accounts configured)". No error about unrecognized parameter.
- **Exit:** 0
- **Source fn:** `it194_abs_1_accepted_empty_store` (in `tests/cli/usage_test.rs`)
- **Source:** [param/046_abs.md](../../../../docs/cli/param/046_abs.md)

---

### EC-2: `abs::0` shows percentages

- **Given:** One account.
- **When:** `clp .usage abs::0`
- **Then:** Exits 0. 5h Left column shows percentage (e.g., `88%`). Default behavior.
- **Exit:** 0
- **Source fn:** `it195_abs_0_accepted` (in `tests/cli/usage_test.rs`)
- **Source:** [param/046_abs.md](../../../../docs/cli/param/046_abs.md)

---

### EC-3: `abs::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage abs::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `it196_abs_bad_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/046_abs.md](../../../../docs/cli/param/046_abs.md)

---

### EC-4: `abs::1` shows absolute token counts instead of percentages

- **Given:** One account with live quota.
- **When:** `clp .usage abs::1`
- **Then:** Exits 0. 5h Left column shows an absolute token count (e.g., `22000`) instead of a percentage string. No `%` suffix in quota columns.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it223_lim_it_abs_1_shows_token_counts` (in `tests/cli/usage_test.rs`)
- **Source:** [param/046_abs.md](../../../../docs/cli/param/046_abs.md)

---

### EC-5: `abs::1` on error row — error message shown unchanged

- **Given:** One account with expired token (🔴 row).
- **When:** `clp .usage abs::1`
- **Then:** Exits 0. Error row shown with error message in last column (unchanged by `abs::1`). `abs::` has no effect on `—` or error cells.
- **Exit:** 0
- **Source fn:** `it197_abs_1_on_error_row` (in `tests/cli/usage_test.rs`)
- **Source:** [param/046_abs.md](../../../../docs/cli/param/046_abs.md)

---

### EC-6: `abs::true` accepted as alias for 1

- **Given:** One account with live quota.
- **When:** `clp .usage abs::true`
- **Then:** Exits 0. Quota columns show absolute counts — same result as `abs::1`.
- **Exit:** 0
- **Source fn:** `it224_lim_it_abs_true_shows_token_counts` (in `tests/cli/usage_test.rs`)
- **Source:** [param/046_abs.md](../../../../docs/cli/param/046_abs.md)
