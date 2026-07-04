# User Story: Interactive With Message

- **Source:** [docs/cli/user_story/003_interactive_with_message.md](../../../../docs/cli/user_story/003_interactive_with_message.md)
- **Primary flags:** `--interactive`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--interactive` with message keeps TTY open after initial prompt |
| US-2 | Default override | `--interactive` overrides message-triggers-print default |
| US-3 | Failure path | `--interactive` without TTY falls back gracefully |
| US-4 | Parameter interaction | `--interactive` with `--new-session` starts fresh conversation |

---

### US-1: interactive with initial message

- **Given:** Terminal with TTY attached
- **When:** `clr "Fix the bug" --interactive`
- **Then:** Message sent as initial prompt; subprocess stays open for follow-up turns; TTY remains connected for multi-turn conversation
- **Exit:** 0

### US-2: overrides print-on-message default

- **Given:** Terminal with TTY attached
- **When:** `clr "Review code" --interactive --dry-run`
- **Then:** Dry-run output shows `--interactive` flag present; `--print` is NOT present despite message being provided
- **Exit:** 0

### US-3: interactive without TTY

- **Given:** No TTY attached (piped stdin)
- **When:** `clr "Fix it" --interactive`
- **Then:** Subprocess attempts interactive mode; behavior depends on Claude Code's handling of non-TTY interactive mode
- **Exit:** 0 or non-zero depending on subprocess

### US-4: interactive with fresh session

- **Given:** Terminal with TTY attached
- **When:** `clr "Start analysis" --interactive --new-session`
- **Then:** Subprocess launches without `-c` (new session); `--interactive` flag present; message sent as initial prompt
- **Exit:** 0
