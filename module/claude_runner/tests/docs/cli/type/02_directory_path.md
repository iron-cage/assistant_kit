# Type :: `DirectoryPath`

Validation tests for the `DirectoryPath` semantic type. Tests validate path acceptance and missing-value handling.

**Source:** [type.md](../../../../docs/cli/type.md#type--2-directorypath)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path → accepted | Valid Input |
| TC-2 | Relative path → accepted | Valid Input |
| TC-3 | `--dir` without value → exit 1 | Missing Value |
| TC-4 | `--session-dir` without value → exit 1 | Missing Value |

## Test Coverage Summary

- Valid Input: 2 tests (TC-1, TC-2)
- Missing Value: 2 tests (TC-3, TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Absolute path → accepted

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug" --dir /tmp`
- **Then:** Exit 0; assembled command uses `/tmp` as working directory
- **Exit:** 0
- **Source:** [type.md — DirectoryPath](../../../../docs/cli/type.md#type--2-directorypath)

---

### TC-2: Relative path → accepted

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug" --dir ./src`
- **Then:** Exit 0; relative path accepted without rejection
- **Exit:** 0
- **Source:** [type.md — DirectoryPath](../../../../docs/cli/type.md#type--2-directorypath)

---

### TC-3: `--dir` without value → exit 1

- **Given:** clean environment
- **When:** `clr --dir`
- **Then:** Exit 1; error indicating `--dir` requires a value
- **Exit:** 1
- **Source:** [type.md — DirectoryPath](../../../../docs/cli/type.md#type--2-directorypath)

---

### TC-4: `--session-dir` without value → exit 1

- **Given:** clean environment
- **When:** `clr --session-dir`
- **Then:** Exit 1; error indicating `--session-dir` requires a value
- **Exit:** 1
- **Source:** [type.md — DirectoryPath](../../../../docs/cli/type.md#type--2-directorypath)
