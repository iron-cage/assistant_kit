# Changelog

All notable changes to claude_runner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Dependency upgrade: `data_fmt ^0.4 → ^0.6`** (TSK-327)
  - Migrated `clr ps` and `clr tools` from `TableCaption`/`.field()`/`.caption()` to `Heading`/`.with_field()`/`.with_heading()` (renamed in data_fmt TSK-009)
  - Removed two-pass probe workaround in `ps.rs::render_plain_table()` and inline in `tools.rs::dispatch_tools()` — data_fmt ≥0.5.1 fills heading rule to rendered table body width automatically (data_fmt TSK-008)
  - No behavioral change to `clr ps` / `clr tools` output content
- **Dependency upgrade: `cli_fmt ^0.9 → ^0.10`** — API-compatible; no code changes required
- **Dependency upgrade: `test_tools ^0.17 → ^0.18`** — API-compatible; no code changes required

### Added

- **Event journaling integration** (Plan 033, TSK-326)
  - `--journal <full|meta|off>` controls event emission level; `full` (default) captures stdout/stderr (≤1MB each), `meta` records metadata only, `off` disables journaling
  - `--journal-dir <PATH>` overrides journal directory; 3-tier resolution: CLI flag > `CLR_JOURNAL_DIR` env var > `~/.clr/journal/`
  - 8 emission points: Execution (success/error), Credential (isolated/refresh), GateWait (session slot acquired), Retry (per-class), Timeout (watchdog), RunnerRetry (spawn failure), Interactive (session start/end)
  - Best-effort: journal write failures never change the runner exit code
  - Parameters: [`--journal`](docs/cli/param/072_journal.md) (072), [`--journal-dir`](docs/cli/param/073_journal_dir.md) (073)
  - Env vars: `CLR_JOURNAL`, `CLR_JOURNAL_DIR`; invalid `CLR_JOURNAL` exits 1
  - New dependency: `claude_journal` (workspace)
  - Tests: `journal_integration_test.rs` (EC-1–EC-10)

- **Help split: `RUNNER OPTIONS` / `CLAUDE CODE OPTIONS (forwarded)` sections** (TSK-232)
  - `clr --help` now uses `CliHelpTemplate` from `cli_fmt ^0.9` instead of 262-line hand-rolled `print!` calls
  - Two option groups: `RUNNER OPTIONS` (47 entries — flags consumed by the runner) and `CLAUDE CODE OPTIONS (forwarded)` (14 entries — flags passed to claude verbatim)
  - Users can now distinguish which flags `clr` processes vs. which pass through to claude
  - `print_help()` refactored into `runner_option_group()` + `claude_code_option_group()` helpers (under Clippy 100-line limit)
  - `cli_fmt` workspace dep upgraded `^0.8` → `^0.9`; all 7 workspace crates using `CliHelpData` migrated from struct literals to `CliHelpData::default()` + field assignment (E0639: struct is `#[non_exhaustive]` in 0.9)
  - Tests: EC-01–EC-06 in `tests/cli_args_ext_test.rs`; spec updated: `tests/docs/cli/command/02_help.md`

- feat: add `--summary-fields` parameter for summary output field selection (TSK-234)
  - Presets: `full` (default, 32 fields), `standard` (14 fields), `minimal` (7 fields)
  - Custom comma-separated field whitelists with validation
  - `CLR_SUMMARY_FIELDS` env var; CLI flag wins when both set
  - Tests: `tests/summary_fields_test.rs` (EC-01–EC-12)

