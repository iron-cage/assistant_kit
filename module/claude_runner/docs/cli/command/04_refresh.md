# CLI Command: refresh

### Description

Refresh OAuth credentials without running an actual Claude task by spawning `claude --print "."` in a temporary isolated HOME and writing the updated token back to `--creds` in-place. Use `clr refresh` to pre-warm tokens before a batch of operations without any task side effects.

-- **Parameters:** `--creds`, `--timeout`, `--trace`, `--dry-run`, `--no-compact-window`, `--journal`, `--journal-dir`, `--args-file`
-- **Exit Codes:** 0 (refreshed) | 1 (error) | 2 (timeout, no refresh)

### Syntax

```sh
clr refresh [--creds <FILE>] [--timeout <SECS>] [--trace] [--dry-run] [--no-compact-window]
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`--creds`](../param/019_creds.md) | [`CredentialsFilePath`](../type/08_credentials_file_path.md) | `~/.claude/.credentials.json` | Credentials JSON file path (optional; defaults to current account credentials) |
| [`--timeout`](../param/020_timeout.md) | [`TimeoutSecs`](../type/09_timeout_secs.md) | 45 | Max seconds to wait for refresh |
| [`--trace`](../param/013_trace.md) | bool | false | Print underlying call details to stderr then execute |
| [`--dry-run`](../param/011_dry_run.md) | bool | false | Print subprocess env+command to stderr (same path as `--trace`); exit 0 without spawning |
| [`--no-compact-window`](../param/075_no_compact_window.md) | bool | false | Suppress `CLAUDE_CODE_AUTO_COMPACT_WINDOW=200000` injection; env: `CLR_NO_COMPACT_WINDOW` |
| [`--journal`](../param/072_journal.md) | enum | `full` | Journal level: `full` (stdout+stderr ≤1MB), `meta` (metadata only), `off` (disabled) |
| [`--journal-dir`](../param/073_journal_dir.md) | path | `~/.clr/journal/` | Directory for journal JSONL files; overrides `CLR_JOURNAL_DIR` |
| [`--args-file`](../param/075_args_file.md) | [`FilePath`](../type/12_file_path.md) | — | Load clr params from JSON config file; stdin JSON auto-detected when no TTY; env: `CLR_ARGS_FILE` |
| `-h`/`--help` | — | — | Print refresh subcommand help and exit 0 |

**Algorithm (5 steps):**
1. Resolve credentials path: `--creds` if given, else `$HOME/.claude/.credentials.json`; exit 1 if file not found.
2. Create temporary HOME directory; write `.claude/.credentials.json` from resolved path.
3. Write minimal `~/.claude/CLAUDE.md` to temp HOME to suppress interactive prompts.
4. Spawn `claude --print "."` with `HOME=<temp>`, `--model "sonnet"` alias, `--effort low`, `--no-session-persistence`, `--no-chrome`; wait up to `--timeout` seconds.
5. If credentials were refreshed at subprocess startup, write updated file back to `--creds` and exit 0; otherwise exit 1 (no refresh) or 2 (timeout); delete temp HOME unconditionally.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Credentials were refreshed and written back to `--creds` |
| 1 | Error (creds file not found, claude not in PATH, I/O failure, no refresh occurred) |
| 2 | Timeout — subprocess did not finish within `--timeout` seconds and no refresh occurred |

### Examples

```sh
# Refresh current account credentials (no --creds needed)
clr refresh

# Refresh specific credentials file with default 45s timeout
clr refresh --creds ~/.claude/.credentials.json

# Refresh with custom timeout for slow networks
clr refresh --creds /path/to/creds.json --timeout 90

# Trace the underlying call to see what happens
clr refresh --trace
```

### Notes

Internally calls `run_isolated_ext()` with fixed args. The `claude` binary refreshes its OAuth token at startup before processing the trivial `.` prompt, then exits. If the token was refreshed, `clr refresh` writes the updated credentials back to `--creds` and exits 0.

The default timeout of 45 seconds (vs 30 for `isolated`) allows headroom for slow networks and API rate limiting during the OAuth token exchange. `--timeout 0` disables the watchdog entirely (unlimited runtime).

Subprocess injected defaults (see [`invariant/005_isolated_subprocess_defaults.md`](../../invariant/005_isolated_subprocess_defaults.md)):
- `--model "claude-sonnet-5"` (`REFRESH_DEFAULT_MODEL` — Sonnet is sufficient for a trivial ping)
- `--effort low` (minimal reasoning for a one-character OAuth-trigger prompt)
- `--no-session-persistence` (temp HOME is discarded after run; session writes are waste)
- `--no-chrome` (OAuth token exchange is pure HTTP; browser context adds overhead with no benefit)
- No `--dangerously-skip-permissions` (refresh invokes no tools; no permission prompts)
- CLAUDE.md written to temp HOME (same as isolated; suppresses interactive prompts)

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`isolated`](03_isolated.md) | Both use `run_isolated_ext()`; `refresh` sends a trivial ping instead of a real task |

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | — |
| 6 | [Running Commands](../param_group/06_running_commands.md) | Subset — `--timeout`, `--trace`, `--dry-run`, `--no-compact-window`, `--journal`, `--journal-dir` | `--creds` is Group 4 |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 8 | [008_trace_execution.md](../user_story/008_trace_execution.md) | Developer |
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |

---

**Category:** Credential management
**Complexity:** 5
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
