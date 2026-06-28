# Invariant: Default Flags

### Scope

- **Purpose**: Document the automatic flag injection behavior that must be maintained across all `clr run` invocations.
- **Responsibility**: State which flags are injected by default, their opt-out mechanism, and why the defaults exist.
- **In Scope**: Automatic `-c` injection, `--dangerously-skip-permissions` default-on, `--chrome` builder default, `"\n\nultrathink"` message suffix default-on, `--effort max` default-on, `CLAUDECODE` env var removal default-on, `--new-session` override, `--no-skip-permissions` opt-out, `--no-ultrathink` opt-out, `--no-effort-max` opt-out, `--no-chrome` opt-out, `--keep-claudecode` opt-out.
- **Out of Scope**: Dependency constraints (→ `invariant/002_dep_constraints.md`), execution mode behavior (→ `feature/001_runner_tool.md`).

### Invariant Statement

`clr run` must inject the following flags on every invocation unless explicitly overridden. These defaults apply to the `run` command (including the implicit default when no subcommand is given). The `ask` command is a pure semantic alias for `run` with identical defaults — see [command/05_ask.md](../cli/command/05_ask.md).

| Flag | Default | Override | Rationale |
|------|---------|----------|-----------|
| `-c` (continue conversation) | ON (when session exists) | `--new-session` | Automation expects session continuity by default; injected only when session storage is non-empty (`session_exists()` guard); expected session UUID captured for post-execution mismatch detection (→ `invariant/009_session_mismatch_detection.md`) |
| `--dangerously-skip-permissions` | ON | `--no-skip-permissions` | Automation pipelines must not stall on permission prompts |
| `--chrome` | ON (interactive only; suppressed in print mode — BUG-304) | `--no-chrome` | Browser context for web-aware automation; suppressed in print mode to prevent permanent session hang |
| `"\n\nultrathink"` message suffix | ON | `--no-ultrathink` | Extended thinking mode should be the automation default |
| `--effort max` | ON | `--effort <level>` or `--no-effort-max` | Agentic automation requires maximum reasoning; claude binary default (`medium`) undershoots |
| `CLAUDECODE` removal | ON | `--keep-claudecode` | Subprocess must behave as standalone; inheriting `CLAUDECODE=1` triggers nested-agent mode which alters permissions, output format, and tool availability |

These defaults are intentional and must not be removed without explicit design decision. They represent the automation-optimized defaults for the `clr` runner.

### Enforcement Mechanism

The flag injection is implemented at three layers:
- `-c`: injected by `build_claude_command()` via `session_exists()` guard — only when `!cli.new_session` AND the configured session directory (or `$HOME/.claude/projects/{encoded(effective_dir)}/` default, checked via `most_recent_session_id()`) contains conversation files. `session_exists()` now returns `Option<SessionId>` (the UUID of the most-recently-modified `.jsonl` file) rather than `bool`; this UUID is threaded as `expected_session_id` through `build_claude_command()` → `run_built_command()` → `run_print_mode()` for post-execution mismatch detection. `--dangerously-skip-permissions`: injected unconditionally unless `--no-skip-permissions` is set.
- `--chrome`: injected via `ClaudeCommand::new()` builder default (`chrome: Some(true)`). CLI opt-out: `--no-chrome`. Rust API callers can also override with `with_chrome(None)` or `with_chrome(Some(false))`. Print-mode suppression (Fix(BUG-304)): `builder.rs` computes `use_print` early and applies `if cli.no_chrome || use_print { chrome = None }` — prevents `--chrome` emission for all print-mode invocations without requiring `--no-chrome`.
- `"\n\nultrathink"` message suffix: appended to the message string inside `build_claude_command()` before `builder.with_message()` is called. Skipped when `cli.no_ultrathink` is set or the message already ends with `"ultrathink"` (idempotent guard — `msg.trim_end().ends_with("ultrathink")`).
- `--effort max`: injected by `build_claude_command()` via `builder.with_effort(cli.effort.unwrap_or(EffortLevel::Max))`. Skipped entirely when `cli.no_effort_max` is set. Overridden to a different level when `cli.effort` is `Some(level)`.
- `CLAUDECODE` removal: `std::env::remove_var("CLAUDECODE")` called on the subprocess environment before spawn. Skipped when `cli.keep_claudecode` is set.

