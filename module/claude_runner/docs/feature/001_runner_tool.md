# Feature: Runner Tool

### Scope

- **Purpose**: Document the clr CLI tool design including execution modes, default flag injection, and the YAML library surface.
- **Responsibility**: Describe the two roles of claude_runner (CLI binary and YAML library), invocation modes, and flag behavior.
- **In Scope**: clr execution modes, automatic `-c`, `--dangerously-skip-permissions`, `--chrome` injection, `"\n\nultrathink"` message suffix default, YAML library role, mode selection logic.
- **Out of Scope**: Dependency constraints (→ `invariant/002_dep_constraints.md`), public API contracts (→ `api/001_public_api.md`).

### Design

claude_runner serves two distinct consumers from one crate:

**YAML library consumer:** The library surface exposes `COMMANDS_YAML` — an absolute path (computed at compile time from the crate manifest directory) to `claude.commands.yaml`. Consumers such as `dream` aggregate this YAML at compile time via a build script to build a PHF static command registry for `.claude` and `.claude.help` commands. The library has zero consumer workspace dependencies.

**CLI binary (`clr`):** The `clr` binary translates `--flag value` syntax to `ClaudeCommand` builder calls and executes Claude Code via `claude_runner_core`. It acts as the user-facing runner for both interactive and non-interactive use.

**Execution modes:** See [command/](../cli/command/readme.md) for the full invocation mode table.

**Default flag injection:** See [invariant/001_default_flags.md](../invariant/001_default_flags.md) for the complete default injection rules and opt-out mechanisms.

**Quiet gate:** The `--quiet` bool flag (default false; env: `CLR_QUIET`) suppresses non-fatal CLR runner diagnostics: gate-wait messages, retry progress, retry-exhaustion messages, and the keep-claudecode nested-agent warning. Fatal errors (spawn failures, binary not found) are always emitted to stderr regardless of `--quiet`. When the `claude` binary is not found, the message is: `claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code`. `--dry-run` and `--trace` output are similarly unaffected by `--quiet`.

**Trace mode:** `--trace` prints environment variables and the full command to stderr (like `set -x`), then executes normally. This is independent of `--quiet`.

**Stdin file piping:** `--file <PATH>` opens the given file and pipes its content as standard input to the `claude` subprocess. Equivalent to `cat file | clr -p "..."` but without a shell pipeline. The file is opened at spawn time; a non-readable path causes `clr` to exit with an error. Applies to `run`, `ask`, and `isolated` subcommands (isolated uses a separate `run_isolated_with_stdin_file()` code path in `credential.rs`).

