# Commands

### Scope

- **Purpose**: Document the eleven clr subcommands and their parameters, modes, and usage examples.
- **Responsibility**: Specify each command's behavior, accepted parameters, and usage.
- **In Scope**: run, ask, isolated, refresh, help, ps, kill, tools, scope, query, topic commands and their invocation modes.
- **Out of Scope**: Parameter definitions (-> `../param/`), type definitions (-> `../type/`), user stories (-> `../user_story/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_run.md | Command spec: default execution with configurable flags |
| 02_help.md | Command spec: print usage information and exit |
| 03_isolated.md | Command spec: credential-isolated subprocess execution |
| 04_refresh.md | Command spec: OAuth credential refresh without running a task |
| 05_ask.md | Command spec: semantic alias for run (identical defaults) |
| 06_ps.md | Command spec: list running Claude Code sessions and queued waiters in two plain-style tables |
| 07_kill.md | Command spec: terminate a running Claude Code session by PID via SIGTERM |
| 08_tools.md | Command spec: list Claude Code tools with version information |
| 09_scope.md | Command spec: print all 6 CLAUDE_* path variables for a directory |
| 10_query.md | Command spec: start or dispatch PID-addressed bidirectional control sessions |
| 11_topic.md | Command spec: `run`/`ask` alias with an auto-naming `--topic` default |
| 12_topics.md | Command spec: list topics (fork + dir modes), or resolve one name to a path or session file |

### All Commands (12 total)

| # | Command | Description | Params | Example |
|---|---------|-------------|--------|---------|
| 1 | `run` (default) | Execute Claude Code with given parameters | 65 | `clr "Fix bug" --model sonnet` |
| 2 | `isolated` | Run Claude with credential-isolated temp HOME | 17 | `clr isolated --creds creds.json "Fix bug"` |
| 3 | `refresh` | Refresh OAuth credentials without running a task | 6 | `clr refresh --creds creds.json` |
| 4 | `help` | Print usage information and exit | 0 | `clr help` |
| 5 | `ask` | Semantic alias for run (identical defaults) | 65 | `clr ask "What does X do?"` |
| 6 | `ps` | List running Claude Code sessions | 5 | `clr ps` |
| 7 | `kill` | Terminate a running Claude Code session by PID | 0 | `clr kill 12345` |
| 8 | `tools` | List Claude Code tools with filter/projection/inspect controls | 5 | `clr tools --category Web --inspect` |
| 9 | `scope` | Print all 6 CLAUDE_* path variables for a directory | 1 | `clr scope --dir /project` |
| 10 | `query` | Start or dispatch PID-addressed bidirectional control sessions | 1 | `clr query "Fix bug"` |
| 11 | `topic` | `run`/`ask` alias with an auto-naming `--topic` default | 65 | `clr topic "Investigate the flaky test"` |
| 12 | `topics` | List topics (fork + dir modes), or resolve one name to its dir-mode path or fork-mode session file | 4 | `clr topics --global --path auth-refactor` |

**Total:** 12 commands

**Maintenance note:** When a new param is added to the Runner Control group (`docs/cli/param_group/02_runner_control.md`), these files must ALL be updated manually: (1) `01_run.md` Parameters table, (2) the Params count column above, (3) `docs/entity.md` param count + row, (4) `docs/cli/003_env_param.md` if it has an env var, (5) `tests/docs/cli/param/readme.md` status. `ask` inherits all `run` params automatically via the "All parameters from run are accepted" shortcut — no separate table update needed for `05_ask.md`.
