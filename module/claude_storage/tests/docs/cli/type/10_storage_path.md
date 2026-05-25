# Type :: 10. `StoragePath`

Type constraint tests for `StoragePath` — filesystem path for storage operations.

**Source:** [type/10_storage_path.md](../../../../docs/cli/type/10_storage_path.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path accepted | Valid Input |
| TC-2 | Tilde-prefixed path accepted | Valid Input |
| TC-3 | Relative path accepted | Valid Input |
| TC-4 | Empty string rejected | Invalid Input |

## Test Coverage Summary

- Valid Input: 3 tests (TC-1, TC-2, TC-3)
- Invalid Input: 1 test (TC-4)

**Total:** 4 cases

## Test Cases

---

### TC-1: Absolute path accepted

- **Given:** Input string `"/home/alice/.claude"`
- **When:** `StoragePath` is parsed
- **Then:** Accepted as `StoragePath("/home/alice/.claude")`; `expanded()` returns the path unchanged

---

### TC-2: Tilde-prefixed path accepted

- **Given:** Input string `"~/.claude"` with `HOME=/home/alice`
- **When:** `StoragePath` is parsed
- **Then:** Accepted; `expanded()` returns `"/home/alice/.claude"` — tilde expanded to home directory

---

### TC-3: Relative path accepted

- **Given:** Input string `"relative/path"`
- **When:** `StoragePath` is parsed
- **Then:** Accepted as `StoragePath("relative/path")`; `expanded()` returns the path unchanged

---

### TC-4: Empty string rejected

- **Given:** Input string `""`
- **When:** `StoragePath` is parsed
- **Then:** Rejected; error message is `path must be non-empty`
