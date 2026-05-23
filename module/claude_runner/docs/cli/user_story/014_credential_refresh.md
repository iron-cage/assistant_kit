# User Story :: 014. Credential Refresh

### Persona

Automation operator who needs to refresh an OAuth token before a batch of Claude operations, without running an actual task.

### Goal

Refresh the OAuth credentials in a given file — triggering the `claude` binary's startup token refresh — and write the updated token back, with no task execution.

### Acceptance Criteria

- `clr refresh --creds <path>` refreshes the OAuth token and writes it back to `--creds`
- Exit 0 when credentials were refreshed; exit 1 on error; exit 2 on timeout
- No Claude task is executed — the subprocess receives `["--print", "."]` which returns immediately after the startup token refresh
- Default timeout is 45 seconds (sufficient for slow networks and API rate limits)
- `--trace` shows the underlying `run_isolated()` call details to stderr before execution

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`refresh`](../001_command.md#command--3-refresh) | Dedicated subcommand for credential refresh |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--creds`](../param/019_creds.md) | Path to credentials JSON file (required) |
| 2 | [`--timeout`](../param/020_timeout.md) | Max seconds to wait (default: 45 for refresh) |
| 3 | [`--trace`](../param/013_trace.md) | Print underlying call details to stderr |

### Related User Stories

| # | User Story | Relationship |
|---|-----------|-------------|
| 1 | [010 Credential-isolated Execution](010_credential_isolated_execution.md) | `refresh` reuses `run_isolated()` internally; `isolated` runs a full task |
