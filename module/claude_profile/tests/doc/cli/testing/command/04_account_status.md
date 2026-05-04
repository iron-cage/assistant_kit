# Test: `.account.status`

Integration test planning for the `.account.status` command. See [commands.md](../../../../../docs/cli/commands.md#command--4-accountstatus) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | No `_active` file → exit 2, stderr reports no active account | Error Handling |
| IT-2 | Empty `_active` file → exit 2, stderr reports no active account | Error Handling |
| IT-3 | Active account with valid token → name and "valid" in output | Basic Invocation |
| IT-4 | Active account with expired token → name and "expired" in output | Token State |
| IT-5 | Active account with near-expiry token → "expiring in Xm" in output | Token State |
| IT-6 | Missing credentials file → token shows "unknown", exit 0 | Error Recovery |
| IT-7 | `v::0` shows bare name then token state (no labels) | Verbosity |
| IT-8 | `v::1` (default) shows Account: and Token: labels; no Expires: line | Verbosity |
| IT-9 | `v::2` shows additional "Expires:" line | Verbosity |
| IT-10 | `format::json` returns `{"account":"...","token":"..."}` | Output Format |
| IT-11 | `name::` = active account → same name and "valid" as no-name path | FR-16 / Named Account |
| IT-12 | `name::` = non-active account → shows that account's own expired token | FR-16 / P2 Guard |
| IT-13 | `name::` = nonexistent account → exit 2 + "not found" in stderr | FR-16 / Error Handling |
| IT-14 | `name::` with invalid chars → exit 1 | FR-16 / Input Validation |
| IT-15 | `name::` + `v::0` → two bare lines (name, token state) | FR-16 / Verbosity |
| IT-16 | `name::` = active + `.claude.json` present → shows Email/Org at `v::1` | FR-16 / Email/Org |
| IT-17 | `name::` = non-active + `v::1` → Email and Org are N/A | FR-16 / Email/Org |
| IT-18 | `name::` = non-active + `v::2` → "Expires:" line present | FR-16 / Verbosity |
| IT-19 | `name::` + `format::json` → JSON with `account` and `token` fields | FR-16 / Output Format |
| IT-20 | `name::` = non-active + `v::0` → two bare lines with named account's state | FR-16 / P2 Guard |
| IT-21 | Active path `v::1` — default `.account.status` shows Sub:, Tier:, Email:, Org: | FR-16 / Sub/Tier |
| IT-22 | Named active `v::1` — `name::work@acme.com v::1` shows Sub:, Tier:, Email:, Org: | FR-16 / Sub/Tier |
| IT-23 | Named non-active `v::1` — `name::personal@home.com v::1` shows own Sub:, Tier: (not active's) | FR-16 / Sub/Tier |
| IT-24 | Active path — empty-string `subscriptionType` in credentials → `Sub: N/A` | FR-16 / N/A Normalization |
| IT-25 | Named non-active — `subscriptionType` absent in account file → `Sub: N/A` | FR-16 / N/A Normalization |
| IT-26 | Named non-active — `rateLimitTier` absent in account file → `Tier: N/A` | FR-16 / N/A Normalization |
| IT-27 | Active account — empty-string `emailAddress`/`organizationName` in `.claude.json` → `Email: N/A`, `Org: N/A` | FR-16 / N/A Normalization |

### Test Coverage Summary

- Basic Invocation: 1 test
- Token State: 2 tests
- Error Handling: 3 tests
- Error Recovery: 1 test
- Verbosity: 5 tests
- Output Format: 2 tests
- FR-16 Named Account: 6 tests (IT-11, IT-12, IT-16, IT-17, IT-18, IT-20)
- FR-16 Sub/Tier: 3 tests (IT-21, IT-22, IT-23)
- FR-16 N/A Normalization: 4 tests (IT-24, IT-25, IT-26, IT-27)

**Total:** 27 integration tests

---

### IT-1: No `_active` file → exit 2, stderr reports no active account

**Goal:** Confirm that a missing `_active` marker produces a clear error and non-zero exit.
**Setup:** Create `~/.claude/.credentials.json` (valid token). Do NOT create `~/.persistent/claude/credential/_active`.
**Command:** `clp .account.status`
**Expected Output:** Empty stdout; stderr contains "no active account set".
**Verification:**
- Capture stdout and stderr
- Assert exit code is 2
- Assert stderr contains `no active account`
- Assert stdout is empty
**Pass Criteria:** Exit 2; actionable error message on stderr.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-2: Empty `_active` file → exit 2, stderr reports no active account

**Goal:** Confirm that an existing but empty `_active` marker is treated the same as absent.
**Setup:** Create `~/.persistent/claude/credential/_active` with empty contents (zero bytes or whitespace only).
**Command:** `clp .account.status`
**Expected Output:** Empty stdout; stderr contains "no active account set".
**Verification:**
- Capture stdout and stderr
- Assert exit code is 2
- Assert stderr contains `no active account`
**Pass Criteria:** Exit 2; empty `_active` is not treated as a valid account name.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-3: Active account with valid token → name and "valid" in output

**Goal:** Confirm the happy-path output shows the account name and "valid" token state.
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Write `~/.claude/.credentials.json` with `expiresAt` far in the future (year ~2286).
**Command:** `clp .account.status`
**Expected Output:** Output contains `work@acme.com` and `valid`.
**Verification:**
- Capture stdout
- Assert exit code is 0
- Assert stdout contains `work@acme.com`
- Assert stdout contains `valid`
**Pass Criteria:** Exit 0; account name and valid token state present.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-4: Active account with expired token → name and "expired" in output

**Goal:** Confirm that an expired token is reported as "expired" in the output.
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Write `~/.claude/.credentials.json` with `expiresAt` in the past (Unix ms 1_000_000_000).
**Command:** `clp .account.status`
**Expected Output:** Output contains `work@acme.com` and `expired`.
**Verification:**
- Capture stdout
- Assert exit code is 0
- Assert stdout contains `work@acme.com`
- Assert stdout contains `expired`
**Pass Criteria:** Exit 0; expired state reported without crashing.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-5: Active account with near-expiry token → "expiring in Xm" in output

**Goal:** Confirm that a token within the warning threshold is reported as expiring.
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Write `~/.claude/.credentials.json` with `expiresAt` 30 minutes from now (within 3600s default threshold).
**Command:** `clp .account.status`
**Expected Output:** Output contains `work@acme.com` and `expiring in`.
**Verification:**
- Capture stdout
- Assert exit code is 0
- Assert stdout contains `work@acme.com`
- Assert stdout contains `expiring in`
**Pass Criteria:** Exit 0; near-expiry state distinguishable from valid and expired.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-6: Missing credentials file → token shows "unknown", exit 0

**Goal:** Confirm that a missing credentials file does not crash the command — account name is still shown with token state "unknown".
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Do NOT create `~/.claude/.credentials.json`.
**Command:** `clp .account.status`
**Expected Output:** Output contains `work@acme.com` and `unknown`. Exit 0.
**Verification:**
- Capture stdout and stderr
- Assert exit code is 0
- Assert stdout contains `work@acme.com`
- Assert stdout contains `unknown`
**Pass Criteria:** Exit 0; graceful degradation when credentials are unreadable.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-7: `v::0` shows bare name then token state (no labels)

**Goal:** Confirm verbosity 0 produces exactly two lines: account name then token state, no "Account:" or "Token:" labels.
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Write valid credentials.
**Command:** `clp .account.status v::0`
**Expected Output:** Two lines: line 1 = `work@acme.com`, line 2 = `valid`. No label prefixes.
**Verification:**
- Capture stdout
- Assert exit code is 0
- Split stdout into lines
- Assert line 0 is `work@acme.com`
- Assert line 1 is `valid`
- Assert stdout does not contain `Account:`
- Assert stdout does not contain `Token:`
**Pass Criteria:** Exit 0; two unlabelled lines only.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-8: `v::1` (default) shows "Account:" and "Token:" labels

**Goal:** Confirm the default verbosity level 1 shows labelled output with "Account:" and "Token:" prefixes.
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Write valid credentials.
**Command:** `clp .account.status` (implicit v::1)
**Expected Output:** Lines like `Account: work@acme.com` and `Token:   valid`.
**Verification:**
- Capture stdout
- Assert exit code is 0
- Assert stdout contains `Account: work@acme.com`
- Assert stdout contains `Token:`
- Assert stdout does not contain `Expires:`
**Pass Criteria:** Exit 0; Account: and Token: labels present, no Expires: line (Sub/Tier/Email/Org presence verified by IT-21).
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-9: `v::2` shows additional "Expires:" line

**Goal:** Confirm verbosity level 2 adds the "Expires:" line beyond what v::1 shows.
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Write valid credentials with far-future expiry.
**Command:** `clp .account.status v::2`
**Expected Output:** Seven lines: `Account: work@acme.com`, `Token:   valid`, `Sub:     pro`, `Tier:    standard`, `Expires: in Xh Ym`, `Email:   N/A`, `Org:     N/A` (no `.claude.json` in setup, so email/org fall back to N/A).
**Verification:**
- Capture stdout
- Assert exit code is 0
- Assert stdout contains `Account: work@acme.com`
- Assert stdout contains `Token:`
- Assert stdout contains `Expires:`
**Pass Criteria:** Exit 0; seven-line output (Account, Token, Sub, Tier, Expires, Email, Org) with expiry detail.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-10: `format::json` returns `{"account":"...","token":"..."}`

**Goal:** Confirm JSON format output is a valid JSON object with `account` and `token` fields.
**Setup:** Write `~/.persistent/claude/credential/_active` = `"work@acme.com"`. Write valid credentials.
**Command:** `clp .account.status format::json`
**Expected Output:** A single JSON object `{"account":"work@acme.com","token":"valid"}`.
**Verification:**
- Capture stdout
- Assert exit code is 0
- Parse stdout as JSON (must not fail)
- Assert parsed value is a JSON object
- Assert object has `account` field equal to `"work@acme.com"`
- Assert object has `token` field equal to `"valid"`
**Pass Criteria:** Exit 0; valid JSON object with both required fields.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus)

---

### IT-11: `name::` = active account → same name and "valid" as no-name path

**Goal:** Confirm `name::work@acme.com` when `work@acme.com` is the active account produces equivalent output to omitting `name::`.
**Setup:** Write `work@acme.com` as active account with valid far-future credentials.
**Command:** `clp .account.status name::work@acme.com`
**Expected Output:** Contains `work@acme.com` and `valid`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `work@acme.com`
- Assert stdout contains `valid`
**Pass Criteria:** Exit 0; named active account shown correctly.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16)

