# Parameter :: `--trace`

Edge case tests for the trace flag. Tests validate command echoing to stderr before execution.

**Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--trace` → assembled command echoed to stderr before invocation; exit 1 (test env has no claude) | Behavioral Divergence |
| EC-2 | Without `--trace` → no command echo to stderr | Behavioral Divergence |
| EC-3 | `--trace` + `--dry-run` → preview on stdout; trace NOT on stderr (dry-run wins) | Interaction |
| EC-4 | `--trace --dry-run` without message → stdout preview; stderr empty (dry-run wins) | Edge Case |
| EC-5 | `--help` lists `--trace` | Documentation |
| EC-6 | `--trace` + env vars → env vars included in trace output | Trace Content |
| EC-7 | `isolated --creds <f> --trace "msg"` → `# clr isolated` / `# creds:` / `# timeout: 30s` on stderr | Trace Content |
| EC-8 | `refresh --creds <f> --trace` → `# clr refresh` / `# creds:` / `# timeout: 45s` on stderr | Trace Content |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Interaction: 1 test (EC-3)
- Edge Case: 1 test (EC-4)
- Documentation: 1 test (EC-5)
- Trace Content: 3 tests (EC-6, EC-7, EC-8)

**Total:** 8 edge cases


## Test Cases
---

### EC-1: `--trace` → command echoed to stderr before invocation

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`; trace fires before invocation)
- **Then:** Stderr contains assembled command (trace output written before claude is invoked); subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md)
- **Commands:** run, isolated, refresh, ask
---

### EC-2: Without `--trace` → no command echo

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Stderr is empty (no command echo)
- **Exit:** 0
- **Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md)
- **Commands:** run, isolated, refresh, ask
---

### EC-3: `--trace` + `--dry-run` → preview on stdout; trace NOT on stderr (dry-run wins)

- **Given:** clean environment
- **When:** `clr --trace --dry-run "Fix bug"`
- **Then:** Command preview on stdout; stderr is EMPTY (`handle_dry_run` returns before trace fires)
- **Exit:** 0
- **Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md)
- **Commands:** run, isolated, refresh, ask
---

### EC-4: `--trace --dry-run` without message → stdout preview; stderr empty (dry-run wins)

- **Given:** clean environment
- **When:** `clr --trace --dry-run`
- **Then:** Exit 0; assembled command on stdout (dry-run output); stderr is EMPTY (trace does not fire when dry-run wins)
- **Exit:** 0
- **Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md)
- **Commands:** run, isolated, refresh, ask
---

### EC-5: `--help` lists `--trace`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--trace`
- **Exit:** 0
- **Source:** [command/04_help.md](../../../../docs/cli/command/04_help.md)
- **Commands:** run, isolated, refresh, ask
---

### EC-6: `--trace` output includes environment context (without `--dry-run`)

- **Given:** clean environment
- **When:** `clr --trace "Fix bug"` (no `--dry-run`; claude absent in test environment)
- **Then:** Stderr contains env vars and assembled command line (trace fires before invocation attempt); stdout empty or shows error from failed subprocess
- **Exit:** 1 (claude absent in test environment)
- **Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md)
- **Commands:** run, isolated, refresh, ask
---

### EC-7: `isolated --creds <f> --trace "msg"` → credential trace format on stderr

- **Given:** credentials JSON written to a temp file `<f>` (file is readable); claude binary absent
- **When:** `clr isolated --creds <f> --trace "Fix bug"` (trace fires before subprocess attempt)
- **Then:** Stderr contains `# clr isolated`, `# creds: <path>`, `# timeout: 30s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`), and `claude --chrome --model claude-opus-4-6 --effort max --no-session-persistence --dangerously-skip-permissions --print "Fix bug"`; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md), [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
- **Commands:** run, isolated, refresh, ask
---

### EC-8: `refresh --creds <f> --trace` → credential trace format on stderr with 45s timeout

- **Given:** credentials JSON written to a temp file `<f>` (file is readable); claude binary absent
- **When:** `clr refresh --creds <f> --trace` (trace fires before subprocess attempt)
- **Then:** Stderr contains `# clr refresh`, `# creds: <path>`, `# timeout: 45s`, env var block (including `CLAUDE_CODE_MAX_OUTPUT_TOKENS=200000`), and `claude --chrome --model claude-sonnet-4-6 --print "."`; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Source:** [013_trace.md](../../../../docs/cli/param/013_trace.md), [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
- **Commands:** run, isolated, refresh, ask
