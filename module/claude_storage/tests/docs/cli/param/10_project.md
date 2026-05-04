# Parameter :: `project::`

Edge case tests for the `project::` parameter. Tests validate multi-format identifier resolution and default behavior.

**Source:** [params.md#parameter--10-project](../../../../docs/cli/params.md#parameter--10-project) | [types.md#projectid](../../../../docs/cli/types.md#projectid)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absolute path format resolves correctly | Format Resolution |
| EC-2 | Path-encoded ID format resolves correctly | Format Resolution |
| EC-3 | UUID format resolves correctly | Format Resolution |
| EC-4 | Path(...) form from .list resolves correctly | Format Resolution |
| EC-5 | Unknown project value exits with error | Error Handling |
| EC-6 | Empty value rejected | Boundary Values |
| EC-7 | Default resolves to cwd project when omitted | Default |
| EC-8 | Default exits with 2 when cwd has no project | Default |

## Test Coverage Summary

- Format Resolution: 4 tests (EC-1, EC-2, EC-3, EC-4)
- Error Handling: 1 test (EC-5)
- Boundary Values: 1 test (EC-6)
- Default: 2 tests (EC-7, EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Absolute path format resolves correctly

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show project::/home/user1/pro/myproject`
- **Then:** Project view for the project at `/home/user1/pro/myproject` (must exist in fixture).; correct project displayed (same as using the path-encoded or UUID form)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Path-encoded ID format resolves correctly

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show project::-home-user1-pro-myproject`
- **Then:** Project view for the project encoded as `-home-user1-pro-myproject`; identical to absolute path form.; + same project displayed as absolute path form
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: UUID format resolves correctly

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show project::8d795a1c-c81d-4010-8d29-b4e678272419`
- **Then:** Project view for the UUID-named project (must exist in fixture as a UUID directory).; + UUID project correctly identified and displayed
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Path(...) form from .list resolves correctly

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show project::Path("/home/user1/pro/myproject")`
- **Then:** Project view for `/home/user1/pro/myproject`; identical to using the plain absolute path form.; + same project displayed as when using the raw absolute path
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Unknown project value exits with error

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show project::nonexistent-project-zzz`
- **Then:** `project not found: nonexistent-project-zzz`; + error message `project not found: nonexistent-project-zzz`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Empty value rejected

- **Given:** clean environment
- **When:** `clg .show project::`
- **Then:** Error about empty project value (e.g., `project must be non-empty`).; + error about empty project identifier
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Default resolves to cwd project when omitted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show` (run from a directory that has a known project in the fixture)
- **Then:** Project view for the project associated with the current working directory.; + cwd project displayed without explicit `project::` argument
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-8: Default exits with 2 when cwd has no project

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .show` (run from `/tmp` or another directory with no project entry in the fixture)
- **Then:** Error indicating no project found for the current directory.; + message indicating no project for cwd
- **Exit:** 2
- **Source:** [params.md](../../../../docs/cli/params.md)