`--dangerously-skip-permissions` is no longer user-facing as a positive flag. Users disable it via `--no-skip-permissions`. This prevents confusion between "skip permissions" as an intentional choice vs. the default behavior.

Dry-run mode (`--dry-run`) shows the injected flags in its output — the preview reflects the actual command that would be run, including all default injections.

### Violation Consequences

If any default injection is removed:
- Interactive automation scripts stall waiting for permission prompts (skip-permissions removed)
- Each invocation starts a new session, losing conversation context (continuation removed)
- Claude performs fast (non-extended) thinking on every automation request (ultrathink suffix removed)
- Claude uses medium-effort reasoning instead of maximum for every automation request (effort max removed)
- Subprocess detects nested-agent context and alters permissions, output format, and tool availability (`CLAUDECODE` removal skipped)
- Automation pipelines that depend on these defaults will behave differently without a version change

### Fixed Defects

**BUG-214 (fixed 2026-05-28) — `-c` was injected unconditionally regardless of session existence:**
`build_claude_command()` now calls `session_exists()` before injecting `-c`. On first invocation or with an empty `--session-dir`, `-c` is suppressed and the REPL opens unconditionally. The invariant above (`-c` on by default) now carries the implicit precondition: session storage must be non-empty.

- **Root cause:** `src/cli/builder.rs` — `session_exists()` + `check_continuation()`, `Fix(BUG-214-reopen)` comment; original `cli/mod.rs` path is now `builder.rs` after refactor
- **Bug report:** `claude_tools/task/claude_runner/bug/214_bare_clr_exits_no_session.md` (external to crate)

**BUG-304 (INT mitigation 2026-06-21) — `claude --print --chrome` sessions never exit:**
`build_claude_command()` now computes `use_print` before the `--no-chrome` guard and applies `if cli.no_chrome || use_print` to suppress `--chrome` in all print-mode invocations. Root cause (EXT): Node.js/libuv registers a ref-counted 1-second timerfd (Chrome CDP reconnect) that is never `unref()`'d after `--print` response flush; event loop cannot drain; `clr`'s `cmd.output()` deadlocks. `--chrome` remains active in interactive mode.

- **Root cause (INT):** `src/cli/builder.rs` — `use_print` computed early; `if cli.no_chrome || use_print` guard; `Fix(BUG-304)` comment
- **Bug report:** `task/claude_runner/bug/304_print_chrome_session_permanent_hang.md`

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Execution modes that consume these injected defaults |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/mod.rs` | `build_claude_command()` entry point and CLI dispatch |
| `../../src/cli/builder.rs` | `session_exists()` guard and `build_claude_command()` implementation |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/cli_args_test.rs` | T01–T35 flag parsing; --new-session, --no-skip-permissions, --no-ultrathink, --no-effort-max, --no-chrome |
| `../../tests/cli_args_ext_test.rs` | T36–T49, S58–S79; --keep-claudecode and extended flag coverage |
| `../../tests/ultrathink_args_test.rs` | T50–T58 ultrathink suffix injection, idempotent guard, and --no-ultrathink opt-out |
| `../../tests/effort_args_test.rs` | T59–T70 --effort max default injection and override behavior |
| `../../tests/param_edge_cases_test.rs` | `bug_214_empty_session_dir_suppresses_continue_flag` — BUG-214 regression (empty `--session-dir` → no `-c`) |
| `../../tests/dry_run_test.rs` | `bug_reproducer_214_no_session_dir_fresh_cwd_no_continue_flag` — BUG-214-reopen regression (fresh CWD, no `--session-dir` → no `-c`) |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Default Flags Principle, Modes table, CLI Flags (--new-session, --no-skip-permissions) |