---

### IT-12: `name::` = non-active account → shows that account's own expired token

**Goal:** Confirm the P2 guard: `name::personal@home.com` where `personal@home.com` has an expired token (but active account `work@acme.com` has valid token) reports `expired`, not `valid`.
**Setup:** Write `work@acme.com` as active with valid far-future token. Write `personal@home.com` as non-active with past-expired token.
**Command:** `clp .account.status name::personal@home.com`
**Expected Output:** Contains `personal@home.com` and `expired`. Does NOT contain `valid`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `personal@home.com`
- Assert stdout contains `expired`
- Assert stdout does NOT contain `valid`
**Pass Criteria:** Exit 0; non-active account reports its own token state, not the active account's.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16, P2)

---

### IT-13: `name::` = nonexistent account → exit 2 + "not found" in stderr

**Goal:** Confirm that querying an unknown account name produces exit 2 with a descriptive error.
**Setup:** Write `work@acme.com` as active with valid credentials. Do NOT create a `ghost@example.com` account.
**Command:** `clp .account.status name::ghost@example.com`
**Expected Output:** Empty stdout; stderr contains `not found` or `ghost@example.com`.
**Verification:**
- Assert exit code is 2
- Assert stderr contains `not found` or `ghost@example.com`
- Assert stdout is empty
**Pass Criteria:** Exit 2; unknown account name reported clearly.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16)