- **`--output-style <MODE>` — runner-level print-mode rendering control** (TSK-231)
  - Two values: `summary` (default) routes stdout through `render_summary()` in `summary.rs` producing a key:val header; `raw` bypasses rendering and passes stdout unchanged
  - **Default changed:** `clr run -m "..."` (no flags) now renders the summary key:val header by default; previous behavior was raw text
  - When `--output-style summary` and `--output-format` is absent, `clr` auto-injects `--output-format json` into the subprocess command; graceful fallback to raw when non-JSON output is received (e.g. `--output-format text` explicitly set)
  - `CLR_OUTPUT_STYLE` env var: accepts `summary` or `raw`; invalid values exit 1 (not silently ignored); CLI flag wins when both set
  - **Separation of concerns:** `--output-style` is runner-level rendering; `--output-format` is a claude passthrough; the two are orthogonal
  - **Legacy alias preserved:** `--output-format summary` remains supported for backward compatibility
  - Sources: `src/cli/parse.rs` (field + dispatch + validation), `src/cli/env.rs` (CLR_OUTPUT_STYLE), `src/cli/builder.rs` (injection branch), `src/cli/execution.rs` (predicate replacement), `src/cli/help.rs` (OPTIONS entry)
  - Tests: `tests/output_style_test.rs` (EC-01–EC-13; EC-14 added by TSK-236); spec: `tests/docs/cli/param/070_output_style.md`

- **Default timeout kill test: `ec_timeout_default_kills` + `default_print_timeout()` helper** (TSK-228)
  - `default_print_timeout() -> u32` helper in `src/cli/execution.rs`: reads `_CLR_DEFAULT_TIMEOUT` env var (test-only override), falls back to `DEFAULT_PRINT_TIMEOUT_SECS` (3600); call site in `run_print_mode()` changed from `unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )` to `unwrap_or( default_print_timeout() )`
  - `ec_timeout_default_kills` integration test in `tests/timeout_test.rs`: sets `_CLR_DEFAULT_TIMEOUT=2`, fake claude sleeps 30s, asserts exit 4 within ~5s and stderr contains "timeout" — proves the `None → default` path fires `poll_timeout()` and kills the subprocess (gap not covered by EC-7 which tests `Some(1)` explicit path only)
  - `ec_timeout_default_constant_value` assertion updated: third assert added for `unwrap_or( default_print_timeout() )` at call site
  - `clr kill --stale` proposal cancelled (TSK-229, MAAV 0/4 FAIL, YAGNI): `--stale` syntax and IT-10..IT-14 reverted from kill docs

- **`clr ps --pid` PID filter and `--inspect` key:value output** (TSK-224)
  - `--pid <PIDs>`: restricts the active sessions table to the specified comma-separated process IDs;
    non-numeric entries exit 1; combined with `--mode` as an AND filter; `CLR_PS_PID` env var fallback
  - `-i`/`--inspect`: emits 12-attribute key:value record blocks per session instead of the table;
    ignores `--columns`/`--wide`; suppresses the Queued CLR Processes table; filters via `--pid`/`--mode`
  - `mode` promoted to `DEFAULT_COLUMNS` — visible in `clr ps` output without any flags
  - Parameters: [`--pid`](docs/cli/param/068_pid.md) (param 068), [`--inspect`](docs/cli/param/069_inspect.md) (param 069)
  - Tests: `ps_pid_test.rs` (EC-1–EC-8), `ps_inspect_test.rs` (EC-1–EC-9)

- **`clr ps` session flags** (TSK-225)
  - Each active session row can display emoji flags: 🐳 (container), 🕰 (ancient), 🐘 (high RAM),
    ⚠ (dead metrics), ⚡ (active), 🖨 (print mode), 👈 (this session)
  - `compute_flags()` + `FLAG_LEGEND` + `build_legend()` in `ps.rs`; legend appended below the table
  - Configurable thresholds: `CLR_PS_ANCIENT_SECS` (default 28800), `CLR_PS_HIGH_RAM_MB` (default 400);
    setting either to `0` triggers the flag on every process
  - Tests: `ps_flags_test.rs` (IT-30–IT-38, US-18–US-26, E41–E42)

