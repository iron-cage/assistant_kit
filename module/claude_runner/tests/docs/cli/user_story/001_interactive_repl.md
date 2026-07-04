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
| US-5 | BUG-214 regression | Empty session dir → no `-c` injected; REPL opens without error |

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
- **Then:** Dry-run output contains `cd /tmp/test_project`; working directory is set to `/tmp/test_project`. Session continuation (`-c`) is injected only when prior sessions exist for `/tmp/test_project` — not asserted here (see US-5 and T10 for session-dir coverage)
- **Exit:** 0

---

### US-5: Empty session dir → REPL opens without error (BUG-214 regression)

- **Given:** Terminal with TTY attached; `--session-dir` points to a freshly created empty directory (no prior claude session)
- **When:** `clr --session-dir /tmp/mre214_empty`
- **Then:** Assembled command (dry-run) does NOT contain `-c`; `session_exists()` guard detected empty directory and suppressed injection; REPL opens without "No conversation found" error
- **Exit:** 0
- **Source:** [invariant/001_default_flags.md § Fixed Defects](../../../../docs/invariant/001_default_flags.md), [bug/214_bare_clr_exits_no_session.md](../../../../../../../task/claude_runner/completed/214_bare_clr_exits_no_session.md)
- **Implementation:** `tests/param_edge_cases_test.rs:562` — `bug_214_empty_session_dir_suppresses_continue_flag` (located in param edge-case suite, not user_story_test.rs)
