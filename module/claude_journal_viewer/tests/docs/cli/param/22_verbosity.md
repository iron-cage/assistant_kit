# Parameter :: `verbosity`

Edge case tests for the `verbosity` parameter. Tests validate the
default level and the per-command meaning of level 0 and level 2.

**Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> standard report (level 1) | Default |
| EC-2 | `verbosity::0` on `.status` -> compact one-line summary | Per-Command Meaning |
| EC-3 | `verbosity::2` on `.stats` -> extended table with percentiles | Per-Command Meaning |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Per-Command Meaning: 2 tests (EC-2, EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent -> standard report (level 1)

- **Given:** journal with events
- **When:** `clj .status`
- **Then:** exit 0; a standard report (files, size, date range, journal level) is shown
- **Exit:** 0
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

---

### EC-2: `verbosity::0` on `.status` -> compact one-line summary

- **Given:** journal with events
- **When:** `clj .status verbosity::0`
- **Then:** exit 0; a single-line summary (files, size, date range) is shown
- **Exit:** 0
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

---

### EC-3: `verbosity::2` on `.stats` -> extended table with percentiles

- **Given:** journal with events of varying duration
- **When:** `clj .stats verbosity::2`
- **Then:** exit 0; the grouped table includes p50/p90/p99 duration columns
- **Exit:** 0
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)
