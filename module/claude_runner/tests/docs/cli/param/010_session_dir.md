# Parameter :: `--session-dir`

Edge case tests for the session directory parameter — deprecated and inert (BUG-493): claude ≥2.x ignores the `CLAUDE_CODE_SESSION_DIR` export this flag used to set, so it has zero effect on session storage. Tests validate the deprecation-warning/no-op behavior, missing-value rejection, and help documentation.

**Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--session-dir /path` → deprecated, inert; `CLAUDE_CODE_SESSION_DIR` never set (BUG-493) | Behavioral Divergence |
| EC-2 | `--session-dir` without value → exit 1 | Missing Value |
| EC-3 | Default (no `--session-dir`) → `CLAUDE_CODE_SESSION_DIR` absent from env block | Behavioral Divergence |
| EC-4 | `--session-dir` + `--new-session` → both accepted (`--session-dir` deprecated, inert) | Interaction |
| EC-5 | `--help` lists `--session-dir` | Documentation |
| EC-6 | Non-existent path accepted without validation at runner layer | Permissive |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-3)
- Missing Value: 1 test (EC-2)
- Interaction: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Permissive: 1 test (EC-6)

**Total:** 6 edge cases


## Test Cases
---

### EC-1: `--session-dir /path` deprecated, inert; env var never set (BUG-493)

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions "Fix bug"`
- **Then:** Env block does NOT contain `CLAUDE_CODE_SESSION_DIR=` — claude ≥2.x ignores this export for both reads and writes, so the flag no longer has any effect on session storage; stderr carries a deprecation warning naming `/tmp/sessions`
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

### EC-3: Default → `CLAUDE_CODE_SESSION_DIR` absent from env block

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Env block does NOT contain `CLAUDE_CODE_SESSION_DIR=`
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
---

### EC-4: `--session-dir` + `--new-session` → both accepted (`--session-dir` deprecated, inert)

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions --new-session "Fix bug"`
- **Then:** Both flags accepted, no conflict; env block does NOT contain `CLAUDE_CODE_SESSION_DIR=` (deprecated, zero effect — BUG-493); no `-c` flag (suppressed by `--new-session`); exit 0
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
---

### EC-5: `--help` lists `--session-dir`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--session-dir`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask
---

### EC-6: Non-existent path accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /no/such/dir "Fix bug"`
- **Then:** Exit 0; no path validation error (runner accepts any string as session dir value)
- **Exit:** 0
- **Source:** [010_session_dir.md](../../../../docs/cli/param/010_session_dir.md)
- **Commands:** run, ask
