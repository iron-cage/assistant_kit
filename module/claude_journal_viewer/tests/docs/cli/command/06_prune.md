# Test: `.prune`

### Scope

- **Purpose**: Verify `.prune` deletes old journal files correctly by age or size, respecting dry-run and confirm gates.
- **Responsibility**: Test case coverage for all 3 `.prune` parameters, including the required `keep`.
- **In Scope**: Age-based retention, size-based retention, dry-run preview, confirm bypass, required-param and format validation.
- **Out of Scope**: Journal health reporting (-> `07_status.md`), non-destructive listing (-> `01_list.md`).

Test case planning for [command/06_prune.md](../../../../docs/cli/command/06_prune.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | `keep::30d` -> deletes files older than 30 days | Age Retention |
| IT-2 | `keep::100mb` -> deletes oldest until under size threshold | Size Retention |
| IT-3 | `keep::4w dry_run::1` -> preview only, no deletion | Dry Run |
| IT-4 | `keep::7d confirm::1` -> deletes without confirmation prompt | Confirm Bypass |
| IT-5 | Missing `keep` -> exit 1, error message | Required Param |
| IT-6 | `keep::badvalue` -> exit 1, invalid format message | Error Handling |

## Test Coverage Summary

- Age Retention: 1 test (IT-1)
- Size Retention: 1 test (IT-2)
- Dry Run: 1 test (IT-3)
- Confirm Bypass: 1 test (IT-4)
- Required Param: 1 test (IT-5)
- Error Handling: 1 test (IT-6)

**Total:** 6 tests

---

### IT-1: `keep::30d` -> deletes files older than 30 days

- **Given:** journal directory with files older and newer than 30 days
- **When:** `clj .prune keep::30d confirm::1`
- **Then:** exit 0; files older than 30 days are deleted, newer files remain
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md)

---

### IT-2: `keep::100mb` -> deletes oldest until under size threshold

- **Given:** journal directory totaling more than 100MB
- **When:** `clj .prune keep::100mb confirm::1`
- **Then:** exit 0; oldest files are deleted first until total size is under 100MB
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md), [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)

---

### IT-3: `keep::4w dry_run::1` -> preview only, no deletion

- **Given:** journal directory with files older than 4 weeks
- **When:** `clj .prune keep::4w dry_run::1`
- **Then:** exit 0; candidate file list printed; no files are actually deleted
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md), [param/19_dry_run.md](../../../../docs/cli/param/19_dry_run.md)

---

### IT-4: `keep::7d confirm::1` -> deletes without confirmation prompt

- **Given:** journal directory with files older than 7 days; non-interactive test environment
- **When:** `clj .prune keep::7d confirm::1`
- **Then:** exit 0; matching files deleted without prompting for confirmation
- **Exit:** 0
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md), [param/20_confirm.md](../../../../docs/cli/param/20_confirm.md)

---

### IT-5: Missing `keep` -> exit 1, error message

- **Given:** clean environment
- **When:** `clj .prune`
- **Then:** exit 1; stderr states that `keep` is required
- **Exit:** 1
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md)

---

### IT-6: `keep::badvalue` -> exit 1, invalid format message

- **Given:** clean environment
- **When:** `clj .prune keep::badvalue`
- **Then:** exit 1; stderr states the retention spec is invalid, expecting duration (e.g. `7d`, `4w`) or size (e.g. `100mb`, `1gb`) format
- **Exit:** 1
- **Source:** [command/06_prune.md](../../../../docs/cli/command/06_prune.md), [type/11_retention_spec.md](../../../../docs/cli/type/11_retention_spec.md)
