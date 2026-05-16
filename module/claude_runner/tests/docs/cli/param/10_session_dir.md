# Parameter :: `--session-dir`

Edge case tests for the session directory parameter. Tests validate path forwarding, missing-value rejection, and help documentation.

**Source:** [10_session_dir.md](../../../../docs/cli/param/10_session_dir.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--session-dir /path` → sets `CLAUDE_CODE_SESSION_DIR` env var | Behavioral Divergence |
| EC-2 | `--session-dir` without value → exit 1 | Missing Value |
| EC-3 | Default (no `--session-dir`) → `CLAUDE_CODE_SESSION_DIR` absent from env block | Behavioral Divergence |
| EC-4 | `--session-dir` + `--new-session` → both accepted | Interaction |
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

### EC-1: `--session-dir /path` sets env var

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions "Fix bug"`
- **Then:** Env block contains `CLAUDE_CODE_SESSION_DIR=/tmp/sessions` (runner converts flag to env var for subprocess)
- **Exit:** 0
- **Source:** [10_session_dir.md](../../../../docs/cli/param/10_session_dir.md)
---

### EC-2: `--session-dir` without value → exit 1

- **Given:** clean environment
- **When:** `clr --session-dir`
- **Then:** Exit 1; error about missing `--session-dir` value
- **Exit:** 1
- **Source:** [10_session_dir.md](../../../../docs/cli/param/10_session_dir.md)
---

### EC-3: Default → `CLAUDE_CODE_SESSION_DIR` absent from env block

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Env block does NOT contain `CLAUDE_CODE_SESSION_DIR=`
- **Exit:** 0
- **Source:** [10_session_dir.md](../../../../docs/cli/param/10_session_dir.md)
---

### EC-4: `--session-dir` + `--new-session` → no conflict

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /tmp/sessions --new-session "Fix bug"`
- **Then:** Env block contains `CLAUDE_CODE_SESSION_DIR=/tmp/sessions`; no `-c` flag; exit 0
- **Exit:** 0
- **Source:** [10_session_dir.md](../../../../docs/cli/param/10_session_dir.md)
---

### EC-5: `--help` lists `--session-dir`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--session-dir`
- **Exit:** 0
- **Source:** [command.md](../../../../docs/cli/command.md#command--2-help)
---

### EC-6: Non-existent path accepted without validation

- **Given:** clean environment
- **When:** `clr --dry-run --session-dir /no/such/dir "Fix bug"`
- **Then:** Exit 0; no path validation error (runner accepts any string as session dir value)
- **Exit:** 0
- **Source:** [10_session_dir.md](../../../../docs/cli/param/10_session_dir.md)
