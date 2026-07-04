# Parameter :: `until`

Edge case tests for the `until` parameter. Tests validate absence
behavior (no upper bound) and combination with `since`.

**Source:** [param/02_until.md](../../../../docs/cli/param/02_until.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> no upper time bound | Default |
| EC-2 | `until::1d` -> only events older than 1 day | Parsing |
| EC-3 | `since::7d until::1d` -> combined range | Combined |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Combined: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> no upper time bound

- **Given:** journal with events up to the current moment
- **When:** `clj .list since::7d`
- **Then:** exit 0; events up to now are included, with no upper time bound applied
- **Exit:** 0
- **Source:** [param/02_until.md](../../../../docs/cli/param/02_until.md)

---

### EC-2: `until::1d` -> only events older than 1 day

- **Given:** journal with events both inside and outside the last day
- **When:** `clj .list until::1d`
- **Then:** exit 0; only events older than 1 day are shown
- **Exit:** 0
- **Source:** [param/02_until.md](../../../../docs/cli/param/02_until.md)

---

### EC-3: `since::7d until::1d` -> combined range

- **Given:** journal with events spread across the last 30 days
- **When:** `clj .list since::7d until::1d`
- **Then:** exit 0; only events between 7 days ago and 1 day ago are shown
- **Exit:** 0
- **Source:** [param/02_until.md](../../../../docs/cli/param/02_until.md)
