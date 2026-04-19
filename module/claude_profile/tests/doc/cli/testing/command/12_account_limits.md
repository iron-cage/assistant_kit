# Test: `.account.limits`

Integration test specifications for the `.account.limits` command. See [commands.md](../../../../../docs/cli/commands.md#command--12-accountlimits) and [feature/013_account_limits.md](../../../../../docs/feature/013_account_limits.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Active account — default output shows session, weekly-all, weekly-sonnet | Happy Path |
| IT-2 | Active account — `v::0` compact output (bare percentages) | Verbosity |
| IT-3 | Active account — `v::2` verbose output (all fields + raw values) | Verbosity |
| IT-4 | Active account — `format::json` returns parseable JSON | Format |
| IT-5 | Named account — `name::work` shows limits for that account | Named Account |
| IT-6 | Named account — `name::ghost` unknown account exits 2 | Not Found |
| IT-7 | No active account set — exits 2 with actionable error | Error Handling |
| IT-8 | Data unavailable — exits 2 with actionable error (not silent 0) | Error Handling |
| IT-9 | `name::` with invalid chars — exits 1 (usage error, not 2) | Parameter Validation |

### Test Coverage Summary

- Happy Path: 1 test (IT-1)
- Verbosity: 2 tests (IT-2, IT-3)
- Format: 1 test (IT-4)
- Named Account: 1 test (IT-5)
- Not Found: 1 test (IT-6)
- Error Handling: 2 tests (IT-7, IT-8)
- Parameter Validation: 1 test (IT-9)

**Total:** 9 integration tests

**Requirement:** FR-18 (feature/013_account_limits.md)

---

### IT-1: Happy Path — Default Output

**Goal:** Confirm `.account.limits` displays session, weekly-all, and weekly-sonnet utilization for the active account.
**Setup:** Active account configured; rate-limit data available.
**Command:** `clp .account.limits`
**Expected Output:** Exit 0; output contains three utilization lines for session (5h), weekly (all), and weekly (sonnet) with percentage and reset time.
**Verification:**
- Exit code is 0
- Output contains `Session` or `5h` keyword
- Output contains `Weekly` keyword
- Output contains `%` and reset time
**Pass Criteria:** Exit 0; all three utilization categories visible.
**Source:** [commands.md — .account.limits](../../../../../docs/cli/commands.md#command--12-accountlimits) (FR-18)

---

### IT-2: Verbosity — `v::0` Compact Output

**Goal:** Confirm `v::0` shows bare percentages without labels or reset times.
**Setup:** Active account configured; rate-limit data available.
**Command:** `clp .account.limits v::0`
**Expected Output:** Exit 0; output contains percentage numbers with minimal formatting.
**Verification:**
- Exit code is 0
- Output contains three numeric values (one per utilization category)
- No reset-time phrases in output
**Pass Criteria:** Exit 0; compact output with percentages only.
**Source:** [params.md — v::](../../../../../docs/cli/params.md#parameter--2-verbosity--v)

---

### IT-3: Verbosity — `v::2` Verbose Output

**Goal:** Confirm `v::2` shows all fields including raw values and extended metadata.
**Setup:** Active account configured; rate-limit data available.
**Command:** `clp .account.limits v::2`
**Expected Output:** Exit 0; output contains percentage, reset time, and any additional fields beyond v::1.
**Verification:**
- Exit code is 0
- Output is a superset of v::1 content
- No missing fields at extended verbosity
**Pass Criteria:** Exit 0; extended output visible.
**Source:** [params.md — v::](../../../../../docs/cli/params.md#parameter--2-verbosity--v)

---

### IT-4: Format — `format::json`

**Goal:** Confirm `format::json` returns parseable JSON with utilization fields.
**Setup:** Active account configured; rate-limit data available.
**Command:** `clp .account.limits format::json`
**Expected Output:** Exit 0; stdout is valid JSON containing utilization percentage fields.
**Verification:**
- Exit code is 0
- `echo $output | jq .` succeeds (valid JSON)
- JSON contains numeric percentage fields
**Pass Criteria:** Exit 0; valid JSON output.
**Source:** [params.md — format::](../../../../../docs/cli/params.md#parameter--3-format)

---

### IT-5: Named Account — `name::work`

**Goal:** Confirm `.account.limits name::work` shows limits for the named account, not the active account.
**Setup:** Two accounts configured: active is `personal`, named `work` exists.
**Command:** `clp .account.limits name::work`
**Expected Output:** Exit 0; output reflects `work` account limits.
**Verification:**
- Exit code is 0
- Output is well-formed (same structure as IT-1)
**Pass Criteria:** Exit 0; named account limits displayed.
**Source:** [commands.md — .account.limits](../../../../../docs/cli/commands.md#command--12-accountlimits) (FR-18)

---

### IT-6: Not Found — Unknown Named Account

**Goal:** Confirm that a syntactically valid but non-existent account name exits 2.
**Setup:** `ghost` account does not exist in `~/.claude/accounts/`.
**Command:** `clp .account.limits name::ghost`
**Expected Output:** Exit 2; stderr contains `not found` or `ghost`.
**Verification:**
- Exit code is 2
- Stderr contains `not found` or `ghost`
**Pass Criteria:** Exit 2; not-found is a runtime error (2), not a usage error (1).
**Source:** [commands.md — .account.limits](../../../../../docs/cli/commands.md#command--12-accountlimits)

---

### IT-7: Error Handling — No Active Account

**Goal:** Confirm that running without a configured active account exits 2 with an actionable error.
**Setup:** No `_active` marker set, no active credentials.
**Command:** `clp .account.limits`
**Expected Output:** Exit 2; stderr contains actionable message.
**Verification:**
- Exit code is 2
- Stderr is non-empty and mentions what to do next
**Pass Criteria:** Exit 2; actionable error message shown.
**Source:** [invariant/003_clear_errors.md](../../../../../docs/invariant/003_clear_errors.md)

---

### IT-8: Error Handling — Data Unavailable

**Goal:** Confirm that when rate-limit data cannot be retrieved, the command exits 2, not silently 0.
**Setup:** Active account configured but rate-limit data source unavailable.
**Command:** `clp .account.limits`
**Expected Output:** Exit 2; stderr contains actionable error naming the missing data source.
**Verification:**
- Exit code is 2
- Stderr is non-empty
- Output is NOT a zero-percentage table (not silent success)
**Pass Criteria:** Exit 2; explicit error, never silent zero.
**Source:** [feature/013_account_limits.md](../../../../../docs/feature/013_account_limits.md) AC-04

---

### IT-9: Parameter Validation — Invalid `name::` Characters

**Goal:** Confirm that `name::` with forbidden characters exits 1 (usage error), not 2 (runtime error).
**Setup:** Any environment.
**Command:** `clp .account.limits name::foo/bar`
**Expected Output:** Exit 1; stderr contains `invalid characters`.
**Verification:**
- Exit code is 1 (not 2)
- Stderr contains `invalid characters` or similar
**Pass Criteria:** Exit 1; character validation is a usage error.
**Source:** [params.md — name::](../../../../../docs/cli/params.md#parameter--1-name)
