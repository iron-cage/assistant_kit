# Parameter :: `--session-dir`

Edge case tests for the **deprecated, inert** session directory parameter
(Fix(BUG-493)). Tests validate that the parameter is still parsed (no hard
failure), applies no effect (no `CLAUDE_CODE_SESSION_DIR` export, no `-c`
gating role), warns loudly on stderr, and remains help-documented.

**Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--session-dir /path` → accepted, warns, NO `CLAUDE_CODE_SESSION_DIR` export | Deprecation |
| EC-2 | `--session-dir` without value → exit 1 | Missing Value |
| EC-3 | Default (no `--session-dir`) → `CLAUDE_CODE_SESSION_DIR` absent, no warning | Deprecation |
| EC-4 | `--session-dir` + `--new-session` → both accepted, inert + no `-c` | Interaction |
| EC-5 | `--help` lists `--session-dir` with DEPRECATED description | Documentation |
| EC-6 | Non-existent path accepted without validation at runner layer | Permissive |
| EC-7 | Override dir WITH a session gates nothing: no `-c` when real storage is empty | Deprecation |

## Test Coverage Summary

- Deprecation: 3 tests (EC-1, EC-3, EC-7)
- Missing Value: 1 test (EC-2)
- Interaction: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Permissive: 1 test (EC-6)

**Total:** 7 edge cases


## Test Cases
---

### EC-1: `--session-dir /path` accepted, inert, warns

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions "Fix bug"`
- **Then:** Env block does NOT contain `CLAUDE_CODE_SESSION_DIR=`; stderr contains the one-line `--session-dir is deprecated` warning
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
---

### EC-2: `--session-dir` without value → exit 1

- **Given:** clean environment
- **When:** `clr --session-dir`
- **Then:** Exit 1; error about missing `--session-dir` value
- **Exit:** 1
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
---

### EC-3: Default → no env var, no warning

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Env block does NOT contain `CLAUDE_CODE_SESSION_DIR=`; stderr does NOT contain the deprecation warning (fires only when the parameter is given)
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
---

### EC-4: `--session-dir` + `--new-session` → no conflict, both inert-compatible

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions --new-session "Fix bug"`
- **Then:** Env block does NOT contain `CLAUDE_CODE_SESSION_DIR=`; no `-c` flag (from `--new-session`); exit 0
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--session-dir` as DEPRECATED

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--session-dir` (described as DEPRECATED, no effect)
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
---

### EC-6: Non-existent path accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /no/such/dir "Fix bug"`
- **Then:** Exit 0; no path validation error (the inert value is never dereferenced)
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
---

### EC-7: Override dir contents gate nothing (bug_reproducer(BUG-493))

- **Given:** override dir containing a `.jsonl` session; empty `CLAUDE_HOME` (real source storage has no session)
- **When:** `clr --dry-run --session-dir <override_dir> "test"`
- **Then:** No `CLAUDE_CODE_SESSION_DIR=` export; no `-c` (the override dir's contents must not gate continuation); deprecation warning on stderr
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
