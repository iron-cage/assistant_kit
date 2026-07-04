# User Story: Keep ClaudeCode Context

- **Source:** [docs/cli/user_story/021_keep_claudecode_context.md](../../../../docs/cli/user_story/021_keep_claudecode_context.md)
- **Primary flags:** `--keep-claudecode`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--keep-claudecode` flag accepted; command assembled |
| US-2 | Env var | `CLR_KEEP_CLAUDECODE=1` accepted; command assembled |
| US-3 | Env var | `CLR_KEEP_CLAUDECODE=true` accepted (alternative truthy) |
| US-4 | Rejection | `CLR_KEEP_CLAUDECODE=yes` silently rejected; exit 0 |

---

### US-1: --keep-claudecode flag accepted

- **Given:** No prior configuration; no CLAUDECODE in parent env
- **When:** `clr --keep-claudecode --dry-run "Fix bug"`
- **Then:** Command assembled without error; dry-run output contains `CLAUDE_CODE_MAX_OUTPUT_TOKENS=`
- **Exit:** 0

### US-2: CLR_KEEP_CLAUDECODE=1 accepted via env var

- **Given:** `CLR_KEEP_CLAUDECODE=1` set; no `--keep-claudecode` CLI flag
- **When:** `CLR_KEEP_CLAUDECODE=1 clr --dry-run "Fix bug"`
- **Then:** Command assembled without error; exit 0
- **Exit:** 0

### US-3: CLR_KEEP_CLAUDECODE=true accepted

- **Given:** `CLR_KEEP_CLAUDECODE=true` set
- **When:** `CLR_KEEP_CLAUDECODE=true clr --dry-run "Fix bug"`
- **Then:** Command assembled without error; exit 0
- **Exit:** 0

### US-4: CLR_KEEP_CLAUDECODE=yes silently rejected

- **Given:** `CLR_KEEP_CLAUDECODE=yes` set (invalid truthy value)
- **When:** `CLR_KEEP_CLAUDECODE=yes clr --dry-run "Fix bug"`
- **Then:** Exit 0 (silently rejected; behaves as default — CLAUDECODE not preserved)
- **Exit:** 0
