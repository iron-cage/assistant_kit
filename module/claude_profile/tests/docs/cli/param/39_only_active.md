# Test: `only_active::` Parameter

Edge case coverage for the `only_active::` parameter on `.usage`. See [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `only_active::1` shows exactly the active account row | Behavioral Divergence |
| EC-2 | `only_active::0` (default) shows all rows | Behavioral Divergence |
| EC-3 | `only_active::bad` exits 1 naming valid values | Invalid Value |
| EC-4 | `only_active::1` with no active account shows 0 rows | Empty Result |
| EC-5 | `only_active::true` accepted (alias for 1) | Alias Acceptance |
| EC-6 | `only_active::false` accepted (alias for 0) | Alias Acceptance |

---

### EC-1: `only_active::1` shows exactly the active account row

- **Given:** Three accounts; one is the active account.
- **When:** `clp .usage only_active::1`
- **Then:** Exits 0. Exactly one row in the table body — the active account.
- **Exit:** 0
- **Source fn:** `it154_only_active_1_shows_active_account_row` (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)

---

### EC-2: `only_active::0` shows all rows

- **Given:** Three accounts.
- **When:** `clp .usage only_active::0`
- **Then:** Exits 0. All rows shown (default behavior).
- **Exit:** 0
- **Source fn:** `it155_only_active_0_shows_all_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)

---

### EC-3: `only_active::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage only_active::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** `it156_only_active_bad_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)

---

### EC-4: `only_active::1` with no active account shows 0 rows

- **Given:** Three accounts; none is marked active.
- **When:** `clp .usage only_active::1`
- **Then:** Exits 0. Table has 0 data rows (no active account to show). No error.
- **Exit:** 0
- **Source fn:** `it157_only_active_1_no_active_marker_shows_empty` (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)

---

### EC-5: `only_active::true` accepted as alias for 1

- **Given:** Two accounts; one is active.
- **When:** `clp .usage only_active::true`
- **Then:** Exits 0. Exactly one row shown — same result as `only_active::1`.
- **Exit:** 0
- **Source fn:** `it158_only_active_true_accepted` (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)

---

### EC-6: `only_active::false` accepted as alias for 0

- **Given:** Two accounts.
- **When:** `clp .usage only_active::false`
- **Then:** Exits 0. Both rows shown — same result as `only_active::0`.
- **Exit:** 0
- **Source fn:** `it159_only_active_false_shows_all_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [param/039_only_active.md](../../../../docs/cli/param/039_only_active.md)
