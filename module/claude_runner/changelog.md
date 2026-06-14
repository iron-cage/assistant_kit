# Changelog

All notable changes to claude_runner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.0] - 2026-06-14

### Added

- **`clr kill <pid>` subcommand** — terminate a running Claude Code session by PID (TSK-201)
  - Sends SIGTERM to the target process; validates PID belongs to a running `claude` process via `find_claude_processes()`
  - Exits 0 with `"Sent SIGTERM to Claude Code session <PID>."` on success; exits 1 if PID not found or not a `claude` process
  - Typo guard: `clr kil`, `clr killl` trigger "Did you mean 'kill'?" and exit 1
  - `clr kill --help` / `clr kill -h` exit 0 with usage info
  - Documented as command 07, user story 027 (6 acceptance criteria)

- **`clr ps` gate state files** — `gate.rs` writes JSON state to `$CLR_GATE_DIR/{pid}.json` on each polling cycle (TSK-200)
  - File contains: `cwd`, `since` (Unix timestamp), `attempt` (0-based), `message` (human-readable status)
  - Deleted when the process acquires a session slot or exhausts its retry budget
  - `CLR_GATE_DIR` environment variable overrides the default `/tmp/clr-gate/` gate state directory

- **`clr ps` queued CLR processes table** — second table shows `clr` processes blocked in `wait_for_session_slot()` (TSK-200)
  - Columns: `#`, `PID`, `CWD`, `Waiting`, `Attempt`
  - Present only when ≥1 gate state file exists in `$CLR_GATE_DIR`
  - No active sessions + queued waiters: prints "No active Claude Code sessions." sentinel above the queued table for context
  - `Waiting` column uses same duration format as `Elapsed`: `"45s"` / `"8m 30s"` / `"2h 15m"`

- **`clr ps` titled caption rule lines** — each table is preceded by a caption showing table name and count
  - Active sessions: `─── Active Sessions · N running ──────`
  - Queued waiters: `─── Queued · N waiting ──────`
  - Rendered via `data_fmt::TableCaption`; `data_fmt` dependency updated `^0.3` → `^0.4`

- **`CLAUDE.md` provisioned into isolated subprocess home** — `clr isolated` writes a fresh `CLAUDE.md` to `<temp_home>/.claude/CLAUDE.md` before invoking the subprocess (TSK-022)
  - Ensures the subprocess receives a clean, governed configuration regardless of host `~/.claude/CLAUDE.md` state
  - The temp home is isolated from the user's real `$HOME` via `--home <temp_home>` flag

### Changed

- **`clr ps` table style** — unicode-box → plain-style; `Started` column renamed `Elapsed` with duration format (TSK-199, TSK-200)
  - Plain-style: no outer borders, dash separator under header row, 2-space column gaps
  - `Elapsed` shows time since process start as `"45s"` / `"8m 30s"` / `"2h 15m"` (was Unix timestamp string)
  - PIDs sorted numerically (ascending) in both tables; previously unordered

- **`clr ps` `$PRO` path shortening** — `Absolute Path` (active) and `CWD` (queued) columns replace the `$PRO` prefix with the literal `"$PRO"` string when the `PRO` env var is set (TSK-199)

- **Isolated subprocess model upgraded to Claude Opus 4.6** — `ISOLATED_DEFAULT_MODEL` changed from `claude-sonnet-4-6` to `claude-opus-4-6`; effort set to `EffortLevel::Max` (TSK-021)
  - `--dangerously-skip-permissions` injected when a message is present; `--no-session-persistence` always injected
  - Rationale: isolated mode runs high-stakes, single-shot tasks requiring maximum capability

- **Refresh subprocess model set to Claude Sonnet 4.6** — new `REFRESH_DEFAULT_MODEL` = `"claude-sonnet-4-6"`; effort set to `EffortLevel::Low`; `--no-chrome` and `--no-session-persistence` always injected (TSK-021)
  - Rationale: refresh is a lightweight credential-ping task; Sonnet at low effort is sufficient and faster

- **`--max-sessions` default raised 25 → 30** (Plan 011)
  - Reflects typical parallel workloads; users with stricter limits can still pass `--max-sessions <N>` explicitly

- **Rate-limit retry defaults tightened** — `--retry-on-rate-limit` default 0 → 1; `--retry-delay` default 60 → 30 (Plan 010)
  - One automatic retry with a 30s backoff is the right default for most automation; `--retry-on-rate-limit 0` disables

### Fixed

