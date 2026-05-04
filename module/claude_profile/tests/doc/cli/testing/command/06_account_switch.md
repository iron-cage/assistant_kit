# Test: `.account.switch`

Integration test planning for the `.account.switch` command. See [commands.md](../../../../../docs/cli/commands.md#command--5-accountswitch) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Switch overwrites `~/.claude/.credentials.json` with named account | Basic Invocation |
| IT-2 | Switch updates `_active` marker to new name | Marker Update |
| IT-3 | Switch to nonexistent account exits 2 with "not found" message | Error Handling |
| IT-4 | Switch with non-email name exits 1 | Validation |
| IT-5 | `dry::1` prints action without modifying credentials | Dry Run |
| IT-6 | Credential file content matches source account after switch | Data Integrity |
| IT-7 | Other accounts in store are not modified by switch | Isolation |
| IT-8 | Switch to already-active account succeeds (idempotent) | Idempotency |
| IT-9 | Atomic write: no partial file on simulated crash | Atomicity |
| IT-10 | Missing `name::` parameter exits 1 (required) | Required Param |

### Test Coverage Summary

- Basic Invocation: 1 test
- Marker Update: 1 test
- Error Handling: 1 test
- Validation: 1 test
- Dry Run: 1 test
- Data Integrity: 1 test
- Isolation: 1 test
- Idempotency: 1 test
- Atomicity: 1 test
- Required Param: 1 test

**Total:** 10 integration tests

---

### IT-1: Switch overwrites credentials with named account

**Goal:** Verify that switching to a saved account replaces `~/.claude/.credentials.json` with that account's stored credentials.
**Setup:** Two accounts saved in `~/.persistent/claude/credential/`: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. `_active` marker set to `work`. `~/.claude/.credentials.json` contains `work` credentials.
**Command:** `clp .account.switch name::personal@home.com`
**Expected Output:** `switched to 'personal@home.com'` on stdout, exit 0.
**Verification:**
- `~/.claude/.credentials.json` now contains the exact content of `~/.persistent/claude/credential/personal@home.com.credentials.json`
- Exit code is 0
**Pass Criteria:** Exit 0; credentials file replaced with `personal` account content.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-2: Switch updates `_active` marker to new name

**Goal:** Verify that the `_active` marker file is updated to reflect the newly switched account name.
**Setup:** Two accounts saved: `work@acme.com` and `personal@home.com`. `_active` contains `work@acme.com`.
**Command:** `clp .account.switch name::personal@home.com`
**Expected Output:** `switched to 'personal@home.com'` on stdout, exit 0.
**Verification:**
- `~/.persistent/claude/credential/_active` file content is exactly `personal@home.com` (no trailing newline or whitespace beyond what the implementation writes)
- Previous marker value `work@acme.com` is no longer present
**Pass Criteria:** Exit 0; `_active` marker reads `personal@home.com`.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-3: Switch to nonexistent account exits 2

**Goal:** Verify that switching to an account name not present in the store produces exit 2 and an error message containing "not found".
**Setup:** Account store contains only `work@acme.com.credentials.json`. No `ghost@example.com.credentials.json` exists.
**Command:** `clp .account.switch name::ghost@example.com`
**Expected Output:** Error message on stderr containing "not found", exit 2.
**Verification:**
- Exit code is 2
- Stderr output contains the substring "not found"
- `~/.claude/.credentials.json` is unchanged from before the command
- `_active` marker is unchanged
**Pass Criteria:** Exit 2; stderr contains "not found"; no state mutation.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-4: Switch with non-email name exits 1

**Goal:** Verify that a name that is not a valid email address is rejected with exit 1.
**Setup:** Account store contains `work@acme.com.credentials.json`. `_active` is `work@acme.com`.
**Command:** `clp .account.switch name::notanemail`
**Expected Output:** Error message on stderr indicating the name must be a valid email address, exit 1.
**Verification:**
- Exit code is 1
- `~/.claude/.credentials.json` is unchanged
- `_active` marker is unchanged
**Pass Criteria:** Exit 1; no state mutation.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-5: Dry run prints action without modifying credentials

**Goal:** Verify that `dry::1` previews the switch action without modifying the credentials file or the `_active` marker.
**Setup:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`. Record SHA-256 of `~/.claude/.credentials.json` and `_active` before command.
**Command:** `clp .account.switch name::personal@home.com dry::1`
**Expected Output:** `[dry-run] would switch to 'personal@home.com'` on stdout, exit 0.
**Verification:**
- Stdout contains `[dry-run]` and `personal@home.com`
- SHA-256 of `~/.claude/.credentials.json` is identical before and after
- SHA-256 of `~/.persistent/claude/credential/_active` is identical before and after
- Exit code is 0
**Pass Criteria:** Exit 0; stdout contains dry-run message; no files modified.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-6: Credential file content matches source account after switch

**Goal:** Verify byte-for-byte data integrity between the source account file and the resulting credentials file.
**Setup:** Account `personal@home.com` saved with known credential content containing specific `expiresAt`, `oauthAccessToken`, and `claudeAiSubscriptionType` values.
**Command:** `clp .account.switch name::personal@home.com`
**Expected Output:** `switched to 'personal@home.com'`, exit 0.
**Verification:**
- Byte-for-byte comparison: `diff ~/.claude/.credentials.json ~/.persistent/claude/credential/personal@home.com.credentials.json` produces no output
- JSON parse of both files yields identical key-value pairs
**Pass Criteria:** Exit 0; credentials file is byte-identical to source account file.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-7: Other accounts in store not modified by switch

**Goal:** Verify that switching to one account does not alter any other account files in the store.
**Setup:** Three accounts saved: `work@acme.com`, `personal@home.com`, `backup@archive.com`. Record SHA-256 of all three `.credentials.json` files in `~/.persistent/claude/credential/`.
**Command:** `clp .account.switch name::personal@home.com`
**Expected Output:** `switched to 'personal@home.com'`, exit 0.
**Verification:**
- SHA-256 of `work@acme.com.credentials.json` is unchanged
- SHA-256 of `backup@archive.com.credentials.json` is unchanged
- SHA-256 of `personal@home.com.credentials.json` is unchanged
- No new files created in account store (aside from `_active` update)
**Pass Criteria:** Exit 0; all non-target account files unchanged; source account file unchanged.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-8: Switch to already-active account succeeds

**Goal:** Verify that switching to the account that is already active is a no-op success (idempotent operation).
**Setup:** Account `work@acme.com` saved and active. `_active` contains `work@acme.com`. `~/.claude/.credentials.json` matches `work@acme.com` credentials.
**Command:** `clp .account.switch name::work@acme.com`
**Expected Output:** `switched to 'work@acme.com'`, exit 0.
**Verification:**
- Exit code is 0
- `~/.claude/.credentials.json` content is unchanged (still matches `work`)
- `_active` marker still reads `work`
- No error or warning output on stderr
**Pass Criteria:** Exit 0; state unchanged; no errors.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-9: Atomic write produces no partial file on simulated crash

**Goal:** Verify that the atomic write-then-rename strategy uses a `.json.tmp` adjacent file and that no partial credentials file can exist after an interrupted write.
**Setup:** Account `personal@home.com` saved. Set up filesystem observation to detect temporary files. Optionally, use a signal or filesystem constraint to interrupt mid-write.
**Command:** `clp .account.switch name::personal@home.com`
**Expected Output:** `switched to 'personal@home.com'`, exit 0.
**Verification:**
- No `.credentials.json.tmp` file remains in `~/.claude/` after successful completion
- The `.credentials.json` file is either the old content or the new content, never a partial mix
- If the process is interrupted before rename completes, the original `.credentials.json` remains intact
**Pass Criteria:** Exit 0; no `.json.tmp` residue; credentials file is always complete.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-10: Missing `name::` parameter exits 1

**Goal:** Verify that omitting the required `name::` parameter produces exit 1 with a usage error.
**Setup:** Account store contains `work@acme.com` account. No special state needed.
**Command:** `clp .account.switch`
**Expected Output:** Error message on stderr indicating missing required parameter `name::`, exit 1.
**Verification:**
- Exit code is 1
- Stderr contains indication of missing `name` parameter
- `~/.claude/.credentials.json` is unchanged
- `_active` marker is unchanged
**Pass Criteria:** Exit 1; no state mutation; error message references missing parameter.
**Source:** [commands.md — .account.switch](../../../../../docs/cli/commands.md#command--5-accountswitch)
