# User Story: Ask Mode

- **Source:** [docs/cli/user_story/015_ask_mode.md](../../../../docs/cli/user_story/015_ask_mode.md)
- **Primary flags:** `[MESSAGE]`, `--effort`, `--max-tokens`
- **Command:** `ask`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `clr ask` applies conservative defaults (no -c, no permissions bypass, no ultrathink) |
| US-2 | Boundary | Print mode always on for ask regardless of message presence |
| US-3 | Parameter interaction | Ask defaults overridable via explicit flags |
| US-4 | Happy path | `--no-persist` and `--no-chrome` default ON for ask |

---

### US-1: conservative ask defaults

- **Given:** No prior configuration
- **When:** `clr ask --dry-run "What does this function do?"`
- **Then:** Assembled command shows: no `-c` (new session), no `--dangerously-skip-permissions`, no ultrathink suffix, `--effort high`, `--max-tokens 16384`; question goes to Claude as a single-turn Q&A
- **Exit:** 0

### US-2: print mode always on

- **Given:** No explicit `--print` flag
- **When:** `clr ask --dry-run "Explain closures"`
- **Then:** Assembled command includes `--print` regardless of whether a message was provided; ask always operates in print mode
- **Exit:** 0

### US-3: override ask defaults

- **Given:** User wants extended output with maximum effort
- **When:** `clr ask --effort max --max-tokens 200000 --dry-run "Write a detailed analysis"`
- **Then:** Assembled command shows `--effort max` and `--max-tokens 200000` overriding the ask defaults of `high` and `16384`; all 25 run parameters are accepted
- **Exit:** 0

### US-4: no-persist and no-chrome defaults

- **Given:** No explicit persistence or chrome flags
- **When:** `clr ask --dry-run "Quick question"`
- **Then:** Assembled command includes `--no-persist` (no session state saved) and `--no-chrome` (no browser context); ask is stateless by default
- **Exit:** 0
