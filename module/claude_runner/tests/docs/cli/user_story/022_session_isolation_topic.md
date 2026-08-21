# User Story: Session Isolation via Topic Directory

- **Source:** [docs/cli/user_story/022_session_isolation_topic.md](../../../../docs/cli/user_story/022_session_isolation_topic.md)
- **Primary flags:** `--topic`
- **Command:** `run`, `ask`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--topic NAME` accepted; dry-run preview names the topic session (`topic=NAME`, fork mode for a fresh name) |
| US-2 | Default | `--topic .` leaves base dir unchanged; no `/-` suffix in dry-run output |
| US-3 | Env var | `CLR_TOPIC=NAME` accepted; dry-run preview names the topic session (`topic=NAME`) |
| US-4 | Env var | `CLR_TOPIC=.` identity semantics; no `/-` suffix in dry-run output |
| US-5 | CLI-wins | `--topic cliname` overrides `CLR_TOPIC=envname`; preview shows `topic=cliname`, never `topic=envname` |

---

### US-1: --topic NAME names an isolated topic conversation

- **Given:** No prior `--dir`; cwd is the base directory; no pre-existing `-build` dir (fresh name → fork mode)
- **When:** `clr --topic build --dry-run "Fix bug"`
- **Then:** Dry-run preview line contains `topic=build ` (fork-mode session plan; no `/-build` directory path); exit 0
- **Exit:** 0

### US-2: --topic . (default) leaves base dir unchanged

- **Given:** No prior `--dir`; cwd is the base directory
- **When:** `clr --topic . --dry-run "Fix bug"`
- **Then:** Dry-run output contains no `/-` path component; identity (`.`) produces same output as bare `clr --dry-run "Fix bug"`; exit 0
- **Exit:** 0

### US-3: CLR_TOPIC=NAME env var accepted

- **Given:** `CLR_TOPIC=debug` set; no `--topic` CLI flag
- **When:** `CLR_TOPIC=debug clr --dry-run "Fix bug"`
- **Then:** Dry-run preview line contains `topic=debug ` (fork-mode session plan); exit 0
- **Exit:** 0

### US-4: CLR_TOPIC=. env var identity semantics

- **Given:** `CLR_TOPIC=.` set; no `--topic` CLI flag
- **When:** `CLR_TOPIC=. clr --dry-run "Fix bug"`
- **Then:** Dry-run output contains no `/-` path component; `CLR_TOPIC=.` is treated as identity — same output as bare `clr --dry-run "Fix bug"`; exit 0
- **Exit:** 0

### US-5: --topic CLI wins over CLR_TOPIC env var

- **Given:** `CLR_TOPIC=envname` set; `--topic cliname` on CLI
- **When:** `CLR_TOPIC=envname clr --topic cliname --dry-run "Fix bug"`
- **Then:** Dry-run preview contains `topic=cliname `, NOT `topic=envname`; exit 0
- **Exit:** 0
