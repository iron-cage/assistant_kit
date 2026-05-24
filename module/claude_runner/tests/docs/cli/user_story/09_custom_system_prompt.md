# User Story: Custom System Prompt

- **Source:** [docs/cli/user_story/009_custom_system_prompt.md](../../../../docs/cli/user_story/009_custom_system_prompt.md)
- **Primary flags:** `--system-prompt`, `--append-system-prompt`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--system-prompt` replaces default system prompt |
| US-2 | Happy path | `--append-system-prompt` extends default system prompt |
| US-3 | Parameter interaction | Both flags combined: replace then append |
| US-4 | Boundary | Empty string system prompt clears default |

---

### US-1: replace system prompt

- **Given:** No prior configuration
- **When:** `clr --system-prompt "You are a Python expert" --dry-run "test"`
- **Then:** Assembled command includes `--system-prompt "You are a Python expert"`; default system prompt is fully replaced
- **Exit:** 0

### US-2: append to system prompt

- **Given:** No prior configuration
- **When:** `clr --append-system-prompt "Always respond in JSON" --dry-run "test"`
- **Then:** Assembled command includes `--append-system-prompt "Always respond in JSON"`; default system prompt preserved with addition appended
- **Exit:** 0

### US-3: replace then append

- **Given:** No prior configuration
- **When:** `clr --system-prompt "Base prompt" --append-system-prompt "Extra rule" --dry-run "test"`
- **Then:** Assembled command includes both `--system-prompt "Base prompt"` and `--append-system-prompt "Extra rule"`; replacement applies first, then append adds to it
- **Exit:** 0

### US-4: empty system prompt

- **Given:** No prior configuration
- **When:** `clr --system-prompt "" --dry-run "test"`
- **Then:** Assembled command includes `--system-prompt ""`; default system prompt is cleared
- **Exit:** 0
