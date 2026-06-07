# Parameter Group :: Credential Operations

Interaction tests for Group 4 (Credential Operations): `--creds`, `--timeout`, `--trace`. Tests validate these flags coexist correctly and configure the credential-isolated subprocess across `isolated` and `refresh` commands.

**Source:** [param_group/04_credential_operations.md](../../../../docs/cli/param_group/04_credential_operations.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `--creds` + `--timeout` both explicit → subprocess launched with both | Combined |
| CC-2 | `isolated` `--creds` only → default 30-second deadline | Default |
| CC-3 | `isolated` `--timeout 0` → unlimited (no watchdog), subprocess runs to natural exit | Interaction |
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
- **Source:** [param_group/04_credential_operations.md](../../../../docs/cli/param_group/04_credential_operations.md)
- **Commands:** isolated, refresh
---

### CC-2: `isolated` `--timeout` omitted → default 30 applied

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json "What is 2+2?"`
- **Then:** isolated subprocess launched with 30-second default deadline; behavior identical to `--timeout 30`
- **Exit:** 0
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group/04_credential_operations.md](../../../../docs/cli/param_group/04_credential_operations.md)
- **Commands:** isolated, refresh
---

### CC-3: `--timeout 0` → unlimited (no watchdog)

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json --timeout 0 "dummy"`
- **Then:** `0` disables the watchdog entirely — subprocess runs until it exits naturally (matching `run`/`ask` semantics); exit code is the subprocess exit code
- **Exit:** 0 or passthrough
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group/04_credential_operations.md](../../../../docs/cli/param_group/04_credential_operations.md)
- **Commands:** isolated, refresh
---

### CC-4: `-- --version` passthrough with `--creds`

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr isolated --creds /tmp/creds.json -- --version`
- **Then:** `--version` forwarded as passthrough arg to subprocess; subprocess prints its version and exits; no message needed
- **Exit:** 0
- **Note:** lim_it — requires claude binary accessible
- **Source:** [param_group/04_credential_operations.md](../../../../docs/cli/param_group/04_credential_operations.md)
- **Commands:** isolated, refresh
---

### CC-5: `refresh` `--timeout` omitted → default 45 applied

- **Given:** valid credentials file `/tmp/creds.json`
- **When:** `clr refresh --creds /tmp/creds.json`
- **Then:** subprocess launched with 45-second default deadline (not 30); deadline differs from `isolated` to allow headroom for slow OAuth exchange; exit 0 on successful refresh
- **Exit:** 0
- **Note:** lim_it — requires valid credentials
- **Source:** [param_group/04_credential_operations.md](../../../../docs/cli/param_group/04_credential_operations.md)
- **Commands:** isolated, refresh
---

### CC-6: `--trace` prints credential trace to stderr before execution

- **Given:** credentials JSON written to a temp file at `/tmp/cc6.creds.json` (file is readable; content `{}`; no live credentials needed — trace fires before subprocess attempt)
- **When:** `clr isolated --creds /tmp/cc6.creds.json --trace`
- **Then:** stderr contains `# clr isolated`, `# creds: /tmp/cc6.creds.json`, and `# timeout: 30s` before any subprocess attempt; subprocess attempt fails (claude absent in test environment)
- **Exit:** 1
- **Note:** verifies `--trace` is honoured for credential operations independently of `run`/`ask` trace path; does not require live credentials
- **Source:** [param_group/04_credential_operations.md](../../../../docs/cli/param_group/04_credential_operations.md), [invariant/004_trace_universality.md](../../../../docs/invariant/004_trace_universality.md)
- **Commands:** isolated, refresh
