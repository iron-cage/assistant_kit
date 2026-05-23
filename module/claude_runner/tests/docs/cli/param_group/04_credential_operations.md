# Parameter Group :: Credential Operations

Interaction tests for Group 4 (Credential Operations): `--creds`, `--timeout`, `--trace`. Tests validate these flags coexist correctly and configure the credential-isolated subprocess across `isolated` and `refresh` commands.

**Source:** [param_group.md#group--4-credential-operations](../../../../docs/cli/param_group.md#group--4-credential-operations)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--creds` + `--timeout` both explicit → subprocess launched with both | Combined |
| CC-2 | `isolated` `--creds` only → default 30-second deadline | Default |
| CC-3 | `isolated` `--timeout 0` → immediate expiry, exit 2, write-back triggered | Interaction |
| CC-4 | `--creds` + `-- --version` passthrough → subprocess flag forwarded | Interaction |
| CC-5 | `refresh` `--creds` only → default 45-second deadline | Default |
| CC-6 | `--trace` on credential ops → call details printed to stderr | Trace |

## Test Coverage Summary

- Combined: 1 test (CC-1)
- Default: 2 tests (CC-2, CC-5)
- Interaction: 2 tests (CC-3, CC-4)
- Trace: 1 test (CC-6)

**Total:** 6 combination cases

## Test Cases
---

### CC-1: `--creds` + `--timeout` both explicit

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json --timeout 60 "What is 2+2?"`
- **Then:** isolated subprocess launched with 60-second deadline; temp HOME contains `.claude/.credentials.json`; response captured to stdout
- **Exit:** 0
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-credential-operations)
---

### CC-2: `isolated` `--timeout` omitted → default 30 applied

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json "What is 2+2?"`
- **Then:** isolated subprocess launched with 30-second default deadline; behavior identical to `--timeout 30`
- **Exit:** 0
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-credential-operations)
---

### CC-3: `--timeout 0` → immediate expiry

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json --timeout 0 "dummy"`
- **Then:** subprocess killed immediately (0-second deadline); exit 2 when no token refresh occurred (typical); exit 0 and creds written back in-place when OAuth refresh completed at subprocess startup before the kill
- **Exit:** 2 (or 0 if creds refreshed at startup)
- **Note:** lim_it — requires valid credentials; fast path (no full session needed)
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-credential-operations)
---

### CC-4: `-- --version` passthrough with `--creds`

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json -- --version`
- **Then:** `--version` forwarded as passthrough arg to subprocess; subprocess prints its version and exits; no message needed
- **Exit:** 0
- **Note:** lim_it — requires claude binary accessible
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-credential-operations)
---

### CC-5: `refresh` `--timeout` omitted → default 45 applied

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr refresh --creds /tmp/creds.json`
- **Then:** subprocess launched with 45-second default deadline (not 30); deadline differs from `isolated` to allow headroom for slow OAuth exchange
- **Exit:** 0 (creds refreshed) or 1 (no refresh occurred)
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-credential-operations)
---

### CC-6: `--trace` prints call details to stderr before execution

- **Given:** `CLR_CREDS=/tmp/cc6.creds.json` (or `--creds`); `--trace` on CLI
- **When:** `clr isolated --creds /tmp/cc6.creds.json --trace` (parse-only path)
- **Then:** stderr contains creds path, temp HOME path, timeout value, and forwarded args before subprocess launch
- **Exit:** varies
- **Note:** verifies `--trace` is honoured by `apply_isolated_env_vars()` / `apply_refresh_env_vars()` independently of `run` trace path
- **Source:** [param_group.md](../../../../docs/cli/param_group.md#group--4-credential-operations)
