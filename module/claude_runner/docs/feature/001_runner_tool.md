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

**Verbosity gate:** The `--verbosity <0-5>` flag (default 3) controls how much runner diagnostic output is emitted. At level 0 all runner diagnostic output is suppressed — except fatal errors (spawn failures, binary not found), which are always emitted to stderr regardless of verbosity level. When the `claude` binary is not found, the message is: `claude binary not found in PATH — install with: npm i -g @anthropic-ai/claude-code`. At level 4 a command preview is printed to stderr before execution. `--dry-run` output is always shown regardless of verbosity level.

**Trace mode:** `--trace` prints environment variables and the full command to stderr (like `set -x`), then executes normally. This is independent of verbosity level.

**Stdin file piping:** `--file <PATH>` opens the given file and pipes its content as standard input to the `claude` subprocess. Equivalent to `cat file | clr -p "..."` but without a shell pipeline. The file is opened at spawn time; a non-readable path causes `clr` to exit with an error.

**Output fence stripping:** `--strip-fences` post-processes captured stdout after the subprocess exits: the first and last markdown code fence lines (`` ``` `` with optional language tag) are removed and the content between them is emitted unchanged. If no fence pair is found, stdout passes through unmodified.

**CLAUDECODE removal:** `clr` removes the `CLAUDECODE` environment variable from the subprocess environment before spawning (default-on). This prevents the subprocess from detecting a parent Claude Code session, which would alter its behaviour. Use `--keep-claudecode` to opt out and preserve `CLAUDECODE` in the subprocess environment.

**Output file capture:** `--output-file <PATH>` captures subprocess stdout to a file in addition to
printing it to stdout (tee behavior). The runner captures output first, then writes to the file
and prints. Write errors (permission denied, directory absent) cause `clr` to exit 1 with the OS
error on stderr. In dry-run mode the file is not created. `--output-file` is orthogonal to
`--file` — `--file` feeds input to the subprocess; `--output-file` captures subprocess output.

**Enum output validation:** `--expect "val1|val2|..."` validates captured stdout against a
pipe-separated list of expected values (case-insensitive, whitespace-trimmed). The
`--expect-strategy` parameter controls mismatch handling: `fail` (exit 3, default), `retry`
(re-invoke up to `--retry-on-validation` times then exit 3), or `default:<VALUE>` (output fallback
and exit 0). Exit code 3 is exclusive to `--expect` mismatch; it does not overlap with
subprocess exit codes. Both parameters are silently ignored in interactive mode.

**Session concurrency gate:** `--max-sessions <N>` (default 30, 0 = unlimited) counts active
`claude` processes via `/proc` scan before spawning a subprocess. When the live count is at or
above the limit, the runner emits a waiting message to stderr and polls every 30 seconds until a
slot opens or the 100-attempt limit is exhausted (fatal exit 1 on exhaustion). Setting `--max-sessions 0`
disables the gate entirely — the scan is skipped and the subprocess is launched immediately.
The gate is also skipped in `--dry-run` mode. Provides deterministic backpressure in CI
environments with parallel `clr` invocations hitting API rate limits. The gate uses
`claude_core::process::find_claude_processes()` (Linux `/proc` scanner) for the session count.

**Session listing (`clr ps`):** Prints two plain-style tables: active Claude Code sessions
and queued `clr` waiters (processes blocked at the concurrency gate). Active table columns:
`#`, `PID`, `Elapsed`, `CPU%`, `RAM`, `State`, `Absolute Path`, `Task`. Data sources:
`/proc/{pid}/stat` (state, CPU jiffies, start time), `/proc/{pid}/status` (VmRSS in MB),
`~/.claude/projects/` JSONL files (Task column — last user message, truncated to 35 chars);
falls back to `"interactive"` when no JSONL found. Queued table reads gate state files from
`$CLR_GATE_DIR` — columns: `#`, `PID`, `CWD`, `Waiting`, `Attempt`. The current `clr ps`
process is never listed. When no sessions are found: prints `No active Claude Code sessions.`
and exits 0. Linux-only (`#[cfg(target_os = "linux")]`).

**Separation of concerns:** `clr` owns CLI flag translation and automation defaults only. Process execution is delegated to `claude_runner_core`. Session storage paths come from `claude_profile` (via `--session-dir` flag passthrough or resolved externally).

### APIs

| File | Relationship |
|------|--------------|
| [api/001_public_api.md](../api/001_public_api.md) | COMMANDS_YAML and VerbosityLevel public API |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_default_flags.md](../invariant/001_default_flags.md) | Default flag injection rules and opt-out mechanism |
| [invariant/002_dep_constraints.md](../invariant/002_dep_constraints.md) | Zero consumer workspace deps, binary deps gated by enabled |
| [invariant/003_command_naming.md](../invariant/003_command_naming.md) | Command naming convention (clr / clr run / clr ask) |
| [invariant/004_trace_universality.md](../invariant/004_trace_universality.md) | --trace applies to all executing subcommands |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/lib.rs` | `run_cli()` entry point |
| `../../src/cli/mod.rs` | Subcommand dispatch, dry-run, guard |
| `../../src/cli/execution.rs` | `run_print_mode`, `run_interactive`, timeout watchdog, expect validation |
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

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/cli_args_test.rs` | T01–T35 flag parsing; --interactive, --print mode dispatch coverage |
| `../../tests/cli_args_ext_test.rs` | T36–T49, S58–S79 extended flags; BUG-212, BUG-215 reproducers |
| `../../tests/dry_run_test.rs` | Validates dry-run preview output including all injected flags |
| `../../tests/execution_mode_test.rs` | E01–E13 live mode dispatch via fake claude binary |
| `../../tests/isolated_test.rs` | Credential-isolated and refresh command execution; trace output for isolated/refresh |
| `../../tests/output_file_test.rs` | T01–T06 --output-file tee behavior, write errors, dry-run skip |
| `../../tests/expect_validation_test.rs` | T01–T17 --expect / --expect-strategy / --retry-on-validation validation loop |
| `../../tests/bug_reproducers_247_test.rs` | BUG-247 stdout-to-stderr forwarding on subprocess failure |
| `../../tests/bug_reproducers_248_test.rs` | BUG-248 --keep-claudecode warning when CLAUDECODE present |

### Provenance

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Purpose, Architecture, Modes, Default Flags Principle, CLI Flags, Separation of Concerns |
