# Command :: `.status`

Integration tests for the `.status` command. Tests verify storage overview, statistics output, and verbosity behavior.

**Source:** [commands.md#command--1-status](../../../../docs/cli/commands.md#command--1-status)

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

---

### IT-1: Default output with real storage

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, 3 sessions total)
- **When:** `clg .status`
- **Then:** A summary table containing project count (`2`) and session count (`3`).; summary table with correct project and session counts
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-2: Verbosity 0 machine-readable output

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, 3 sessions total)
- **When:** `clg .status verbosity::0`
- **Then:** Two key-value lines: `projects: 2` and `sessions: 3` (no table borders, no labels).; + exactly `projects: N, sessions: N` format (machine-parseable)
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-3: Verbosity 2 detailed per-project output

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, each with 1 session containing known entry counts)
- **When:** `clg .status verbosity::2`
- **Then:** Summary table plus per-project rows showing session counts and entry type breakdown (user/assistant).; + per-project breakdown with entry type counts present
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-4: Custom storage path via path::

- **Given:** Create fixture at `/tmp/alt-fixture` with 1 project, 1 session; use a different default storage root that has different counts.
- **When:** `clg .status path::/tmp/alt-fixture`
- **Then:** Summary showing counts from `/tmp/alt-fixture` (1 project, 1 session), not from default storage.; + counts reflect `/tmp/alt-fixture` content
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-5: Custom storage path via CLAUDE_STORAGE_ROOT env

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, 3 sessions)
- **When:** `clg .status`
- **Then:** Summary reflecting the fixture's 2 projects and 3 sessions, not the real `~/.claude/` counts.; + counts match `CLAUDE_STORAGE_ROOT` fixture, not `~/.claude/`
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-6: Exit code 0 on success

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (any valid fixture)
- **When:** `clg .status`
- **Then:** Normal summary output on stdout.
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-7: Exit code 2 on unreadable storage path

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/nonexistent-storage-xyz`
- **When:** `clg .status`
- **Then:** Error message on stderr indicating storage read failure; no summary on stdout.; + error message on stderr
- **Exit:** 2
- **Source:** [commands.md](../../../../docs/cli/commands.md)

---

### IT-8: Output contains project count and session count

- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 3 projects, 5 sessions total)
- **When:** `clg .status`
- **Then:** Output includes labeled project count (`3`) and labeled session count (`5`).; + both `projects` and `sessions` counts present and accurate
- **Exit:** 0
- **Source:** [commands.md](../../../../docs/cli/commands.md)
