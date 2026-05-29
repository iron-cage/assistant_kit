# Test: `exclude_exhausted::` Parameter

Edge case coverage for the `exclude_exhausted::` parameter on `.usage`. See [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `exclude_exhausted::1` shows only 🟢 rows | Behavioral Divergence |
| EC-2 | `exclude_exhausted::0` (default) shows all rows | Behavioral Divergence |
| EC-3 | `exclude_exhausted::1` is stricter than `only_valid::1` (also removes 🟡) | Comparison |
| EC-4 | `exclude_exhausted::bad` exits 1 naming valid values | Invalid Value |
| EC-5 | `exclude_exhausted::1` with all 🔴 accounts shows 0 rows | Empty Result |
| EC-6 | `exclude_exhausted::true` accepted (alias for 1) | Alias Acceptance |

---

### EC-1: `exclude_exhausted::1` shows only 🟢 rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage exclude_exhausted::1`
- **Then:** Exits 0. Only 🟢 row shown; both 🟡 and 🔴 rows hidden.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-2: `exclude_exhausted::0` shows all rows

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage exclude_exhausted::0`
- **Then:** Exits 0. All rows shown.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-3: `exclude_exhausted::1` removes 🟡 while `only_valid::1` keeps 🟡

- **Given:** Two accounts: one 🟢, one 🟡.
- **When-A:** `clp .usage only_valid::1`
- **When-B:** `clp .usage exclude_exhausted::1`
- **Then-A:** Both rows shown (🟡 passes `only_valid::1`).
- **Then-B:** Only 🟢 row shown (🟡 hidden by `exclude_exhausted::1`).
- **Exit:** 0 both
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-4: `exclude_exhausted::bad` exits 1 naming valid values

- **Given:** Any environment.
- **When:** `clp .usage exclude_exhausted::bad`
- **Then:** Exits 1. Stderr names valid values: `0`, `1`, `false`, `true`.
- **Exit:** 1
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-5: `exclude_exhausted::1` with all 🔴 accounts shows 0 rows

- **Given:** Two accounts; both are 🔴.
- **When:** `clp .usage exclude_exhausted::1`
- **Then:** Exits 0. Table has 0 data rows (all filtered). No error.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)

---

### EC-6: `exclude_exhausted::true` accepted as alias for 1

- **Given:** Three accounts: one 🟢, one 🟡, one 🔴.
- **When:** `clp .usage exclude_exhausted::true`
- **Then:** Exits 0. Only 🟢 row shown — same result as `exclude_exhausted::1`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/044_exclude_exhausted.md](../../../../docs/cli/param/044_exclude_exhausted.md)
