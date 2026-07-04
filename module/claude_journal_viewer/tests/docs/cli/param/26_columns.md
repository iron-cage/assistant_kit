# Parameter :: `columns`

Edge case tests for the `columns` parameter. Tests validate the
default column set and a custom column selection.

**Source:** [param/26_columns.md](../../../../docs/cli/param/26_columns.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> default column set shown | Default |
| EC-2 | `columns::time,command,cost,exit` -> only those 4 columns shown | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> default column set shown

- **Given:** journal with events
- **When:** `clj .list`
- **Then:** exit 0; the table shows `time,type,command,exit,duration,cost,model`
- **Exit:** 0
- **Source:** [param/26_columns.md](../../../../docs/cli/param/26_columns.md)

---

### EC-2: `columns::time,command,cost,exit` -> only those 4 columns shown

- **Given:** journal with events
- **When:** `clj .list columns::time,command,cost,exit`
- **Then:** exit 0; the table shows exactly the 4 named columns, in the given order
- **Exit:** 0
- **Source:** [param/26_columns.md](../../../../docs/cli/param/26_columns.md)
