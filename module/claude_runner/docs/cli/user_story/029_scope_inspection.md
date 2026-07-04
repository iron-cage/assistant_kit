# User Story 029: Scope Inspection

### Scope

- **Persona**: Developer
- **Goal**: Inspect all 6 `CLAUDE_*` path variables for any directory to understand where Claude Code stores sessions and memory, or to verify paths before cross-loading.

### User Story

> As a developer,
> I want to see all 6 CLAUDE_* path variables for any directory in a single command,
> so I can understand where Claude Code stores data and verify cross-loading paths before running.

### Acceptance Criteria

- **AC-1 (Default dir):** `clr scope` prints all 6 `CLAUDE_*` variables for CWD.
- **AC-2 (Explicit dir):** `clr scope --dir /home/alice/project` prints all 6 variables for the specified directory.
- **AC-3 (Session file present):** `CLAUDE_SESSION_FILE` contains the full path to the most recently modified qualifying `.jsonl` file.
- **AC-4 (No session history):** `CLAUDE_SESSION_FILE=` is printed as an empty string when no qualifying session file exists.
- **AC-5 (CLAUDE_HOME override):** When `CLAUDE_HOME=/custom` is set, all 6 variables reflect the custom home.
- **AC-6 (Memory override):** When `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE=/shared` is set, `CLAUDE_MEMORY_DIR` and `CLAUDE_MEMORY_FILE` reflect the override.
- **AC-7 (Eval-safe output):** Output is `key=value` format, one variable per line, with no extra whitespace or shell metacharacters in the values.
- **AC-8 (Exit code):** Exit 0 on success; exit 1 when `--dir` path does not exist.

### Primary Flags

| Flag | Role |
|------|------|
| (none) | Inspect CWD |
| `--dir <DIR>` | Inspect a specific directory |

### Examples

```sh
# Inspect CWD
clr scope

# Inspect a specific project
clr scope --dir /home/alice/project

# Capture variables into shell env
eval "$(clr scope --dir /home/alice/project)"
echo "Session: $CLAUDE_SESSION_FILE"
echo "Memory:  $CLAUDE_MEMORY_FILE"

# Verify cross-loading paths before running
clr scope --dir /home/alice/project-a   # check source
clr scope --dir /home/alice/project-b   # check target
clr run --to /home/alice/project-b --session-from /home/alice/project-a "Continue"
```

### Related Commands

| Command | Role |
|---------|------|
| `scope` | Primary command for this user story |
| `run` | Uses `scope_for()` when `--session-from` is given |
| `ask` | Same as `run` |

### Related Doc Instances

| File | Relationship |
|------|--------------|
| [`../command/09_scope.md`](../command/09_scope.md) | `clr scope` command reference |
| [`../feature/005_session_path_resolution.md`](../../feature/005_session_path_resolution.md) | Feature hub: `scope_for()` |
| [`../variable/001_claude_home.md`](../../variable/001_claude_home.md) | CLAUDE_HOME |
| [`../variable/006_claude_session_file.md`](../../variable/006_claude_session_file.md) | CLAUDE_SESSION_FILE |

### Related User Stories

| # | Title | Relationship |
|---|-------|--------------|
| 028 | [Session Cross-Loading](028_session_transplant.md) | `clr scope` used to verify paths before `--session-from` |
| 005 | [Project-specific Execution](005_project_specific_execution.md) | `--dir` for targeting specific directories |
