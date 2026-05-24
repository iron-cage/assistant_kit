# User Story: Print Mode Capture

- **Source:** [docs/cli/user_story/002_print_mode_capture.md](../../../../docs/cli/user_story/002_print_mode_capture.md)
- **Primary flags:** `[MESSAGE]`, `--print`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | Message argument triggers print mode by default |
| US-2 | Parameter interaction | Explicit `--print` with message for scripting |
| US-3 | Failure path | `--print` without message errors |
| US-4 | Boundary | Print mode output is capturable via shell redirect |

---

### US-1: message triggers print mode

- **Given:** No TTY required
- **When:** `clr "Explain this function"`
- **Then:** Output goes to stdout in print mode (`--print` is implicit when message is provided); subprocess executes non-interactively
- **Exit:** 0

### US-2: explicit print flag with message

- **Given:** No TTY required
- **When:** `clr -p "List all files"`
- **Then:** Subprocess runs with `--print` flag; stdout contains Claude's response; stderr is empty (no runner diagnostics at default verbosity)
- **Exit:** 0

### US-3: print without message

- **Given:** No TTY attached
- **When:** `clr --print`
- **Then:** Error — print mode requires a message argument
- **Exit:** non-zero

### US-4: output redirect captures response

- **Given:** No TTY required; shell redirect to file
- **When:** `clr "Generate a greeting" > /tmp/output.txt`
- **Then:** `/tmp/output.txt` contains Claude's response; no runner diagnostic output mixed into stdout
- **Exit:** 0
