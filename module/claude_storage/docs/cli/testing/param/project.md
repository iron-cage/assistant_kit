# Parameter :: `project::`

Edge case tests for the `project::` parameter. Tests validate multi-format identifier resolution and default behavior.

**Source:** [params.md#parameter--10-project](../../params.md#parameter--10-project) | [types.md#projectid](../../types.md#projectid)

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

## Test Cases

### EC-1: Absolute path format resolves correctly

**Goal:** Verify that an absolute filesystem path is accepted as a project identifier and resolves to the correct project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show project::/home/user1/pro/myproject`
**Expected Output:** Project view for the project at `/home/user1/pro/myproject` (must exist in fixture).
**Verification:**
- Command exits with code 0
- Output shows the project associated with `/home/user1/pro/myproject`
- No error about unrecognized format appears on stderr
**Pass Criteria:** exit 0 + correct project displayed (same as using the path-encoded or UUID form)
**Source:** [params.md](../../params.md)

---

### EC-2: Path-encoded ID format resolves correctly

**Goal:** Verify that a path-encoded ID (hyphen-separated path segments) resolves to the same project as the absolute path form.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show project::-home-user1-pro-myproject`
**Expected Output:** Project view for the project encoded as `-home-user1-pro-myproject`; identical to absolute path form.
**Verification:**
- Command exits with code 0
- Output is identical to `project::/home/user1/pro/myproject` output
- No error about unrecognized format appears on stderr
**Pass Criteria:** exit 0 + same project displayed as absolute path form
**Source:** [params.md](../../params.md)

---

### EC-3: UUID format resolves correctly

**Goal:** Verify that a UUID project identifier resolves to the correct project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show project::8d795a1c-c81d-4010-8d29-b4e678272419`
**Expected Output:** Project view for the UUID-named project (must exist in fixture as a UUID directory).
**Verification:**
- Command exits with code 0
- Output shows the project stored under the UUID directory name
- No error about unrecognized format appears on stderr
**Pass Criteria:** exit 0 + UUID project correctly identified and displayed
**Source:** [params.md](../../params.md)

---

### EC-4: Path(...) form from .list resolves correctly

**Goal:** Verify that the `Path(...)` output form produced by `.list` can be passed back as a `project::` value.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show project::Path("/home/user1/pro/myproject")`
**Expected Output:** Project view for `/home/user1/pro/myproject`; identical to using the plain absolute path form.
**Verification:**
- Command exits with code 0
- Output matches `project::/home/user1/pro/myproject` output
- `Path(...)` wrapper syntax is parsed and the inner path extracted correctly
**Pass Criteria:** exit 0 + same project displayed as when using the raw absolute path
**Source:** [params.md](../../params.md)

---

### EC-5: Unknown project value exits with error

**Goal:** Verify that a project identifier that doesn't exist in storage produces the exact not-found error message.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show project::nonexistent-project-zzz`
**Expected Output:** `project not found: nonexistent-project-zzz`
**Verification:**
- Command exits with code 1
- Stderr contains the string `project not found: nonexistent-project-zzz`
**Pass Criteria:** exit 1 + error message `project not found: nonexistent-project-zzz`
**Source:** [params.md](../../params.md)

---

### EC-6: Empty value rejected

**Goal:** Verify that an empty `project::` value is rejected before any storage lookup is attempted.
**Setup:** None
**Command:** `clg .show project::`
**Expected Output:** Error about empty project value (e.g., `project must be non-empty`).
**Verification:**
- Command exits with code 1
- Stderr contains an error message about the empty value
- No storage access occurs (validation runs first)
**Pass Criteria:** exit 1 + error about empty project identifier
**Source:** [params.md](../../params.md)

---

### EC-7: Default resolves to cwd project when omitted

**Goal:** Verify that omitting `project::` causes the command to use the current working directory's project.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show` (run from a directory that has a known project in the fixture)
**Expected Output:** Project view for the project associated with the current working directory.
**Verification:**
- Command exits with code 0
- Output reflects the project for the cwd, not some other project
- No `project::` argument was needed
**Pass Criteria:** exit 0 + cwd project displayed without explicit `project::` argument
**Source:** [params.md](../../params.md)

---

### EC-8: Default exits with 2 when cwd has no project

**Goal:** Verify that when no `project::` is given and the cwd has no associated project in storage, the command exits with code 2.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .show` (run from `/tmp` or another directory with no project entry in the fixture)
**Expected Output:** Error indicating no project found for the current directory.
**Verification:**
- Command exits with code 2 (distinct from code 1 used for explicit bad values)
- Output or stderr indicates the cwd has no corresponding project in storage
**Pass Criteria:** exit 2 + message indicating no project for cwd
**Source:** [params.md](../../params.md)
