# Parameter :: `reverse`

Edge case tests for the `reverse` parameter. Tests validate the
default ascending order and the reversed descending order.

**Source:** [param/12_reverse.md](../../../../docs/cli/param/12_reverse.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> ascending order | Default |
| EC-2 | `reverse::1` -> descending order (newest first) | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

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
