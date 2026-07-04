# Parameter :: `dry_run`

Edge case tests for the `dry_run` parameter. Tests validate the
default (live deletion) and the preview mode.

**Source:** [param/19_dry_run.md](../../../../docs/cli/param/19_dry_run.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> live deletion occurs | Default |
| EC-2 | `dry_run::1` -> preview only, no files deleted | Parsing |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)

**Total:** 2 edge cases

## Test Cases

---

### EC-1: Absent -> live deletion occurs

- **Given:** journal directory with files older than 30 days
- **When:** `clj .prune keep::30d confirm::1`
- **Then:** exit 0; matching files are actually deleted from disk
- **Exit:** 0
- **Source:** [param/19_dry_run.md](../../../../docs/cli/param/19_dry_run.md)

---

### EC-2: `dry_run::1` -> preview only, no files deleted

- **Given:** journal directory with files older than 30 days
- **When:** `clj .prune keep::30d dry_run::1`
- **Then:** exit 0; candidate files are listed with size and age, but none are deleted from disk
- **Exit:** 0
- **Source:** [param/19_dry_run.md](../../../../docs/cli/param/19_dry_run.md)
