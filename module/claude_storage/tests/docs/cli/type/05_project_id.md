# Type :: 5. `ProjectId`

Type constraint tests for `ProjectId` — multi-format project identifier.

**Source:** [type/05_project_id.md](../../../../docs/cli/type/05_project_id.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Absolute path format accepted | Valid Input |
| TC-2 | Path-encoded ID format accepted | Valid Input |
| TC-3 | UUID format accepted | Valid Input |
| TC-4 | Path(...) form accepted | Valid Input |
| TC-5 | Empty input rejected | Invalid Input |
| TC-6 | Unresolvable project rejected | Not Found |

## Test Coverage Summary

- Valid Input: 4 tests (TC-1, TC-2, TC-3, TC-4)
- Invalid Input: 1 test (TC-5)
- Not Found: 1 test (TC-6)

**Total:** 6 cases

## Test Cases

---

### TC-1: Absolute path format accepted

- **Given:** Input string `"/home/alice/projects/my-app"` and project exists in storage
- **When:** `ProjectId` is parsed and resolved
- **Then:** Accepted; `path_encoded()` returns the path-encoded form of the input path

---

### TC-2: Path-encoded ID format accepted

- **Given:** Input string `"-home-alice-projects-my-app"` (starts with `-`) and project exists
- **When:** `ProjectId` is parsed and resolved
- **Then:** Accepted as path-encoded ID directly; `get()` returns the raw input

---

### TC-3: UUID format accepted

- **Given:** Input string `"8d795a1c-c81d-4010-8d29-b4e678272419"` and project exists
- **When:** `ProjectId` is parsed and resolved
- **Then:** Accepted; `is_uuid()` returns true

---

### TC-4: Path(...) form accepted

- **Given:** Input string `"Path(\"/home/alice/projects/my-app\")"` and project exists
- **When:** `ProjectId` is parsed and resolved
- **Then:** Accepted; path extracted from the `Path(...)` wrapper and encoded to path-encoded form

---

### TC-5: Empty input rejected

- **Given:** Input string `""`
- **When:** `ProjectId` is parsed
- **Then:** Rejected; error message is `project must be non-empty`

---

### TC-6: Unresolvable project rejected

- **Given:** Input string `"/home/alice/no-such-project"` and no matching project in storage
- **When:** `ProjectId` is resolved
- **Then:** Rejected; error message is `project not found: /home/alice/no-such-project`
