# User Story: Session Isolation via Subdirectory

- **Source:** [docs/cli/user_story/022_session_isolation_subdir.md](../../../../docs/cli/user_story/022_session_isolation_subdir.md)
- **Primary flags:** `--subdir`
- **Command:** `run`, `ask`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--subdir NAME` accepted; dry-run output shows effective dir with `/-NAME` suffix |
| US-2 | Default | `--subdir .` leaves base dir unchanged; no `/-` suffix in dry-run output |
| US-3 | Env var | `CLR_SUBDIR=NAME` accepted; effective dir shows `/-NAME` suffix |
| US-4 | Env var | `CLR_SUBDIR=.` identity semantics; no `/-` suffix in dry-run output |
| US-5 | CLI-wins | `--subdir cliname` overrides `CLR_SUBDIR=envname`; effective dir ends in `/-cliname` |

---

### US-1: --subdir NAME appends subdirectory

- **Given:** No prior `--dir`; cwd is the base directory
- **When:** `clr --subdir build --dry-run "Fix bug"`
- **Then:** Dry-run output contains the effective dir ending in `/-build`; exit 0
- **Exit:** 0

### US-2: --subdir . (default) leaves base dir unchanged

- **Given:** No prior `--dir`; cwd is the base directory
- **When:** `clr --subdir . --dry-run "Fix bug"`
- **Then:** Dry-run output contains no `/-` path component; identity (`.`) produces same output as bare `clr --dry-run "Fix bug"`; exit 0
- **Exit:** 0

### US-3: CLR_SUBDIR=NAME env var accepted

- **Given:** `CLR_SUBDIR=debug` set; no `--subdir` CLI flag
- **When:** `CLR_SUBDIR=debug clr --dry-run "Fix bug"`
- **Then:** Dry-run output contains the effective dir ending in `/-debug`; exit 0
- **Exit:** 0

### US-4: CLR_SUBDIR=. env var identity semantics

- **Given:** `CLR_SUBDIR=.` set; no `--subdir` CLI flag
- **When:** `CLR_SUBDIR=. clr --dry-run "Fix bug"`
- **Then:** Dry-run output contains no `/-` path component; `CLR_SUBDIR=.` is treated as identity — same output as bare `clr --dry-run "Fix bug"`; exit 0
- **Exit:** 0

### US-5: --subdir CLI wins over CLR_SUBDIR env var

- **Given:** `CLR_SUBDIR=envname` set; `--subdir cliname` on CLI
- **When:** `CLR_SUBDIR=envname clr --subdir cliname --dry-run "Fix bug"`
- **Then:** Dry-run output contains effective dir ending in `/-cliname`, NOT `/-envname`; exit 0
- **Exit:** 0
