# Test: `offset::` Parameter

Edge case coverage for the `offset::` parameter on `.usage`. See [param/038_offset.md](../../../../docs/cli/param/038_offset.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `offset::2` skips first 2 rows | Row Skip |
| EC-2 | `offset::0` (default) shows from first row | No Skip |
| EC-3 | `offset::99` with 2 accounts shows 0 rows, exits 0 | Over-offset |
| EC-4 | `offset::abc` exits 1 with type error | Invalid Value |

---

### EC-1: `offset::2` skips first 2 rows

- **Given:** Four accounts; `sort::name` for deterministic order.
- **When-A:** `clp .usage sort::name`
- **When-B:** `clp .usage sort::name offset::2`
- **Then:** When-B rows match rows 3–4 from When-A.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-2: `offset::0` shows from first row

- **Given:** Three accounts.
- **When:** `clp .usage offset::0`
- **Then:** Exits 0. All rows shown (offset::0 = no skip).
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-3: `offset::99` with 2 accounts shows 0 data rows

- **Given:** Two accounts.
- **When:** `clp .usage offset::99`
- **Then:** Exits 0. Table has 0 data rows (offset exceeds account count). Header still shown.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-4: `offset::abc` exits 1 with type error

- **Given:** Any environment.
- **When:** `clp .usage offset::abc`
- **Then:** Exits 1. Stderr contains a type error message.
- **Exit:** 1
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)