- **`clr ps --mode`/`--columns`/`--wide` session listing params** (TSK-214)
  - `--mode <MODE>`: filter active sessions by mode (`print`/`interactive`/`all`); `CLR_PS_MODE` env var
  - `--columns <KEYS>`: select and reorder columns by comma-separated key names; `CLR_PS_COLUMNS` env var
  - `--wide`: shorthand for all columns; overrides `--columns`
  - Implementation: `PsConfig` + `classify_mode()` + `resolve_columns()` + `validate_columns()` +
    `COLUMN_KEYS`/`DEFAULT_COLUMNS` constants in `ps.rs`; `apply_ps_env_vars()` in `env.rs`
  - Parameters: [`--mode`](docs/cli/param/058_mode.md) (058), [`--columns`](docs/cli/param/059_columns.md) (059),
    [`--wide`](docs/cli/param/060_wide.md) (060)
  - Tests: `ps_mode_test.rs` (EC-1–EC-8), `ps_columns_test.rs` (EC-1–EC-10), `ps_wide_test.rs` (EC-1–EC-5)

- **7 Claude-native passthrough params** (Plan 021, TSK-215–221)
  - `--output-format` (061), `--max-turns` (062), `--allowed-tools` (063), `--disallowed-tools` (064),
    `--max-budget-usd` (065), `--add-dir` (066), `--fallback-model` (067)
  - Forwarded directly to the `claude` subprocess; `CLR_*` env var fallbacks for each
  - `--output-format summary` legacy alias routes through `render_summary()` in `summary.rs`
    (renders CLR result envelope as key:val header + text body)

- **`clr tools` subcommand** (Plan 021, TSK-222)
  - Lists all 26 Claude Code built-in tools in a plain-style table with Name, Category, Description columns
  - Source: `src/cli/tools.rs`; static `TOOLS` array sourced from `contract/claude_code/docs/tool/readme.md`
  - Unknown arguments exit 1 (e.g. `clr tools --bogus`)
  - Tests: `tools_command_test.rs` (IT-1–IT-9)

### Fixed

- **Timeout stderr double-emission on retry lines eliminated** (BUG-317)
  - `run_print_mode()` unconditionally forwarded `output.stderr` via `eprint!("{}", output.stderr)` at
    `execution.rs:454` with no trailing newline; `execute_print_attempt()` stores the CLR-synthesized
    timeout label `"timeout after Ns"` in the `stderr` field (exit code 4); the retry formatter then
    extracted the same string via `first_message()` and emitted `[Process] timeout after Ns — retrying…`;
    the two emissions concatenated without a separator → `"timeout after Ns[Process] timeout after Ns —
    retrying…"` on a single terminal line for every retry + exhaustion event
  - Fix: `eprint!` at line 454 gated on `output.exit_code != 4`; exit 4 is the exclusive CLR-internal
    timeout sentinel — no subprocess can produce it in CLR's taxonomy; the retry formatter already surfaces
    the full `[Process]` structured message, so suppressing the bare forward eliminates the double-emission
  - **Pitfall:** storing CLR-synthesized strings in `ExecutionOutput.stderr` violates the field's implicit
    contract (subprocess-originated only); any unconditional `eprint!` of `output.stderr` will double-emit
    when the retry formatter also surfaces the same string via `first_message()`
  - Tests: `ec_timeout_retry_no_double_emission` in `tests/timeout_test.rs` — asserts no stderr line
    starts with `"timeout after"` when timeout fires during a Process-class retry loop; spec: `36_timeout.md`

