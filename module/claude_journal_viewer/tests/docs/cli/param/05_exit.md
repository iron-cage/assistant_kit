# Parameter :: `exit`

Edge case tests for the `exit` parameter. Tests validate absence
behavior (all exit codes) and specific error-class filtering.

**Source:** [param/05_exit.md](../../../../docs/cli/param/05_exit.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> all exit codes shown | Default |
| EC-2 | `exit::0` -> only successful executions | Parsing |
| EC-3 | `exit::2` -> only rate-limit failures | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 2 tests (EC-2, EC-3)

**Total:** 3 edge cases

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

- **Given:** journal with events of varying exit codes, including exit code 2 (rate limit)
- **When:** `clj .list exit::2`
- **Then:** exit 0; only events with exit code 2 are shown
- **Exit:** 0
- **Source:** [param/05_exit.md](../../../../docs/cli/param/05_exit.md)
