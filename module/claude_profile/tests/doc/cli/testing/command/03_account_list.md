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
| IT-9 | Accounts dir missing returns empty, not error | Edge Case |
| IT-10 | `format::json` with `v::0` returns full JSON | Interaction |

### Test Coverage Summary

- Basic Invocation: 1 test
- Output Format: 2 tests
- Empty State: 1 test
- Verbosity: 3 tests
- Output Order: 1 test
- Edge Case: 1 test
- Interaction: 1 test

**Total:** 10 integration tests

---

### IT-1: Lists all accounts when multiple exist

**Goal:** Confirm all saved accounts appear in the listing when multiple credential files exist.
**Setup:** Create `~/.persistent/claude/credential/` with two credential files: `work.credentials.json` and `personal.credentials.json`. Set `work` as the active account via the `_active` marker.
**Command:** `clp .account.list`
**Expected Output:** Output contains both `work` and `personal` account entries.
**Verification:**
- Capture stdout
- Assert stdout contains the string `work`
- Assert stdout contains the string `personal`
- Assert exactly 2 account entries appear in output
**Pass Criteria:** Exit 0; both accounts listed.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-2: Active account marked with `<- active` indicator

**Goal:** Confirm the currently active account is visually distinguished with the `<- active` marker.
**Setup:** Create `~/.persistent/claude/credential/` with `work.credentials.json` and `personal.credentials.json`. Set `work` as active via the `_active` marker.
**Command:** `clp .account.list`
**Expected Output:** The `work` line includes `<- active`; the `personal` line does not.
**Verification:**
- Capture stdout
- Assert the line containing `work` also contains `<- active`
- Assert the line containing `personal` does not contain `<- active`
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
**Setup:** Create `~/.persistent/claude/credential/` with `work.credentials.json` and `personal.credentials.json`. Set `work` as active.
**Command:** `clp .account.list v::0`
**Expected Output:** Two lines, each containing only a bare account name (no `<- active`, no subscription type, no tier, no expiry).
**Verification:**
- Capture stdout
- Assert stdout contains `work` on its own line
- Assert stdout contains `personal` on its own line
- Assert stdout does not contain `<- active`
- Assert stdout does not contain subscription type labels (e.g., `max`, `pro`)
**Pass Criteria:** Exit 0; bare names only, no metadata.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-5: `v::1` shows names with active indicator and subscription type

**Goal:** Confirm verbosity level 1 (default) includes account name, active marker, and subscription type.
**Setup:** Create `~/.persistent/claude/credential/` with `work.credentials.json` (subscription type `max`) and `personal.credentials.json` (subscription type `pro`). Set `work` as active.
**Command:** `clp .account.list v::1`
**Expected Output:** Lines like `work <- active (max, standard, expires in 47m)` and `personal (pro, standard, expires in 3h12m)`.
**Verification:**
- Capture stdout
- Assert the `work` line contains `<- active`
- Assert the `work` line contains a subscription type indicator (e.g., `max`)
- Assert the `personal` line contains a subscription type indicator (e.g., `pro`)
- Assert expiry information is present on both lines
**Pass Criteria:** Exit 0; names with active indicator and subscription type displayed.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-6: `v::2` shows full metadata including tier and expiry

**Goal:** Confirm verbosity level 2 displays all available metadata: name, active marker, subscription type, rate-limit tier, and token expiry time.
**Setup:** Create `~/.persistent/claude/credential/` with `work.credentials.json` containing full metadata (subscription type `max`, rate-limit tier `standard`, valid expiry timestamp). Set `work` as active.
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
**Setup:** Create `~/.persistent/claude/credential/` with `work.credentials.json` and `personal.credentials.json`. Set `work` as active.
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
**Setup:** Create accounts in reverse alphabetical order: `work.credentials.json` first, then `alpha.credentials.json`, then `personal.credentials.json`.
**Command:** `clp .account.list`
**Expected Output:** Output lines ordered: `alpha`, `personal`, `work`.
**Verification:**
- Capture stdout
- Extract account names from output lines
- Assert the sequence is `alpha`, `personal`, `work` (alphabetical order)
**Pass Criteria:** Exit 0; accounts appear in alphabetical order.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)

---

### IT-9: Accounts dir missing returns empty, not error

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
**Setup:** Create `~/.persistent/claude/credential/` with `work.credentials.json`. Set `work` as active.
**Command:** `clp .account.list format::json v::0`
**Expected Output:** Full JSON array identical to `format::json` without `v::0` -- format takes precedence over verbosity.
**Verification:**
- Capture stdout of `clp .account.list format::json v::0`
- Capture stdout of `clp .account.list format::json`
- Parse both as JSON
- Assert both produce identical JSON structure (same fields, same values)
**Pass Criteria:** Exit 0; JSON output is complete despite `v::0`.
**Source:** [commands.md — .account.list](../../../../../docs/cli/commands.md#command--3-accountlist)
