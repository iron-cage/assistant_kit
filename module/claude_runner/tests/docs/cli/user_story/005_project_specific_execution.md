# User Story: Project-specific Execution

- **Source:** [docs/cli/user_story/005_project_specific_execution.md](../../../../docs/cli/user_story/005_project_specific_execution.md)
- **Primary flags:** `--dir`, `--session-dir`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--dir` sets subprocess working directory |
| US-2 | Parameter interaction | `--dir` with `--session-dir` for full project isolation |
| US-3 | Failure path | `--dir` with non-existent path errors |
| US-4 | Boundary | `--dir` combined with `--new-session` prevents context bleed |

---

### US-1: set subprocess working directory

- **Given:** Directory `/tmp/my_project` exists
- **When:** `clr --dir /tmp/my_project --dry-run "fix tests"`
- **Then:** Assembled command includes `--dir /tmp/my_project`; subprocess would execute in that directory
- **Exit:** 0

### US-2: full project isolation with session-dir

- **Given:** Directories `/tmp/project_a` and `/tmp/sessions_a` exist
- **When:** `clr --dir /tmp/project_a --session-dir /tmp/sessions_a --dry-run "analyze"`
- **Then:** Assembled command includes both `--dir /tmp/project_a` and `--session-dir /tmp/sessions_a`; session state isolated from default location
- **Exit:** 0

### US-3: non-existent directory path

- **Given:** Directory `/tmp/nonexistent_project` does not exist
- **When:** `clr --dir /tmp/nonexistent_project "fix it"`
- **Then:** Error — directory path does not exist or is not accessible
- **Exit:** non-zero

### US-4: project isolation with fresh session

- **Given:** Directory `/tmp/my_project` exists; prior sessions may exist
- **When:** `clr --dir /tmp/my_project --new-session --dry-run "start fresh"`
- **Then:** `-c` absent from assembled command (new session); `--dir` still present; no prior session context loaded
- **Exit:** 0
