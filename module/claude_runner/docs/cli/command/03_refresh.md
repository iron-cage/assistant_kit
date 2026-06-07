# CLI Command: refresh

Refresh OAuth credentials without running an actual Claude task. Creates a
temporary `HOME` (like `isolated`), spawns `claude --print "."` to trigger the
startup token refresh, then writes the updated credentials back to `--creds`
in-place. No user task is executed — the subprocess returns immediately after
the token refresh completes.

**Syntax:**

```sh
clr refresh [--creds <FILE>] [--timeout <SECS>] [--trace]
```

**Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| [`--creds`](../param/019_creds.md) | [`CredentialsFilePath`](../type/08_credentials_file_path.md) | `~/.claude/.credentials.json` | Credentials JSON file path (optional; defaults to current account credentials) |
| [`--timeout`](../param/020_timeout.md) | [`TimeoutSecs`](../type/09_timeout_secs.md) | 45 | Max seconds to wait for refresh |
| [`--trace`](../param/013_trace.md) | bool | false | Print underlying call details to stderr then execute |
| `-h`/`--help` | — | — | Print refresh subcommand help and exit 0 |

**Exit Codes:**

| Code | Meaning |
|------|---------|
| 0 | Credentials were refreshed and written back to `--creds` |
| 1 | Error (creds file not found, claude not in PATH, I/O failure, no refresh occurred) |
| 2 | Timeout — subprocess did not finish within `--timeout` seconds and no refresh occurred |

**Examples:**

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

**Notes:**

Internally calls `run_isolated()` with fixed args. The `claude` binary refreshes
its OAuth token at startup before processing the trivial `.` prompt, then exits.
If the token was refreshed, `clr refresh` writes the updated credentials back to
`--creds` and exits 0.

The default timeout of 45 seconds (vs 30 for `isolated`) allows headroom for slow
networks and API rate limiting during the OAuth token exchange. `--timeout 0`
disables the watchdog entirely (unlimited runtime).

The subprocess is invoked with the following injected defaults (see
[`invariant/005_isolated_subprocess_defaults.md`](../../invariant/005_isolated_subprocess_defaults.md)):

- `--model claude-sonnet-4-6` (`REFRESH_DEFAULT_MODEL` — Sonnet is sufficient for a trivial ping)
- `--effort low` (minimal reasoning for a one-character OAuth-trigger prompt)
- `--no-session-persistence` (temp HOME is discarded after run; session writes are waste)
- `--no-chrome` (OAuth token exchange is pure HTTP; browser context adds overhead with no benefit)
- No `--dangerously-skip-permissions` (refresh invokes no tools; no permission prompts)
- CLAUDE.md written to temp HOME (same as isolated; suppresses interactive prompts)

### Referenced Parameter Groups

| # | Group | Membership | Excluded Params |
|---|-------|------------|-----------------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | Full | — |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 14 | [014_credential_refresh.md](../user_story/014_credential_refresh.md) | Developer |
