# Parameter :: `output::`

Edge case tests for the `output::` parameter. Tests validate required enforcement, path handling, and overwrite behavior.

**Source:** [params.md#parameter--8-output](../../params.md#parameter--8-output)

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

## Test Cases

### EC-1: Required — missing output:: exits with 1

**Goal:** Verify that `.export` without `output::` fails because the parameter is required.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic`
**Expected Output:** stderr contains an error indicating `output::` is required or missing.
**Verification:**
- Exit code is 1
- stderr contains a message indicating the output path is required
- No file is created
**Pass Criteria:** exit 1 + error indicating output path is required
**Source:** [params.md](../../params.md)

### EC-2: Absolute path accepted

**Goal:** Verify that an absolute filesystem path is accepted as a valid `output::` value.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic output::/tmp/test-out/absolute.md`
**Expected Output:** No error; file written at the exact absolute path `/tmp/test-out/absolute.md`.
**Verification:**
- Exit code is 0
- File exists at `/tmp/test-out/absolute.md`
- File contains non-empty content
**Pass Criteria:** exit 0 + file created at specified absolute path
**Source:** [params.md](../../params.md)

### EC-3: ~ prefix path accepted

**Goal:** Verify that a `~`-prefixed path is accepted and expanded to the home directory.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic output::~/tmp/clg-test-output.md`
**Expected Output:** No error; file written at the expanded home-relative path.
**Verification:**
- Exit code is 0
- File exists at `$HOME/tmp/clg-test-output.md` (tilde expanded)
- File contains non-empty content
**Pass Criteria:** exit 0 + file created at tilde-expanded path
**Source:** [params.md](../../params.md)

### EC-4: Relative path accepted

**Goal:** Verify that a relative path is accepted as a valid `output::` value.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic output::session-output.md`
**Expected Output:** No error; file written at `session-output.md` relative to cwd.
**Verification:**
- Exit code is 0
- File `session-output.md` exists in the current working directory
- File contains non-empty content
**Pass Criteria:** exit 0 + file created at relative path (resolved from cwd)
**Source:** [params.md](../../params.md)

### EC-5: Empty value rejected

**Goal:** Verify that `output::` with an empty value is rejected as an invalid path.
**Setup:** None
**Command:** `clg .export session_id::-default_topic output::`
**Expected Output:** stderr contains an error indicating the path must be non-empty.
**Verification:**
- Exit code is 1
- stderr contains a message about empty or invalid path
- No file is created
**Pass Criteria:** exit 1 + error indicating empty path is invalid
**Source:** [params.md](../../params.md)

### EC-6: Nonexistent parent directory exits with 2

**Goal:** Verify that specifying a path whose parent directory does not exist produces an error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`
**Command:** `clg .export session_id::-default_topic output::/nonexistent/dir/file.md`
**Expected Output:** stderr contains an error indicating the parent directory does not exist.
**Verification:**
- Exit code is 2 (or 1 per implementation)
- stderr contains an error about the parent directory not existing
- No file is created at `/nonexistent/dir/file.md`
**Pass Criteria:** exit non-zero + error about nonexistent parent directory
**Source:** [params.md](../../params.md)

### EC-7: Existing file is overwritten without error

**Goal:** Verify that if the output file already exists, it is silently overwritten without any warning or error.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` + create `/tmp/test-out/existing.md` with sentinel content `ORIGINAL`
**Command:** `clg .export session_id::-default_topic output::/tmp/test-out/existing.md`
**Expected Output:** No error; file at `/tmp/test-out/existing.md` contains new export content (original sentinel content gone).
**Verification:**
- Exit code is 0
- File at `/tmp/test-out/existing.md` no longer contains the string `ORIGINAL`
- File contains valid export content
**Pass Criteria:** exit 0 + file overwritten (original content replaced)
**Source:** [params.md](../../params.md)

### EC-8: Whitespace-only path rejected

**Goal:** Verify that a path containing only whitespace is rejected as effectively empty.
**Setup:** None
**Command:** `clg .export session_id::-default_topic output::" "`
**Expected Output:** stderr contains an error indicating the path is invalid or empty.
**Verification:**
- Exit code is 1
- stderr contains a message about invalid or empty path
- No file is created
**Pass Criteria:** exit 1 + error indicating whitespace-only path is invalid
**Source:** [params.md](../../params.md)
