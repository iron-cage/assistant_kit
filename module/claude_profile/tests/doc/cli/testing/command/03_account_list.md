# Test: `.account.list`

Integration test planning for the `.account.list` command. See [commands.md](../../../../../docs/cli/commands.md#command--3-accountlist) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Lists all accounts when multiple exist | Basic Invocation |
| IT-2 | Active account marked with `<- active` indicator | Output Format |
| IT-3 | Empty account store returns empty output, exit 0 | Empty State |
| IT-4 | `v::0` shows bare account names only | Verbosity |
| IT-5 | `v::1` shows names with active indicator and subscription type | Verbosity |
| IT-6 | `v::2` shows full metadata including tier and expiry | Verbosity |
| IT-7 | `format::json` returns valid JSON array | Output Format |
| IT-8 | Accounts sorted alphabetically by name | Output Order |
| IT-9 | Credential store missing returns empty, not error | Edge Case |
| IT-10 | `format::json` with `v::0` returns full JSON | Interaction |
| IT-11 | `name::EMAIL` → single account status view (Account:/Token: labels) | Named Account |
| IT-12 | `name::EMAIL` → output identical to `.account.status name::EMAIL` | Named Account |
| IT-13 | `name::` account not in store → exit 2 | Named Account / Error |
| IT-14 | `name::notanemail` → exit 1 | Named Account / Validation |

### Test Coverage Summary

- Basic Invocation: 1 test
- Output Format: 2 tests
- Empty State: 1 test
- Verbosity: 3 tests
- Output Order: 1 test
- Edge Case: 1 test
- Interaction: 1 test
- Named Account: 3 tests (IT-11, IT-12, IT-13)
- Named Account / Validation: 1 test (IT-14)

**Total:** 14 integration tests

---

### IT-1: Lists all accounts when multiple exist

**Goal:** Confirm all saved accounts appear in the listing when multiple credential files exist.
**Setup:** Create `~/.persistent/claude/credential/` with two credential files: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Set `work@acme.com` as the active account via the `_active` marker.
**Command:** `clp .account.list`
**Expected Output:** Output contains both `work@acme.com` and `personal@home.com` account entries.
**Verification:**
- Capture stdout
- Assert stdout contains `work@acme.com`
- Assert stdout contains `personal@home.com`
- Assert exactly 2 account entries appear in output
**Pass Criteria:** Exit 0; both accounts listed.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-2: Active account marked with `<- active` indicator

**Goal:** Confirm the currently active account is visually distinguished with the `<- active` marker.
**Setup:** Create `~/.persistent/claude/credential/` with `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Set `work@acme.com` as active via the `_active` marker.
**Command:** `clp .account.list`
**Expected Output:** The `work@acme.com` line includes `<- active`; the `personal@home.com` line does not.
**Verification:**
- Capture stdout
- Assert the line containing `work@acme.com` also contains `<- active`
- Assert the line containing `personal@home.com` does not contain `<- active`
**Pass Criteria:** Exit 0; active indicator appears only on the active account.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-3: Empty account store returns empty output, exit 0

**Goal:** Confirm that an empty account store produces no output lines and exits successfully.
**Setup:** Create `~/.persistent/claude/credential/` as an empty directory. No credential files present.
**Command:** `clp .account.list`
**Expected Output:** Empty stdout (no lines).
**Verification:**
- Capture stdout
- Assert stdout is empty or contains only whitespace
- Assert exit code is 0
**Pass Criteria:** Exit 0; stdout is empty.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-4: `v::0` shows bare account names only

**Goal:** Confirm verbosity level 0 strips all metadata and shows only account names, one per line.
**Setup:** Create `~/.persistent/claude/credential/` with `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Set `work@acme.com` as active.
**Command:** `clp .account.list v::0`
**Expected Output:** Two lines: `work@acme.com` and `personal@home.com` (no `<- active`, no subscription type, no tier, no expiry).
**Verification:**
- Capture stdout
- Assert stdout contains `work@acme.com` on its own line
- Assert stdout contains `personal@home.com` on its own line
- Assert stdout does not contain `<- active`
- Assert stdout does not contain subscription type labels (e.g., `max`, `pro`)
**Pass Criteria:** Exit 0; bare names only, no metadata.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-5: `v::1` shows names with active indicator and subscription type

**Goal:** Confirm verbosity level 1 (default) includes account name, active marker, and subscription type.
**Setup:** Create `~/.persistent/claude/credential/` with `work@acme.com.credentials.json` (subscription type `max`) and `personal@home.com.credentials.json` (subscription type `pro`). Set `work@acme.com` as active.
**Command:** `clp .account.list v::1`
**Expected Output:** Lines like `work@acme.com <- active (max, standard, ...)` and `personal@home.com (pro, standard, ...)`.
**Verification:**
- Capture stdout
- Assert the `work@acme.com` line contains `<- active`
- Assert the `work@acme.com` line contains a subscription type indicator (e.g., `max`)
- Assert the `personal@home.com` line contains a subscription type indicator (e.g., `pro`)
- Assert expiry information is present on both lines
**Pass Criteria:** Exit 0; names with active indicator and subscription type displayed.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-6: `v::2` shows full metadata including tier and expiry

**Goal:** Confirm verbosity level 2 displays all available metadata: name, active marker, subscription type, rate-limit tier, and token expiry time.
**Setup:** Create `~/.persistent/claude/credential/` with `work@acme.com.credentials.json` containing full metadata (subscription type `max`, rate-limit tier `standard`, valid expiry timestamp). Set `work@acme.com` as active.
**Command:** `clp .account.list v::2`
**Expected Output:** Expanded output with all metadata fields visible, including tier and expiry details beyond what `v::1` shows.
**Verification:**
- Capture stdout
- Assert output contains subscription type (e.g., `max`)
- Assert output contains rate-limit tier (e.g., `standard`)
- Assert output contains token expiry information
- Assert output contains more detail than `v::1` output for the same data
**Pass Criteria:** Exit 0; full metadata including tier and expiry visible.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-7: `format::json` returns valid JSON array

**Goal:** Confirm JSON format output is a valid JSON array of account objects with expected fields.
**Setup:** Create `~/.persistent/claude/credential/` with `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. Set `work@acme.com` as active.
**Command:** `clp .account.list format::json`
**Expected Output:** A valid JSON array containing objects with at least: `name`, `subscription_type`, `rate_limit_tier`, `expires_at_ms`, `is_active`.
**Verification:**
- Capture stdout
- Parse stdout as JSON (must not fail)
- Assert parsed value is a JSON array
- Assert array length is 2
- Assert each element has `name` field (string)
- Assert each element has `is_active` field (boolean)
- Assert exactly one element has `is_active: true`
**Pass Criteria:** Exit 0; valid JSON array with expected schema.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-8: Accounts sorted alphabetically by name

**Goal:** Confirm accounts are listed in alphabetical order by name regardless of file creation order.
**Setup:** Create accounts in reverse alphabetical order: `work@acme.com.credentials.json` first, then `alpha@test.com.credentials.json`, then `personal@home.com.credentials.json`.
**Command:** `clp .account.list`
**Expected Output:** Output lines ordered: `alpha@test.com`, `personal@home.com`, `work@acme.com`.
**Verification:**
- Capture stdout
- Extract account names from output lines
- Assert the sequence is `alpha@test.com`, `personal@home.com`, `work@acme.com` (alphabetical order)
**Pass Criteria:** Exit 0; accounts appear in alphabetical order.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-9: Credential store missing returns empty, not error

**Goal:** Confirm that a missing `~/.persistent/claude/credential/` directory is treated as empty, not as an error.
**Setup:** Ensure `~/.persistent/claude/credential/` does not exist (delete if present). Ensure `~/.claude/` itself exists.
**Command:** `clp .account.list`
**Expected Output:** Empty stdout; no error message on stderr.
**Verification:**
- Capture stdout and stderr
- Assert stdout is empty
- Assert stderr is empty (no error message)
- Assert exit code is 0
**Pass Criteria:** Exit 0; empty output, no error.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-10: `format::json` with `v::0` returns full JSON

**Goal:** Confirm that `format::json` overrides verbosity, returning complete JSON regardless of `v::0`.
**Setup:** Create `~/.persistent/claude/credential/` with `work@acme.com.credentials.json`. Set `work@acme.com` as active.
**Command:** `clp .account.list format::json v::0`
**Expected Output:** Full JSON array identical to `format::json` without `v::0` -- format takes precedence over verbosity.
**Verification:**
- Capture stdout of `clp .account.list format::json v::0`
- Capture stdout of `clp .account.list format::json`
- Parse both as JSON
- Assert both produce identical JSON structure (same fields, same values)
**Pass Criteria:** Exit 0; JSON output is complete despite `v::0`.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-11: `name::EMAIL` → single account status view

**Goal:** Confirm that `.account.list name::EMAIL` shows a single-account status view with `Account:` and `Token:` labels — not a list.
**Setup:** Create `work@acme.com` as active with a valid far-future token. Create `personal@home.com` as non-active with an expired token.
**Command:** `clp .account.list name::personal@home.com`
**Expected Output:** Contains `personal@home.com`, `Account:`, `Token:`, `expired`. Does NOT show `work@acme.com`.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `personal@home.com`
- Assert stdout contains `Account:`
- Assert stdout contains `Token:`
- Assert stdout contains `expired`
- Assert stdout does NOT contain `work@acme.com`
**Pass Criteria:** Exit 0; single-account status view shown for named account.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-12: `name::EMAIL` output identical to `.account.status name::EMAIL`

**Goal:** Confirm that `.account.list name::X` and `.account.status name::X` produce byte-for-byte identical output.
**Setup:** Create `work@acme.com` as active with valid credentials. Create `personal@home.com` as non-active.
**Commands:** Run both `clp .account.list name::personal@home.com` and `clp .account.status name::personal@home.com`.
**Expected Output:** Both commands produce exactly the same stdout.
**Verification:**
- Capture stdout of both commands
- Assert both exit with code 0
- Assert the two stdout strings are identical
**Pass Criteria:** Exit 0; output is identical between the two commands.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-13: `name::` account not in store → exit 2

**Goal:** Confirm that querying a non-existent account name via `.account.list name::` produces exit 2.
**Setup:** Create `work@acme.com` as active. Do NOT create `ghost@example.com`.
**Command:** `clp .account.list name::ghost@example.com`
**Expected Output:** Empty stdout; stderr contains `not found` or `ghost@example.com`.
**Verification:**
- Assert exit code is 2
- Assert stderr contains `not found` or `ghost@example.com`
- Assert stdout is empty
**Pass Criteria:** Exit 2; unknown account reported clearly.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-14: `name::notanemail` → exit 1

**Goal:** Confirm that a `name::` value that is not a valid email address is rejected before any lookup.
**Setup:** Any valid active account exists.
**Command:** `clp .account.list name::notanemail`
**Expected Output:** Exit 1; stderr contains validation error.
**Verification:**
- Assert exit code is 1
**Pass Criteria:** Exit 1; non-email name rejected before lookup.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)
