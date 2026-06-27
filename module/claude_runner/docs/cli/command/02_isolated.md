# CLI Command: isolated

Run Claude in a credential-isolated subprocess. Creates a temporary `HOME`
directory containing only `.claude/.credentials.json` populated from
`--creds`, then spawns Claude with `HOME=<temp>`. Waits at most `--timeout`
seconds, then deletes the temp HOME unconditionally. If Claude refreshes its
OAuth token, the updated credentials are written back to `--creds` in-place.

**Syntax:**

```sh
clr isolated [--creds <FILE>] [--timeout <SECS>] [MESSAGE]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`[MESSAGE]`](../param/001_message.md) | [`MessageText`](../type/01_message_text.md) | — | Prompt forwarded to Claude |
| [`--creds`](../param/019_creds.md) | [`CredentialsFilePath`](../type/08_credentials_file_path.md) | `~/.claude/.credentials.json` | Credentials JSON file path (optional; defaults to current account credentials) |
| [`--timeout`](../param/020_timeout.md) | [`TimeoutSecs`](../type/09_timeout_secs.md) | 30 | Max seconds to wait for subprocess |
| [`--trace`](../param/013_trace.md) | bool | false | Print underlying call details to stderr then execute |
| [`--journal`](../param/072_journal.md) | enum | `full` | Journal level: `full` (stdout+stderr ≤1MB), `meta` (metadata only), `off` (disabled) |
| [`--journal-dir`](../param/073_journal_dir.md) | path | `~/.clr/journal/` | Directory for journal JSONL files; overrides `CLR_JOURNAL_DIR` |
| `-h`/`--help` | — | — | Print isolated subcommand help and exit 0 |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Claude exited successfully (may have refreshed creds in-place) |
| 1 | Error (creds file not found, claude not in PATH, I/O failure) |
| 2 | Timeout — subprocess did not finish within `--timeout` seconds; any partial stdout accumulated before the timeout is preserved in the error output |
| N | Passthrough from claude subprocess (non-zero) |
| 128+signal | POSIX signal termination — subprocess killed by signal (e.g., 130 = SIGINT, 143 = SIGTERM); passes through from subprocess identically to any other non-zero `N` |

**Examples:**

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

**Notes:**

The isolated subprocess has no access to the caller's real `$HOME` — no
`~/.claude/settings.json`, no previous conversation state. A minimal
`~/.claude/CLAUDE.md` is written to the temp HOME before spawn instructing
the subprocess to execute immediately without asking clarifying questions or
requesting confirmation.

The subprocess is invoked with the following injected defaults (see
[`invariant/005_isolated_subprocess_defaults.md`](../../invariant/005_isolated_subprocess_defaults.md)):

- `--model claude-opus-4-6` (`ISOLATED_DEFAULT_MODEL` — maximum capability for real tasks)
- `--effort max` (maximum reasoning effort)
- `--no-session-persistence` (temp HOME is discarded after every run; session writes are waste)
- `--dangerously-skip-permissions` — injected when `[MESSAGE]` is present (tool calls must
  not block interactively); omitted in interactive mode (no message)
- `--chrome` active (ClaudeCommand default; isolated tasks may use browser tools)

Injected flags are prepended before `--print` and message so passthrough args can
override them via last-wins semantics:

```sh
# Override effort for a lighter task:
clr isolated "summarize this file" -- --effort medium
# Opt out of skip-permissions for a read-only task:
clr isolated "what is 2+2?" -- --no-skip-permissions
```

If the subprocess times out but already wrote refreshed credentials (OAuth
token refresh at startup before blocking on input), `clr isolated` exits 0
and writes updated credentials back to `--creds` instead of returning exit 2.
This matches the `IsolatedRunResult { exit_code: -1, credentials: Some(…) }`
path in `claude_runner_core::run_isolated()`.

`--timeout 0` disables the watchdog entirely (unlimited runtime), matching
`run`/`ask` semantics.

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 10 | [010_credential_isolated_execution.md](../user_story/010_credential_isolated_execution.md) | Developer |
