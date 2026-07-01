# CLI Command: isolated

### Description

Run Claude in a credential-isolated subprocess with a temporary HOME containing only the provided credentials file. Use `clr isolated` when running Claude with alternate accounts, test tokens, or deployment-specific credentials without exposing the caller's real HOME, settings, or session history.

-- **Parameters:** `--creds`, `--timeout`, `--trace`, `--dry-run`, `--dir`, `--add-dir`, `--file`, `--expect`, `--expect-strategy`, `--journal`, `--journal-dir`, `--output-file`, `--strip-fences`, `--output-style`, `--summary-fields`, `--args-file`
-- **Exit Codes:** 0 (success) | 1 (error) | 2 (timeout) | 3 (expect mismatch) | N (subprocess passthrough) | 128+signal (signal)

### Syntax

```sh
clr isolated [--creds <FILE>] [--timeout <SECS>] [OPTIONS] [MESSAGE] [-- PASSTHROUGH...]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`[MESSAGE]`](../param/001_message.md) | [`MessageText`](../type/01_message_text.md) | — | Prompt forwarded to Claude |
| [`--creds`](../param/019_creds.md) | [`CredentialsFilePath`](../type/08_credentials_file_path.md) | `~/.claude/.credentials.json` | Credentials JSON file path (optional; defaults to current account credentials) |
| [`--timeout`](../param/020_timeout.md) | [`TimeoutSecs`](../type/09_timeout_secs.md) | 30 | Max seconds to wait for subprocess |
| [`--trace`](../param/013_trace.md) | bool | false | Print underlying call details to stderr then execute |
| [`--dry-run`](../param/011_dry_run.md) | bool | false | Print subprocess command preview to stdout; exit 0 without spawning |
| [`--dir`](../param/008_dir.md) | path | — | Working directory injected into subprocess command; validated to exist before spawn; env: `CLR_DIR` |
| [`--add-dir`](../param/066_add_dir.md) | path (repeatable) | — | Additional directory Claude may access; injected per entry into subprocess command; env: `CLR_ADD_DIR` |
| [`--file`](../param/025_file.md) | path | — | File piped as stdin to the subprocess; validated to exist before spawn |
| [`--expect`](../param/030_expect.md) | string | — | Pipe-separated expected values; mismatch triggers `--expect-strategy` (case-insensitive, trimmed) |
| [`--expect-strategy`](../param/031_expect_strategy.md) | enum | `fail` | Mismatch strategy: `fail` → exit 3; `default:<V>` → print `<V>`, exit 0; `retry` → exit 1 (unsupported for isolated) |
| [`--journal`](../param/072_journal.md) | enum | `full` | Journal level: `full` (stdout+stderr ≤1MB), `meta` (metadata only), `off` (disabled) |
| [`--journal-dir`](../param/073_journal_dir.md) | path | `~/.clr/journal/` | Directory for journal JSONL files; overrides `CLR_JOURNAL_DIR` |
| [`--output-file`](../param/029_output_file.md) | path | — | Write output to file (also prints to stdout); env: `CLR_OUTPUT_FILE` |
| [`--strip-fences`](../param/026_strip_fences.md) | bool | false | Strip outermost markdown code fences from output; env: `CLR_STRIP_FENCES` |
| [`--output-style`](../param/070_output_style.md) | enum | `raw` | Output rendering: `raw` (default), `summary`; env: `CLR_OUTPUT_STYLE` |
| [`--summary-fields`](../param/071_summary_fields.md) | string | — | Summary field selection: `full`, `standard`, `minimal`, or comma-separated; env: `CLR_SUMMARY_FIELDS` |
| [`--args-file`](../param/075_args_file.md) | [`FilePath`](../type/12_file_path.md) | — | Load clr params from JSON config file; stdin JSON auto-detected when no TTY; env: `CLR_ARGS_FILE` |
| `-h`/`--help` | — | — | Print isolated subcommand help and exit 0 |

**Algorithm (6 steps):**
1. Resolve credentials path: `--creds` if given, else `$HOME/.claude/.credentials.json`; exit 1 if file not found.
2. Create temporary HOME directory; write `.claude/.credentials.json` from resolved credentials.
3. Write minimal `~/.claude/CLAUDE.md` to temp HOME to suppress interactive prompts.
4. Build subprocess command with injected defaults (`--model claude-opus-4-6`, `--effort max`, `--no-session-persistence`, `--dangerously-skip-permissions` when message present); prepend before `--print` and message; passthrough args appended last for last-wins override.
5. Spawn `claude` with `HOME=<temp>`; wait up to `--timeout` seconds (0 = unlimited).
6. If credentials were refreshed at startup, write updated file back to `--creds`; delete temp HOME unconditionally; propagate subprocess exit code (or exit 2 on timeout without refresh).

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Claude exited successfully (may have refreshed creds in-place) |
| 1 | Error (creds file not found, claude not in PATH, I/O failure, unsupported `--expect-strategy retry`) |
| 2 | Timeout — subprocess did not finish within `--timeout` seconds; any partial stdout accumulated before the timeout is preserved in the error output |
| 3 | `--expect` mismatch with `fail` strategy |
| N | Passthrough from claude subprocess (non-zero) |
| 128+signal | POSIX signal termination — subprocess killed by signal (e.g., 130 = SIGINT, 143 = SIGTERM); passes through from subprocess identically to any other non-zero `N` |

### Examples

```sh
# Quick prompt with isolated credentials
clr isolated --creds ~/.claude/.credentials.json "What is 2+2?"

# Custom timeout for long-running tasks
clr isolated --creds /path/to/creds.json --timeout 120 "Refactor this module"

# Verify credentials work (--version exits fast)
clr isolated --creds /path/to/creds.json -- --version

# Interactive isolated session (no message — REPL mode)
clr isolated --creds /path/to/creds.json
```

### Notes

The isolated subprocess has no access to the caller's real `$HOME` — no `~/.claude/settings.json`, no previous conversation state. A minimal `~/.claude/CLAUDE.md` is written to the temp HOME before spawn instructing the subprocess to execute immediately without asking clarifying questions or requesting confirmation.

Subprocess injected defaults (see [`invariant/005_isolated_subprocess_defaults.md`](../../invariant/005_isolated_subprocess_defaults.md)):
- `--model claude-opus-4-6` (`ISOLATED_DEFAULT_MODEL` — maximum capability for real tasks)
- `--effort max` (maximum reasoning effort)
- `--no-session-persistence` (temp HOME is discarded after every run; session writes are waste)
- `--dangerously-skip-permissions` — injected when `[MESSAGE]` is present; omitted in interactive mode (no message)
- `--chrome` active (ClaudeCommand default; isolated tasks may use browser tools)

Injected flags are prepended before `--print` and message so passthrough args override via last-wins:

```sh
# Override effort for a lighter task:
clr isolated "summarize this file" -- --effort medium
# Opt out of skip-permissions for a read-only task:
clr isolated "what is 2+2?" -- --no-skip-permissions
```

If the subprocess times out but already wrote refreshed credentials, `clr isolated` exits 0 and writes updated credentials back to `--creds` instead of returning exit 2. This matches the `IsolatedRunResult { exit_code: -1, credentials: Some(…) }` path in `claude_runner_core::run_isolated()`.

`--timeout 0` disables the watchdog entirely (unlimited runtime), matching `run`/`ask` semantics.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`refresh`](03_refresh.md) | Both use `run_isolated()`; `refresh` sends a trivial ping to trigger token refresh only |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |

---

**Category:** Credential management
**Complexity:** 15
**API Requirement:** Write
**Idempotent:** No
**Risk Level:** Low
