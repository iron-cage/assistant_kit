# User Story: Env-var Configuration

- **Source:** [docs/cli/user_story/018_env_var_configuration.md](../../../../docs/cli/user_story/018_env_var_configuration.md)
- **Primary flags:** (env vars — no primary CLI flag)
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Fallback | `CLR_*` applies when CLI flag absent |
| US-2 | Precedence | CLI flag always wins over `CLR_*` env var |
| US-3 | Bool semantics | `"true"` (literal) accepted as truthy for bool vars |
| US-4 | Rejection | `"yes"` rejected — not a valid truthy value |

---

### US-1: env var applies when CLI flag absent

- **Given:** `CLR_MODEL=haiku` set; no `--model` CLI flag
- **When:** `CLR_MODEL=haiku clr --dry-run "task"`
- **Then:** Assembled command contains `--model haiku`
- **Exit:** 0

### US-2: CLI flag always wins over env var

- **Given:** `CLR_MODEL=haiku` set; `--model opus` passed on CLI
- **When:** `CLR_MODEL=haiku clr --model opus --dry-run "task"`
- **Then:** Assembled command contains `--model opus`; `haiku` is not present as model
- **Exit:** 0

### US-3: bool env var accepts "true" literal

- **Given:** `CLR_NO_ULTRATHINK=true` set; no `--no-ultrathink` CLI flag; message provided
- **When:** `CLR_NO_ULTRATHINK=true clr --dry-run "task"`
- **Then:** Assembled command does NOT contain `ultrathink` ("true" is a valid truthy value → no-ultrathink active)
- **Exit:** 0

### US-4: bool env var rejects "yes" as invalid

- **Given:** `CLR_NO_ULTRATHINK=yes` set; no `--no-ultrathink` CLI flag
- **When:** `CLR_NO_ULTRATHINK=yes clr --dry-run "task"`
- **Then:** Assembled command DOES contain `ultrathink` ("yes" is not truthy → env var rejected → ultrathink default active)
- **Exit:** 0
