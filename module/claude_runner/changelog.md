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

- **3-tier retry hierarchy with 20 parameters** — class-specific retry counts and delays for all 8 error classes (TSK-205)
  - Tier 1 (`--retry-override`/`--retry-override-delay`): forces retry count/delay for all classes
  - Tier 2: per-class params (`--retry-on-<class>`/`--<class>-delay`) for Transient, Account, Auth, Service, Process, Validation, Runner, Unknown
  - Tier 3 (`--retry-default`/`--retry-default-delay`): fallback for unset classes (default: count=2, delay=30s)
  - Resolution: `resolve_count(override, class_specific, fallback).unwrap_or(2)`
  - Stderr error labels use `[Class]` prefix: `"Error: [Transient] rate limit (exit 2)"`
  - Param doc files 040–057; env vars `CLR_RETRY_ON_ACCOUNT` through `CLR_RETRY_DEFAULT_DELAY`

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

- **Retry param renames** — 6 params renamed to align with error class taxonomy (TSK-205)
  - `--retry-on-rate-limit` → `--retry-on-transient`; `--retry-delay` → `--transient-delay`
  - `--retry-on-api-error` → `--retry-on-service`; `--api-error-delay` → `--service-delay`
  - `--retry-on-unknown-error` → `--retry-on-unknown`; `--expect-retries` → `--retry-on-validation`
  - Old names rejected at parse time (exit 1)

- **Retry defaults now uniform via fallback tier** — all 8 error classes default to count=2/delay=30s via `--retry-default`/`--retry-default-delay` (TSK-205)
  - Previously only Transient had retry support (count=1, delay=30); all other classes were immediate-fail

### Fixed

- **`clr run`/`ask` timeout now exits 4** — disambiguates from rate-limit exit 2 (TSK-202)
  - `poll_timeout()` in `execution.rs` calls `std::process::exit(4)` instead of `exit(2)`
  - `clr isolated`/`refresh` timeout still exits 2 (preserves "no credentials refreshed" semantics)
  - Exit code contract tests added: `exit_code_contract_test.rs` (EC-1/EC-2/EC-3)

- **Stale gate files no longer displayed as live waiting processes in `clr ps`** (BUG-293)
  - `build_queued_table()` now probes `/proc/{pid}` before rendering; orphaned files self-heal via `remove_file`
  - `GateFile` RAII struct with `Drop` impl in `gate.rs` ensures cleanup on normal exit and panic unwind
  - Regression test: IT-13 in `ps_command_test.rs`

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
  - `fail`: exit 3 immediately; `retry`: re-invoke up to `--retry-on-validation` times; `default:<VAL>`: substitute value on mismatch
  - Env var fallback: `CLR_EXPECT_STRATEGY`
  - `--expect-strategy "default:"` (empty VALUE) is valid — substitutes empty string on mismatch
  - Documented as param 031, Group 2 (Runner Control)

- **`--retry-on-validation <N>` parameter** — retry count for Validation (expect-mismatch) errors (0–255; Tier 2, falls back to `--retry-default`)
  - Retries when `--expect-strategy retry` and output mismatches `--expect` pattern
  - Silently ignored when strategy is not `retry`
  - Env var fallback: `CLR_RETRY_ON_VALIDATION`
  - Documented as param 048, Group 2 (Runner Control)
  - Renamed from `--expect-retries` (TSK-205)

- **`--max-sessions <N>` parameter** — max concurrent claude sessions before blocking (0=unlimited, default: 25)
  - Blocks up to 100 attempts (30s each) polling `/proc/*/cmdline` for running `claude` processes
  - Env var fallback: `CLR_MAX_SESSIONS`
  - Documented as param 033, Group 2 (Runner Control)

- **`--retry-on-transient <N>` parameter** — retry count for Transient (rate-limit) errors (0–255; Tier 2 class-specific, falls back to `--retry-default`)
  - When subprocess exits 2 (`ErrorKind::RateLimit`) and retries remain, waits `--transient-delay` seconds and re-invokes
  - Applies to print-mode only; interactive mode not retried
  - Env var fallback: `CLR_RETRY_ON_TRANSIENT`
  - Documented as param 034, Group 2 (Runner Control)
  - Renamed from `--retry-on-rate-limit` (TSK-205)

- **`--transient-delay <SECS>` parameter** — seconds between Transient retries (u32; Tier 2, falls back to `--retry-default-delay`)
  - 0 = immediate retry (no sleep)
  - Env var fallback: `CLR_TRANSIENT_DELAY`
  - Documented as param 035, Group 2 (Runner Control)
  - Renamed from `--retry-delay` (TSK-205)

- **`--timeout <SECS>` parameter for `run`/`ask`** — kill subprocess after N seconds (u32, default: 0 = unlimited)
  - Spawns watchdog via `spawn_piped()` + `try_wait()` polling at 50ms intervals; sends SIGKILL on deadline
  - Applies to both print-mode and interactive mode
  - Semantic contrast with `isolated`/`refresh` where `--timeout 0` means immediate expiry
  - Env var fallback: `CLR_TIMEOUT`
  - Documented as param 036, Group 2 (Runner Control)

- **3-tier retry parameter hierarchy** — 20 retry parameters organized in override/class-specific/fallback tiers (TSK-205)
  - Tier 1 (Override): `--retry-override` and `--retry-override-delay` beat all class-specific settings
  - Tier 2 (Class-specific): per-error-class retry count and delay pairs for all 8 error classes
  - Tier 3 (Fallback): `--retry-default` (default: 2) and `--retry-default-delay` (default: 30s) apply when no class-specific value is set
  - Resolution: `effective_count(class) = override ?? class_specific ?? fallback`

- **Class-specific retry params** — per-error-class count/delay pairs (TSK-203, TSK-205)
  - `--retry-on-account`/`--account-delay` (params 040/041) — Account errors
  - `--retry-on-auth`/`--auth-delay` (params 042/043) — Auth errors
  - `--retry-on-service`/`--service-delay` (params 044/045) — Service (API) errors
  - `--retry-on-process`/`--process-delay` (params 046/047) — Process errors
  - `--validation-delay` (param 049) — delay between Validation retries
  - `--retry-on-runner`/`--runner-delay` (params 050/051) — Runner errors
  - `--retry-on-unknown`/`--unknown-delay` (params 052/053) — Unknown errors
  - All class-specific params use `Option<T>` (absent = defer to fallback tier)
  - Env var fallbacks: `CLR_RETRY_ON_{CLASS}` / `CLR_{CLASS}_DELAY`

- **Override and fallback tier params** — global retry control (TSK-205)
  - `--retry-override <N>` / `--retry-override-delay <SECS>` (params 054/055) — Tier 1; beats all class-specific
  - `--retry-default <N>` / `--retry-default-delay <SECS>` (params 056/057) — Tier 3; default 2/30s
  - Env var fallbacks: `CLR_RETRY_OVERRIDE`, `CLR_RETRY_OVERRIDE_DELAY`, `CLR_RETRY_DEFAULT`, `CLR_RETRY_DEFAULT_DELAY`

- **`[Class]` prefix in console error and retry output** — all retry progress and terminal error messages include error class (TSK-205)
  - Retry: `[Transient] <message> — retrying in Xs (attempt M/N)…`
  - Terminal: `Error: [Transient] <message> (exit N)`
  - Classes: Transient, Account, Auth, Service, Process, Validation, Runner, Unknown

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
