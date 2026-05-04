# Parameter :: `path::`

Edge case tests for the `path::` parameter. Tests validate semantics per-command, path expansion, and empty value handling.

**Source:** [params.md#parameter--9-path](../../../../docs/cli/params.md#parameter--9-path) | [types.md#storagepath](../../../../docs/cli/types.md#storagepath)

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

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Absolute path accepted in .status

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status path::/tmp/test-fixture`
- **Then:** Status output reflecting the storage at `/tmp/test-fixture` (same as default in this setup).; status output references the given absolute path as storage root
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: ~ prefix expanded in .status

- **Given:** clean environment
- **When:** `clg .status path::~/.claude/`
- **Then:** Status output reflecting the storage at the expanded home path.; + `~` expanded correctly and storage at that path is reported
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: Relative path accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .exists path::subdir/project`
- **Then:** Exit 0 if the resolved path has history, exit 1 if not — either way no format error.; or 1 + relative path accepted without format error
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Empty value rejected

- **Given:** clean environment
- **When:** `clg .status path::`
- **Then:** `path must be non-empty`; + error message `path must be non-empty`
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Substring filter in .list matches case-insensitively

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list path::MYPROJECT`
- **Then:** Projects whose paths contain `myproject` (case-insensitive) — same result as `path::myproject`.; + results match what lowercase `path::myproject` would return
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Substring filter in .list with no match returns empty list

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list path::zzznomatch999`
- **Then:** Empty list or "no projects found" message; no error.; + empty result set (no projects match)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Default in .exists resolves to cwd

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .exists` (run from a directory that has a known project in the fixture)
- **Then:** Exit 0 indicating the cwd project has history.; + cwd project recognized as having history without explicit `path::` argument
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-8: Nonexistent path in .exists exits with code 1 (not error)

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .exists path::/tmp/nonexistent-dir-xyzabc`
- **Then:** "No session history found" or similar not-found message; no stack trace or exception output.; + graceful not-found message (not a crash or unhandled error)
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)
