# Parameter :: `refresh`

Edge case tests for the `refresh` parameter. Tests validate the
default interval, the disable shortcut, and a custom interval.

**Source:** [param/27_refresh.md](../../../../docs/cli/param/27_refresh.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> 10-second auto-refresh | Default |
| EC-2 | `refresh::0` -> auto-refresh disabled | Special Value |
| EC-3 | `refresh::30` -> 30-second interval | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Special Value: 1 test (EC-2)
- Parsing: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> 10-second auto-refresh

- **Given:** clean environment
- **When:** `clj .serve`
- **Then:** exit 0 on shutdown; the dashboard page polls for new data every 10 seconds
- **Exit:** 0
- **Source:** [param/27_refresh.md](../../../../docs/cli/param/27_refresh.md)

---

### EC-2: `refresh::0` -> auto-refresh disabled

- **Given:** clean environment
- **When:** `clj .serve refresh::0`
- **Then:** exit 0 on shutdown; the dashboard page does not auto-poll; only manual reload updates data
- **Exit:** 0
- **Source:** [param/27_refresh.md](../../../../docs/cli/param/27_refresh.md)

---

### EC-3: `refresh::30` -> 30-second interval

- **Given:** clean environment
- **When:** `clj .serve refresh::30`
- **Then:** exit 0 on shutdown; the dashboard page polls for new data every 30 seconds
- **Exit:** 0
- **Source:** [param/27_refresh.md](../../../../docs/cli/param/27_refresh.md)
