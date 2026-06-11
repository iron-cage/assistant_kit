# Test: noun::token

Noun contract tests for the `token` domain noun. Verifies stateless read behavior,
JSON output schema fidelity, and error code contract as defined in
[docs/cli/command_noun/002_token.md](../../../../docs/cli/command_noun/002_token.md).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| NC-1 | Token status is stateless — no persistent state written | Lifecycle |
| NC-2 | `.token.status format::json` output matches documented schema | Output Schema |
| NC-3 | Missing credentials file exits 2 | Error Code Contract |

### Test Coverage Summary

- Lifecycle: 1 test
- Output Schema: 1 test
- Error Code Contract: 1 test

**Total:** 3 noun contract tests

---

### NC-1: Token status is stateless — no persistent state written

- **Given:** `~/.claude/.credentials.json` exists with valid `expiresAt`. Record mtime of `~/.claude/.credentials.json` and all credential store files.
- **When:** `clp .token.status`
- **Then:** Exit 0. mtime of `~/.claude/.credentials.json` unchanged. No files created or modified. Token classification printed to stdout. Confirms token noun is a pure read path — the token lifecycle is managed externally by the OAuth flow; `clp` never writes token state.
- **Exit:** 0
- **Source:** [002_token.md — Lifecycle](../../../../docs/cli/command_noun/002_token.md#lifecycle)

---

### NC-2: `.token.status format::json` output matches documented schema

- **Given:** `~/.claude/.credentials.json` exists with valid `expiresAt` in the future.
- **When:** `clp .token.status format::json`
- **Then:** Exit 0. Output is valid JSON object. Contains `status` field (string; one of `"valid"`, `"expiring_soon"`, `"expired"`) and `expires_in_secs` field (non-negative integer). No undocumented fields required.
- **Exit:** 0
- **Source:** [002_token.md — Output Schema](../../../../docs/cli/command_noun/002_token.md#output-schema)

---

### NC-3: Missing credentials file exits 2

- **Given:** `~/.claude/.credentials.json` does NOT exist.
- **When:** `clp .token.status`
- **Then:** Exit 2. Error message on stderr referencing absent or unreadable credentials file. No stdout output.
- **Exit:** 2
- **Source:** [002_token.md — Error Codes](../../../../docs/cli/command_noun/002_token.md#error-codes)
