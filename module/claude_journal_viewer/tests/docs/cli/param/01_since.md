# Parameter :: `since`

Edge case tests for the `since` parameter. Tests validate absence
behavior (no lower bound) and per-command default variance.

**Source:** [param/01_since.md](../../../../docs/cli/param/01_since.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent on `.list` -> no lower time bound | Default |
| EC-2 | `since::1h` -> only events from last hour | Parsing |
| EC-3 | Absent on `.stats` -> defaults to last 7 days | Per-Command Default |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Per-Command Default: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent on `.list` -> no lower time bound

- **Given:** journal with events spanning multiple years
- **When:** `clj .list`
- **Then:** exit 0; events are not filtered by a lower time bound
- **Exit:** 0
- **Source:** [param/01_since.md](../../../../docs/cli/param/01_since.md)

---

### EC-2: `since::1h` -> only events from last hour

- **Given:** journal with events both inside and outside the last hour
- **When:** `clj .list since::1h`
- **Then:** exit 0; only events from the last hour are shown
- **Exit:** 0
- **Source:** [param/01_since.md](../../../../docs/cli/param/01_since.md)

---

### EC-3: Absent on `.stats` -> defaults to last 7 days

- **Given:** journal with events both inside and outside the last 7 days
- **When:** `clj .stats`
- **Then:** exit 0; only events from the last 7 days are included in the aggregate
- **Exit:** 0
- **Source:** [param/01_since.md](../../../../docs/cli/param/01_since.md)
