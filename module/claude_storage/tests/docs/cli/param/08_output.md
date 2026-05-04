# Parameter :: `output::`

Edge case tests for the `output::` parameter. Tests validate required enforcement, path handling, and overwrite behavior.

**Source:** [params.md#parameter--8-output](../../../../docs/cli/params.md#parameter--8-output)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Required — missing output:: exits with 1 | Required Enforcement |
| EC-2 | Absolute path accepted | Type Validation |
| EC-3 | ~ prefix path accepted | Path Expansion |
| EC-4 | Relative path accepted | Type Validation |
| EC-5 | Empty value rejected | Boundary Values |
| EC-6 | Nonexistent parent directory exits with 2 | Error Handling |
| EC-7 | Existing file is overwritten without error | Overwrite |
| EC-8 | Whitespace-only path rejected | Boundary Values |

## Test Coverage Summary

- Required Enforcement: 1 test (EC-1)
- Type Validation: 2 tests (EC-2, EC-4)
- Path Expansion: 1 test (EC-3)
- Boundary Values: 2 tests (EC-5, EC-8)
- Error Handling: 1 test (EC-6)
- Overwrite: 1 test (EC-7)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (valid/expected path) ↔ EC-2 (invalid/rejected path)

## Test Cases

---

### EC-1: Required — missing output:: exits with 1

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic`
- **Then:** stderr contains an error indicating `output::` is required or missing.; error indicating output path is required
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-2: Absolute path accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic output::/tmp/test-out/absolute.md`
- **Then:** No error; file written at the exact absolute path `/tmp/test-out/absolute.md`.; file created at specified absolute path
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-3: ~ prefix path accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic output::~/tmp/clg-test-output.md`
- **Then:** No error; file written at the expanded home-relative path.; file created at tilde-expanded path
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-4: Relative path accepted

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic output::session-output.md`
- **Then:** No error; file written at `session-output.md` relative to cwd.; file created at relative path (resolved from cwd)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-5: Empty value rejected

- **Given:** clean environment
- **When:** `clg .export session_id::-default_topic output::`
- **Then:** stderr contains an error indicating the path must be non-empty.; error indicating empty path is invalid
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-6: Nonexistent parent directory exits with 2

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
- **When:** `clg .export session_id::-default_topic output::/nonexistent/dir/file.md`
- **Then:** stderr contains an error indicating the parent directory does not exist.; exit non-zero + error about nonexistent parent directory
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-7: Existing file is overwritten without error

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` + create `/tmp/test-out/existing.md` with sentinel content `ORIGINAL`
- **When:** `clg .export session_id::-default_topic output::/tmp/test-out/existing.md`
- **Then:** No error; file at `/tmp/test-out/existing.md` contains new export content (original sentinel content gone).; file overwritten (original content replaced)
- **Exit:** 0
- **Source:** [params.md](../../../../docs/cli/params.md)

---

### EC-8: Whitespace-only path rejected

- **Given:** clean environment
- **When:** `clg .export session_id::-default_topic output::" "`
- **Then:** stderr contains an error indicating the path is invalid or empty.; error indicating whitespace-only path is invalid
- **Exit:** 1
- **Source:** [params.md](../../../../docs/cli/params.md)
