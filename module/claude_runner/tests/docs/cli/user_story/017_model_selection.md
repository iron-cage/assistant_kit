# User Story: Model Selection

- **Source:** [docs/cli/user_story/017_model_selection.md](../../../../docs/cli/user_story/017_model_selection.md)
- **Primary flags:** `--model`
- **Command:** `run`, `ask`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--model sonnet` appears in assembled command |
| US-2 | Env var | `CLR_MODEL` sets model when CLI flag absent |
| US-3 | Precedence | CLI `--model` overrides `CLR_MODEL` env var |
| US-4 | Command scope | `--model` accepted in `ask` command |

---

### US-1: model flag appears in assembled command

- **Given:** No prior configuration
- **When:** `clr --model sonnet --dry-run "Fix bug"`
- **Then:** Assembled command contains `--model sonnet`
- **Exit:** 0

### US-2: CLR_MODEL env var sets model

- **Given:** `CLR_MODEL=haiku` set in environment; no `--model` CLI flag
- **When:** `CLR_MODEL=haiku clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--model haiku`
- **Exit:** 0

### US-3: CLI --model overrides CLR_MODEL

- **Given:** `CLR_MODEL=haiku` set; `--model opus` passed on CLI
- **When:** `CLR_MODEL=haiku clr --model opus --dry-run "Fix bug"`
- **Then:** Assembled command contains `--model opus`; `--model haiku` does not appear
- **Exit:** 0

### US-4: --model accepted in ask command

- **Given:** No prior configuration
- **When:** `clr ask --model sonnet --dry-run "What is X?"`
- **Then:** Assembled command contains `--model sonnet`
- **Exit:** 0
