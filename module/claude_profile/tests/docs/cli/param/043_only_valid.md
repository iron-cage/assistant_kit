# Test: `only_valid::` Parameter

Edge case coverage for the `only_valid::` parameter on `.usage`. See [param/043_only_valid.md](../../../../docs/cli/param/043_only_valid.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `only_valid::1` hides 🔴 rows; shows 🟢 and 🟡 | Behavioral Divergence |
| EC-2 | `only_valid::0` (default) shows all rows | Behavioral Divergence |
| EC-3 | `only_valid::bad` exits 1 naming valid values | Invalid Value |
| EC-4 | `only_valid::1` with all 🔴 accounts shows 0 rows | Empty Result |
| EC-5 | `only_valid::true` accepted (alias for 1) | Alias Acceptance |
| EC-6 | `only_valid::false` accepted (alias for 0) | Alias Acceptance |

---

### EC-1: `only_valid::1` hides 🔴 rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage only_valid::1`
- **Then:** Exits 0. 🟢 and 🟡 rows shown; 🔴 row hidden.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/043_only_valid.md](../../../../docs/cli/param/043_only_valid.md)

---

### EC-2: `only_valid::0` shows all rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage only_valid::0`
- **Then:** Exits 0. All rows shown.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/043_only_valid.md](../../../../docs/cli/param/043_only_valid.md)

---

### EC-3: `only_valid::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage only_valid::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/043_only_valid.md](../../../../docs/cli/param/043_only_valid.md)

---

### EC-4: `only_valid::1` with all 🔴 accounts shows 0 rows

- **Given:** Two accounts; both are 🔴 (expired token or network error).
- **When:** `clp .usage only_valid::1`
- **Then:** Exits 0. Table has 0 data rows (all filtered). No error.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/043_only_valid.md](../../../../docs/cli/param/043_only_valid.md)

---

### EC-5: `only_valid::true` accepted as alias for 1

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage only_valid::true`
- **Then:** Exits 0. 🟢 and 🟡 rows shown — same result as `only_valid::1`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/043_only_valid.md](../../../../docs/cli/param/043_only_valid.md)

---

### EC-6: `only_valid::false` accepted as alias for 0

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage only_valid::false`
- **Then:** Exits 0. All rows shown — same result as `only_valid::0`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/043_only_valid.md](../../../../docs/cli/param/043_only_valid.md)
