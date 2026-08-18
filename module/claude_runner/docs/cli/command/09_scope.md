# CLI Command: scope

### Description

Print all 6 `CLAUDE_*` path variables for a given directory (default: CWD). Use `clr scope` to inspect where Claude Code stores sessions and memory for any project directory, or to verify that `--from` / `--dir` resolve to the expected paths before running.

-- **Parameters:** `--dir`
-- **Exit Codes:** 0 (success) | 1 (error)

### Syntax

```sh
clr scope [--dir <DIR>]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`--dir`](../param/008_dir.md) | path | CWD | Directory to compute paths for |
| `-h`/`--help` | — | — | Print scope subcommand help and exit 0 |

**Algorithm (3 steps):**
1. Resolve target directory: `--dir` if given, otherwise CWD.
2. Call `scope_for(target_dir)` to compute all 6 `CLAUDE_*` variables.
3. Print each variable as `NAME=value`, one per line. `CLAUDE_SESSION_FILE` is printed as empty string when no session history exists.

### Output Format

```sh
CLAUDE_HOME=/home/alice/.claude
CLAUDE_PROJECTS_DIR=/home/alice/.claude/projects/
CLAUDE_SESSION_DIR=/home/alice/.claude/projects/-home-alice-project
CLAUDE_MEMORY_DIR=/home/alice/.claude/projects/-home-alice-project/memory/
CLAUDE_MEMORY_FILE=/home/alice/.claude/projects/-home-alice-project/memory/MEMORY.md
CLAUDE_SESSION_FILE=/home/alice/.claude/projects/-home-alice-project/9a3f8a12-cdef-4567-8901-abcdef012345.jsonl
```

When no session history exists:
```sh
CLAUDE_SESSION_FILE=
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All 6 variables printed successfully |
| 1 | Error (e.g., `--dir` path does not exist) |

### Examples

```sh
# Inspect paths for current directory
clr scope

# Inspect paths for a specific project
clr scope --dir /home/alice/project

# Use in shell eval (capture all 6 vars)
eval "$(clr scope --dir /home/alice/project)"
echo "$CLAUDE_SESSION_FILE"

# Verify cross-loading paths before running
clr scope --dir /home/alice/project-b   # target session dir
clr scope --dir /home/alice/project-a   # source session dir
clr run --to /home/alice/project-b --from /home/alice/project-a "Continue"
```

### Notes

`CLAUDE_HOME` respects the `CLAUDE_HOME` env var override. `CLAUDE_MEMORY_DIR` respects the `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` env var. Both overrides are applied identically to how `scope_for()` computes them at subprocess launch time.

`clr scope` does not require Claude to be installed or credentials to be present — it is a pure filesystem path computation.

### Referenced Command Group

Evaluated against `run`/`ask` under the strict [command_group](../command_group/readme.md) identity test (same dispatch function, same parameter set) — does not qualify. `dispatch_scope()` (`src/cli/scope.rs:13`) has zero cross-calls with `dispatch_run()`/`dispatch_ask()`. Both call `claude_storage_core::scope_for()` internally, but that's a shared external library utility, not a shared dispatch function — `scope` uses it for its entire output (all 6 returned fields), while `run`'s builder uses only 1 of 6 fields as one step in `--from` resolution (defaults to CWD when omitted). `scope`'s single `--dir` parameter is a documented subset of the `Running Commands` param_group (see [`param_group/06_running_commands.md`](../param_group/06_running_commands.md)), a parameter-level relationship distinct from command_group's dispatch-level test. See [`command_group/readme.md`](../command_group/readme.md) Evaluated, Not Qualifying for the full analysis.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`run`](01_run.md) | Uses `scope_for()` internally for `--from` resolution (defaults to CWD) |
| 5 | [`ask`](05_ask.md) | Same as `run` — uses `scope_for()` for `--from` |
| 11 | [`topic`](11_topic.md) | Same as `run` — uses `scope_for()` for `--from`, once dispatch wiring lands per task 521 |

### Referenced Parameter Groups

| # | Group | Membership |
|---|-------|------------|
| 6 | [Running Commands](../param_group/06_running_commands.md) | Subset — `--dir` only |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 29 | [029_scope_inspection.md](../user_story/029_scope_inspection.md) | Developer |

---

**Category:** Inspection / path resolution
**Complexity:** 3
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Zero (read-only)
