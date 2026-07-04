# Parameter :: `confirm`

Edge case tests for the `confirm` parameter. Tests validate the
default interactive prompt and the skip-prompt shortcut.

**Source:** [param/20_confirm.md](../../../../docs/cli/param/20_confirm.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> interactive confirmation prompt shown | Default |
| EC-2 | `confirm::1` -> deletion proceeds without prompting | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> interactive confirmation prompt shown

- **Given:** journal directory with files older than 30 days
- **When:** `clj .prune keep::30d`
- **Then:** exit 0; `Delete N files? [y/N]` prompt is shown before any deletion
- **Exit:** 0
- **Source:** [param/20_confirm.md](../../../../docs/cli/param/20_confirm.md)

---

### EC-2: `confirm::1` -> deletion proceeds without prompting

- **Given:** journal directory with files older than 30 days
- **When:** `clj .prune keep::30d confirm::1`
- **Then:** exit 0; matching files are deleted with no interactive prompt
- **Exit:** 0
- **Source:** [param/20_confirm.md](../../../../docs/cli/param/20_confirm.md)
