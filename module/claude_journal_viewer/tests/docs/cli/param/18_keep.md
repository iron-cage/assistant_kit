# Parameter :: `keep`

Edge case tests for the `keep` parameter. Tests validate the
required-parameter constraint and both age-based and size-based deletion.

**Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent on `.prune` -> error, required parameter missing | Required |
| EC-2 | `keep::30d` -> age-based deletion | Parsing |
| EC-3 | `keep::100mb` -> size-based deletion | Parsing |

## Test Coverage Summary

- Required: 1 test (EC-1)
- Parsing: 2 tests (EC-2, EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent on `.prune` -> error, required parameter missing

- **Given:** clean environment
- **When:** `clj .prune`
- **Then:** exit 1; stderr indicates `keep` is a required parameter
- **Exit:** 1
- **Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md)

---

### EC-2: `keep::30d` -> age-based deletion

- **Given:** journal directory with files older and newer than 30 days
- **When:** `clj .prune keep::30d confirm::1`
- **Then:** exit 0; files older than 30 days are deleted; newer files remain
- **Exit:** 0
- **Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md)

---

### EC-3: `keep::100mb` -> size-based deletion

- **Given:** journal directory totaling more than 100MB
- **When:** `clj .prune keep::100mb confirm::1`
- **Then:** exit 0; oldest files are deleted first until total size is under 100MB
- **Exit:** 0
- **Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md)
