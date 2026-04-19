# Parameter :: `type::`

Edge case tests for the `type::` parameter. Tests validate enum parsing and project filtering behavior.

**Source:** [params.md#parameter--17-type](../../../../../docs/cli/params.md#parameter--17-type) | [types.md#projecttype](../../../../../docs/cli/types.md#projecttype)

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

## Test Cases

### EC-1: Value "uuid" accepted

**Goal:** Verify that `type::uuid` is accepted and `.list` returns only UUID-named projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list type::uuid`
**Expected Output:** stdout lists only projects whose directory name matches a UUID pattern (e.g., `8d795a1c-c81d-4010-8d29-b4e678272419`); no path-encoded entries like `-home-user1-pro`.
**Verification:**
- Exit code is 0
- Every project entry in output has a UUID-format directory name
- No path-encoded project names (starting with `-home-`) appear in output
**Pass Criteria:** exit 0 + output contains only UUID-named projects (or empty if none exist)
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-2: Value "path" accepted

**Goal:** Verify that `type::path` is accepted and `.list` returns only path-encoded projects.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list type::path`
**Expected Output:** stdout lists only projects whose directory name is path-encoded (e.g., `-home-user1-pro-lib`); no UUID-format entries.
**Verification:**
- Exit code is 0
- Every project entry in output has a path-encoded directory name (starts with `-`)
- No UUID-format project names appear in output
**Pass Criteria:** exit 0 + output contains only path-encoded projects
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-3: Value "all" accepted

**Goal:** Verify that `type::all` is accepted and `.list` returns all projects regardless of naming scheme.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list type::all`
**Expected Output:** stdout lists all projects in storage — both path-encoded and UUID-named.
**Verification:**
- Exit code is 0
- Output count equals the sum of `type::path` and `type::uuid` results
**Pass Criteria:** exit 0 + output contains all projects (no naming-scheme filter)
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-4: Value "PATH" accepted (case-insensitive)

**Goal:** Verify that enum parsing is case-insensitive and `type::PATH` is treated identically to `type::path`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list type::PATH`
**Expected Output:** No error; output is identical to using lowercase `type::path`.
**Verification:**
- Exit code is 0
- Output matches the result of `clg .list type::path`
**Pass Criteria:** exit 0 + output identical to lowercase variant (case normalization applied)
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-5: Invalid value "both" rejected with error

**Goal:** Verify that `type::both` is rejected with the exact error message `"type must be uuid|path|all, got both"`.
**Setup:** None
**Command:** `clg .list type::both`
**Expected Output:** stderr contains `type must be uuid|path|all, got both`
**Verification:**
- Exit code is 1
- stderr contains the exact string `type must be uuid|path|all, got both`
**Pass Criteria:** exit 1 + error message `type must be uuid|path|all, got both`
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-6: Omitted defaults to "all"

**Goal:** Verify that omitting `type::` causes `.list` to show all projects (same as `type::all`).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list`
**Expected Output:** stdout lists all projects in storage, identical to running with `type::all`.
**Verification:**
- Exit code is 0
- Output matches the result of `clg .list type::all`
**Pass Criteria:** exit 0 + output includes all projects (default applied)
**Source:** [params.md](../../../../../docs/cli/params.md)

### EC-7: type::uuid returns only UUID-named projects

**Goal:** Verify that `type::uuid` filters correctly so that no path-encoded projects appear in the output.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture must contain at least one path-encoded project and at least one UUID project)
**Command:** `clg .list type::uuid`
**Expected Output:** Output contains UUID-named entries only; path-encoded project entries are absent.
**Verification:**
- Exit code is 0
- No lines in output contain path-encoded directory names (starting with `-home-` or `-root-`)
- Any UUID entries in the fixture appear in output
**Pass Criteria:** exit 0 + path-encoded projects absent from output when fixture contains both types
**Source:** [params.md](../../../../../docs/cli/params.md)