- **Isolated subprocess timeout semantics corrected** — `--timeout 0` now means "no deadline" (unlimited) for `clr isolated`, consistent with `clr run`/`clr ask` (TSK-022)
  - Previously `timeout=0` was passed to `wait_for_output()` which treated 0 as "expire immediately"
  - Fix: `run_isolated_command` uses `Option<Instant>` for deadline; `None` = no deadline

### Hygiene

- **code_hyg_l1 audit** — test file sizes reduced (all under 500 lines), duplicate helpers consolidated, Fix() doc comments completed
  - `user_story_test.rs` (1911 lines) split into `user_story_creds_isolated_test.rs` + `user_story_output_test.rs`
  - `cli_args_test.rs` (1076 lines) split into `cli_args_ext_test.rs`; `env_var_test.rs` (1027 lines) split into `env_var_ext_test.rs`
  - 7 bare `Fix(BUG-NNN)` source comments in `src/cli/mod.rs` completed with `Root cause:` and `Pitfall:` fields

## [1.1.0] - 2026-06-07

### Fixed

- **`guard_unknown_subcommand` now catches edit-distance-1 typos** (BUG-250)
  - `clr assk`, `clr runn`, etc. now emit "Did you mean '…'?" and exit 1 instead of silently falling through
  - Root cause: guard only used `starts_with` checks; mid-word insertions like "assk" matched neither direction
  - Fix: added `is_close_typo()` helper (first-char guard + Levenshtein ≤ 1); extended guard condition with `|| is_close_typo(first, sub)`
  - First-char constraint prevents false positives for words like "task" (edit distance 1 from "ask" but different initial letter)
  - Reproducer: `t12_ask_edit_distance_typo_caught_by_guard` in `tests/ask_command_test.rs`

- **`clr ask help` now shows ask help instead of hitting session gate** (BUG-249)
  - Previously treated positional "help" as a message and blocked on the `--max-sessions` gate
  - Root cause: only `--help`/`-h` flags were intercepted; positional "help" flowed into `dispatch_run`
  - Fix: positional "help" intercept added to `dispatch_ask()`, mirroring BUG-215 fix for `clr run help`
  - Reproducer: `t11_ask_positional_help_shows_help` in `tests/ask_command_test.rs`

- **`run_print_mode()` no longer discards stdout on non-zero subprocess exit** (BUG-247)
  - Captured stdout was silently dropped when exit code was non-zero; callers saw no diagnostic output
  - Fix: forward captured stdout to stderr before `std::process::exit()` on all failure paths

- **`--keep-claudecode` warning placement corrected** (BUG-248)
  - Warning now fires in all execution modes including `--dry-run` (placed before dry-run branch)
  - Gated on `shows_warnings()` (verbosity ≥ 2) so `--verbosity 0/1` remains silent

- **`ask` default `--max-tokens` corrected to 200000** (matching `run`)
  - Documentation incorrectly stated 8096; `ask` is a pure semantic alias with identical defaults to `run`

- **`run_print_mode()` error label now distinguishes quota exhaustion from rate limit** (BUG-037 follow-up)
  - `ErrorKind::QuotaExhausted` emits "quota exhausted" label; `ErrorKind::RateLimit` emits "rate limit"
  - Previously both were labelled identically, obscuring whether the issue was transient or period-bounded

- **Empty positional arg `""` after `--` separator no longer produces degenerate `"ultrathink "` message** (issue-empty-msg-double-dash)
  - `clr -- ""` now behaves identically to `clr --` (no message, no `--print`, interactive REPL)
  - Root cause: the `--` arm in `parse_args` used `positional.extend()` which copies all tokens verbatim; the empty-token guard in the `_` arm did not apply to this code path
  - Fix: filter empty tokens in the `--` arm via `.filter(|t| !t.is_empty())` before extending positional
  - Reproducer: `t57_empty_positional_after_double_dash_ignored` in `tests/ultrathink_args_test.rs`

- **Empty positional arg `""` no longer produces degenerate `"ultrathink "` message** (issue-empty-msg-ultrathink)
  - `clr ""` now behaves identically to bare `clr` (no message, no `--print`, interactive REPL)
  - Root cause: empty token was pushed to positional list, joined to `message = Some("")`, then the ultrathink prefix produced `"ultrathink "` (trailing space) and triggered print mode
  - Fix: skip empty tokens in the positional-arg collection path of `parse_args`
  - Reproducer: `t54_empty_positional_arg_ignored` in `tests/ultrathink_args_test.rs`

