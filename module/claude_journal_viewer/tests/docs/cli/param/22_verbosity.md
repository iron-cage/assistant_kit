# Parameter :: `verbosity`

Edge case tests for the `verbosity` parameter. Tests validate the default level,
the two non-default levels, the clamp above the documented range, and the
rejection of values that are not integers at all.

**Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> standard report (level 1) | Default | ✅ | `viewer_integration_test.rs::ec7_status_shows_health_report` |
| EC-2 | `verbosity::0` -> compact one-line summary | Level Selection | ✅ | `viewer_integration_test.rs::ec35_status_verbosity_levels_and_clamping` |
| EC-3 | `verbosity::2` -> per-file breakdown | Level Selection | ✅ | `viewer_integration_test.rs::ec35_status_verbosity_levels_and_clamping` |
| EC-4 | `verbosity::9` -> clamped to level 2 | Clamping | ✅ | `viewer_integration_test.rs::ec35_status_verbosity_levels_and_clamping` |
| EC-5 | Non-integer or negative -> exit 1 | Error Handling | ✅ | `viewer_integration_test.rs::ec35_status_verbosity_levels_and_clamping` |
| EC-6 | Empty journal -> every level degrades without a placeholder table | Empty Journal | ✅ | `viewer_integration_test.rs::ec35_status_verbosity_levels_and_clamping` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Level Selection: 2 tests (EC-2, EC-3)
- Clamping: 1 test (EC-4)
- Error Handling: 1 test (EC-5)
- Empty Journal: 1 test (EC-6)

**Total:** 6 edge cases (6 executable)

EC-4 and EC-5 are a pair and neither means much alone: clamping without EC-5
would be indistinguishable from accepting any input at all, and EC-5 without
EC-4 would not pin that an out-of-range *integer* is still a valid request.

`.stats` had `verbosity` levels of its own in an earlier revision of the
parameter page. They were retracted rather than built, so no case here covers
`.stats` — [`command/03_stats.md`](../command/03_stats.md) is where that
command's coverage lives.

## Test Cases

---

### EC-1: Absent -> standard report (level 1)

- **Given:** journal with events
- **When:** `clj .status`
- **Then:** exit 0; the standard report is shown — journal directory, file count, total size, date range, and journal level
- **Exit:** 0
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

---

### EC-2: `verbosity::0` -> compact one-line summary

- **Given:** journal with two dated files
- **When:** `clj .status verbosity::0`
- **Then:** exit 0; output is exactly one line carrying file count, total size, and date range, and does not contain the multi-line report's `Journal directory:` label
- **Exit:** 0
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

---

### EC-3: `verbosity::2` -> per-file breakdown

- **Given:** journal with two dated files of clearly different sizes
- **When:** `clj .status verbosity::2`
- **Then:** exit 0; the level-1 report is still present, followed by a `DATE`/`SIZE` header and one row per file showing that file's own size — not the total repeated
- **Exit:** 0
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)

---

### EC-4: `verbosity::9` -> clamped to level 2

- **Given:** journal with two dated files
- **When:** `clj .status verbosity::9`
- **Then:** exit 0; output is byte-identical to `verbosity::2`
- **Exit:** 0
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md), [param_group/02_display.md](../../../../docs/cli/param_group/02_display.md)

---

### EC-5: Non-integer or negative -> exit 1

- **Given:** clean environment
- **When:** `clj .status verbosity::abc`, `verbosity::-1`, and `verbosity::1.5`
- **Then:** exit 1 for each; stderr says `invalid integer` and names the `verbosity` parameter
- **Exit:** 1
- **Source:** [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md), [type/04_integer.md](../../../../docs/cli/type/04_integer.md)

---

### EC-6: Empty journal -> every level degrades without a placeholder table

- **Given:** an empty journal directory
- **When:** `clj .status verbosity::2` and `clj .status verbosity::0`
- **Then:** exit 0 for both; the report shows `Files: 0` and `Date range: no events`, and the level-2 breakdown reads `(no journal files)` with no `DATE`/`SIZE` column header above zero rows
- **Exit:** 0
- **Source:** [command/07_status.md](../../../../docs/cli/command/07_status.md), [param/22_verbosity.md](../../../../docs/cli/param/22_verbosity.md)
