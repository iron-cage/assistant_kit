# Test: `.credentials.status`

Integration test planning for the `.credentials.status` command. See [commands.md](../../commands.md#command--11-credentialsstatus) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| cred01 | No `accounts/` dir — exits 0 with sub and token state | Account-Store Independence |
| cred02 | `v::2` with `.claude.json` — shows all fields including expiry | Verbosity |
| cred03 | `format::json` — returns parseable JSON with required fields | Output Format |
| cred04 | Missing `.credentials.json` — exits non-zero with actionable error | Error Handling |
| cred05 | `v::1` without `.claude.json` — email and org show N/A | Missing Optional File |

## Test Coverage Summary

- Account-Store Independence: 1 test
- Verbosity: 1 test
- Output Format: 1 test
- Error Handling: 1 test
- Missing Optional File: 1 test

**Total:** 5 integration tests

---

### cred01: No `accounts/` dir — exits 0 with sub and token state

**Goal:** Verify that `.credentials.status` does not depend on the account store. Command must succeed on a machine with only `~/.claude/.credentials.json` and no `~/.claude/accounts/` directory.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). No `accounts/` directory created.
**Command:** `clp .credentials.status`
**Expected Output:** Stdout contains subscription type ("pro") and a token state classification ("valid"), exit 0.
**Verification:**
- Exit code is 0
- Stdout contains "pro" (subscription type)
- Stdout contains a token state word ("valid", "expiring", or "expired")
- No stderr output
**Pass Criteria:** Exit 0; subscription type and token state visible.
**Source:** [FR-17](../../../feature/012_live_credentials_status.md)

---

### cred02: `v::2` with `.claude.json` — shows all fields including expiry

**Goal:** Verify that `v::2` displays all available fields: sub, tier, token state, expiry time, email, and org.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). `~/.claude/.claude.json` present (emailAddress="user@example.com", organizationName="Acme Corp"). No `accounts/` directory.
**Command:** `clp .credentials.status v::2`
**Expected Output:** Stdout contains all six fields, exit 0.
**Verification:**
- Exit code is 0
- Stdout contains "pro" (subscription)
- Stdout contains "standard" (tier)
- Stdout contains "user@example.com" (email)
- Stdout contains "Acme Corp" (org)
- Stdout contains "Expires" or "expires" (expiry line)
**Pass Criteria:** Exit 0; all fields present.
**Source:** [FR-17](../../../feature/012_live_credentials_status.md)

---

### cred03: `format::json` — returns parseable JSON with required fields

**Goal:** Verify that `format::json` produces a valid JSON object with `subscription`, `tier`, and `token` keys.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future).
**Command:** `clp .credentials.status format::json`
**Expected Output:** Valid JSON object on stdout, exit 0.
**Verification:**
- Exit code is 0
- Stdout is a JSON object (starts with `{`, ends with `}`)
- Parsed JSON contains key `subscription`
- Parsed JSON contains key `tier`
- Parsed JSON contains key `token`
**Pass Criteria:** Exit 0; output is valid JSON with required fields.
**Source:** [FR-17](../../../feature/012_live_credentials_status.md)

---

### cred04: Missing `.credentials.json` — exits non-zero with actionable error

**Goal:** Verify that when `~/.claude/.credentials.json` is absent, the command exits non-zero and the error message references the credential file.
**Setup:** `~/.claude/` directory exists but `.credentials.json` is not present.
**Command:** `clp .credentials.status`
**Expected Output:** Error message on stderr referencing "credential", exit non-zero.
**Verification:**
- Exit code is non-zero (1 or 2)
- Stderr contains the word "credential" (case-insensitive)
- No misleading success output on stdout
**Pass Criteria:** Exit non-zero; stderr references the missing credential file.
**Source:** [FR-17](../../../feature/012_live_credentials_status.md)

---

### cred05: `v::1` without `.claude.json` — email and org show N/A

**Goal:** Verify that when `~/.claude/.claude.json` is absent, email and org gracefully default to "N/A" rather than erroring or showing blank lines.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro"). No `~/.claude/.claude.json`.
**Command:** `clp .credentials.status v::1`
**Expected Output:** Stdout shows at least two "N/A" values (for email and org), exit 0.
**Verification:**
- Exit code is 0
- Stdout contains at least two occurrences of "N/A"
- No stderr output
**Pass Criteria:** Exit 0; N/A displayed for missing email and org.
**Source:** [FR-17](../../../feature/012_live_credentials_status.md)
