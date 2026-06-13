# Commands

### Scope

- **Purpose**: Document the six clr subcommands and their parameters, modes, and usage examples.
- **Responsibility**: Specify each command's behavior, accepted parameters, and usage.
- **In Scope**: run, ask, isolated, refresh, help, ps commands and their invocation modes.
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

### All Commands (6 total)

| # | Command | Description | Params | Example |
|---|---------|-------------|--------|---------|
| 1 | `run` (default) | Execute Claude Code with given parameters | 31 | `clr "Fix bug" --model sonnet` |
| 2 | `isolated` | Run Claude with credential-isolated temp HOME | 4 | `clr isolated --creds creds.json "Fix bug"` |
| 3 | `refresh` | Refresh OAuth credentials without running a task | 3 | `clr refresh --creds creds.json` |
| 4 | `help` | Print usage information and exit | 0 | `clr help` |
| 5 | `ask` | Semantic alias for run (identical defaults) | 31 | `clr ask "What does X do?"` |
| 6 | `ps` | List running Claude Code sessions | 0 | `clr ps` |

**Total:** 6 commands
