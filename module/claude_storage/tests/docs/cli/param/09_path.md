# Parameter :: `path::`

Edge case tests for the `path::` parameter. Tests validate semantics per-command, path expansion, and empty value handling.

**Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md) | [type/10_storage_path.md](../../../../docs/cli/type/10_storage_path.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Absolute path accepted in .status | Type Validation |
| EC-2 | ~ prefix expanded in .status | Path Expansion |
| EC-3 | Relative path accepted in .project.exists | Type Validation |
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

**Behavioral Divergence Pair:** EC-1 (absolute path in .status) ↔ EC-2 (~ prefix in .status)

## Test Cases

---

### EC-1: Absolute path accepted in .status

- **Commands:** `.status`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .status path::/tmp/test-fixture`
- **Then:** Status output reflecting the storage at `/tmp/test-fixture` (same as default in this setup).; status output references the given absolute path as storage root
- **Exit:** 0
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### EC-2: ~ prefix expanded in .status

- **Commands:** `.status`
- **Given:** clean environment
- **When:** `clg .status path::~/.claude/`
- **Then:** Status output reflecting the storage at the expanded home path.; + `~` expanded correctly and storage at that path is reported
- **Exit:** 0
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### EC-3: Relative path accepted in .project.exists

- **Commands:** `.project.exists`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture has no storage for `subdir/project`)
- **When:** `clg .project.exists path::subdir/project`
- **Then:** Exit 1 (no history for this path); relative path format accepted without a format-validation error
- **Exit:** 1
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### EC-4: Empty value rejected

- **Commands:** `.status`
- **Given:** clean environment
- **When:** `clg .status path::`
- **Then:** `path must be non-empty`; + error message `path must be non-empty`
- **Exit:** 1
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### EC-5: Substring filter in .list matches case-insensitively

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list path::MYPROJECT`
- **Then:** Projects whose paths contain `myproject` (case-insensitive) — same result as `path::myproject`.; + results match what lowercase `path::myproject` would return
- **Exit:** 0
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### EC-6: Substring filter in .list with no match returns empty list

- **Commands:** `.list`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .list path::zzznomatch999`
- **Then:** Empty list or "no projects found" message; no error.; + empty result set (no projects match)
- **Exit:** 0
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### EC-7: Default in .exists resolves to cwd

- **Commands:** `.project.exists`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .project.exists` (run from a directory that has a known project in the fixture)
- **Then:** Exit 0 indicating the cwd project has history.; + cwd project recognized as having history without explicit `path::` argument
- **Exit:** 0
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)

---

### EC-8: Nonexistent path in .exists exits with code 1 (not error)

- **Commands:** `.project.exists`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .project.exists path::/tmp/nonexistent-dir-xyzabc`
- **Then:** "No session history found" or similar not-found message; no stack trace or exception output.; + graceful not-found message (not a crash or unhandled error)
- **Exit:** 1
- **Source:** [param/09_path.md](../../../../docs/cli/param/09_path.md)
