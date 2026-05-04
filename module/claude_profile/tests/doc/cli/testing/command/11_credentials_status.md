# Test: `.credentials.status`

Integration test planning for the `.credentials.status` command. See [commands.md](../../../../../docs/cli/commands.md#command--11-credentialsstatus) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| cred01 | No credential store — exits 0, default output | Account-Store Independence |
| cred02 | Default output with `.claude.json` — all 7 default-on fields shown | Field Presence (default) |
| cred03 | `format::json` — returns parseable JSON with all 9 fields | Output Format |
| cred04 | Missing `.credentials.json` — exits non-zero with actionable error | Error Handling |
| cred05 | Default output without `.claude.json` — email, org, account show N/A | Missing Optional File |
| cred06 | All default-on fields suppressed — only token line shown | Field Presence (suppress) |
| cred07 | `file::1 saved::1` — File and Saved lines appended | Field Presence (opt-in) |

### Test Coverage Summary

- Account-Store Independence: 1 test
- Field Presence (default): 1 test
- Output Format: 1 test
- Error Handling: 1 test
- Missing Optional File: 1 test
- Field Presence (suppress): 1 test
- Field Presence (opt-in): 1 test

**Total:** 7 integration tests

---

### cred01: No credential store — exits 0, default output

**Goal:** Verify that `.credentials.status` does not depend on the account store. Command must succeed on a machine with only `~/.claude/.credentials.json` and no `~/.persistent/claude/credential/` directory. `Account:` shows N/A (no `_active` marker).
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). No credential store directory created.
**Command:** `clp .credentials.status`
**Expected Output:** Stdout contains subscription type ("pro") and a token state classification ("valid"), `Account: N/A`, exit 0.
**Verification:**
- Exit code is 0
- Stdout contains "pro" (subscription type)
- Stdout contains a token state word ("valid", "expiring", or "expired")
- Stdout contains "N/A" (account field — no `_active` marker)
- No stderr output
**Pass Criteria:** Exit 0; default fields visible including N/A account.
**Source:** [FR-17](../../../../../docs/feature/012_live_credentials_status.md)

---

### cred02: Default output with `.claude.json` — all 7 default-on fields shown

**Goal:** Verify that default invocation (no params) shows all 7 default-on fields: account, sub, tier, token, expires, email, org.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). `~/.claude/.claude.json` present (emailAddress="user@example.com", organizationName="Acme Corp"). No credential store directory.
**Command:** `clp .credentials.status`
**Expected Output:** Stdout contains all 7 default-on fields, exit 0.
**Verification:**
- Exit code is 0
- Stdout contains "pro" (subscription — Sub: line)
- Stdout contains "standard" (tier — Tier: line)
- Stdout contains a token state word ("valid", "expiring", or "expired") (Token: line)
- Stdout contains "Expires" or "expires" (Expires: line)
- Stdout contains "user@example.com" (email — Email: line)
- Stdout contains "Acme Corp" (org — Org: line)
- Stdout contains "Account:" (account line)
- No stderr output
**Pass Criteria:** Exit 0; all 7 default-on fields present in output.
**Source:** [FR-17](../../../../../docs/feature/012_live_credentials_status.md)

---

### cred03: `format::json` — returns parseable JSON with all 9 fields

**Goal:** Verify that `format::json` produces a valid JSON object with all 9 required keys regardless of field-presence param defaults.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future). `~/.claude/.claude.json` present (emailAddress="user@example.com", organizationName="Acme Corp").
**Command:** `clp .credentials.status format::json`
**Expected Output:** Valid JSON object on stdout containing all 9 keys, exit 0.
**Verification:**
- Exit code is 0
- Stdout is a JSON object (starts with `{`, ends with `}`)
- Parsed JSON contains key `subscription`
- Parsed JSON contains key `tier`
- Parsed JSON contains key `token`
- Parsed JSON contains key `expires_in_secs`
- Parsed JSON contains key `email`
- Parsed JSON contains key `org`
- Parsed JSON contains key `account`
- Parsed JSON contains key `file`
- Parsed JSON contains key `saved`
**Pass Criteria:** Exit 0; output is valid JSON with all 9 required fields (including opt-in `file` and `saved`).
**Source:** [FR-17](../../../../../docs/feature/012_live_credentials_status.md)

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
**Source:** [FR-17](../../../../../docs/feature/012_live_credentials_status.md)

---

### cred05: Default output without `.claude.json` — email, org, account show N/A

**Goal:** Verify that when `~/.claude/.claude.json` is absent and no `_active` marker exists, email, org, and account gracefully show "N/A" rather than erroring or showing blank lines.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro"). No `~/.claude/.claude.json`. No credential store directory (no `_active` marker).
**Command:** `clp .credentials.status`
**Expected Output:** Stdout shows "N/A" for email, org, and account fields, exit 0.
**Verification:**
- Exit code is 0
- Stdout contains at least three occurrences of "N/A" (account, email, org)
- No stderr output
**Pass Criteria:** Exit 0; N/A displayed for missing email, org, and account.
**Source:** [FR-17](../../../../../docs/feature/012_live_credentials_status.md)

---

### cred06: All default-on fields suppressed — only token line shown

**Goal:** Verify that suppressing all default-on boolean params except `token::` leaves only the Token line in output.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro", rateLimitTier="standard", expiresAt=far future).
**Command:** `clp .credentials.status account::0 sub::0 tier::0 expires::0 email::0 org::0`
**Expected Output:** Stdout contains only the Token line, exit 0.
**Verification:**
- Exit code is 0
- Stdout contains a token state word ("valid", "expiring", or "expired")
- Stdout does not contain "Sub:" or "Tier:" or "Expires:" or "Email:" or "Org:" or "Account:"
- No stderr output
**Pass Criteria:** Exit 0; only Token: line in output, all other default-on lines suppressed.
**Source:** [FR-17](../../../../../docs/feature/012_live_credentials_status.md)

---

### cred07: `file::1 saved::1` — File and Saved lines appended

**Goal:** Verify that opt-in fields `file::1` and `saved::1` append the File and Saved lines after the default-on fields.
**Setup:** `~/.claude/.credentials.json` present (subscriptionType="pro"). Credential store may or may not exist.
**Command:** `clp .credentials.status file::1 saved::1`
**Expected Output:** Stdout contains the default-on fields plus File and Saved lines, exit 0.
**Verification:**
- Exit code is 0
- Stdout contains "File:" (file path line)
- Stdout contains "Saved:" (saved account count line)
- Stdout contains ".credentials.json" (path in File: line)
- No stderr output
**Pass Criteria:** Exit 0; File: and Saved: lines present in output alongside default-on fields.
**Source:** [FR-17](../../../../../docs/feature/012_live_credentials_status.md)
