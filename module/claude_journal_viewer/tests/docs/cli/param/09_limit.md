# Parameter :: `limit`

Edge case tests for the `limit` parameter. Tests validate the default
cap, the unlimited shortcut, and a custom cap.

**Source:** [param/09_limit.md](../../../../docs/cli/param/09_limit.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> defaults to 50 | Default | ⏳ | — |
| EC-2 | `limit::0` -> unlimited, all matching events shown | Special Value | ✅ | `viewer_integration_test.rs::ec33_limit_applies_after_sort_and_zero_means_unlimited` |
| EC-3 | `limit::100` -> up to 100 events | Parsing | ⏳ | — |
| EC-4 | `sort::cost reverse::1 limit::1` -> the journal's most expensive event | Ordering | ✅ | `viewer_integration_test.rs::ec33_limit_applies_after_sort_and_zero_means_unlimited` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Special Value: 1 test (EC-2)
- Parsing: 1 test (EC-3)
- Ordering: 1 test (EC-4)

**Total:** 4 edge cases (2 executable)

EC-2 and EC-4 are the two cases where "cap" and "stop reading early" come apart.
`JournalReader::query()` caps by stopping early, which is equivalent to a cap
only while the output order matches the read order — EC-4 puts a sort between
the two, and EC-2 asks for a cap of zero, which as an early stop reads nothing
at all instead of everything.

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

---

### EC-4: `sort::cost reverse::1 limit::1` -> the journal's most expensive event

- **Given:** journal whose most expensive event is *not* its first-appended one
- **When:** `clj .list sort::cost reverse::1 limit::1`
- **Then:** exit 0; the single returned event is the most expensive in the whole
  matching window, not the priciest among the first `limit` appended
- **And:** `clj .list sort::cost limit::1` returns the cheapest, which the fixture
  places last — so a cap applied before the sort fails in both directions
- **Exit:** 0
- **Source:** [param/09_limit.md](../../../../docs/cli/param/09_limit.md), [command/01_list.md](../../../../docs/cli/command/01_list.md)
