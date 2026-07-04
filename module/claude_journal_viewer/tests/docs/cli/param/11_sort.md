# Parameter :: `sort`

Edge case tests for the `sort` parameter. Tests validate the default
field and combination with `reverse`.

**Source:** [param/11_sort.md](../../../../docs/cli/param/11_sort.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> sorted by time (chronological) | Default |
| EC-2 | `sort::cost` -> ascending by default | Parsing |
| EC-3 | `sort::cost reverse::1` -> most expensive first | Combined |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Combined: 1 test (EC-3)

**Total:** 3 edge cases

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
