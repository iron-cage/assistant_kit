# User Story: Dry-run Preview

- **Source:** [docs/cli/user_story/004_dry_run_preview.md](../../../../docs/cli/user_story/004_dry_run_preview.md)
- **Primary flags:** `--dry-run`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--dry-run` prints assembled command without executing |
| US-2 | Default visibility | Dry-run output shows all injected defaults |
| US-3 | Failure path | Dry-run with invalid parameter still shows assembled command |
| US-4 | Boundary | Dry-run with `--verbosity` does not alter dry-run output |

---

### US-1: dry-run prints command without executing

- **Given:** No subprocess execution expected
- **When:** `clr --dry-run "test message"`
- **Then:** stdout contains the full assembled command starting with `env -u CLAUDECODE claude` (default: CLAUDECODE removal is visible in output per WYSIWYG invariant — BUG-246 fix); no subprocess is spawned; output includes `--dangerously-skip-permissions`, `--effort max`, ultrathink suffix; `--chrome` is absent (print mode — BUG-304 mitigation)
- **Exit:** 0

### US-2: all injected defaults visible

- **Given:** No prior configuration
- **When:** `clr --dry-run "test"`
- **Then:** Assembled command contains: `-c`, `--dangerously-skip-permissions`, `--effort max`; message has ultrathink suffix; `--chrome` is absent (print mode — BUG-304 mitigation)
- **Exit:** 0

### US-3: dry-run with model override

- **Given:** No prior configuration
- **When:** `clr --dry-run --model sonnet "test"`
- **Then:** Assembled command includes `--model sonnet`; all other defaults still present; no subprocess spawned
- **Exit:** 0

### US-4: dry-run exit is always zero

- **Given:** Any combination of valid flags
- **When:** `clr --dry-run --verbose --new-session "test"`
- **Then:** Command printed to stdout; `--new-session` causes `-c` to be absent from output; exit is always 0 regardless of flags
- **Exit:** 0
