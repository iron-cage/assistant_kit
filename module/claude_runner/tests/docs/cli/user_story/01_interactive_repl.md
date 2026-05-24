# User Story: Interactive REPL

- **Source:** [docs/cli/user_story/001_interactive_repl.md](../../../../docs/cli/user_story/001_interactive_repl.md)
- **Primary flags:** (none)
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | Bare `clr` opens REPL with session continuation |
| US-2 | Default injection | Session continuation flag `-c` present in subprocess args |
| US-3 | Failure path | Non-interactive environment without message errors |
| US-4 | Boundary | REPL with `--dir` changes subprocess working directory |

---

### US-1: bare clr opens interactive REPL

- **Given:** Terminal with TTY attached; no prior session state required
- **When:** `clr`
- **Then:** Subprocess launches with `-c` and `--dangerously-skip-permissions`; stdin/stdout connected to subprocess; user can type prompts interactively
- **Exit:** 0

### US-2: session continuation flag present

- **Given:** Terminal with TTY attached
- **When:** `clr --dry-run`
- **Then:** Dry-run output includes `-c` flag in the assembled command, confirming session continuation is injected by default
- **Exit:** 0

### US-3: non-interactive environment without message

- **Given:** No TTY attached (piped stdin); no `[MESSAGE]` argument provided
- **When:** `clr`
- **Then:** Process exits with error — interactive mode requires either TTY or a message
- **Exit:** non-zero

### US-4: REPL with custom working directory

- **Given:** Terminal with TTY; directory `/tmp/test_project` exists
- **When:** `clr --dir /tmp/test_project`
- **Then:** Subprocess launches with working directory set to `/tmp/test_project`; session continuation remains active
- **Exit:** 0
