# Test: `offset::` Parameter

Edge case coverage for the `offset::` parameter on `.usage`. See [param/038_offset.md](../../../../docs/cli/param/038_offset.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `offset::2` skips first 2 rows | Behavioral Divergence |
| EC-2 | `offset::0` (default) shows from first row | Behavioral Divergence |
| EC-3 | `offset::99` with 2 accounts shows 0 rows, exits 0 | Over-offset |
| EC-4 | `offset::abc` exits 1 with type error | Invalid Value |
| EC-5 | `count::1 offset::1 sort::name` shows second row only | Pagination Composition |
| EC-6 | `offset::-1` exits 1 (negative integer rejected) | Invalid Value |

---

### EC-1: `offset::2` skips first 2 rows

- **Given:** Four accounts; `sort::name` for deterministic order.
- **When-A:** `clp .usage sort::name`
- **When-B:** `clp .usage sort::name offset::2`
- **Then:** When-B rows match rows 3–4 from When-A.
- **Exit:** 0
- **Source fn:** `it184_offset_2_skips_first_2_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-2: `offset::0` shows from first row

- **Given:** Three accounts.
- **When:** `clp .usage offset::0`
- **Then:** Exits 0. All rows shown (offset::0 = no skip).
- **Exit:** 0
- **Source fn:** `it185_offset_0_shows_all_rows` (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-3: `offset::99` with 2 accounts shows 0 data rows

- **Given:** Two accounts.
- **When:** `clp .usage offset::99`
- **Then:** Exits 0. Table has 0 data rows (offset exceeds account count). Header still shown.
- **Exit:** 0
- **Source fn:** `it186_offset_99_shows_empty` (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-4: `offset::abc` exits 1 with type error

- **Given:** Any environment.
- **When:** `clp .usage offset::abc`
- **Then:** Exits 1. Stderr contains a type error message.
- **Exit:** 1
- **Source fn:** `it187_offset_abc_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-5: `count::1 offset::1 sort::name` shows second row only

- **Given:** Three accounts with deterministic name sort (sorted: `alice`, `bob`, `charlie`).
- **When:** `clp .usage count::1 offset::1 sort::name`
- **Then:** Exits 0. Exactly 1 row shown — `bob` (second alphabetically). `alice` skipped by offset; `charlie` excluded by count.
- **Exit:** 0
- **Source fn:** `it188_offset_1_count_1_shows_second_row` (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)

---

### EC-6: `offset::-1` exits 1 (negative value rejected)

- **Given:** Any environment.
- **When:** `clp .usage offset::-1`
- **Then:** Exits 1. Stderr indicates value must be a non-negative integer.
- **Exit:** 1
- **Source fn:** `it189_offset_minus_1_exits_1` (in `tests/cli/usage_test.rs`)
- **Source:** [param/038_offset.md](../../../../docs/cli/param/038_offset.md)
