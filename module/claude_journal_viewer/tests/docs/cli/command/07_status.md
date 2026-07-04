# Test: `.status`

### Scope

- **Purpose**: Verify `.status` reports journal health, size, and configuration at each verbosity level.
- **Responsibility**: Test case coverage for both `.status` parameters.
- **In Scope**: Default (verbosity 1) report, compact (0) and per-file (2) verbosity, journal_dir override.
- **Out of Scope**: Retention/pruning (-> `06_prune.md`), aggregate cost/token stats (-> `03_stats.md`).

Test case planning for [command/07_status.md](../../../../docs/cli/command/07_status.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No args -> standard health report | Default |
| IT-2 | `verbosity::0` -> compact one-line summary | Verbosity |
| IT-3 | `verbosity::2` -> per-file breakdown | Verbosity |
| IT-4 | `journal_dir::PATH` -> reports on custom directory | Directory Override |

## Test Coverage Summary

- Default: 1 test (IT-1)
- Verbosity: 2 tests (IT-2, IT-3)
- Directory Override: 1 test (IT-4)

**Total:** 4 tests

---

### IT-1: No args -> standard health report

- **Given:** journal directory with a known number of files and total size
- **When:** `clj .status`
- **Then:** exit 0; output shows journal directory path, file count, total size, and date range
- **Exit:** 0
- **Source:** [command/07_status.md](../../../../docs/cli/command/07_status.md)

---

### IT-2: `verbosity::0` -> compact one-line summary

- **Given:** journal directory with a known number of files
- **When:** `clj .status verbosity::0`
- **Then:** exit 0; output is a single compact summary line
- **Exit:** 0
- **Source:** [command/07_status.md](../../../../docs/cli/command/07_status.md), [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

---

### IT-3: `verbosity::2` -> per-file breakdown

- **Given:** journal directory containing multiple files
- **When:** `clj .status verbosity::2`
- **Then:** exit 0; output lists each journal file individually with its own size/date
- **Exit:** 0
- **Source:** [command/07_status.md](../../../../docs/cli/command/07_status.md), [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

---

### IT-4: `journal_dir::PATH` -> reports on custom directory

- **Given:** a non-default journal directory containing files
- **When:** `clj .status journal_dir::/tmp/custom_journal`
- **Then:** exit 0; reported directory path, file count, and size reflect the custom directory, not the default
- **Exit:** 0
- **Source:** [command/07_status.md](../../../../docs/cli/command/07_status.md), [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)