- **Windows compilation regressions in `claude_runner/tests/` fixed** (BUG-316)
  - Feature 064 pull introduced 3 failures on Windows: (F1) `bug_reproducers_247_test.rs` missing
    `#![cfg(unix)]` → E0432 on `use std::os::unix::process::ExitStatusExt`; (F2) 17 test files had
    `#![allow(missing_docs)]` after `#![cfg(unix)]` — inner attrs must precede cfg gates; (F3)
    `ps_flags_test.rs` ungated `stdout_str` import → E0432 on non-Linux
  - Fix: `#![allow(missing_docs)]` moved before `#![cfg(unix)]` across affected files; `ps_flags_test.rs`
    given crate-level `#![allow(missing_docs)]` + `#![cfg(unix)]` gate; ungated `Command` import
    removed from `output_file_test.rs`; `bug_reproducers_247_test.rs` given `#![cfg(unix)]` gate
  - Cross-platform path separator sweep across 4 test files (`ask_command_test.rs` t10,
    `env_var_ext_test.rs` e29/bug233, `param_extended_flags_test.rs` s81–s86, `param_group_test.rs`
    G2CC4/G2CC6) — `/-NAME` subdir assertions replaced with `std::path::MAIN_SEPARATOR` for both
    positive and negative assertions; tests with hardcoded `/tmp/` paths gated `#[cfg(unix)]`
  - Rule: (1) lint-suppression inner attrs must precede `#![cfg(unix)]`; (2) `/-NAME` assertions
    must use `MAIN_SEPARATOR`; (3) negative `!contains("/-")` assertions are equally broken on
    Windows — `\-` goes undetected; fix both directions when adding cross-platform guards

- **`run_print_mode` retry loop now fails fast on auth errors** (BUG-315, TSK-323)
  - Auth errors (`ErrorKind::AuthError`) are persistent — the auth token is invalid and requires
    explicit credential recovery; the retry loop had no recovery mechanism, so each `sleep + continue`
    iteration reproduced the same 401 failure and exhausted N×delay seconds without useful output
  - Fix: guard the retry-block entry with `!is_auth_error &&` condition; when `kind` is
    `ErrorKind::AuthError`, the sleep+retry block is skipped and execution falls through to the
    non-retriable exit path, which prints `[Auth] …` and calls `process::exit(output.exit_code)`
  - `// Fix(BUG-315)` comment at guard: Root cause: auth errors are persistent without credential
    recovery; retrying consumes budget without recovery path; Pitfall: a retry loop operating on
    auth-class errors without a credential recovery hook deterministically exhausts its budget
  - Tests: `mre_bug315_auth_error_exits_retry_loop_immediately` in `tests/retry_auth_test.rs`

- **`authentication_error` 401 now correctly classified as `[Auth]`** (BUG-314, TSK-322)
  - `ERROR_PATTERNS` had no entry for `"authentication_error"`; the `"API Error: "` catch-all
    matched first → `ApiError` → `ErrorClass::Service`; auth-specific retry logic (`--retry-on-auth`,
    BUG-315 fail-fast guard) never fired on actual 401 credential failures
  - Fix: inserted `("authentication_error", ErrorKind::AuthError)` before `"API Error: "` in
    `claude_runner_core/src/types.rs` `ERROR_PATTERNS`; the Claude CLI 401 output string
    `"Failed to authenticate. API Error: 401 {\"type\":\"authentication_error\",...}"` now classifies
    as `Some(ErrorKind::AuthError)`, not `Some(ErrorKind::ApiError)`
  - `// Fix(BUG-314)` comment at insertion: this pattern must precede the `"API Error: "` catch-all;
    swapping their positions is a silent regression that re-classifies 401 as Service
  - Tests: `mre_bug314_authentication_error_classifies_as_auth_error` in `error_classification_test.rs`

- **`render_summary()` rewritten for CLR result envelope schema** (BUG-309, TSK-233)
  - `render_summary()` hard-gated on `"id"` field (Messages API schema) which is absent from the
    CLR result envelope emitted by `claude --output-format json`; `extract_str(json,"id")?` returned
    `None` on every live invocation; caller's `.unwrap_or(out)` silently printed raw JSON
  - Fix: gate changed to `extract_str(json,"session_id")?`; header now renders CLR envelope fields
    (`type`, `subtype`, `session_id`, `is_error`, `usage`, `total_cost_usd`); `result` field
    appears as text body after `---` separator
  - Removed: `ContentBlock` struct, `parse_content_blocks()`, `parse_input_keys()`, `find_close()`,
    3 unused color constants (`MAGENTA`, `BRIGHT_BLACK`, `BOLD_GREEN`)
  - Added: `extract_f64()` helper for `total_cost_usd` parsing
  - Both test fixtures (`output_style_test.rs`, `output_format_test.rs`) updated from Messages API
    to CLR result envelope; contract test `ec14_render_summary_clr_envelope_accepted` added
  - Tests: all 13 `output_style_test` EC pass, all 14 `output_format_test` EC pass, g2cc4 pass

