# Parameter :: `output`

Edge case tests for the `output` parameter. Tests validate the
default (stdout) and writing to a file.

**Source:** [param/23_output.md](../../../../docs/cli/param/23_output.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> output goes to stdout | Default |
| EC-2 | `output::/tmp/events.csv` -> written to file instead | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> output goes to stdout

- **Given:** journal with events
- **When:** `clj .export format::jsonl`
- **Then:** exit 0; serialized events are printed to stdout
- **Exit:** 0
- **Source:** [param/23_output.md](../../../../docs/cli/param/23_output.md)

---

### EC-2: `output::/tmp/events.csv` -> written to file instead

- **Given:** journal with events; `/tmp` exists and is writable
- **When:** `clj .export format::csv output::/tmp/events.csv`
- **Then:** exit 0; `/tmp/events.csv` is created containing the serialized events; nothing is printed to stdout
- **Exit:** 0
- **Source:** [param/23_output.md](../../../../docs/cli/param/23_output.md)