- **`--help`/`-h` now wins over unknown flags regardless of position** (issue-help-loses-to-unknown)
  - `clr --help --unknown` and `clr --unknown --help` both now exit 0 and show USAGE
  - Root cause: `parse_args` returned `Err` immediately on the first unknown flag; `main()` then took the error path before ever consulting `cli.help`
  - Fix: pre-scan for `--help`/`-h` at the top of `parse_args`; if found, return `CliArgs { help: true, .. }` immediately without full parsing
  - Reproducers: `t55_help_wins_over_subsequent_unknown_flag` and `t56_help_wins_over_preceding_unknown_flag` in `tests/ultrathink_args_test.rs`

### Added

- **`clr ps` subcommand** — list all running Claude Code sessions in a unicode-box table
  - Columns: `#`, `PID`, `Started`, `CPU%`, `RAM`, `State`, `Absolute Path`, `Task`
  - Data sourced from `/proc/{pid}/stat`, `/proc/{pid}/status`, `~/.claude/projects/` JSONL logs
  - "No active Claude Code sessions." message when 0 processes found; self-PID excluded from output
  - Typo guard: `clr p` and `clr pss` trigger "Did you mean 'ps'?" and exit 1
  - Linux-only (`#[cfg(target_os = "linux")]`); depends on `data_fmt` for table formatting
  - Documented as command 06, user story 026

- **`--output-file <PATH>` parameter** — write captured output to file (tee: stdout + file simultaneously)
  - Output is both printed to stdout and written to the specified path; file is created/truncated on each run
  - Env var fallback: `CLR_OUTPUT_FILE`
  - Documented as param 029, Group 2 (Runner Control)

- **`--expect <VALS>` parameter** — pipe-separated expected values; output mismatch exits 3
  - Case-insensitive, whitespace-trimmed comparison against captured stdout
  - Env var fallback: `CLR_EXPECT`
  - Documented as param 030, Group 2 (Runner Control)

- **`--expect-strategy <STRAT>` parameter** — mismatch handling: `fail` (default), `retry`, `default:<VAL>`
  - `fail`: exit 3 immediately; `retry`: re-invoke up to `--expect-retries` times; `default:<VAL>`: substitute value on mismatch
  - Env var fallback: `CLR_EXPECT_STRATEGY`
  - `--expect-strategy "default:"` (empty VALUE) is valid — substitutes empty string on mismatch
  - Documented as param 031, Group 2 (Runner Control)

- **`--expect-retries <N>` parameter** — retry attempts when `--expect-strategy retry` (0–255, default: 0)
  - Silently ignored when strategy is not `retry`
  - Env var fallback: `CLR_EXPECT_RETRIES`
  - Documented as param 032, Group 2 (Runner Control)

- **`--max-sessions <N>` parameter** — max concurrent claude sessions before blocking (0=unlimited, default: 25)
  - Blocks up to 100 attempts (30s each) polling `/proc/*/cmdline` for running `claude` processes
  - Env var fallback: `CLR_MAX_SESSIONS`
  - Documented as param 033, Group 2 (Runner Control)

- **`--retry-on-rate-limit <N>` parameter** — automatic retry on transient rate-limit exit (0–255, default: 1)
  - When subprocess exits 2 (`ErrorKind::RateLimit`) and retries remain, waits `--retry-delay` seconds and re-invokes
  - `QuotaExhausted`, `AuthError`, `ApiError`, `Signal`, `Unknown` are never retried
  - On exhaustion: emits "rate limit retries exhausted" to stderr, propagates exit 2
  - Applies to print-mode (`run_print_mode()`) only; interactive mode not retried
  - Env var fallback: `CLR_RETRY_ON_RATE_LIMIT`
  - Documented as param 034, Group 2 (Runner Control)

- **`--retry-delay <SECS>` parameter** — seconds between rate-limit retries (u32, default: 30)
  - 0 = immediate retry (no sleep); silently ignored when `--retry-on-rate-limit` is 0
  - Env var fallback: `CLR_RETRY_DELAY`
  - Documented as param 035, Group 2 (Runner Control)

- **`--timeout <SECS>` parameter for `run`/`ask`** — kill subprocess after N seconds (u32, default: 0 = unlimited)
  - Spawns watchdog via `spawn_piped()` + `try_wait()` polling at 50ms intervals; sends SIGKILL on deadline
  - Applies to both print-mode and interactive mode
  - Semantic contrast with `isolated`/`refresh` where `--timeout 0` means immediate expiry
  - Env var fallback: `CLR_TIMEOUT`
  - Documented as param 036, Group 2 (Runner Control)

