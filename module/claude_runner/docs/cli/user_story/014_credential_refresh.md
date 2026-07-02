# Refresh OAuth credentials without running a task

**Persona:** Automation operator who needs to refresh an OAuth token before a batch of Claude operations, without running an actual task.
**Goal:** Refresh the OAuth credentials in a given file — triggering the `claude` binary's startup token refresh — and write the updated token back, with no task execution.
**Benefit:** Enables pre-batch token refresh without side-effecting task execution, ensuring credentials are fresh.
**Priority:** Medium

### Acceptance Criteria

- `clr refresh` (no `--creds`) defaults to `$HOME/.claude/.credentials.json` and refreshes the OAuth token in-place
- `clr refresh --creds <path>` refreshes the OAuth token in the specified file and writes it back in-place
- Exit 0 when credentials were refreshed; exit 1 on error; exit 2 on timeout
- No Claude task is executed — the subprocess receives `["--print", "."]` which returns immediately after the startup token refresh
- Default timeout is 45 seconds (sufficient for slow networks and API rate limits)
- `--trace` shows the underlying `run_isolated()` call details to stderr before execution

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 3 | [`refresh`](../command/04_refresh.md) | OAuth credential refresh without task execution |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | `--creds`, `--timeout`, `--trace` configure refresh |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 13 | [`--trace`](../param/013_trace.md) | Print underlying call details to stderr |
| 19 | [`--creds`](../param/019_creds.md) | Path to credentials JSON file (optional; defaults to `~/.claude/.credentials.json`) |
| 20 | [`--timeout`](../param/020_timeout.md) | Max seconds to wait (default: 45 for refresh) |

### Workflow Steps

1. `clr refresh` — refresh the default credentials file at `~/.claude/.credentials.json`
2. `clr refresh --creds /path/to/creds.json` — refresh an alternate credentials file in-place
3. `clr refresh --creds /path/to/creds.json --timeout 90` — refresh with a custom timeout
4. `clr refresh --trace` — show underlying call details to stderr before refreshing

### Related User Stories

| # | User Story | Relationship |
|---|------------|--------------|
| 10 | [Credential-isolated Execution](010_credential_isolated_execution.md) | `refresh` reuses `run_isolated()` internally; `isolated` runs a full task |
