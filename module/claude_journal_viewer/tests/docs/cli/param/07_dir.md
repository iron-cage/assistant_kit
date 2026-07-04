# Parameter :: `dir`

Edge case tests for the `dir` parameter. Tests validate absence
behavior (all directories) and substring matching including subdirectories.

**Source:** [param/07_dir.md](../../../../docs/cli/param/07_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absent -> all directories shown | Default |
| EC-2 | `dir::/home/user/myproject` -> matches subdirectory events too | Substring Match |
| EC-3 | Unrelated substring -> no match | Substring Match |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Substring Match: 2 tests (EC-2, EC-3)

**Total:** 3 edge cases

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
