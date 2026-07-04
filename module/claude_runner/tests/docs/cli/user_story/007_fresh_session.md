# User Story: Fresh Session

- **Source:** [docs/cli/user_story/007_fresh_session.md](../../../../docs/cli/user_story/007_fresh_session.md)
- **Primary flags:** `--new-session`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--new-session` omits `-c` from assembled command |
| US-2 | Default preservation | Other defaults remain when `--new-session` is active |
| US-3 | Parameter interaction | `--new-session` with `--print` for fresh non-interactive run |
| US-4 | Boundary | `--new-session` with `--interactive` for fresh REPL |

---

### US-1: new session omits continuation flag

- **Given:** Prior sessions may exist
- **When:** `clr --new-session --dry-run "start fresh"`
- **Then:** Assembled command does NOT contain `-c`; conversation starts without prior context
- **Exit:** 0

### US-2: other defaults preserved

- **Given:** No prior configuration
- **When:** `clr --new-session --dry-run "test"`
- **Then:** `-c` absent; `--dangerously-skip-permissions`, `--effort max` still present; ultrathink suffix still appended; `--chrome` absent (print mode — BUG-304 mitigation)
- **Exit:** 0

### US-3: fresh session in print mode

- **Given:** No TTY required
- **When:** `clr --new-session "Review this code" --dry-run`
- **Then:** Assembled command has `--print` (message triggers it) but no `-c`; clean-slate conversation with print output
- **Exit:** 0

### US-4: fresh session in interactive mode

- **Given:** Terminal with TTY attached
- **When:** `clr --new-session --interactive --dry-run "Begin analysis"`
- **Then:** Assembled command has `--interactive` and no `-c`; user gets fresh REPL with initial prompt
- **Exit:** 0
