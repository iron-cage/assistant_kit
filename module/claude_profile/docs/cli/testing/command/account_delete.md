# Test: `.account.delete`

Integration test planning for the `.account.delete` command. See [commands.md](../../commands.md#command--6-accountdelete) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Delete removes credential file from account store | Basic Invocation |
| IT-2 | Delete active account exits 2 with "cannot delete active" message | Active Guard |
| IT-3 | Delete nonexistent account exits 2 with "not found" message | Error Handling |
| IT-4 | Delete with invalid name (contains `*`) exits 1 | Validation |
| IT-5 | `dry::1` prints action without removing file | Dry Run |
| IT-6 | Delete preserves `_active` marker when deleting non-active account | Marker Preservation |
| IT-7 | Delete preserves other accounts in store | Isolation |
| IT-8 | Delete with empty `name::` exits 1 | Validation |
| IT-9 | After delete, `.account.list` no longer shows deleted account | Post-Condition |
| IT-10 | Missing `name::` parameter exits 1 (required) | Required Param |

## Test Coverage Summary

- Basic Invocation: 1 test
- Active Guard: 1 test
- Error Handling: 1 test
- Validation: 2 tests
- Dry Run: 1 test
- Marker Preservation: 1 test
- Isolation: 1 test
- Post-Condition: 1 test
- Required Param: 1 test

**Total:** 10 integration tests

---

### IT-1: Delete removes credential file from account store

**Goal:** Verify that deleting a non-active account removes its `.credentials.json` file from the account store.
**Setup:** Two accounts saved: `work` (active) and `old`. Both have `.credentials.json` files in `~/.claude/accounts/`. `_active` marker reads `work`.
**Command:** `clp .account.delete name::old`
**Expected Output:** `deleted account 'old'` on stdout, exit 0.
**Verification:**
- `~/.claude/accounts/old.credentials.json` no longer exists on disk
- Exit code is 0
**Pass Criteria:** Exit 0; account file removed from store.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-2: Delete active account exits 2 with "cannot delete active"

**Goal:** Verify that attempting to delete the currently active account is refused with exit 2 and an error mentioning "cannot delete active".
**Setup:** Account `work` saved and active. `_active` marker reads `work`.
**Command:** `clp .account.delete name::work`
**Expected Output:** Error message on stderr containing "cannot delete active", exit 2.
**Verification:**
- Exit code is 2
- Stderr contains the substring "cannot delete active"
- `~/.claude/accounts/work.credentials.json` still exists on disk
- `_active` marker is unchanged
**Pass Criteria:** Exit 2; stderr contains "cannot delete active"; no state mutation.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-3: Delete nonexistent account exits 2 with "not found"

**Goal:** Verify that deleting an account name not present in the store produces exit 2 and an error containing "not found".
**Setup:** Account store contains only `work.credentials.json`. No `phantom.credentials.json` exists.
**Command:** `clp .account.delete name::phantom`
**Expected Output:** Error message on stderr containing "not found", exit 2.
**Verification:**
- Exit code is 2
- Stderr contains the substring "not found"
- No files in account store are modified or removed
**Pass Criteria:** Exit 2; stderr contains "not found"; no state mutation.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-4: Delete with invalid name exits 1

**Goal:** Verify that a name containing the asterisk character (`*`) is rejected as invalid with exit 1.
**Setup:** Account store contains `work.credentials.json`. No special state needed.
**Command:** `clp .account.delete name::bad*name`
**Expected Output:** Error message on stderr indicating invalid name, exit 1.
**Verification:**
- Exit code is 1
- No files in account store are modified or removed
- `_active` marker is unchanged
**Pass Criteria:** Exit 1; no state mutation.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-5: Dry run prints action without removing file

**Goal:** Verify that `dry::1` previews the delete action without actually removing the account file.
**Setup:** Two accounts saved: `work` (active) and `old`. Record SHA-256 of `old.credentials.json` before command.
**Command:** `clp .account.delete name::old dry::1`
**Expected Output:** `[dry-run] would delete account 'old'` on stdout, exit 0.
**Verification:**
- Stdout contains `[dry-run]` and `old`
- `~/.claude/accounts/old.credentials.json` still exists on disk
- SHA-256 of `old.credentials.json` is identical before and after
- Exit code is 0
**Pass Criteria:** Exit 0; stdout contains dry-run message; account file not removed.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-6: Delete preserves `_active` marker when deleting non-active

**Goal:** Verify that deleting a non-active account does not alter the `_active` marker file.
**Setup:** Two accounts saved: `work` (active) and `old`. `_active` marker reads `work`. Record SHA-256 of `_active` before command.
**Command:** `clp .account.delete name::old`
**Expected Output:** `deleted account 'old'`, exit 0.
**Verification:**
- SHA-256 of `~/.claude/accounts/_active` is identical before and after
- `_active` file content still reads `work`
**Pass Criteria:** Exit 0; `_active` marker unchanged.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-7: Delete preserves other accounts in store

**Goal:** Verify that deleting one account does not modify or remove any other account files.
**Setup:** Three accounts saved: `work` (active), `personal`, `old`. Record SHA-256 of all three `.credentials.json` files.
**Command:** `clp .account.delete name::old`
**Expected Output:** `deleted account 'old'`, exit 0.
**Verification:**
- SHA-256 of `work.credentials.json` is unchanged
- SHA-256 of `personal.credentials.json` is unchanged
- `old.credentials.json` no longer exists
- No new files created in account store
**Pass Criteria:** Exit 0; only target account removed; all other files intact.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-8: Delete with empty `name::` exits 1

**Goal:** Verify that providing an empty string for `name::` is rejected as invalid with exit 1.
**Setup:** Account store contains `work.credentials.json`. No special state needed.
**Command:** `clp .account.delete name::`
**Expected Output:** Error message on stderr indicating invalid or empty name, exit 1.
**Verification:**
- Exit code is 1
- No files in account store are modified or removed
**Pass Criteria:** Exit 1; no state mutation.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-9: After delete, `.account.list` no longer shows deleted account

**Goal:** Verify end-to-end consistency: after deleting an account, it no longer appears in the `.account.list` output.
**Setup:** Three accounts saved: `work` (active), `personal`, `old`. Confirm `old` appears in `.account.list` output before deletion.
**Command:** `clp .account.delete name::old` then `clp .account.list`
**Expected Output:** Delete outputs `deleted account 'old'`, exit 0. Subsequent list output contains `work` and `personal` but not `old`.
**Verification:**
- First command exits 0
- Second command exits 0
- Second command stdout contains `work` and `personal`
- Second command stdout does not contain `old`
**Pass Criteria:** Exit 0 for both commands; deleted account absent from list output.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)

---

### IT-10: Missing `name::` parameter exits 1

**Goal:** Verify that omitting the required `name::` parameter produces exit 1 with a usage error.
**Setup:** Account store contains `work` account. No special state needed.
**Command:** `clp .account.delete`
**Expected Output:** Error message on stderr indicating missing required parameter `name::`, exit 1.
**Verification:**
- Exit code is 1
- Stderr contains indication of missing `name` parameter
- No files in account store are modified or removed
**Pass Criteria:** Exit 1; no state mutation; error message references missing parameter.
**Source:** [commands.md — .account.delete](../../commands.md#command--6-accountdelete)
