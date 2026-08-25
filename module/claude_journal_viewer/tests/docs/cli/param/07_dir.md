# Parameter :: `dir`

Edge case tests for the `dir` parameter. Tests validate absence
behavior (all directories) and substring matching including subdirectories.

**Source:** [param/07_dir.md](../../../../docs/cli/param/07_dir.md)

## Test Case Index

| ID | Test Name | Category | Status | Implemented as |
|----|-----------|----------|--------|----------------|
| EC-1 | Absent -> all directories shown | Default | ⏳ | — |
| EC-2 | `dir::/home/user/myproject` -> matches subdirectory events too | Substring Match | ⏳ | — |
| EC-3 | Unrelated substring -> no match | Substring Match | ⏳ | — |
| EC-4 | `dir::` filters events; it does not relocate the journal | Name Collision | ✅ | `viewer_integration_test.rs::ec27_dir_param_filters_by_event_working_directory` |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Substring Match: 2 tests (EC-2, EC-3)
- Name Collision: 1 test (EC-4)

**Total:** 4 edge cases (1 executable)

EC-4 is the case whose absence let the collision survive: every earlier test
passed `dir::<tmpdir>` *expecting the journal-location reading*, so the wrong
meaning worked by coincidence — the path handed to it happened to be a real
journal. EC-4 passes `journal_dir::` and `dir::` together with different
values, so neither key can stand in for the other.

## Test Cases

---

### EC-1: Absent -> all directories shown

- **Given:** journal with events from multiple working directories
- **When:** `clj .list`
- **Then:** exit 0; events from all directories are shown
- **Exit:** 0
- **Source:** [param/07_dir.md](../../../../docs/cli/param/07_dir.md)

---

### EC-2: `dir::/home/user/myproject` -> matches subdirectory events too

- **Given:** journal with events from `/home/user/myproject` and `/home/user/myproject/subdir`
- **When:** `clj .list dir::/home/user/myproject`
- **Then:** exit 0; events from both the directory and its subdirectory are shown
- **Exit:** 0
- **Source:** [param/07_dir.md](../../../../docs/cli/param/07_dir.md)

---

### EC-3: Unrelated substring -> no match

- **Given:** journal with events from `/home/user/myproject`
- **When:** `clj .list dir::/home/user/otherproject`
- **Then:** exit 0; no events are shown, since the substring does not appear in any recorded `dir` field
- **Exit:** 0
- **Source:** [param/07_dir.md](../../../../docs/cli/param/07_dir.md)

---

### EC-4: `dir::` filters events; it does not relocate the journal

- **Given:** a temp journal holding three events — two recorded under `/work/alpha`, one under `/work/beta`
- **When:** `clj .list journal_dir::<tmpdir> dir::/work/beta`
- **Then:** exit 0; only the `/work/beta` event is shown. Both keys are supplied with different values in one invocation, so a build in which `dir::` were still consumed as the journal location would read a nonexistent path and print no events at all
- **Exit:** 0
- **Source:** [param/07_dir.md](../../../../docs/cli/param/07_dir.md), [param/21_journal_dir.md](../../../../docs/cli/param/21_journal_dir.md)
