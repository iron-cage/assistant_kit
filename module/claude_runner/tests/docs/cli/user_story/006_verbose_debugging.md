# User Story: Output Suppression

- **Source:** [docs/cli/user_story/006_verbose_debugging.md](../../../../docs/cli/user_story/006_verbose_debugging.md)
- **Primary flags:** `--quiet`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--quiet --dry-run` still shows assembled command on stdout |
| US-2 | Boundary | without `--quiet`, dry-run output always visible |
| US-3 | Suppression | `--quiet` suppresses keep-claudecode warning |
| US-4 | Independence | `--dry-run` output always visible regardless of `--quiet` |

---

### US-1: --quiet with dry-run shows command on stdout

- **Given:** Terminal with TTY attached
- **When:** `clr --quiet "test" --dry-run`
- **Then:** stdout contains assembled command; no runner diagnostic output contaminating stderr
- **Exit:** 0

### US-2: without --quiet, default behavior unchanged

- **Given:** Terminal with TTY attached
- **When:** `clr "test" --dry-run`
- **Then:** stdout contains the assembled command; --quiet absent means no suppression
- **Exit:** 0

### US-3: --quiet suppresses keep-claudecode warning

- **Given:** CLAUDECODE env var is set and --keep-claudecode flag is used
- **When:** `clr --keep-claudecode --quiet --dry-run "task"` with `CLAUDECODE=1`
- **Then:** stderr does NOT contain "nested-agent" warning
- **Exit:** 0

### US-4: dry-run output always visible with --quiet

- **Given:** Any
- **When:** `clr --quiet --dry-run "test"`
- **Then:** stdout contains the assembled command even with --quiet — dry-run output is independent of quiet gating
- **Exit:** 0