**Output fence stripping:** `--strip-fences` post-processes captured stdout after the subprocess exits: the first and last markdown code fence lines (`` ``` `` with optional language tag) are removed and the content between them is emitted unchanged. If no fence pair is found, stdout passes through unmodified.

**CLAUDECODE removal:** `clr` removes the `CLAUDECODE` environment variable from the subprocess environment before spawning (default-on). This prevents the subprocess from detecting a parent Claude Code session, which would alter its behaviour. Use `--keep-claudecode` to opt out and preserve `CLAUDECODE` in the subprocess environment.

**Output file capture:** `--output-file <PATH>` captures subprocess stdout to a file in addition to
printing it to stdout (tee behavior). The runner captures output first, then writes to the file
and prints. Write errors (permission denied, directory absent) cause `clr` to exit 1 with the OS
error on stderr. In dry-run mode the file is not created. `--output-file` is orthogonal to
`--file` — `--file` feeds input to the subprocess; `--output-file` captures subprocess output.

**Print-mode output rendering:** `run_print_mode()` renders output through `summary.rs` by default — `--output-style summary` is the default; `--output-style raw` bypasses `render_summary()` and returns raw claude output unchanged. When `--output-style summary` is active and `--output-format` is absent, `clr` automatically injects `--output-format json` into the subprocess command so `render_summary()` receives parseable input. If rendering fails (e.g. when `--output-format text` was explicitly set), `render_summary()` returns `None` and `clr` falls back to raw output unchanged. `CLR_OUTPUT_STYLE` env var accepted; CLI flag wins. Invalid values (`--output-style invalid` or `CLR_OUTPUT_STYLE=bogus`) exit 1 immediately.

**Enum output validation:** `--expect "val1|val2|..."` validates captured stdout against a
pipe-separated list of expected values (case-insensitive, whitespace-trimmed). The
`--expect-strategy` parameter controls mismatch handling: `fail` (exit 3, default), `retry`
(re-invoke up to `--retry-on-validation` times then exit 3), or `default:<VALUE>` (output fallback
and exit 0). Exit code 3 is exclusive to `--expect` mismatch; it does not overlap with
subprocess exit codes. Both parameters are silently ignored in interactive mode. Also supported by
`isolated` — with the restriction that `retry` strategy is unsupported (exits 1 with error:
one-shot semantics have no retry loop); `fail` and `default:<VALUE>` work identically.

**Session concurrency gate:** `--max-sessions <N>` (default 30, 0 = unlimited) counts active
`claude` processes via `/proc` scan before spawning a subprocess. When the live count is at or
above the limit, the runner emits a waiting message to stderr and polls every 30 seconds until a
slot opens or the 100-attempt limit is exhausted (fatal exit 1 on exhaustion). Setting `--max-sessions 0`
disables the gate entirely — the scan is skipped and the subprocess is launched immediately.
The gate is also skipped in `--dry-run` mode. Provides deterministic backpressure in CI
environments with parallel `clr` invocations hitting API rate limits. The gate uses
`claude_core::process::find_claude_processes()` (Linux `/proc` scanner) for the session count.

**Session listing (`clr ps`):** Prints two plain-style tables: active Claude Code sessions
and queued `clr` waiters (processes blocked at the concurrency gate). Default active table
columns: `#`, `PID`, `Elapsed`, `CPU%`, `RAM`, `State`, `Mode`, `Absolute Path`, `Task`;
optional columns: `Command`, `Binary` (shown via `--wide` or `--columns`). `--mode` filters
rows by execution mode (interactive/print). `--columns` selects a custom column subset.
`--pid <PIDs>` restricts the active table to comma-separated PIDs (AND with `--mode`).
`-i`/`--inspect` switches to 12-attribute key:value record blocks per session (suppresses
queued table, ignores `--columns`/`--wide`). Each row can display emoji session flags:
🐳 (container), 🕰 (ancient), 🐘 (high RAM), ⚠ (dead metrics), ⚡ (active), 🖨 (print mode),
👈 (this session); thresholds configurable via `CLR_PS_ANCIENT_SECS` (default 28800) and
`CLR_PS_HIGH_RAM_MB` (default 400). Env vars: `CLR_PS_MODE`, `CLR_PS_COLUMNS`, `CLR_PS_PID`,
`CLR_PS_ANCIENT_SECS`, `CLR_PS_HIGH_RAM_MB`. Data sources: `/proc/{pid}/stat` (state, CPU
jiffies, start time), `/proc/{pid}/status` (VmRSS in MB), `~/.claude/projects/` JSONL files
(Task column — last user message, truncated to 35 chars); falls back to `"interactive"` when
no JSONL found. Active session rows are ordered by start time (oldest first). Queued table
reads gate state files from `$CLR_GATE_DIR` — columns: `#`, `PID`, `CWD`, `Waiting`,
`Attempt`; gate files whose PID no longer exists are filtered out and self-heal-deleted
(BUG-293). The current `clr ps` process is never listed. When no sessions are found: prints
`No active Claude Code sessions.` and exits 0. Linux-only (`#[cfg(target_os = "linux")]`).

**Session termination (`clr kill <pid>`):** Sends SIGTERM to a running Claude Code session
identified by PID. Validates the PID belongs to a running `claude` process via
`find_claude_processes()`. Exits 0 on success, 1 if PID not found or not a `claude` process.
Typo guard: `clr kil` and `clr killl` trigger "Did you mean 'kill'?" and exit 1.

**Tool listing (`clr tools`):** Lists all 26 Claude Code built-in tools in a plain-style
table with Name, Category, and Description columns. Static data sourced from
`contract/claude_code/docs/tool/readme.md`. Unknown arguments exit 1.

**3-tier retry hierarchy:** When a subprocess exits with a classifiable error, the runner retries according to a 3-tier resolution: Tier 1 (`--retry-override`) forces count/delay for all classes; Tier 2 (`--retry-on-<class>`/`--<class>-delay`) overrides per error class; Tier 3 (`--retry-default`/`--retry-default-delay`) provides fallback defaults (count=2, delay=30s). All 8 error classes use the same 3-tier resolution with no class-level default overrides. The 8 error classes are Transient, Account, Auth, Service, Process, Validation, Runner, and Unknown. Each class maps from `ErrorKind` via `classify_to_class()` in `execution.rs`. Stderr error labels use `[Class]` prefix for traceability (e.g., `"[Account] You've hit your limit · resets 2:40pm — retrying in 30s (attempt 1/3)…"`). Validation class retries are only active when `--expect-strategy retry` is set. Parameters 040–057 cover all 20 retry flags; each has a `CLR_*` env var fallback.

**Error output in summary mode:** In summary mode (default), `first_message()` in `execution.rs` extracts the `"result"` field from the JSON envelope when stdout is JSON — yielding a human-readable single line (e.g., `"You've hit your limit · resets 2:40pm"`) rather than the full raw JSON blob. On error exhaustion, captured stdout is rendered through `render_summary()` before emit to stderr — the same formatted key:val output produced on the success path. In raw mode (`--output-style raw`), both paths emit stdout unmodified.

**Journaling:** `--journal <level>` (default: `full`) enables automatic event recording via `claude_journal::JournalWriter`. At `full` level all event fields are recorded including complete stdout and stderr (each truncated at 1 MB); at `meta` only metadata (timestamp, command, exit code, duration, cost, model) is recorded; at `off` journaling is disabled. `--journal-dir <path>` overrides the default `~/.clr/journal/` directory. Events are emitted at eight execution boundaries: subprocess exit (`execution`), credential ops completion (`credential`), concurrency gate block (`gate_wait`), rate-limit retry (`retry`), watchdog timeout (`timeout`), runner retry (`runner_retry`), expect-validation retry (`validation_retry`), and interactive session start/end (`interactive`). Journal write failures are logged to stderr unless `--quiet` but never alter `clr`'s exit code — journaling is best-effort. Applies to all four executing subcommands: `run`, `ask`, `isolated`, `refresh`. See [feature/002_journaling_integration.md](002_journaling_integration.md).

**Isolated subcommand extended params (Plan 034):** `clr isolated` gained six params that mirror existing `run`/`ask` capabilities: `--dry-run` (print injected temp-HOME command without creating a directory or spawning; exit 0), `--dir <PATH>` (working directory injected into subprocess; validated before spawn; `CLR_DIR` env fallback), `--add-dir <PATH>` (repeatable; additional read paths; `CLR_ADD_DIR` env fallback), `--file <PATH>` (pipe file content as stdin; pre-spawn existence check; exit 1 if missing), `--expect <VALS>` (pipe-separated match patterns; case-insensitive trimmed), `--expect-strategy <STRAT>` (`fail`→exit 3, `default:<V>`→exit 0; `retry` unsupported → exit 1). `isolated` param count went from 6 to 12; it now shares 9 params with `run`/`ask` (was 3). Exit code 3 applies to `isolated` on `--expect` mismatch with `fail` strategy.

**Session mismatch detection (BUG-320 hardening):** When `-c` is injected, `session_exists()` captures the UUID of the most-recently-modified `.jsonl` file in the session storage directory as `expected_session_id: Option<SessionId>`. On the success path in `run_print_mode()`, `extract_session_id()` in `summary.rs` parses the `session_id` field from claude's JSON result envelope. If `expected_session_id` is `Some` and the actual UUID differs, a `[Runner] warning: session mismatch` line is emitted to stderr. The run is not failed — the warning is diagnostic only. See [invariant/009_session_mismatch_detection.md](../invariant/009_session_mismatch_detection.md).

**Separation of concerns:** `clr` owns CLI flag translation and automation defaults only. Process execution is delegated to `claude_runner_core`. Session storage paths come from `claude_profile` (via `--session-dir` flag passthrough or resolved externally).

### APIs

| File | Relationship |
|------|--------------|
| [api/001_public_api.md](../api/001_public_api.md) | COMMANDS_YAML and register_commands public API |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_default_flags.md](../invariant/001_default_flags.md) | Default flag injection rules and opt-out mechanism |
| [invariant/002_dep_constraints.md](../invariant/002_dep_constraints.md) | Zero consumer workspace deps, binary deps gated by enabled |
| [invariant/003_command_naming.md](../invariant/003_command_naming.md) | Command naming convention (clr / clr run / clr ask) |
| [invariant/004_trace_universality.md](../invariant/004_trace_universality.md) | --trace applies to all executing subcommands |
| [invariant/005_isolated_subprocess_defaults.md](../invariant/005_isolated_subprocess_defaults.md) | Isolated subprocess model, effort, and flag defaults |
| [invariant/006_exit_codes.md](../invariant/006_exit_codes.md) | Exit code semantics (0-4, 128+signal) |
| [invariant/007_print_mode_timeout.md](../invariant/007_print_mode_timeout.md) | Print-mode default timeout (3600s) vs interactive (unlimited) |
| [invariant/008_render_summary_gate.md](../invariant/008_render_summary_gate.md) | `render_summary()` must gate on `"type":"result"`; optional fields use `.unwrap_or_default()` |
| [invariant/009_session_mismatch_detection.md](../invariant/009_session_mismatch_detection.md) | Diagnostic warning when `-c` resumes a different session than expected |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/lib.rs` | `run_cli()` entry point |
| `../../src/cli/mod.rs` | Subcommand dispatch, dry-run, guard |
| `../../src/cli/execution.rs` | `run_print_mode`, `run_interactive`, `first_message` (JSON result extraction), timeout watchdog, expect validation |
| `../../src/cli/parse.rs` | CLI argument parsing (`parse_args`, `CliArgs`, `ExpectStrategy`) |
| `../../src/cli/env.rs` | `apply_env_vars` — CLR_* env-variable fallbacks for all run params |
| `../../src/cli/help.rs` | Help text printing for all subcommands (`clr`, `ask`, `isolated`, `refresh`) |
| `../../src/cli/gate.rs` | Session concurrency gate (`wait_for_session_slot`; delegates process scan to `claude_core`) |
| `../../src/cli/ps.rs` | Session listing (`dispatch_ps`; reads `/proc` metrics, formats plain-style tables) |
| `../../src/cli/kill.rs` | Session termination (`dispatch_kill`; validates PID, sends SIGTERM) |
| `../../src/cli/builder.rs` | `build_claude_command()` implementation, `session_exists()` guard, effective-dir resolution |
| `../../src/cli/credential.rs` | Credential-isolated execution (`run_isolated_command`, `run_refresh_command`), trace emission for isolated/refresh |
| `../../src/cli/cred_parse.rs` | `IsolatedArgs`, `RefreshArgs`, their parsers and env-var fallbacks |
| `../../src/cli/fence.rs` | `strip_fences` utility — outermost code-fence stripping for `--strip-fences` |
| `../../src/cli/tools.rs` | `clr tools` — list Claude Code built-in tools in a plain-style table |
| `../../src/cli/summary.rs` | `render_summary()` — JSON→key:val summary; called in `run_print_mode()` when `--output-style summary` (default); falls back to raw on non-JSON input; `extract_structured_output()` — extracts `structured_output` field from CLR envelope for raw mode + `--json-schema` (BUG-318 fix) |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/cli_args_test.rs` | T01–T35 flag parsing; --interactive, --print mode dispatch coverage |
| `../../tests/cli_args_ext_test.rs` | T36–T49, S58–S79 extended flags; BUG-212, BUG-215 reproducers |
| `../../tests/dry_run_test.rs` | Validates dry-run preview output including all injected flags |
| `../../tests/execution_mode_test.rs` | E01–E13 live mode dispatch via fake claude binary |
| `../../tests/isolated_test.rs` | `clr isolated` parsing, error cases, lim_it live runs, unknown-subcommand detection; Plan 034: `--dry-run` (IT-12–15), `--dir`/`--add-dir` (IT-16–20), `--file` (IT-21–23), `--expect`/`--expect-strategy` (IT-24–27), pipe buffering (IT-28); Plan 035: `--output-file` (IT-29), `--strip-fences` (IT-30), `--output-style` (IT-31), `--summary-fields` (IT-32), env fallbacks (IT-33–36), journal env validation (IT-37) |
| `../../tests/output_file_test.rs` | T01–T06 --output-file tee behavior, write errors, dry-run skip |
| `../../tests/expect_validation_test.rs` | T01–T17 --expect / --expect-strategy / --retry-on-validation validation loop |
| `../../tests/bug_reproducers_247_test.rs` | BUG-247 stdout-to-stderr forwarding on subprocess failure |
| `../../tests/bug_reproducers_248_test.rs` | BUG-248 --keep-claudecode warning when CLAUDECODE present |
| `../../tests/retry_transient_test.rs` | Transient class retry count, delay, [Class] prefix, old-flag rejection |
| `../../tests/retry_account_test.rs` | Account class retry count, delay, env fallback |
| `../../tests/retry_auth_test.rs` | Auth class retry count, delay, env fallback |
| `../../tests/retry_service_test.rs` | Service class retry count, delay, old-flag rejection |
| `../../tests/retry_process_test.rs` | Process class retry count, delay |
| `../../tests/retry_validation_test.rs` | Validation class retry count, old-flag rejection |
| `../../tests/retry_runner_test.rs` | Runner class retry count, delay |
| `../../tests/retry_unknown_test.rs` | Unknown class retry count, delay, old-flag rejection |
| `../../tests/retry_override_test.rs` | Tier 1 override count/delay, tier priority tests |
| `../../tests/retry_default_test.rs` | Tier 3 fallback count/delay, effective defaults |
| `../../tests/error_classification_test.rs` | ErrorKind → ErrorClass mapping, [Class] stderr prefix |
| `../../tests/exit_code_contract_test.rs` | Exit code 4 for timeout (TSK-202) |
| `../../tests/ps_command_test.rs` | IT-01–IT-29 clr ps tables, gate file rendering, sort order, mode filter, column select, BUG-293/294/295/296/297/301 |
| `../../tests/user_story_ps_test.rs` | US-01–US-17 session listing acceptance criteria |
| `../../tests/kill_command_test.rs` | IT-01–IT-09 clr kill SIGTERM delivery and guards |
| `../../tests/isolated_defaults_test.rs` | ISD-01–ISD-13 isolated subprocess model, effort, flags |
| `../../tests/isolated_correctness_test.rs` | CT-1–CT-6 isolated correctness invariants |
| `../../tests/timeout_test.rs` | Default timeout constant, watchdog activation, unlimited flag/env, default-path kill via `_CLR_DEFAULT_TIMEOUT` (TSK-228); BUG-317 double-emission guard (`ec_timeout_retry_no_double_emission`) |
| `../../tests/tools_command_test.rs` | IT-01–IT-09 clr tools table output, help, unknown args |
| `../../tests/output_format_test.rs` | --output-format summary rendering and fallback |
| `../../tests/output_style_test.rs` | EC-01–EC-15 --output-style summary/raw rendering, CLR_OUTPUT_STYLE env var, legacy alias, graceful fallback, minimal CLR envelope (BUG-310 regression), raw+json-schema structured output (BUG-318 fix) |
| `../../tests/summary_fields_test.rs` | EC-01–EC-12 --summary-fields profile/custom/env field selection |
| `../../tests/ask_command_test.rs` | clr ask dispatch, help intercept, BUG-249/250 |
| `../../tests/env_var_test.rs` | E01–E17 CLR_* env-variable fallback for run params |
| `../../tests/env_var_ext_test.rs` | E18–E34 extended env-variable fallback (output-file, expect, retry) |
| `../../tests/quiet_test.rs` | Quiet gate: non-fatal diagnostic suppression, fatal error passthrough |
| `../../tests/param_group_test.rs` | G1–G5 param group composition and count contracts |
| `../../tests/param_edge_cases_test.rs` | Edge cases for flag parsing (empty values, duplicates, ordering) |
| `../../tests/param_extended_flags_test.rs` | Extended flag parsing for retry, gate, output params |
| `../../tests/param_trace_edge_cases_test.rs` | --trace edge cases (combined with other flags) |
| `../../tests/ultrathink_args_test.rs` | --ultrathink / --no-ultrathink suffix injection |
| `../../tests/effort_args_test.rs` | --effort level flag parsing and forwarding |
| `../../tests/fence_test.rs` | strip_fences utility — fence pair detection and stripping |
| `../../tests/add_dir_test.rs` | --add-dir flag parsing and forwarding |
| `../../tests/allowed_tools_test.rs` | --allowed-tools flag parsing and forwarding |
| `../../tests/disallowed_tools_test.rs` | --disallowed-tools flag parsing and forwarding |
| `../../tests/max_turns_test.rs` | --max-turns flag parsing and forwarding |
| `../../tests/max_budget_usd_test.rs` | --max-budget-usd flag parsing and forwarding |
| `../../tests/fallback_model_test.rs` | --fallback-model flag parsing and forwarding |
| `../../tests/stale_ref_guard_test.rs` | Guard against stale cross-module references |
| `../../tests/refresh_test.rs` | clr refresh command execution and trace output |
| `../../tests/creds_default_test.rs` | Credential command default model/effort values |
| `../../tests/invariant_trace_universality_test.rs` | IT-01–IT-02 --trace universality across all subcommands |
| `../../tests/commands_yaml_test.rs` | COMMANDS_YAML path validity and content checks |
| `../../tests/lib_test.rs` | Library surface smoke tests |
| `../../tests/user_story_test.rs` | US-01–US-09 core user stories (run, ask, flags) |
| `../../tests/user_story_creds_isolated_test.rs` | US-10–US-18 credential-isolated user stories |
| `../../tests/user_story_output_test.rs` | US-19–US-25 output capture and validation user stories |
| `../../tests/user_story_kill_test.rs` | US-01–US-06 clr kill acceptance criteria |
| `../../tests/bug_reproducers_239_244_test.rs` | BUG-239–BUG-244 signal handling and spawn failure reproducers |
| `../../tests/bug_reproducers_246_test.rs` | BUG-246 reproducer |
| `../../tests/ps_pid_test.rs` | EC-1–EC-8 --pid filter for clr ps |
| `../../tests/ps_inspect_test.rs` | EC-1–EC-9 --inspect key:value block output |
| `../../tests/ps_mode_test.rs` | EC-1–EC-8 --mode filter for clr ps |
| `../../tests/ps_columns_test.rs` | EC-1–EC-10 --columns custom column selection, BUG-303 |
| `../../tests/ps_wide_test.rs` | EC-1–EC-5 --wide optional column display |
| `../../tests/ps_flags_test.rs` | IT-30–IT-40, US-18–US-26, E41–E42 session flag emoji computation |
| `../../tests/journal_integration_test.rs` | EC-1–EC-15 --journal/--journal-dir event emission, level filtering, retry/timeout/gate_wait/validation_retry events, truncation, CLI-wins precedence |
| `../../tests/session_verification_test.rs` | SV-1–SV-4 session mismatch detection (BUG-320 hardening): match/mismatch/no-session/raw-output |
| `../../tests/summary_unit_test.rs` | IT-1–IT-7 render_summary unit tests; extract_session_id unit tests (invariant/008, invariant/009) |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Purpose, Architecture, Modes, Default Flags Principle, CLI Flags, Separation of Concerns |
