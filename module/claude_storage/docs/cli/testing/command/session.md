# Command :: `.session`

Integration tests for the `.session` command. Tests verify history detection, exit code semantics, and path handling.

**Source:** [commands.md#command--7-session](../../commands.md#command--7-session)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | cwd with history exits with code 0 | Exit Codes |
| IT-2 | cwd without history exits with code 1 | Exit Codes |
| IT-3 | path:: with history exits with code 0 | Exit Codes |
| IT-4 | path:: without history exits with code 1 | Exit Codes |
| IT-5 | Output contains path-encoded project ID when found | Output Format |
| IT-6 | Output indicates not found when missing | Output Format |
| IT-7 | Does not list sessions (exits after status check) | Behavior Boundary |
| IT-8 | Nonexistent path exits with code 1 (not an error) | Exit Codes |

## Test Coverage Summary

- Exit Codes: 5 tests (IT-1, IT-2, IT-3, IT-4, IT-8)
- Output Format: 2 tests (IT-5, IT-6)
- Behavior Boundary: 1 test (IT-7)

## Test Cases

### IT-1: cwd with history exits with code 0

**Goal:** Verify `.session` exits with code `0` when cwd corresponds to a project with history in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project whose path-encoding matches the test cwd); run from that cwd.
**Command:** `clg .session`
**Expected Output:** Output confirming history exists for the current directory; exit code 0.
**Verification:**
- `$?` is `0`
- stdout contains an indication that history was found (e.g., project ID or "found" message)
- stderr is empty
**Pass Criteria:** exit 0 + history-found indication in output

**Source:** [commands.md](../../commands.md)

---

### IT-2: cwd without history exits with code 1

**Goal:** Verify `.session` exits with code `1` when cwd does not correspond to any project in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; run from a directory (e.g., `/tmp`) with no matching project in the fixture.
**Command:** `clg .session`
**Expected Output:** Output indicating no history found; exit code 1 (not an error, just not found).
**Verification:**
- `$?` is `1`
- stderr is empty (exit code 1 is not an error condition)
- stdout may be empty or contain a "not found" indication
**Pass Criteria:** exit 1 + no error on stderr

**Source:** [commands.md](../../commands.md)

---

### IT-3: path:: with history exits with code 0

**Goal:** Verify `.session path::PATH` exits with code `0` when the specified path has history in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project whose path-encoding matches `/home/user1/pro/alpha`)
**Command:** `clg .session path::/home/user1/pro/alpha`
**Expected Output:** Output confirming history exists for `/home/user1/pro/alpha`; exit code 0.
**Verification:**
- `$?` is `0`
- stdout contains an indication that history was found for the path
- stderr is empty
**Pass Criteria:** exit 0 + history found for specified path

**Source:** [commands.md](../../commands.md)

---

### IT-4: path:: without history exits with code 1

**Goal:** Verify `.session path::PATH` exits with code `1` when the specified path has no history in storage.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; use a path `/home/user1/pro/no-history` that has no matching project in the fixture.
**Command:** `clg .session path::/home/user1/pro/no-history`
**Expected Output:** No-history indication or empty output; exit code 1.
**Verification:**
- `$?` is `1`
- stderr is empty (not an error)
- stdout is empty or contains a "not found" message
**Pass Criteria:** exit 1 + no error on stderr

**Source:** [commands.md](../../commands.md)

---

### IT-5: Output contains path-encoded project ID when found

**Goal:** Verify that when history is found, stdout includes the path-encoded project ID corresponding to the directory.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project stored as path-encoded ID `-home-user1-pro-alpha`)
**Command:** `clg .session path::/home/user1/pro/alpha`
**Expected Output:** stdout contains the path-encoded project ID `-home-user1-pro-alpha` or the resolved project path.
**Verification:**
- stdout contains `-home-user1-pro-alpha` (or equivalent path-encoded representation)
- the project ID in output corresponds to the path provided
- stderr is empty
**Pass Criteria:** exit 0 + path-encoded project ID present in output

**Source:** [commands.md](../../commands.md)

---

### IT-6: Output indicates not found when missing

**Goal:** Verify that when no history exists for a path, stdout or stderr contains a clear "not found" indication.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; use a path with no matching project.
**Command:** `clg .session path::/home/user1/pro/no-history`
**Expected Output:** A message indicating no history was found (or empty output that is accepted by callers checking exit code).
**Verification:**
- `$?` is `1`
- output (stdout or stderr) is consistent: either empty (scripts rely on exit code) or contains a "not found" message
- no confusing partial output (e.g., no partial project ID for the wrong project)
**Pass Criteria:** exit 1 + unambiguous "not found" output or empty output

**Source:** [commands.md](../../commands.md)

---

### IT-7: Does not list sessions (exits after status check)

**Goal:** Verify `.session` reports only whether history exists and does not enumerate individual sessions — distinguishing it from `.list` or `.sessions`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project at matching path with 3 sessions)
**Command:** `clg .session path::/home/user1/pro/alpha`
**Expected Output:** A single-line or brief found/not-found output; no session IDs, session counts, or session listing rows.
**Verification:**
- stdout does not contain individual session ID entries (no UUID or `-default_topic` style session IDs enumerated)
- stdout does not contain a multi-row session table
- output is brief (one line or a few lines at most)
- exit code is `0` (history exists)
**Pass Criteria:** exit 0 + output contains no session listing

**Source:** [commands.md](../../commands.md)

---

### IT-8: Nonexistent path exits with code 1 (not an error)

**Goal:** Verify that a completely nonexistent filesystem path treated as having no history exits with `1`, not `2`.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture`; the path `/home/user1/pro/totally-nonexistent-xyz` does not exist on the filesystem.
**Command:** `clg .session path::/home/user1/pro/totally-nonexistent-xyz`
**Expected Output:** No-history exit code; not a storage read error.
**Verification:**
- `$?` is `1` (not found, not an error)
- `$?` is NOT `2` (nonexistent path is treated as "no history", not as storage failure)
- stderr is empty
**Pass Criteria:** exit 1 (not 2) + no error on stderr for nonexistent filesystem path

**Source:** [commands.md](../../commands.md)
