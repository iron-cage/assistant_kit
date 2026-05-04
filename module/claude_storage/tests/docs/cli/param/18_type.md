# Parameter :: `type::`

Edge case tests for the `type::` parameter. Tests validate enum parsing and project filtering behavior.

**Source:** [params.md#parameter--17-type](../../../../docs/cli/params.md#parameter--17-type) | [types.md#projecttype](../../../../docs/cli/types.md#projecttype)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Value "uuid" accepted | Enum Values |
| EC-2 | Value "path" accepted | Enum Values |
| EC-3 | Value "all" accepted | Enum Values |
| EC-4 | Value "PATH" accepted (case-insensitive) | Case Insensitivity |
| EC-5 | Invalid value "both" rejected with error | Error Handling |
| EC-6 | Omitted defaults to "all" | Default |
| EC-7 | type::uuid returns only UUID-named projects | Behavior |

## Test Coverage Summary

- Enum Values: 3 tests (EC-1, EC-2, EC-3)
- Case Insensitivity: 1 test (EC-4)
- Error Handling: 1 test (EC-5)
- Default: 1 test (EC-6)
- Behavior: 1 test (EC-7)

**Total:** 7 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Value "uuid" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list type::uuid`
- **Then:** stdout lists only projects whose directory name matches a UUID pattern (e.g., `8d795a1c-c81d-4010-8d29-b4e678272419`); no path-encoded entries like `-home-user1-pro`.; output contains only UUID-named projects (or empty if none exist)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Value "path" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list type::path`
- **Then:** stdout lists only projects whose directory name is path-encoded (e.g., `-home-user1-pro-lib`); no UUID-format entries.; output contains only path-encoded projects
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Value "all" accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list type::all`
- **Then:** stdout lists all projects in storage — both path-encoded and UUID-named.; output contains all projects (no naming-scheme filter)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Value "PATH" accepted (case-insensitive)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list type::PATH`
- **Then:** No error; output is identical to using lowercase `type::path`.; output identical to lowercase variant (case normalization applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Invalid value "both" rejected with error

- **Given:** clean environment
- **When:** `clg .list type::both`
- **Then:** stderr contains `type must be uuid|path|all, got both`; error message `type must be uuid|path|all, got both`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Omitted defaults to "all"

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list`
- **Then:** stdout lists all projects in storage, identical to running with `type::all`.; output includes all projects (default applied)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: type::uuid returns only UUID-named projects

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture must contain at least one path-encoded project and at least one UUID project)
- **When:** `clg .list type::uuid`
- **Then:** Output contains UUID-named entries only; path-encoded project entries are absent.; path-encoded projects absent from output when fixture contains both types
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)
