# User Story: Credential Refresh

- **Source:** [docs/cli/user_story/014_credential_refresh.md](../../../../docs/cli/user_story/014_credential_refresh.md)
- **Primary flags:** `--creds`, `--timeout`, `--trace`
- **Command:** `refresh`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `clr refresh --creds` refreshes OAuth token and writes back |
| US-2 | Boundary | Default timeout is 45 seconds for refresh |
| US-3 | Failure path | Non-existent credentials file errors |
| US-4 | Parameter interaction | `--trace` shows underlying `run_isolated()` call details |

---

### US-1: refresh credentials

- **Given:** A valid credentials file exists at a known path
- **When:** `clr refresh --creds /path/to/creds.json --dry-run`
- **Then:** Assembled command shows isolated execution with `["--print", "."]` as subprocess args; no Claude task is executed — only the startup token refresh occurs; updated token would be written back to `--creds` path
- **Exit:** 0

### US-2: default 45-second timeout

- **Given:** A valid credentials file exists
- **When:** `clr refresh --creds /path/to/creds.json --dry-run`
- **Then:** Assembled command uses 45-second default timeout (sufficient for slow networks and API rate limits); timeout is distinct from `isolated` command's default
- **Exit:** 0

### US-3: non-existent credentials file

- **Given:** No file at `/tmp/nonexistent_creds.json`
- **When:** `clr refresh --creds /tmp/nonexistent_creds.json`
- **Then:** Error — credentials file not found or not readable
- **Exit:** non-zero

### US-4: trace shows run_isolated details

- **Given:** A valid credentials file exists
- **When:** `clr refresh --creds /path/to/creds.json --trace --dry-run`
- **Then:** stderr contains the underlying `run_isolated()` call details including credential path and subprocess args; stdout has dry-run output
- **Exit:** 0
