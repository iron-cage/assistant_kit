# Parameter Group :: Isolated Subcommand

Interaction tests for Group 4 (Isolated Subcommand): `--creds`, `--timeout`. Tests validate these flags coexist correctly and configure the credential-isolated subprocess.

**Source:** [param_group.md#group--4-isolated-subcommand](../../../../docs/cli/param_group.md#group--4-isolated-subcommand)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--creds` + `--timeout` both explicit → subprocess launched with both | Combined |
| CC-2 | `--creds` only, `--timeout` omitted → default 30-second deadline | Default |
| CC-3 | `--creds` + `--timeout 0` → immediate expiry, exit 2, write-back triggered | Interaction |
| CC-4 | `--creds` + `-- --version` passthrough → subprocess flag forwarded | Interaction |

## Test Coverage Summary

- Combined: 1 test (CC-1)
- Default: 1 test (CC-2)
- Interaction: 2 tests (CC-3, CC-4)

**Total:** 4 combination cases

## Test Cases
---

### CC-1: `--creds` + `--timeout` both explicit

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json --timeout 60 "What is 2+2?"`
- **Then:** isolated subprocess launched with 60-second deadline; temp HOME contains `.claude/.credentials.json`; response captured to stdout
- **Exit:** 0
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-isolated-subcommand)
---

### CC-2: `--timeout` omitted → default 30 applied

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json "What is 2+2?"`
- **Then:** isolated subprocess launched with 30-second default deadline; behavior identical to `--timeout 30`
- **Exit:** 0
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-isolated-subcommand)
---

### CC-3: `--timeout 0` → immediate expiry

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json --timeout 0 "dummy"`
- **Then:** subprocess killed immediately (0-second deadline); exit 2 when no token refresh occurred (typical); exit 0 and creds written back in-place when OAuth refresh completed at subprocess startup before the kill
- **Exit:** 2 (or 0 if creds refreshed at startup)
- **Note:** lim_it — requires valid credentials; fast path (no full session needed)
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-isolated-subcommand)
---

### CC-4: `-- --version` passthrough with `--creds`

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json -- --version`
- **Then:** `--version` forwarded as passthrough arg to subprocess; subprocess prints its version and exits; no message needed
- **Exit:** 0
- **Note:** lim_it — requires claude binary accessible
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-isolated-subcommand)
