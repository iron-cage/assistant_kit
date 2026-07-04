# Parameter :: `format`

Edge case tests for the `format` parameter. Tests validate the
per-command default variance between `.list`/`.tail` and `.export`.

**Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent on `.list` -> defaults to table | Per-Command Default |
| EC-2 | Absent on `.export` -> defaults to jsonl | Per-Command Default |
| EC-3 | `format::csv` with `output` on `.export` -> CSV file written | Parsing |

## Test Coverage Summary

- Per-Command Default: 2 tests (EC-1, EC-2)
- Parsing: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent on `.list` -> defaults to table

- **Given:** journal with events
- **When:** `clj .list`
- **Then:** exit 0; output is rendered as an aligned table
- **Exit:** 0
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md)

---

### EC-2: Absent on `.export` -> defaults to jsonl

- **Given:** journal with events
- **When:** `clj .export`
- **Then:** exit 0; output is one JSON object per line
- **Exit:** 0
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md)

---

### EC-3: `format::csv` with `output` on `.export` -> CSV file written

- **Given:** journal with events; `/tmp/events.csv` parent directory exists
- **When:** `clj .export format::csv output::/tmp/events.csv`
- **Then:** exit 0; `/tmp/events.csv` is written with a header row and comma-separated values
- **Exit:** 0
- **Source:** [param/10_format.md](../../../../docs/cli/param/10_format.md)
