# User Story: Suppress Effort Max

- **Source:** [docs/cli/user_story/020_suppress_effort_max.md](../../../../docs/cli/user_story/020_suppress_effort_max.md)
- **Primary flags:** `--no-effort-max`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Default | Default assembled command includes `--effort max` |
| US-2 | Happy path | `--no-effort-max` suppresses all `--effort` injection |
| US-3 | Precedence | `--no-effort-max` wins over explicit `--effort` flag |
| US-4 | Env var | `CLR_NO_EFFORT_MAX=1` suppresses via env var |

---

### US-1: default has --effort max

- **Given:** No effort flags set
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--effort max`
- **Exit:** 0

### US-2: --no-effort-max suppresses all effort

- **Given:** No prior configuration
- **When:** `clr --no-effort-max --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain any `--effort` flag
- **Exit:** 0

### US-3: --no-effort-max wins over --effort

- **Given:** Both `--no-effort-max` and `--effort medium` provided
- **When:** `clr --no-effort-max --effort medium --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain any `--effort` flag (suppression wins)
- **Exit:** 0

### US-4: CLR_NO_EFFORT_MAX env var suppresses

- **Given:** `CLR_NO_EFFORT_MAX=1` set in environment
- **When:** `CLR_NO_EFFORT_MAX=1 clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain any `--effort` flag
- **Exit:** 0
