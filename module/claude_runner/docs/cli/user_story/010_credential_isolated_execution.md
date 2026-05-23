# User Story :: 010. Credential-isolated Execution

### Persona

Developer running Claude with a separate set of credentials — a different account, a test token, or a credentials file for a specific deployment — in a fully isolated environment.

### Goal

Execute a Claude task using a specified credentials file with no access to the caller's real HOME, settings, or prior conversation history.

### Acceptance Criteria

- `clr isolated --creds <path>` runs the subprocess with a temporary HOME containing only the provided credentials file
- Subprocess has no access to the caller's real HOME, settings, or session history
- OAuth tokens refreshed during the run are written back to the `--creds` file before exit
- `--timeout <secs>` sets the max wait time; `--timeout 0` forces the credential-refresh path without a full session
- Temp HOME is deleted unconditionally on exit regardless of timeout or error

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [`isolated`](../command.md#command--2-isolated) | Dedicated subcommand for credential isolation |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`--creds`](../param/19_creds.md) | Path to credentials JSON file (required) |
| 2 | [`--timeout`](../param/20_timeout.md) | Max seconds to wait for isolated subprocess |
