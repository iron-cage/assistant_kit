# Parameter :: `limit`

Edge case tests for the `limit` parameter. Tests validate the default
cap, the unlimited shortcut, and a custom cap.

**Source:** [param/09_limit.md](../../../../docs/cli/param/09_limit.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> defaults to 50 | Default |
| EC-2 | `limit::0` -> unlimited, all matching events shown | Special Value |
| EC-3 | `limit::100` -> up to 100 events | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Special Value: 1 test (EC-2)
- Parsing: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> defaults to 50

- **Given:** journal with more than 50 matching events
- **When:** `clj .list`
- **Then:** exit 0; exactly 50 events are shown
- **Exit:** 0
- **Source:** [param/09_limit.md](../../../../docs/cli/param/09_limit.md)

---

### EC-2: `limit::0` -> unlimited, all matching events shown

- **Given:** journal with 200 matching events
- **When:** `clj .list limit::0`
- **Then:** exit 0; all 200 events are shown, with no cap applied
- **Exit:** 0
- **Source:** [param/09_limit.md](../../../../docs/cli/param/09_limit.md)

---

### EC-3: `limit::100` -> up to 100 events

- **Given:** journal with 200 matching events
- **When:** `clj .list limit::100`
- **Then:** exit 0; exactly 100 events are shown
- **Exit:** 0
- **Source:** [param/09_limit.md](../../../../docs/cli/param/09_limit.md)
