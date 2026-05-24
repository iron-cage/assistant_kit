# User Story: Verbose Debugging

- **Source:** [docs/cli/user_story/006_verbose_debugging.md](../../../../docs/cli/user_story/006_verbose_debugging.md)
- **Primary flags:** `--verbosity`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--verbosity 4` shows command preview on stderr |
| US-2 | Boundary | `--verbosity 0` suppresses all runner output |
| US-3 | Boundary | `--verbosity 5` adds timing and state diagnostics |
| US-4 | Parameter interaction | `--dry-run` always shows command regardless of verbosity |

---

### US-1: verbosity 4 shows command preview

- **Given:** Terminal with TTY attached
- **When:** `clr --verbosity 4 "test" --dry-run`
- **Then:** stderr contains command preview line; stdout contains the assembled command (dry-run output)
- **Exit:** 0

### US-2: verbosity 0 suppresses runner output

- **Given:** Terminal with TTY attached
- **When:** `clr --verbosity 0 "test" --dry-run`
- **Then:** No runner diagnostic output on stderr; stdout contains only the assembled command
- **Exit:** 0

### US-3: verbosity 5 adds full diagnostics

- **Given:** Terminal with TTY attached
- **When:** `clr --verbosity 5 "test" --dry-run`
- **Then:** stderr contains internal state, timing information, and resolved paths in addition to command preview
- **Exit:** 0

### US-4: dry-run always shows regardless of verbosity

- **Given:** Any verbosity level
- **When:** `clr --verbosity 0 --dry-run "test"`
- **Then:** stdout contains the assembled command even at verbosity 0 — dry-run output is independent of verbosity gating
- **Exit:** 0
