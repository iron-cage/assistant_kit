# Parameter :: `sort`

Edge case tests for the `sort` parameter. Tests validate the default
field and combination with `reverse`.

**Source:** [param/11_sort.md](../../../../docs/cli/param/11_sort.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> sorted by time (chronological) | Default | ✅ | `viewer_integration_test.rs::ec30_sort_orders_by_every_documented_field` |
| EC-2 | `sort::cost` -> ascending by default | Parsing | ✅ | `viewer_integration_test.rs::ec30_sort_orders_by_every_documented_field` |
| EC-3 | `sort::cost reverse::1` -> most expensive first | Combined | ✅ | `viewer_integration_test.rs::ec30_sort_orders_by_every_documented_field` |
| EC-4 | `sort::popularity` -> exit 1 listing the valid fields | Error Handling | ✅ | `viewer_integration_test.rs::ec31_sort_case_insensitive_and_invalid_values_exit_1` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Combined: 1 test (EC-3)
- Error Handling: 1 test (EC-4)

**Total:** 4 edge cases (4 executable)

EC-1 is only meaningful against a fixture whose append order differs from every
other sort field's order — otherwise a `sort::` that silently ignored its
argument would satisfy it. `write_sortable_events` is built that way on purpose.

## Test Cases

---

### EC-1: Absent -> sorted by time (chronological)

- **Given:** journal with events at different timestamps
- **When:** `clj .list`
- **Then:** exit 0; events are ordered chronologically by time
- **Exit:** 0
- **Source:** [param/11_sort.md](../../../../docs/cli/param/11_sort.md)

---

### EC-2: `sort::cost` -> ascending by default

- **Given:** journal with events of varying cost
- **When:** `clj .list sort::cost`
- **Then:** exit 0; events are ordered cheapest first
- **Exit:** 0
- **Source:** [param/11_sort.md](../../../../docs/cli/param/11_sort.md)

---

### EC-3: `sort::cost reverse::1` -> most expensive first

- **Given:** journal with events of varying cost
- **When:** `clj .list sort::cost reverse::1`
- **Then:** exit 0; events are ordered most expensive first
- **Exit:** 0
- **Source:** [param/11_sort.md](../../../../docs/cli/param/11_sort.md)

---

### EC-4: `sort::popularity` -> exit 1 listing the valid fields

- **Given:** clean environment
- **When:** `clj .list sort::popularity`
- **Then:** exit 1; stderr names all six valid fields
- **And:** the command does **not** fall back to the default sort and exit 0 — a
  silent fallback is indistinguishable from a sort that worked
- **Exit:** 1
- **Source:** [param/11_sort.md](../../../../docs/cli/param/11_sort.md), [type/07_sort_field.md](../../../../docs/cli/type/07_sort_field.md)
