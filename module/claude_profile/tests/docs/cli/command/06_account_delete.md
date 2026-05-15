# Test: `.account.delete`

Integration test planning for the `.account.delete` command. See [commands.md](../../../../docs/cli/commands.md#command--6-accountdelete) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Delete removes credential file from account store | Basic Invocation |
| IT-2 | Delete active account exits 0, credential file and _active marker removed | Active Account Deletion |
| IT-3 | Delete nonexistent account exits 2 with "not found" message | Error Handling |
| IT-4 | Delete with non-email name exits 1 | Validation |
| IT-5 | `dry::1` prints action without removing file | Dry Run |
| IT-6 | Delete preserves `_active` marker when deleting non-active account | Marker Preservation |
| IT-7 | Delete preserves other accounts in store | Isolation |
| IT-8 | Delete with empty `name::` exits 1 | Validation |
| IT-9 | After delete, `.accounts` no longer shows deleted account | Post-Condition |
| IT-10 | Missing `name::` parameter exits 1 (required) | Required Param |
| IT-11 | Delete also removes `{name}.claude.json` and `{name}.settings.json` snapshots | Snapshot Cleanup |
| IT-12 | Positional bare arg `alice@oldco.com` (no `name::`) deletes account | Positional Syntax |
| IT-13 | Prefix `old` resolves to `old@archive.com` and deletes | Prefix Resolution |

### Test Coverage Summary

- Basic Invocation: 1 test
- Active Account Deletion: 1 test
- Error Handling: 1 test
- Validation: 2 tests
- Dry Run: 1 test
- Marker Preservation: 1 test
- Isolation: 1 test
- Post-Condition: 1 test
- Required Param: 1 test
- Snapshot Cleanup: 1 test
- Positional Syntax: 1 test
- Prefix Resolution: 1 test

**Total:** 13 integration tests

---

### IT-1: Delete removes credential file from account store

- **Given:** Two accounts saved: `work@acme.com` (active) and `old@archive.com`. Both have `.credentials.json` files in `~/.persistent/claude/credential/`. `_active` marker reads `work@acme.com`.
- **When:** `clp .account.delete name::old@archive.com`
- **Then:** `deleted account 'old@archive.com'` on stdout, exit 0.; account file removed from store
- **Exit:** 0
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-2: Delete active account exits 0, credential file and `_active` marker removed

- **Given:** Account `work@acme.com` saved and active. `_active` marker reads `work@acme.com`.
- **When:** `clp .account.delete name::work@acme.com`
- **Then:** `deleted account 'work@acme.com'` on stdout, exit 0. Both `work@acme.com.credentials.json` and `_active` are absent from the store.
- **Exit:** 0
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-3: Delete nonexistent account exits 2 with "not found"

- **Given:** Account store contains only `work@acme.com.credentials.json`. No `phantom@example.com.credentials.json` exists.
- **When:** `clp .account.delete name::phantom@example.com`
- **Then:** Error message on stderr containing "not found", exit 2.; stderr contains "not found"; no state mutation
- **Exit:** 2
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-4: Delete with non-email name exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`. No special state needed.
- **When:** `clp .account.delete name::notanemail`
- **Then:** Error message on stderr indicating the name must be a valid email address, exit 1.; no state mutation
- **Exit:** 1
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-5: Dry run prints action without removing file

- **Given:** Two accounts saved: `work@acme.com` (active) and `old@archive.com`. Record SHA-256 of `old@archive.com.credentials.json` before command.
- **When:** `clp .account.delete name::old@archive.com dry::1`
- **Then:** `[dry-run] would delete account 'old@archive.com'` on stdout, exit 0.; stdout contains dry-run message; account file not removed
- **Exit:** 0
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-6: Delete preserves `_active` marker when deleting non-active

- **Given:** Two accounts saved: `work@acme.com` (active) and `old@archive.com`. `_active` marker reads `work@acme.com`. Record SHA-256 of `_active` before command.
- **When:** `clp .account.delete name::old@archive.com`
- **Then:** `deleted account 'old@archive.com'`, exit 0.; `_active` marker unchanged
- **Exit:** 0
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-7: Delete preserves other accounts in store

- **Given:** Three accounts saved: `work@acme.com` (active), `personal@home.com`, `old@archive.com`. Record SHA-256 of all three `.credentials.json` files.
- **When:** `clp .account.delete name::old@archive.com`
- **Then:** `deleted account 'old@archive.com'`, exit 0.; only target account removed; all other files intact
- **Exit:** 0
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-8: Delete with empty `name::` exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`. No special state needed.
- **When:** `clp .account.delete name::`
- **Then:** Error message on stderr indicating invalid or empty name, exit 1.; no state mutation
- **Exit:** 1
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-9: After delete, `.accounts` no longer shows deleted account

- **Given:** Three accounts saved: `work@acme.com` (active), `personal@home.com`, `old@archive.com`. Confirm `old@archive.com` appears in `.accounts` output before deletion.
- **When:** `clp .account.delete name::old@archive.com` then `clp .accounts`
- **Then:** Delete outputs `deleted account 'old@archive.com'`, exit 0. Subsequent accounts output contains `work@acme.com` and `personal@home.com` but not `old@archive.com`.; for both commands; deleted account absent from accounts output
- **Exit:** 0
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-10: Missing `name::` parameter exits 1

- **Given:** Account store contains `work@acme.com` account. No special state needed.
- **When:** `clp .account.delete`
- **Then:** Error message on stderr indicating missing required parameter `name::`, exit 1.; no state mutation; error message references missing parameter
- **Exit:** 1
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-11: Delete removes snapshot files alongside credential file

- **Given:** Account `old@archive.com` saved (non-active). Store contains `old@archive.com.credentials.json`, `old@archive.com.claude.json`, and `old@archive.com.settings.json`. Active account is `work@acme.com`.
- **When:** `clp .account.delete name::old@archive.com`
- **Then:** `deleted account 'old@archive.com'`, exit 0. All three files — `.credentials.json`, `.claude.json`, `.settings.json` — are absent from the store. No error even if either snapshot file was missing.; credential and both snapshot files removed; no error for absent snapshots
- **Exit:** 0
- **Source:** [commands.md — .account.delete](../../../../docs/cli/commands.md#command--6-accountdelete)

---

### IT-12: Positional bare arg deletes account

- **Given:** Two accounts saved: `work@acme.com` (active) and `old@archive.com`.
- **When:** `clp .account.delete old@archive.com` (no `name::` prefix)
- **Then:** Exits 0; `deleted account 'old@archive.com'` on stdout; `old@archive.com.credentials.json` absent from store.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-02](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-13: Prefix resolves to single account

- **Given:** Two accounts saved: `old@archive.com` and `work@acme.com` (active).
- **When:** `clp .account.delete old` (prefix form, no `@`)
- **Then:** Exits 0; `deleted account 'old@archive.com'` on stdout; `old@archive.com.credentials.json` absent from store.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-05](../../../../docs/feature/015_name_shortcut_syntax.md)
