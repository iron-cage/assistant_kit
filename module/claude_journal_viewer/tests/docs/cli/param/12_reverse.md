# Parameter :: `reverse`

Edge case tests for the `reverse` parameter. Tests validate the
default ascending order and the reversed descending order.

**Source:** [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> ascending order | Default | ✅ | `viewer_integration_test.rs::ec30_sort_orders_by_every_documented_field` |
| EC-2 | `reverse::1` -> descending order (newest first) | Parsing | ✅ | `viewer_integration_test.rs::ec30_sort_orders_by_every_documented_field` |
| EC-3 | `reverse::2` -> exit 1, not silently treated as true | Error Handling | ✅ | `viewer_integration_test.rs::ec31_sort_case_insensitive_and_invalid_values_exit_1` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Error Handling: 1 test (EC-3)

**Total:** 3 edge cases (3 executable)

## Test Cases

---

### EC-1: Absent -> ascending order

- **Given:** journal with events at different timestamps
- **When:** `clj .list`
- **Then:** exit 0; events are shown in ascending order by the active sort field
- **Exit:** 0
- **Source:** [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

---

### EC-2: `reverse::1` -> descending order (newest first)

- **Given:** journal with events at different timestamps
- **When:** `clj .list reverse::1`
- **Then:** exit 0; events are shown newest first
- **Exit:** 0
- **Source:** [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

---

### EC-3: `reverse::2` -> exit 1, not silently treated as true

- **Given:** clean environment
- **When:** `clj .list reverse::2`
- **Then:** exit 1; stderr names the parameter and states that 0 or 1 is expected
- **Exit:** 1
- **Source:** [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md), [type/08_boolean.md](../../../../docs/cli/type/08_boolean.md)
