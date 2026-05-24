# User Story: Credential-isolated Execution

- **Source:** [docs/cli/user_story/010_credential_isolated_execution.md](../../../../docs/cli/user_story/010_credential_isolated_execution.md)
- **Primary flags:** `--creds`, `--timeout`
- **Command:** `isolated`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `clr isolated --creds` runs with temp HOME isolation |
| US-2 | Parameter interaction | `--timeout` controls subprocess wait time |
| US-3 | Failure path | `--creds` with non-existent file errors |
| US-4 | Boundary | OAuth tokens written back to credentials file |

---

### US-1: credential isolation with temp HOME

- **Given:** Credentials file exists at a known path
- **When:** `clr isolated --creds /path/to/creds.json --dry-run "test"`
- **Then:** Assembled command shows isolated execution; subprocess would run with temporary HOME containing only the provided credentials; no access to caller's settings or session
- **Exit:** 0

### US-2: timeout controls subprocess duration

- **Given:** Credentials file exists
- **When:** `clr isolated --creds /path/to/creds.json --timeout 120 --dry-run "long task"`
- **Then:** Assembled command includes `--timeout 120`; subprocess would be terminated after 120 seconds if not finished
- **Exit:** 0

### US-3: non-existent credentials file

- **Given:** No file at `/tmp/nonexistent_creds.json`
- **When:** `clr isolated --creds /tmp/nonexistent_creds.json "test"`
- **Then:** Error — credentials file not found or not readable
- **Exit:** non-zero

### US-4: isolated without creds flag

- **Given:** No `--creds` flag provided
- **When:** `clr isolated "test"`
- **Then:** Error — `isolated` command requires `--creds` flag
- **Exit:** non-zero