- **`render_summary()` gate changed from optional `session_id` to invariant `type` field** (BUG-310, TSK-236)
  - `render_summary()` gated on `extract_str(json,"session_id")?` (the BUG-309 fix) which returned
    `None` for any CLR envelope missing `session_id`; at least one observed claude binary version
    emits a minimal 7-field envelope without `session_id`, restoring the BUG-309 raw-JSON symptom
  - Fix: gate replaced with `extract_str(json,"type")?` + `if msg_type != "result" { return None; }`;
    `session_id` now extracted with `.unwrap_or_default()` — absent = empty string, not `None`
  - **Structural anti-pattern documented (D15):** gating `render_summary()` on any optional CLR field
    with `?` is forbidden; only `"type":"result"` (invariant field) is a safe gate; see `docs/001_design_decisions.md`
  - Added: invariant doc `docs/invariant/008_render_summary_gate.md`; IT-7 structural guard in
    `tests/output_style_test.rs` asserts the anti-pattern substring is absent from source
  - Unit tests added to `src/cli/summary.rs`: IT-1 (minimal envelope → `Some`), IT-4 (type≠result → `None`),
    IT-5 (no type field → `None`), IT-6 (non-JSON → `None`)
  - Tests: EC-14 (`ec14_render_summary_minimal_envelope_no_session_id`) in `tests/output_style_test.rs`;
    IT-7 (`render_summary_gate_uses_type_not_session_id`) structural guard in same file

- **Retry diagnostic message shows human text, not raw JSON blob** (TSK-235)
  - In summary mode, `first_message()` returned the raw CLR JSON envelope when the subprocess
    exited with a retriable error — users saw `[Account] {"type":"result","session_id":...}` instead
    of the human-readable result text
  - Fix: `first_message()` gains a `use_summary: bool` parameter; when `use_summary` and stdout
    starts with `{`, it calls `super::summary::extract_result_text()` to extract the `"result"` field
    before falling back to line-scan; `use_summary` binding in `run_print_mode()` set from
    `cli.output_style.as_deref().unwrap_or("summary") == "summary"`
  - On error exhaustion, the raw `eprint!("{}", output.stdout)` is replaced with a `render_summary()`
    gate — produces the same key:val `---` header as the success path in summary mode
  - `pub(super) fn extract_result_text(json: &str) -> Option<String>` added to `summary.rs`
    as a thin wrapper over `extract_str(json, "result")`; raw mode (`--output-style raw`) is
    unaffected on both paths
  - Tests: EC-10 (`ec10_retry_message_shows_result_not_json_blob`),
    EC-11 (`ec11_exhaustion_output_rendered_not_raw_json`) in `retry_account_test.rs`

- **`clr ps` active sessions sorted oldest-first** (BUG-301, TSK-210)
  - `build_active_table()` was emitting rows in arbitrary filesystem order from `/proc`
  - Fix: sort by `read_process_metrics().started_at` ascending; `#[cfg(target_os = "linux")]` guard
  - Tests: IT-20, US-09

- **`clr ps` table width no longer overflows terminal** (BUG-300, TSK-211)
  - Single-pass render used terminal width for both caption and body, causing misaligned wrapping
  - Fix: two-pass render — first probe measures `body_width`, second pass uses `terminal_width(Some(body_width))`
    via `data_fmt 0.4` `build_view()` + `Format::format(&view)`
  - Tests: verified via existing ps_command_test suite

