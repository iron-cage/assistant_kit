# Parameter :: `keep`

Edge case tests for the `keep` parameter. Tests validate the `30d`
default when absent and age-based duration parsing, including rejection
of size units.

**Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent on `.prune` -> defaults to `30d` | Default |
| EC-2 | `keep::30d` -> age-based deletion | Parsing |
| EC-3 | `keep::100mb` -> exit 1, size units rejected | Error Handling |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Parsing: 1 test (EC-2)
- Error Handling: 1 test (EC-3)

**Total:** 3 edge cases

## Test Cases

---

### EC-1: Absent on `.prune` -> defaults to `30d`

- **Given:** journal directory with files older and newer than 30 days
- **When:** `clj .prune`
- **Then:** exit 0; behaves identically to `keep::30d` — `keep` is optional, not required
- **Exit:** 0
- **Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md) — Required = No, Default = `30d`

---

### EC-2: `keep::30d` -> age-based deletion

- **Given:** journal directory with `YYYY-MM-DD.jsonl` files older and newer than 30 days
- **When:** `clj .prune keep::30d`
- **Then:** exit 0; files whose filename date is older than the window are deleted; newer files remain. Deletion is immediate — there is no confirmation prompt, and `dry_run::1` is the preview mechanism
- **Exit:** 0
- **Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md)

---

### EC-3: `keep::100mb` -> exit 1, size units rejected

- **Given:** clean environment
- **When:** `clj .prune keep::100mb`
- **Then:** exit 1; stderr carries `Error: invalid duration '100mb' (expected e.g. 30s, 5m, 1h, 7d, 2w)` — a RetentionSpec is a duration only; size-based retention was never adopted
- **Exit:** 1
- **Source:** [param/18_keep.md](../../../../docs/cli/param/18_keep.md), [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)