- **`ErrorKind::QuotaExhausted` variant** — distinct from `ErrorKind::RateLimit`
  - Matched by "You've hit your limit" pattern in subprocess stdout/stderr
  - `RateLimit` = transient HTTP 429 (retry in seconds); `QuotaExhausted` = period boundary (wait for reset or switch account)
  - Priority-ordered before `AuthError` and `ApiError` in pattern scan

- **`ask` subcommand documented as pure semantic alias** — no behavioral differences from `run`
  - All flags, defaults, and exit codes are identical; `ask` exists for readability at call sites

- **Default `"ultrathink "` message prefix with `--no-ultrathink` opt-out** (task 090)
  - Every `clr` invocation prepends `"ultrathink "` to the message before forwarding to the `claude` subprocess, activating extended thinking mode for all automation without user intervention
  - `--no-ultrathink` flag disables the automatic prefix for callers managing their own prompt structure
  - Idempotent guard: messages already beginning with `"ultrathink"` are not double-prefixed
  - Single injection site in `build_claude_command()` via `effective_msg` local; all paths share the same transformation
  - Documented in `docs/invariant/001_default_flags.md` as the fourth default injection alongside `-c`, `--dangerously-skip-permissions`, and `--chrome`
  - 7 new tests (T50-T53 + 3 dry-run tests); 9 stale test assertions updated to reflect prefixed output

- **`--system-prompt <TEXT>` and `--append-system-prompt <TEXT>` flags** (tasks 084-085)
  - `--system-prompt` replaces the default Claude system prompt for the invocation
  - `--append-system-prompt` appends text to the active system prompt (compatible with `--system-prompt`)
  - Both flags forwarded to the `claude` subprocess; CLI delegates to `with_system_prompt()` / `with_append_system_prompt()` builder methods in `claude_runner_core`
  - Documented as Type 6 (`SystemPromptText`), Group 3 (`System Prompt`), params 15–16, Workflow 9
  - `docs/cli/parameter_interactions.md` and `docs/cli/testing/` added to bring crate to L4

### Changed

- **`guard_unknown_subcommand` threshold loosened** — now fires for any non-empty first token
  - Previously required `first.len() >= 4`; short typos like `clr p` and `clr pss` were unguarded
  - Safe because known subcommands (run, ask, isolated, refresh, ps) are dispatched before the guard
  - First-char check in `is_close_typo` prevents false positives for common words

- **Gate dedup: private `count_claude_sessions()` removed from `gate.rs`**
  - Replaced with `find_claude_processes().len()` from `claude_core::process` — single canonical source
  - No behavioral change; eliminates redundant `/proc` scanning logic

- **`src/cli/mod.rs` split into focused submodules** (Plan 007 refactor)
  - `cli/help.rs` — all 4 help-printing functions (`print_help`, `print_ask_help`, `print_isolated_help`, `print_refresh_help`)
  - `cli/gate.rs` — `wait_for_session_slot()` concurrency gate (uses `find_claude_processes()` from `claude_core::process`)
  - `cli/mod.rs` reduced from ~600 lines to ~440 lines

- **`--dangerously-skip-permissions` is now default-on** (task 058)
  - Every `clr` invocation silently injects `--dangerously-skip-permissions` to avoid stalling automation pipelines
  - New flag `--no-skip-permissions` disables the automatic bypass for contexts requiring human approval gates
  - `--dangerously-skip-permissions` is no longer a user-facing flag; passing it explicitly produces "unknown option" error
  - Default Flags Principle migrated to `docs/invariant/001_default_flags.md` (spec.md deleted post-migration)

- **Process management moved to `claude_runner_core`** (task 037)
  - `ClaudeCommand` builder and `execute()` moved from `claude_runner` to `claude_runner_core`
  - `claude_runner` binary now delegates process execution to `claude_runner_core`
  - Library surface intentionally minimal: only `COMMANDS_YAML` path and `VerbosityLevel`
  - `stale_ref_guard_test.rs` guards against any regression to pre-migration import paths

- **`architecture_migration_plan.md` removed** (task 037)
  - Post-migration artifact; no longer needed now that the move is complete

### Documentation

- **CLI documentation synced with unilang 5-phase pipeline migration** (task 040)
  - `docs/cli/commands.md`, `params.md`, `types.md`, `dictionary.md`,
    `parameter_groups.md`, `workflows.md` updated to reflect 5-phase architecture
  - `docs/001_design_decisions.md` updated with post-migration rationale
