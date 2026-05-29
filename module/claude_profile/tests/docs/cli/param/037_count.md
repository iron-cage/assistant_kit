# Test: `count::` Parameter

Edge case coverage for the `count::` parameter on `.usage`. See [param/037_count.md](../../../../docs/cli/param/037_count.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `count::3` with 5 accounts shows at most 3 rows | Row Limit |
| EC-2 | `count::0` (default) shows all rows | No Limit |
| EC-3 | `count::100` with 2 accounts shows all 2 rows | Over-count |
| EC-4 | `count::abc` exits 1 with type error | Invalid Value |

---

### EC-1: `count::3` shows at most 3 rows

- **Given:** Five or more accounts.
- **When:** `clp .usage count::3`
- **Then:** Exits 0. Table body has at most 3 data rows. Header and footer shown.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/037_count.md](../../../../docs/cli/param/037_count.md)

---

### EC-2: `count::0` shows all rows

- **Given:** Three accounts.
- **When:** `clp .usage count::0`
- **Then:** Exits 0. All 3 rows shown (count::0 = no limit).
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/037_count.md](../../../../docs/cli/param/037_count.md)

---

### EC-3: `count::100` with 2 accounts shows both rows

- **Given:** Two accounts.
- **When:** `clp .usage count::100`
- **Then:** Exits 0. Both rows shown (count exceeds available rows).
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/037_count.md](../../../../docs/cli/param/037_count.md)

---

### EC-4: `count::abc` exits 1 with type error

- **Given:** Any environment.
- **When:** `clp .usage count::abc`
- **Then:** Exits 1. Stderr contains a type error message (expected non-negative integer).
- **Exit:** 1
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/037_count.md](../../../../docs/cli/param/037_count.md)
