# Test: `.prune`

### Scope

- **Purpose**: Verify `.prune` deletes old journal files correctly by filename-date age, respecting the dry-run preview gate.
- **Responsibility**: Test case coverage for both `.prune` parameters, `keep` and `dry_run`, each optional.
- **In Scope**: Age-based retention, the `keep` default, dry-run preview, and retention-spec format validation.
- **Out of Scope**: Journal health reporting (-> `07_status.md`), non-destructive listing (-> `01_list.md`).

Test case planning for [command/06_prune.md](../../../../docs/cli/command/06_prune.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `keep::30d` -> deletes files older than 30 days | Age Retention |
| IT-2 | `keep::4w dry_run::1` -> preview only, no deletion | Dry Run |
| IT-3 | Absent `keep` -> defaults to `30d`, prunes, exit 0 | Default |
| IT-4 | `keep::12h` -> floors to 0 days, only today's file survives | Boundary |
| IT-5 | `keep::badvalue` -> exit 1, invalid duration message | Error Handling |

## Test Coverage Summary

- Age Retention: 1 test (IT-1)
- Dry Run: 1 test (IT-2)
- Default: 1 test (IT-3)
- Boundary: 1 test (IT-4)
- Error Handling: 1 test (IT-5)

**Total:** 5 tests

---

### IT-1: `keep::30d` -> deletes files older than 30 days

- **Given:** journal directory with `YYYY-MM-DD.jsonl` files dated older and newer than 30 days
- **When:** `clj .prune keep::30d`
- **Then:** exit 0; files whose filename date is strictly before `today - 30d` are deleted, newer files remain; a `Deleted: <path>` line per file plus a final count line
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md)

---

### IT-2: `keep::4w dry_run::1` -> preview only, no deletion

- **Given:** journal directory with files older than 4 weeks
- **When:** `clj .prune keep::4w dry_run::1`
- **Then:** exit 0; each candidate printed as `Would delete: <path>`; no files are actually deleted
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md), [param/19_dry_run.md](../../../../docs/cli/param/19_dry_run.md)

---

### IT-3: Absent `keep` -> defaults to `30d`, prunes, exit 0

- **Given:** journal directory with files older than 30 days
- **When:** `clj .prune`
- **Then:** exit 0; behaves identically to `keep::30d` — `keep` is optional with default `30d`, not a required parameter
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md) — Parameters table, `keep` Required = No

---

### IT-4: `keep::12h` -> floors to 0 days, only today's file survives

- **Given:** journal directory with today's file plus files dated earlier
- **When:** `clj .prune keep::12h`
- **Then:** exit 0; the sub-day duration floors to 0 whole days, so every file dated strictly before today is deleted; today's file is structurally never a candidate and survives
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md) — Algorithm steps 1-2, [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)

---

### IT-5: `keep::badvalue` -> exit 1, invalid duration message

- **Given:** clean environment
- **When:** `clj .prune keep::badvalue`
- **Then:** exit 1; stderr carries `Error: invalid duration 'badvalue' (expected e.g. 30s, 5m, 1h, 7d, 2w)` — a RetentionSpec is a duration only; size units such as `100mb` are not accepted
- **Exit:** 1
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md), [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)
