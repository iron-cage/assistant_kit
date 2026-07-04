# User Story: Trace Execution

- **Source:** [docs/cli/user_story/008_trace_execution.md](../../../../docs/cli/user_story/008_trace_execution.md)
- **Primary flags:** `--trace`
- **Command:** `run`, `isolated`, `refresh`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--trace` prints command to stderr before executing |
| US-2 | Cross-command | `--trace` works on `isolated` command |
| US-3 | Independence | `--trace` output independent of `--quiet` flag |
| US-4 | Parameter interaction | `--trace` with `--dry-run` shows trace but no execution |

---

### US-1: trace prints command to stderr

- **Given:** Terminal with TTY attached
- **When:** `clr --trace "test message" --dry-run`
- **Then:** stderr contains the full assembled command (like `set -x` output); stdout contains the dry-run output
- **Exit:** 0

### US-2: trace on isolated command

- **Given:** Credentials file exists at `/tmp/test_creds.json`
- **When:** `clr isolated --creds /tmp/test_creds.json --trace --dry-run "test"`
- **Then:** stderr contains the trace of the assembled `isolated` command including credential path; stdout has dry-run output
- **Exit:** 0

### US-3: trace independent of --quiet

- **Given:** Terminal with TTY attached
- **When:** `clr --trace --quiet "test"` (PATH=/nonexistent)
- **Then:** stderr still contains trace output even with --quiet — trace is a separate channel from runner diagnostic suppression
- **Exit:** non-zero (binary not found, but trace fires first)

### US-4: trace with dry-run

- **Given:** No subprocess execution expected
- **When:** `clr --trace --dry-run "test"`
- **Then:** stderr shows trace; stdout shows assembled command; no subprocess spawned
- **Exit:** 0