---

### IT-14: `name::` with invalid chars → exit 1

**Goal:** Confirm that a `name::` value that is not a valid email address is rejected as a usage error.
**Setup:** Any valid active account exists.
**Command:** `clp .account.status name::notanemail`
**Expected Output:** Exit 1; stderr contains validation error.
**Verification:**
- Assert exit code is 1
**Pass Criteria:** Exit 1; non-email name rejected before lookup.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16)

---

### IT-15: `name::` + `v::0` → two bare lines (name, token state)

**Goal:** Confirm `v::0` with a named account produces exactly two unlabelled lines.
**Setup:** Write `work@acme.com` as active with valid far-future credentials.
**Command:** `clp .account.status name::work@acme.com v::0`
**Expected Output:** Exactly two lines: `work@acme.com` and `valid`.
**Verification:**
- Assert exit code is 0
- Assert stdout splits into exactly 2 lines
- Assert line 0 is `work@acme.com`
- Assert line 1 is `valid`
- Assert stdout does NOT contain `Account:`
**Pass Criteria:** Exit 0; bare two-line output matching account name and token state.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16)

---

### IT-16: `name::` = active + `.claude.json` present → shows Email/Org at `v::1`

**Goal:** Confirm that at `v::1`, querying the active account shows email and org from `.claude.json`.
**Setup:** Write `work@acme.com` as active with valid credentials. Write `~/.claude/.claude.json` with `emailAddress` = `alice@example.com` and `organizationName` = `Acme Corp`.
**Command:** `clp .account.status name::work@acme.com v::1`
**Expected Output:** Contains `alice@example.com` and `Acme Corp`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `alice@example.com`
- Assert stdout contains `Acme Corp`
**Pass Criteria:** Exit 0; active account's email/org shown from `.claude.json`.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16)

