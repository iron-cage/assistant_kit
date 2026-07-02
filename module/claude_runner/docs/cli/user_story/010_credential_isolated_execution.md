# Execute a Claude task in a credential-isolated environment

**Persona:** Developer running Claude with a separate set of credentials — a different account, a test token, or a credentials file for a specific deployment — in a fully isolated environment.
**Goal:** Execute a Claude task using a specified credentials file with no access to the caller's real HOME, settings, or prior conversation history.
**Benefit:** Prevents cross-contamination of credentials and session history when running Claude with alternate accounts.
**Priority:** High

### Acceptance Criteria

- `clr isolated` (no `--creds`) defaults to `$HOME/.claude/.credentials.json` and runs the subprocess in isolation using the current account's credentials
- `clr isolated --creds <path>` runs the subprocess with a temporary HOME containing only the provided credentials file
- Subprocess has no access to the caller's real HOME, settings, or session history
- OAuth tokens refreshed during the run are written back to the resolved credentials file (explicit or default) before exit
- `--timeout <secs>` sets the max wait time; `--timeout 0` forces the credential-refresh path without a full session
- Temp HOME is deleted unconditionally on exit regardless of timeout or error

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 2 | [`isolated`](../command/03_isolated.md) | Credential-isolated subprocess execution |

### Referenced Parameter Groups

| # | Parameter Group | Role |
|---|-----------------|------|
| 4 | [Credential Operations](../param_group/04_credential_operations.md) | `--creds`, `--timeout`, `--trace` configure isolation |

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 13 | [`--trace`](../param/013_trace.md) | Print underlying call details to stderr |
| 19 | [`--creds`](../param/019_creds.md) | Path to credentials JSON file (optional; defaults to `~/.claude/.credentials.json`) |
| 20 | [`--timeout`](../param/020_timeout.md) | Max seconds to wait for isolated subprocess |

### Workflow Steps

1. `clr isolated "Fix the failing test"` — run with current account credentials in isolation
2. `clr isolated --creds /path/to/creds.json "Fix the failing test"` — run with an alternate credentials file
3. `clr isolated --creds /path/to/creds.json --timeout 120 "task"` — set a custom timeout for the isolated run
