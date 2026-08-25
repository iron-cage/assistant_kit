# Parameter :: `exit`

Edge case tests for the `exit` parameter. Tests validate absence
behavior (all exit codes) and specific error-class filtering.

**Source:** [param/05_exit.md](../../../../docs/cli/param/05_exit.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> all exit codes shown | Default | ⏳ | — |
| EC-2 | `exit::0` -> only successful executions | Parsing | ⏳ | — |
| EC-3 | `exit::2` -> only rate-limit failures | Parsing | ✅ | `viewer_integration_test.rs::ec26_exit_param_filters_by_exit_code` |
| EC-4 | `exit_code::2` -> rejected, not silently accepted | Name Collision | ✅ | `viewer_integration_test.rs::ec28_unknown_param_exits_1` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 2 tests (EC-2, EC-3)
- Name Collision: 1 test (EC-4)

**Total:** 4 edge cases (2 executable)

EC-3 and EC-4 are a pair. EC-3 proves `exit::` — the spelling this document and
the help text have always printed — actually narrows the result set; EC-4
proves the field name `exit_code` is *not* silently accepted as a synonym. Only
EC-3 is load-bearing against the original defect: an ignored filter widens
output rather than erroring, so a fixture must contain non-matching events for
its absence to be visible at all.

## Test Cases

---

### EC-1: Absent -> all exit codes shown

- **Given:** journal with events of varying exit codes
- **When:** `clj .list`
- **Then:** exit 0; events with any exit code are shown
- **Exit:** 0
- **Source:** [param/05_exit.md](../../../../docs/cli/param/05_exit.md)

---

### EC-2: `exit::0` -> only successful executions

- **Given:** journal with both successful (exit 0) and failed events
- **When:** `clj .list exit::0`
- **Then:** exit 0; only events with exit code 0 are shown
- **Exit:** 0
- **Source:** [param/05_exit.md](../../../../docs/cli/param/05_exit.md)

---

### EC-3: `exit::2` -> only rate-limit failures

- **Given:** a temp journal holding three events — one exit 0, two exit 2
- **When:** `clj .list exit::2`
- **Then:** exit 0; both exit-2 events are shown and the exit-0 event is not. The fixture deliberately contains an event that must be *excluded*, since an ignored filter returns everything and would otherwise pass
- **Exit:** 0
- **Source:** [param/05_exit.md](../../../../docs/cli/param/05_exit.md)

---

### EC-4: `exit_code::2` -> rejected, not silently accepted

- **Given:** any journal
- **When:** `clj .list exit_code::2`
- **Then:** exit 1; stderr names `exit_code` as an unknown parameter and lists the accepted set. `exit_code` is the JSON field name, not a CLI synonym — accepting it silently is what made the original defect invisible
- **Exit:** 1
- **Source:** [param/05_exit.md](../../../../docs/cli/param/05_exit.md)
