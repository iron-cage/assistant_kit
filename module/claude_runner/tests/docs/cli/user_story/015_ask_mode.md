# User Story: Ask Mode

- **Source:** [docs/cli/user_story/015_ask_mode.md](../../../../docs/cli/user_story/015_ask_mode.md)
- **Primary flags:** `[MESSAGE]` (all run params accepted identically)
- **Command:** `ask`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Equivalence | `clr ask --dry-run "X"` produces identical output to `clr run --dry-run "X"` |
| US-2 | Equivalence | `clr ask --dry-run` (no message) produces identical output to `clr --dry-run` |
| US-3 | No forced flags | ask injects no extra flags vs run — no forced `--new-session`, `--no-chrome`, `--no-persist`, etc. |
| US-4 | Param passthrough | All run params accepted by ask with identical behavior |

---

### US-1: dry-run output identical to run (with message)

- **Given:** Same message text; no env vars set
- **When:** `clr ask --dry-run "What does X do?"` vs `clr run --dry-run "What does X do?"`
- **Then:** Both produce byte-for-byte identical stdout; no extra flags injected by ask
- **Exit:** 0

### US-2: dry-run output identical to run (no message)

- **Given:** No message; no env vars set
- **When:** `clr ask --dry-run` vs `clr --dry-run` (run with no message)
- **Then:** Both produce identical stdout
- **Exit:** 0

### US-3: no flags forced by ask subcommand

- **Given:** No CLI flags beyond `--dry-run`
- **When:** `clr ask --dry-run "task"`
- **Then:** Assembled command contains no `--new-session`, no extra `--no-chrome`, no `--no-persist`, no `--no-ultrathink` beyond what run would inject; `--effort max` present (run default, not ask-specific); effectively identical to `clr run --dry-run "task"`
- **Exit:** 0

### US-4: all run params accepted by ask

- **Given:** Multiple run params passed including new Phase 2/3/7 params
- **When:** `clr ask --dry-run --effort high --max-tokens 100000 --output-file /tmp/out.txt "task"`
- **Then:** Exit 0; all flags accepted without unknown-flag error; assembled command reflects the values
- **Exit:** 0
