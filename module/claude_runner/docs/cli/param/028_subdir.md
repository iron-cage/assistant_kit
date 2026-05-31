# CLI Parameter: --subdir

Appends a named subdirectory under the effective working directory to produce the
actual execution directory passed to the Claude subprocess. Default `.` is the
identity value — the working directory is used as-is, with no subdirectory appended.

- **Type:** string (directory name component or `.` identity)
- **Default:** `.` (identity — no subdirectory appended)
- **Command:** [`run`](../command/01_run.md), [`ask`](../command/05_ask.md)
- **Group:** [Runner Control](../param_group/02_runner_control.md)

```sh
clr "Fix bug"                           # effective dir = cwd (default: --subdir .)
clr --subdir build "Fix bug"            # effective dir = cwd/-build (auto-created)
clr --dir /project --subdir debug "x"  # effective dir = /project/-debug
clr --subdir . "Fix bug"               # explicit identity — same as default
```

**How it works:** When `--subdir` is a non-`.` value, `/-<name>` is appended to the
base directory (`--dir` value or cwd). The resulting directory is created automatically
(`create_dir_all`) before subprocess spawn — no manual `mkdir` needed.

**Session isolation:** Claude Code session state is keyed by working directory, so
`--subdir build` and `--subdir debug` within the same `--dir` produce independent
conversation histories. This is the mechanism wplan uses to isolate per-topic workspaces:
`dream .claude topic::build` resolves to `clr --dir /project/-build "..."`.

**Note:** The `-` prefix in the generated subdirectory name (`/-build`) follows the
project transient-directory convention — directories beginning with `-` are git-excluded
by `.gitignore` patterns.

**Env var:** `CLR_SUBDIR` — string; applied when `--subdir` is absent from the CLI
and `CLR_SUBDIR` is non-empty. `CLR_SUBDIR=build clr "task"` is equivalent to
`clr --subdir build "task"`.

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| string | Primitive | &str | `.` (identity) or valid directory name component (no `/` separators) |

### Referenced Parameter Groups

| # | Group | Membership | Co-members |
|---|-------|------------|------------|
| 2 | [Runner Control](../param_group/02_runner_control.md) | Full | 16 other params |

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [`run`](../command/01_run.md) | `.` (identity) | — |
| 5 | [`ask`](../command/05_ask.md) | `.` (identity) | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 22 | [022_session_isolation_subdir.md](../user_story/022_session_isolation_subdir.md) | Developer |
