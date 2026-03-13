# Command :: `.status`

Integration tests for the `.status` command. Tests verify storage overview, statistics output, and verbosity behavior.

**Source:** [commands.md#command--1-status](../../commands.md#command--1-status)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Default output with real storage | Read Operations |
| IT-2 | Verbosity 0 machine-readable output | Output Format |
| IT-3 | Verbosity 2 detailed per-project output | Output Format |
| IT-4 | Custom storage path via path:: | Configuration |
| IT-5 | Custom storage path via CLAUDE_STORAGE_ROOT env | Configuration |
| IT-6 | Exit code 0 on success | Exit Codes |
| IT-7 | Exit code 2 on unreadable storage path | Exit Codes |
| IT-8 | Output contains project count and session count | Read Operations |

## Test Coverage Summary

- Read Operations: 2 tests (IT-1, IT-8)
- Output Format: 2 tests (IT-2, IT-3)
- Configuration: 2 tests (IT-4, IT-5)
- Exit Codes: 2 tests (IT-6, IT-7)

## Test Cases

### IT-1: Default output with real storage

**Goal:** Verify `.status` runs without arguments and produces a summary table with project and session totals.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, 3 sessions total)
**Command:** `clg .status`
**Expected Output:** A summary table containing project count (`2`) and session count (`3`).
**Verification:**
- stdout contains the word `projects` or a project-count label
- stdout contains a numeric project count matching fixture (2)
- stdout contains a numeric session count matching fixture (3)
- stderr is empty
**Pass Criteria:** exit 0 + summary table with correct project and session counts

**Source:** [commands.md](../../commands.md)

---

### IT-2: Verbosity 0 machine-readable output

**Goal:** Verify `verbosity::0` produces bare numeric counts with no decorative formatting.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, 3 sessions total)
**Command:** `clg .status verbosity::0`
**Expected Output:** Two key-value lines: `projects: 2` and `sessions: 3` (no table borders, no labels).
**Verification:**
- stdout matches pattern `projects: \d+`
- stdout matches pattern `sessions: \d+`
- stdout does not contain table-border characters (`|`, `+`, `-` in formatting context)
- numeric values match fixture project and session counts
**Pass Criteria:** exit 0 + exactly `projects: N, sessions: N` format (machine-parseable)

**Source:** [commands.md](../../commands.md)

---

### IT-3: Verbosity 2 detailed per-project output

**Goal:** Verify `verbosity::2` adds per-project session counts and user/assistant entry breakdowns beyond the default summary.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, each with 1 session containing known entry counts)
**Command:** `clg .status verbosity::2`
**Expected Output:** Summary table plus per-project rows showing session counts and entry type breakdown (user/assistant).
**Verification:**
- stdout contains all content that `verbosity::1` produces
- stdout also contains per-project breakdown (individual project paths or IDs listed)
- stdout contains entry type labels (`user` and/or `assistant`)
- output is longer than the `verbosity::1` output for the same fixture
**Pass Criteria:** exit 0 + per-project breakdown with entry type counts present

**Source:** [commands.md](../../commands.md)

---

### IT-4: Custom storage path via path::

**Goal:** Verify `path::` parameter redirects storage root to the specified directory.
**Setup:** Create fixture at `/tmp/alt-fixture` with 1 project, 1 session; use a different default storage root that has different counts.
**Command:** `clg .status path::/tmp/alt-fixture`
**Expected Output:** Summary showing counts from `/tmp/alt-fixture` (1 project, 1 session), not from default storage.
**Verification:**
- stdout project count is `1` (matches `/tmp/alt-fixture`, not default storage)
- stdout session count is `1` (matches `/tmp/alt-fixture`, not default storage)
- no reference to default storage path in output
**Pass Criteria:** exit 0 + counts reflect `/tmp/alt-fixture` content

**Source:** [commands.md](../../commands.md)

---

### IT-5: Custom storage path via CLAUDE_STORAGE_ROOT env

**Goal:** Verify `CLAUDE_STORAGE_ROOT` environment variable overrides the default `~/.claude/` storage root.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, 3 sessions)
**Command:** `clg .status`
**Expected Output:** Summary reflecting the fixture's 2 projects and 3 sessions, not the real `~/.claude/` counts.
**Verification:**
- stdout project count matches fixture (2), not the real `~/.claude/` project count
- stdout session count matches fixture (3)
- command runs without error
**Pass Criteria:** exit 0 + counts match `CLAUDE_STORAGE_ROOT` fixture, not `~/.claude/`

**Source:** [commands.md](../../commands.md)

---

### IT-6: Exit code 0 on success

**Goal:** Verify the command exits with code `0` when storage is readable and the command completes normally.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (any valid fixture)
**Command:** `clg .status`
**Expected Output:** Normal summary output on stdout.
**Verification:**
- `$?` is `0` after command completes
- stdout is non-empty
- stderr is empty
**Pass Criteria:** exit 0

**Source:** [commands.md](../../commands.md)

---

### IT-7: Exit code 2 on unreadable storage path

**Goal:** Verify the command exits with code `2` when the storage root path cannot be read (permission denied or nonexistent).
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/nonexistent-storage-xyz`
**Command:** `clg .status`
**Expected Output:** Error message on stderr indicating storage read failure; no summary on stdout.
**Verification:**
- `$?` is `2` after command completes
- stderr is non-empty and contains an error indication
- stdout is empty or contains no valid summary data
**Pass Criteria:** exit 2 + error message on stderr

**Source:** [commands.md](../../commands.md)

---

### IT-8: Output contains project count and session count

**Goal:** Verify both project count and session count fields are always present in default output regardless of fixture size.
**Setup:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects, 5 sessions total)
**Command:** `clg .status`
**Expected Output:** Output includes labeled project count (`3`) and labeled session count (`5`).
**Verification:**
- stdout contains a project count value of `3`
- stdout contains a session count value of `5`
- both counts appear in the same output (not on separate invocations)
- values are accurate to the fixture state
**Pass Criteria:** exit 0 + both `projects` and `sessions` counts present and accurate

**Source:** [commands.md](../../commands.md)
