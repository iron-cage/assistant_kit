# Parameter :: `wide`

Edge case tests for the `wide` parameter. Tests validate the
default compact table and the full-width mode.

**Source:** [param/25_wide.md](../../../../docs/cli/param/25_wide.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> compact table, columns may be truncated | Default |
| EC-2 | `wide::1` -> full-width table, no truncation | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> compact table, columns may be truncated

- **Given:** journal with events carrying long stdout/model/dir values
- **When:** `clj .list`
- **Then:** exit 0; the table shows the default column set without full-width fields
- **Exit:** 0
- **Source:** [param/25_wide.md](../../../../docs/cli/param/25_wide.md)

---

### EC-2: `wide::1` -> full-width table, no truncation

- **Given:** journal with events carrying long stdout/model/dir values
- **When:** `clj .list wide::1`
- **Then:** exit 0; all available columns are shown untruncated, including stdout/stderr preview, full model name, and full directory path
- **Exit:** 0
- **Source:** [param/25_wide.md](../../../../docs/cli/param/25_wide.md)
