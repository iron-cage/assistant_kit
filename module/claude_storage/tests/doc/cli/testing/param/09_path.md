# Parameter :: `path::`

Edge case tests for the `path::` parameter. Tests validate semantics per-command, path expansion, and empty value handling.

**Source:** [params.md#parameter--9-path](../../../../../docs/cli/params.md#parameter--9-path) | [types.md#storagepath](../../../../../docs/cli/types.md#storagepath)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absolute path accepted in .status | Type Validation |
| EC-2 | ~ prefix expanded in .status | Path Expansion |
| EC-3 | Relative path accepted | Type Validation |
| EC-4 | Empty value rejected | Boundary Values |
| EC-5 | Substring filter in .list matches case-insensitively | Semantics (.list) |
| EC-6 | Substring filter in .list with no match returns empty list | Semantics (.list) |
| EC-7 | Default in .exists resolves to cwd | Default |
| EC-8 | Nonexistent path in .exists exits with code 1 (not error) | Semantics (.exists) |

## Test Coverage Summary

- Type Validation: 2 tests (EC-1, EC-3)
- Path Expansion: 1 test (EC-2)
- Boundary Values: 1 test (EC-4)
- Semantics (.list): 2 tests (EC-5, EC-6)
- Default: 1 test (EC-7)
- Semantics (.exists): 1 test (EC-8)

## Test Cases

### EC-1: Absolute path accepted in .status

**Goal:** Verify that an absolute filesystem path is accepted as the storage root override for `.status`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .status path::/tmp/test-fixture`
**Expected Output:** Status output reflecting the storage at `/tmp/test-fixture` (same as default in this setup).
**Verification:**
- Command exits with code 0
- No error message about invalid path appears on stderr
- Output reflects content at the specified absolute path
**Pass Criteria:** exit 0 + status output references the given absolute path as storage root
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-2: ~ prefix expanded in .status

**Goal:** Verify that a `~`-prefixed path is expanded to the home directory for `.status`.
**Setup:** None (uses real `~/.claude/` directory which must exist)
**Command:** `clg .status path::~/.claude/`
**Expected Output:** Status output reflecting the storage at the expanded home path.
**Verification:**
- Command exits with code 0
- No error about unexpanded `~` appears on stderr
- Output is equivalent to running `.status` with the full absolute home path
**Pass Criteria:** exit 0 + `~` expanded correctly and storage at that path is reported
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-3: Relative path accepted

**Goal:** Verify that a relative path is accepted (relative to cwd at invocation time).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .exists path::subdir/project`
**Expected Output:** Exit 0 if the resolved path has history, exit 1 if not — either way no format error.
**Verification:**
- Command exits with code 0 or 1 (path accepted syntactically; exit reflects presence/absence)
- No error about path format rejection appears on stderr
**Pass Criteria:** exit 0 or 1 + relative path accepted without format error
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-4: Empty value rejected

**Goal:** Verify that an empty `path::` value is rejected with an error.
**Setup:** None
**Command:** `clg .status path::`
**Expected Output:** `path must be non-empty`
**Verification:**
- Command exits with code 1
- Stderr contains the string `path must be non-empty`
**Pass Criteria:** exit 1 + error message `path must be non-empty`
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-5: Substring filter in .list matches case-insensitively

**Goal:** Verify that `path::` in `.list` performs case-insensitive substring matching against project paths.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list path::MYPROJECT`
**Expected Output:** Projects whose paths contain `myproject` (case-insensitive) — same result as `path::myproject`.
**Verification:**
- Command exits with code 0
- Projects matching `myproject` (lowercase) in their path appear in output
- No error about case sensitivity appears on stderr
**Pass Criteria:** exit 0 + results match what lowercase `path::myproject` would return
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-6: Substring filter in .list with no match returns empty list

**Goal:** Verify that `path::` in `.list` with a non-matching substring returns an empty list (not an error).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .list path::zzznomatch999`
**Expected Output:** Empty list or "no projects found" message; no error.
**Verification:**
- Command exits with code 0
- Output contains no project entries
- No error about the filter value appears on stderr
**Pass Criteria:** exit 0 + empty result set (no projects match)
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-7: Default in .exists resolves to cwd

**Goal:** Verify that omitting `path::` in `.exists` uses the current working directory as the project path.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .exists` (run from a directory that has a known project in the fixture)
**Expected Output:** Exit 0 indicating the cwd project has history.
**Verification:**
- Command exits with code 0
- No `path::` argument was needed to identify the project
- No error about missing path appears on stderr
**Pass Criteria:** exit 0 + cwd project recognized as having history without explicit `path::` argument
**Source:** [params.md](../../../../../docs/cli/params.md)

---

### EC-8: Nonexistent path in .exists exits with code 1 (not error)

**Goal:** Verify that a nonexistent directory path in `.exists` exits with code 1 indicating no history (not a hard error).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .exists path::/tmp/nonexistent-dir-xyzabc`
**Expected Output:** "No session history found" or similar not-found message; no stack trace or exception output.
**Verification:**
- Command exits with code 1
- Output or stderr indicates no history was found for that path
- No exception backtrace or unexpected error format in output
**Pass Criteria:** exit 1 + graceful not-found message (not a crash or unhandled error)
**Source:** [params.md](../../../../../docs/cli/params.md)