- **`guard_unknown_subcommand()` no longer false-matches short prefixes** (BUG-302, TSK-212)
  - 2–3 character inputs like `"is"` matched `"isolated"` via `sub.starts_with(first)`; the reverse
    branch `first.starts_with(sub)` matched morphological extensions like `"asking"` → `"ask"`
  - Fix: `first.len() >= 4` guard added to prefix branch; reverse extension branch removed entirely
  - Tests: `bug_reproducer_302_*` + 3 regression tests in `cli_args_ext_test.rs`; IN-7/IN-8

- **`print_ps_help()` key names aligned with `COLUMN_KEYS`** (BUG-303)
  - Help text used `num`/`command` but `COLUMN_KEYS` defined `idx`/`cmd`; users following help
    text got "unknown column" errors
  - Fix: 3 lines in `help.rs` corrected to `idx`/`cmd`
  - Tests: EC-10 regression in `ps_columns_test.rs`

- **`dispatch_tools()` now rejects unknown arguments** (Plan 021 post-verification)
  - `clr tools --bogus` silently exited 0; should exit 1 with error
  - Fix: unknown-arg guard in `src/cli/tools.rs` (lines 53–62)
  - Tests: IT-9 in `tools_command_test.rs`

- **`--chrome` suppressed in print mode to prevent permanent session hang** (BUG-304, INT mitigation)
  - `claude --print --chrome` sessions never exit: Node.js/libuv registers a ref-counted 1-second
    timerfd (Chrome CDP reconnect) that is never `unref()`'d after `--print` response flush; event
    loop cannot drain; `clr`'s `cmd.output()` deadlocks waiting for subprocess EOF
  - Fix: `builder.rs` computes `use_print` before the `no_chrome` guard; `if cli.no_chrome || use_print`
    suppresses `--chrome` in all print-mode invocations automatically
  - `--chrome` still injected in interactive mode (no message); `--no-chrome` remains the explicit opt-out
  - Root fix (EXT) required in upstream `claude` binary (`process.exit(0)` after flush)
  - Tests: `s35_default_chrome_injected_interactive`, `s35b_print_mode_suppresses_chrome`

- **`clr ps --help`/`-h`/`help` now exits 0 and prints help** (BUG-294, TSK-206)
  - `dispatch_ps()` lacked the `--help`/`-h`/`help` intercept present in all peer dispatch
    functions (`dispatch_kill`, `dispatch_ask`, `dispatch_isolated`, `dispatch_refresh`)
  - Fix: added help intercept arm; added `print_ps_help()` to `src/cli/help.rs`
  - `clr ps --unknown` still exits 1 (genuinely unknown flags unaffected)
  - Tests: IT-14 (`--help`), IT-15 (`-h`), IT-18 (positional `help`), US-8

- **`clr ps` Task column now correctly shows the last human-typed question** (BUG-295/296/297, TSK-207)
  - Three compounding bugs in `try_jsonl_task()` caused the column to always show "interactive":
    - (BUG-295) Path encoding replaced `/` with `-` but not `_`; Claude encodes both
    - (BUG-296) Content marker `"text":"` did not match Claude's actual JSONL field `"content":"`
    - (BUG-297) Line-selection predicate picked Form B tool_result lines instead of Form A
  - Fix: three one-line changes in `src/cli/ps.rs` `try_jsonl_task()`
  - Tests: IT-16 (end-to-end with underscore CWD), IT-17 (Form A preferred over Form B),
    IT-19 (underscore-free CWD regression)

- **Spawn errors now include `[Runner]` class prefix on all paths** (BUG-298, TSK-208)
  - `spawn_error_msg()` emitted unclassed errors; three bare `eprintln!("Error: {e}")` sites
    in `execution.rs` (expect-validation arm, no-timeout print arm, interactive arm)
    and the gate-timeout message in `gate.rs` all lacked the `[Runner]` prefix
  - Fix: `spawn_error_msg()` updated; bare arms changed to `eprintln!("Error: [Runner] {e}")`;
    gate-timeout message updated to `"Error: [Runner] session gate timed out — ..."` per spec
  - Tests: TC-12 in `error_classification_test.rs`

