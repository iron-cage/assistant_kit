# User Story: CLI Discoverability

- **Source:** [docs/cli/user_story/016_cli_discoverability.md](../../../../docs/cli/user_story/016_cli_discoverability.md)
- **Primary flags:** (none)
- **Command:** `help`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `clr help` prints usage and exits 0 |
| US-2 | Equivalence | `clr -h` and `clr --help` produce identical output to `clr help` |
| US-3 | Coverage | Help output lists all subcommands |
| US-4 | No side effects | No subprocess launched, no credentials required, no session state |

---

### US-1: help prints usage

- **Given:** Claude binary is in PATH
- **When:** `clr help`
- **Then:** Prints usage information to stdout listing commands and flags
- **Exit:** 0

### US-2: flag aliases identical

- **Given:** Claude binary is in PATH
- **When:** `clr -h` and `clr --help`
- **Then:** Output is identical to `clr help`
- **Exit:** 0

### US-3: all subcommands listed

- **Given:** No prior configuration
- **When:** `clr help`
- **Then:** Output lists all 5 subcommands: run, isolated, refresh, ask, help; available flags shown with short descriptions
- **Exit:** 0

### US-4: no side effects

- **Given:** No credentials file, no existing session
- **When:** `clr help`
- **Then:** No Claude subprocess is spawned; no credentials are read or written; no session state is read or written; completes without network access
- **Exit:** 0