---

### IT-17: `name::` = non-active + `v::1` → Email and Org are N/A

**Goal:** Confirm that at `v::1`, querying a non-active account shows N/A for email/org even if `.claude.json` exists.
**Setup:** Write `work@acme.com` as active. Write `personal@home.com` as non-active. Write `.claude.json` with `alice@example.com`.
**Command:** `clp .account.status name::personal@home.com v::1`
**Expected Output:** Contains `personal@home.com` and `N/A`. Does NOT contain `alice@example.com`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `personal@home.com`
- Assert stdout contains `N/A`
- Assert stdout does NOT contain `alice@example.com`
**Pass Criteria:** Exit 0; active session's email not leaked to non-active account query.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16, P3)

---

### IT-18: `name::` = non-active + `v::2` → "Expires:" line present

**Goal:** Confirm that `v::2` includes the `Expires:` line for a named account query.
**Setup:** Write `work@acme.com` as active. Write `personal@home.com` as non-active with far-future token.
**Command:** `clp .account.status name::personal@home.com v::2`
**Expected Output:** Contains `personal@home.com` and `Expires:`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `personal@home.com`
- Assert stdout contains `Expires:`
**Pass Criteria:** Exit 0; expiry detail line present at v::2.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16)

---

### IT-19: `name::` + `format::json` → JSON with `account` and `token` fields

**Goal:** Confirm JSON output for a named account query has the correct `account` and `token` fields.
**Setup:** Write `work@acme.com` as active with valid credentials.
**Command:** `clp .account.status name::work@acme.com format::json`
**Expected Output:** `{"account":"work@acme.com","token":"valid"}`.
**Verification:**
- Assert exit code is 0
- Assert stdout starts with `{`
- Assert stdout contains `"account":"work@acme.com"`
- Assert stdout contains `"token":"valid"`
**Pass Criteria:** Exit 0; valid JSON with correct field values.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16)