- **`--retry-on-runner`/`--runner-delay` now have runtime effect** (BUG-299, TSK-209)
  - Both params were parsed and documented but the `apply_runner_retry()` call site was never
    built; all Runner-class exits called `std::process::exit(1)` directly, bypassing retry
  - Fix: `execute_print_attempt()` returns `Result<ExecutionOutput, io::Error>` instead of
    calling `exit(1)` on spawn failure; `run_print_mode()` calls `apply_runner_retry()` on
    `Err`; `wait_for_session_slot()` in `gate.rs` accepts `cli: &CliArgs` and calls
    `apply_runner_retry()` on gate timeout
  - Tests: EC-7 (retry fires on absent binary), EC-8 (retry disabled with `--retry-on-runner 0`)

### Changed

- **⚡ flag trigger redesigned from kernel state R to two-sample CPU delta** (TSK-230)
  - Previous trigger: `/proc/{pid}/stat` field 3 == `R` (kernel scheduler state at sample instant);
    detected only 1–2 of 20 active sessions — effectively useless for monitoring
  - New trigger: read `/proc/{pid}/stat` fields 14+15 (utime+stime) twice with 1 s sleep; fire ⚡
    when delta >= 3 ticks (30 ms CPU in 1 s window); detected 16/20 active sessions in validation
  - Threshold of 3 ticks separates active work (6–100 ticks/s) from BUG-304 timer noise (1–2 ticks)
  - Flag renamed "On CPU" → "Active" in legend and help text
  - No-process optimization: pre-pass sleep skipped when `procs.is_empty()`
  - Tests: IT-39 (sleeping → no ⚡), IT-40 (busy-loop → ⚡), US-23 updated

- **Account error class retry default changed from 0 to auto** (BUG-307, BUG-308)
  - `class_default_count()` removed; `resolve_count()` simplified from 4-tier to 3-tier resolution
    (`--retry-override` ?? `--retry-on-<class>` ?? `--retry-default`); Account now inherits Tier 3
    fallback (effective default = 2) like all other error classes

- **Print-mode sessions now have a 1-hour default timeout watchdog** (BUG-305, TSK-227)
  - `run_print_mode()` used `cli.timeout.unwrap_or( 0 )`, leaving unattended sessions unbounded;
    four `clr run --print --chrome` sessions were found alive after 76–79 hours
  - Fix: `DEFAULT_PRINT_TIMEOUT_SECS: u32 = 3600` constant; `unwrap_or( DEFAULT_PRINT_TIMEOUT_SECS )`
    in `run_print_mode()` only; `run_interactive()` retains `unwrap_or( 0 )` (user-attended)
  - `--timeout 0` or `CLR_TIMEOUT=0` explicitly restores unlimited behavior
  - Stdin-file pre-check added before the retry loop — missing `--file` path now fast-fails with
    `"Error: cannot open stdin file '<path>': ..."` instead of entering runner retry
  - Tests: 7 new in `timeout_test.rs` + 1 in `env_var_test.rs`; invariant 007 created

- **Account class (`--retry-on-account`) default retry count changed from `auto` (effective 2) to `0`** (TSK-213)
  - Quota resets are measured in hours; 30-second retry delays are counterproductive for
    `QuotaExhausted` errors; retrying immediately after quota exhaustion wastes wall-clock time
  - Account class is now opt-in only: pass `--retry-on-account N` with a long `--account-delay`
    only for batch workflows that may span a billing-period boundary
  - Implementation: `class_default_count(ErrorClass)` returns `Some(0)` for Account, `None`
    for all others; inserted as new tier between class-specific and fallback in `resolve_count()`
  - `--retry-override` (Tier 1) still overrides this default when set
  - Tests: EC-9 (no retry without explicit `--retry-on-account`; Account class_default=0)

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
