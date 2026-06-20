# Commands

### Scope

- **Purpose**: Document the seven clr subcommands and their parameters, modes, and usage examples.
- **Responsibility**: Specify each command's behavior, accepted parameters, and usage.
- **In Scope**: run, ask, isolated, refresh, help, ps, kill commands and their invocation modes.
- **Out of Scope**: Parameter definitions (-> `../param/`), type definitions (-> `../type/`), user stories (-> `../user_story/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_run.md | Command spec: default execution with configurable flags |
| 02_isolated.md | Command spec: credential-isolated subprocess execution |
| 03_refresh.md | Command spec: OAuth credential refresh without running a task |
| 04_help.md | Command spec: print usage information and exit |
| 05_ask.md | Command spec: semantic alias for run (identical defaults) |
| 06_ps.md | Command spec: list running Claude Code sessions and queued waiters in two plain-style tables |
| 07_kill.md | Command spec: terminate a running Claude Code session by PID via SIGTERM |
| 08_tools.md | Command spec: list Claude Code tools with version information |

### All Commands (8 total)

| # | Command | Description | Params | Example |
|---|---------|-------------|--------|---------|
| 1 | `run` (default) | Execute Claude Code with given parameters | 58 | `clr "Fix bug" --model sonnet` |
| 2 | `isolated` | Run Claude with credential-isolated temp HOME | 4 | `clr isolated --creds creds.json "Fix bug"` |
| 3 | `refresh` | Refresh OAuth credentials without running a task | 3 | `clr refresh --creds creds.json` |
| 4 | `help` | Print usage information and exit | 0 | `clr help` |
| 5 | `ask` | Semantic alias for run (identical defaults) | 58 | `clr ask "What does X do?"` |
| 6 | `ps` | List running Claude Code sessions | 3 | `clr ps` |
| 7 | `kill` | Terminate a running Claude Code session by PID | 0 | `clr kill 12345` |
| 8 | `tools` | List Claude Code tools with version info | 0 | `clr tools` |

**Total:** 8 commands

**Maintenance note:** When a new param is added to the Runner Control group (`docs/cli/param_group/02_runner_control.md`), these files must ALL be updated manually: (1) `01_run.md` Parameters table, (2) the Params count column above, (3) `docs/002_entities.md` param count + row, (4) `docs/cli/env_param.md` if it has an env var, (5) `tests/docs/cli/param/readme.md` status. `ask` inherits all `run` params automatically via the "All parameters from run are accepted" shortcut — no separate table update needed for `05_ask.md`.