---

### IT-20: `name::` = non-active + `v::0` → two bare lines with named account's state

**Goal:** Confirm the P2 guard at `v::0`: `name::personal@home.com` where `personal@home.com` has an expired token produces bare `personal@home.com` then `expired`.
**Setup:** Write `work@acme.com` as active with valid token. Write `personal@home.com` as non-active with past-expired token.
**Command:** `clp .account.status name::personal@home.com v::0`
**Expected Output:** Exactly two lines: `personal@home.com` and `expired`.
**Verification:**
- Assert exit code is 0
- Assert stdout splits into exactly 2 lines
- Assert line 0 is `personal@home.com`
- Assert line 1 is `expired`
**Pass Criteria:** Exit 0; non-active account's own expired state shown in bare format.
**Source:** [commands.md — .account.status](../../../../../docs/cli/commands.md#command--4-accountstatus) (FR-16, P2)

---

### IT-21: Active path `v::1` — default `.account.status` shows Sub:, Tier:, Email:, Org:

**Goal:** Confirm the active-path `v::1` output includes `Sub:`, `Tier:`, `Email:`, and `Org:` fields after the FR-16 spec compliance fix.
**Setup:** Write `work@acme.com` as active with valid far-future credentials (subscriptionType=`pro`, rateLimitTier=`standard`). Write `~/.claude/.claude.json` with emailAddress=`alice@example.com` and organizationName=`Acme Corp`.
**Command:** `clp .account.status` (implicit v::1)
**Expected Output:** Contains `Sub:`, `Tier:`, `pro`, `standard`, `alice@example.com`, `Acme Corp`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `Sub:`
- Assert stdout contains `Tier:`
- Assert stdout contains `pro`
- Assert stdout contains `standard`
- Assert stdout contains `alice@example.com`
- Assert stdout contains `Acme Corp`
**Pass Criteria:** Exit 0; all four fields present in v::1 output.
**Source:** `account_list_status_test.rs::astat11_v1_shows_sub_tier_email_org` (spec FR-16 line 283)

---

### IT-22: Named active `v::1` — `name::work@acme.com v::1` shows Sub:, Tier:, Email:, Org:

**Goal:** Confirm `name::work@acme.com v::1` when `work@acme.com` is the active account shows `Sub:`, `Tier:`, `Email:`, and `Org:`.
**Setup:** Write `work@acme.com` as active with valid far-future credentials (subscriptionType=`pro`, rateLimitTier=`standard`). Write `.claude.json` with email and org.
**Command:** `clp .account.status name::work@acme.com v::1`
**Expected Output:** Contains `Sub:`, `Tier:`, `pro`, `standard`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `Sub:`
- Assert stdout contains `Tier:`
- Assert stdout contains `pro`
- Assert stdout contains `standard`
**Pass Criteria:** Exit 0; Sub/Tier fields present in named-active v::1 output.
**Source:** `account_status_name_test.rs::astname11_active_named_v1_shows_sub_tier` (spec FR-16 line 283)

---

### IT-23: Named non-active `v::1` — `name::personal@home.com v::1` shows own Sub:, Tier: (not active's)

**Goal:** Confirm that querying a non-active account at `v::1` shows that account's own `Sub:` and `Tier:` — not the active account's values.
**Setup:** Write `work@acme.com` as active with subscriptionType=`max` and rateLimitTier=`tier4`. Write `personal@home.com` as non-active with subscriptionType=`pro` and rateLimitTier=`standard`.
**Command:** `clp .account.status name::personal@home.com v::1`
**Expected Output:** Contains `pro` and `standard`. Does NOT contain `max` or `tier4`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `Sub:`
- Assert stdout contains `Tier:`
- Assert stdout contains `pro`
- Assert stdout contains `standard`
- Assert stdout does NOT contain `max`
- Assert stdout does NOT contain `tier4`
**Pass Criteria:** Exit 0; non-active account's own Sub/Tier shown without leaking active account's values.
**Source:** `account_status_name_test.rs::astname12_nonactive_named_v1_shows_own_sub_tier` (spec FR-16 line 283)

---

### IT-24: Active path — empty-string `subscriptionType` in credentials → `Sub: N/A`

**Goal:** Confirm that an empty-string `subscriptionType` in `~/.claude/.credentials.json` produces `Sub:     N/A` rather than a blank `Sub:     ` line.
**Setup:** Write `work@acme.com` as active with valid far-future expiry. Write `~/.claude/.credentials.json` manually with `"subscriptionType":""` (explicit empty string) and `"rateLimitTier":"standard"`.
**Command:** `clp .account.status` (implicit v::1)
**Expected Output:** Contains `Sub:     N/A`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `Sub:     N/A`
**Pass Criteria:** Exit 0; empty-string subscriptionType normalizes to N/A, not a blank value.
**Source:** `account_list_status_test.rs::astat12_v1_empty_sub_in_creds_shows_n_a` (spec FR-16)

---

### IT-25: Named non-active — `subscriptionType` absent in account file → `Sub: N/A`

**Goal:** Confirm that a missing `subscriptionType` field in a named account's credential file produces `Sub:     N/A` rather than a blank line.
**Setup:** Write `work@acme.com` as active with `subscriptionType=max`. Write `personal@home.com` as non-active with a credentials file that has no `subscriptionType` field at all (only `rateLimitTier` and `expiresAt`).
**Command:** `clp .account.status name::personal@home.com`
**Expected Output:** Contains `Sub:     N/A`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `Sub:     N/A`
**Pass Criteria:** Exit 0; absent subscriptionType field normalizes to N/A via empty-string path from `unwrap_or_default()`.
**Source:** `account_status_name_test.rs::astname13_missing_sub_in_file_shows_n_a` (spec FR-16)

---

### IT-26: Named non-active — `rateLimitTier` absent in account file → `Tier: N/A`

**Goal:** Confirm that a missing `rateLimitTier` field in a named account's credential file produces `Tier:    N/A` rather than a blank line.
**Setup:** Write `work@acme.com` as active with `rateLimitTier=tier4`. Write `personal@home.com` as non-active with a credentials file that has no `rateLimitTier` field (only `subscriptionType` and `expiresAt`).
**Command:** `clp .account.status name::personal@home.com`
**Expected Output:** Contains `Tier:    N/A`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `Tier:    N/A`
**Pass Criteria:** Exit 0; absent rateLimitTier field normalizes to N/A via empty-string path from `unwrap_or_default()`.
**Source:** `account_status_name_test.rs::astname14_missing_tier_in_file_shows_n_a` (spec FR-16)

---

### IT-27: Active account — empty-string `emailAddress`/`organizationName` in `.claude.json` → `Email: N/A`, `Org: N/A`

**Goal:** Confirm that empty-string `emailAddress` and `organizationName` fields in `~/.claude/.claude.json` produce `Email:   N/A` and `Org:     N/A` rather than blank lines.
**Setup:** Write `work@acme.com` as active with valid far-future credentials. Write `~/.claude/.claude.json` with `"emailAddress":""` and `"organizationName":""` (explicit empty strings).
**Command:** `clp .account.status` (implicit v::1)
**Expected Output:** Contains `Email:   N/A` and `Org:     N/A`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `Email:   N/A`
- Assert stdout contains `Org:     N/A`
**Pass Criteria:** Exit 0; empty-string email/org fields normalize to N/A rather than showing blank values.
**Source:** `account_status_name_test.rs::astname15_active_empty_email_org_in_claude_json_shows_na` (spec FR-16, bug_reproducer issue-empty-field-blank-status-named)
