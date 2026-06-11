# Test: noun::credentials

Noun contract tests for the `credentials` domain noun. Verifies stateless read behavior,
JSON output schema fidelity, and error code contract as defined in
[docs/cli/command_noun/003_credentials.md](../../../../docs/cli/command_noun/003_credentials.md).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| NC-1 | Credentials status is stateless — no persistent state written | Lifecycle |
| NC-2 | `.credentials.status format::json` output matches documented schema | Output Schema |
| NC-3 | Missing `~/.claude/.credentials.json` exits 2 | Error Code Contract |

### Test Coverage Summary

- Lifecycle: 1 test
- Output Schema: 1 test
- Error Code Contract: 1 test

**Total:** 3 noun contract tests

---

### NC-1: Credentials status is stateless — no persistent state written

- **Given:** `~/.claude/.credentials.json` exists. Active account `alice@acme.com` in store. Record mtime of all credential store files.
- **When:** `clp .credentials.status`
- **Then:** Exit 0. Credential metadata printed to stdout. mtime of `~/.claude/.credentials.json` unchanged. No credential store files written. Confirms credentials noun does not require account store setup and is a pure read from `~/.claude/.credentials.json` plus optional supplementary reads.
- **Exit:** 0
- **Source:** [003_credentials.md — Lifecycle](../../../../docs/cli/command_noun/003_credentials.md#lifecycle)

---

### NC-2: `.credentials.status format::json` output matches documented schema

- **Given:** `~/.claude/.credentials.json` exists with full credentials including `oauthAccount`, subscription, and tier fields.
- **When:** `clp .credentials.status format::json`
- **Then:** Exit 0. Output is valid JSON object. Contains at minimum: `account` (string or null), `subscription` (string), `tier` (string), `token` (string), `expires_in_secs` (number), `email` (string), `file` (string). `format::json` always includes all documented fields regardless of field-presence params.
- **Exit:** 0
- **Source:** [003_credentials.md — Output Schema](../../../../docs/cli/command_noun/003_credentials.md#output-schema)

---

### NC-3: Missing `~/.claude/.credentials.json` exits 2

- **Given:** `~/.claude/.credentials.json` does NOT exist.
- **When:** `clp .credentials.status`
- **Then:** Exit 2. Error message on stderr referencing absent credentials file.
- **Exit:** 2
- **Source:** [003_credentials.md — Error Codes](../../../../docs/cli/command_noun/003_credentials.md#error-codes)
