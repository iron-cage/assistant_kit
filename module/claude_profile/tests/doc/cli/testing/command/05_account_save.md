# Test: `.account.save`

Integration test planning for the `.account.save` command. See [commands.md](../../../../../docs/cli/commands.md#command--4-accountsave) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Save creates credential file in `~/.claude/accounts/` | Basic Invocation |
| IT-2 | Save creates `accounts/` directory if missing | Directory Init |
| IT-3 | Save with existing name overwrites silently | Overwrite |
| IT-4 | Save with invalid name (contains `/`) exits 1 | Validation |
| IT-5 | Save with empty `name::` exits 1 | Validation |
| IT-6 | Save when `~/.claude/.credentials.json` missing exits 2 | Error Handling |
| IT-7 | `dry::1` prints action without creating file | Dry Run |
| IT-8 | `dry::1` then `dry::0` creates file as previewed | Dry Run Fidelity |
| IT-9 | Saved file content matches active credentials exactly | Data Integrity |
| IT-10 | Missing `name::` parameter exits 1 | Required Param |

### Test Coverage Summary

- Basic Invocation: 1 test
- Directory Init: 1 test
- Overwrite: 1 test
- Validation: 2 tests
- Error Handling: 1 test
- Dry Run: 2 tests
- Data Integrity: 1 test
- Required Param: 1 test

**Total:** 10 integration tests

---

### IT-1: Save creates credential file in `~/.claude/accounts/`

**Goal:** Confirm that saving an account creates the named credential file in the accounts directory.
**Setup:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.claude/accounts/` directory (empty).
**Command:** `clp .account.save name::work`
**Expected Output:** stdout: `saved current credentials as 'work'`. File `~/.claude/accounts/work.credentials.json` now exists.
**Verification:**
- Assert exit code is 0
- Assert `~/.claude/accounts/work.credentials.json` exists on disk
- Assert stdout contains `saved current credentials as 'work'`
**Pass Criteria:** Exit 0; credential file created at expected path.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-2: Save creates `accounts/` directory if missing

**Goal:** Confirm the command auto-creates the `~/.claude/accounts/` directory when it does not exist.
**Setup:** Create `~/.claude/.credentials.json` with valid credential content. Ensure `~/.claude/accounts/` does not exist.
**Command:** `clp .account.save name::work`
**Expected Output:** stdout: `saved current credentials as 'work'`. Both `~/.claude/accounts/` directory and `~/.claude/accounts/work.credentials.json` file now exist.
**Verification:**
- Assert exit code is 0
- Assert `~/.claude/accounts/` directory exists
- Assert `~/.claude/accounts/work.credentials.json` file exists
**Pass Criteria:** Exit 0; directory auto-created; credential file created.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-3: Save with existing name overwrites silently

**Goal:** Confirm that saving to an already-existing account name overwrites the file without error or confirmation prompt.
**Setup:** Create `~/.claude/.credentials.json` with credential content V2. Create `~/.claude/accounts/work.credentials.json` with older credential content V1.
**Command:** `clp .account.save name::work`
**Expected Output:** stdout: `saved current credentials as 'work'`. File `~/.claude/accounts/work.credentials.json` now contains V2 content.
**Verification:**
- Assert exit code is 0
- Read `~/.claude/accounts/work.credentials.json`
- Assert file content matches V2 (current `.credentials.json`), not V1 (previous)
- Assert no error or warning on stderr
**Pass Criteria:** Exit 0; file overwritten with current credentials; no error output.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-4: Save with invalid name (contains `/`) exits 1

**Goal:** Confirm that a name containing the filesystem-forbidden character `/` is rejected with exit 1.
**Setup:** Create `~/.claude/.credentials.json` with valid credential content.
**Command:** `clp .account.save name::bad/name`
**Expected Output:** Error message on stderr indicating invalid account name. No file created.
**Verification:**
- Assert exit code is 1
- Assert stderr contains an error message referencing invalid characters
- Assert no file matching `bad` or `name` exists under `~/.claude/accounts/`
**Pass Criteria:** Exit 1; no file created; error message on stderr.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-5: Save with empty `name::` exits 1

**Goal:** Confirm that an empty name value is rejected with exit 1.
**Setup:** Create `~/.claude/.credentials.json` with valid credential content.
**Command:** `clp .account.save name::`
**Expected Output:** Error message indicating the account name must not be empty. No file created.
**Verification:**
- Assert exit code is 1
- Assert stderr contains an error message about empty name
- Assert no new files created in `~/.claude/accounts/`
**Pass Criteria:** Exit 1; no file created; error message about empty name.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-6: Save when `~/.claude/.credentials.json` missing exits 2

**Goal:** Confirm that a missing source credential file causes a runtime error exit 2.
**Setup:** Ensure `~/.claude/.credentials.json` does not exist. Create `~/.claude/accounts/` directory.
**Command:** `clp .account.save name::work`
**Expected Output:** Error message indicating credentials file is unreadable or missing. No account file created.
**Verification:**
- Assert exit code is 2
- Assert stderr contains an error message about credentials
- Assert `~/.claude/accounts/work.credentials.json` does not exist
**Pass Criteria:** Exit 2; no file created; error message about missing credentials.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-7: `dry::1` prints action without creating file

**Goal:** Confirm that dry-run mode previews the save action without writing any file to disk.
**Setup:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.claude/accounts/` directory (empty).
**Command:** `clp .account.save name::work dry::1`
**Expected Output:** stdout: `[dry-run] would save current credentials as 'work'`. No file created.
**Verification:**
- Assert exit code is 0
- Assert stdout contains `[dry-run]`
- Assert stdout contains `would save current credentials as 'work'`
- Assert `~/.claude/accounts/work.credentials.json` does not exist
**Pass Criteria:** Exit 0; dry-run message printed; no file created on disk.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-8: `dry::1` then `dry::0` creates file as previewed

**Goal:** Confirm that executing after a dry run produces the exact action that was previewed.
**Setup:** Create `~/.claude/.credentials.json` with valid credential content. Create `~/.claude/accounts/` directory (empty).
**Command:** `clp .account.save name::work dry::1` followed by `clp .account.save name::work`
**Expected Output:** First command: `[dry-run] would save current credentials as 'work'` (no file). Second command: `saved current credentials as 'work'` (file created).
**Verification:**
- Run dry-run command; assert exit 0 and no file created
- Run real command; assert exit 0
- Assert `~/.claude/accounts/work.credentials.json` now exists
- Assert file content matches `~/.claude/.credentials.json`
**Pass Criteria:** Exit 0 on both; dry run creates nothing; real run creates exactly the previewed file.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-9: Saved file content matches active credentials exactly

**Goal:** Confirm the saved credential file is a byte-exact copy of the source `.credentials.json`.
**Setup:** Create `~/.claude/.credentials.json` with known credential content (e.g., specific JSON with `accessToken`, `refreshToken`, `expiresAt` fields).
**Command:** `clp .account.save name::work`
**Expected Output:** `~/.claude/accounts/work.credentials.json` is a byte-identical copy of `~/.claude/.credentials.json`.
**Verification:**
- Assert exit code is 0
- Read `~/.claude/.credentials.json` into variable SOURCE
- Read `~/.claude/accounts/work.credentials.json` into variable SAVED
- Assert SOURCE == SAVED (byte-equal comparison)
**Pass Criteria:** Exit 0; saved file is byte-identical to source credentials.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)

---

### IT-10: Missing `name::` parameter exits 1

**Goal:** Confirm that omitting the required `name::` parameter is a usage error with exit 1.
**Setup:** Create `~/.claude/.credentials.json` with valid credential content.
**Command:** `clp .account.save`
**Expected Output:** Error message indicating the `name::` parameter is required. No file created.
**Verification:**
- Assert exit code is 1
- Assert stderr contains an error message about missing or required `name` parameter
- Assert no new files created in `~/.claude/accounts/`
**Pass Criteria:** Exit 1; no file created; error message about required parameter.
**Source:** [commands.md — .account.save](../../../../../docs/cli/commands.md#command--4-accountsave)
