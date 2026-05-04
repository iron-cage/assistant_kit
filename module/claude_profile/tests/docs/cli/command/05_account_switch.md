# Test: `.account.switch`

Integration test planning for the `.account.switch` command. See [commands.md](../../../../docs/cli/commands.md#command--5-accountswitch) for specification.

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

- **Given:** Two accounts saved in `~/.persistent/claude/credential/`: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. `_active` marker set to `work`. `~/.claude/.credentials.json` contains `work` credentials.
- **When:** `clp .account.switch name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; credentials file replaced with `personal` account content
- **Exit:** 0
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-2: Switch updates `_active` marker to new name

- **Given:** Two accounts saved: `work@acme.com` and `personal@home.com`. `_active` contains `work@acme.com`.
- **When:** `clp .account.switch name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; `_active` marker reads `personal@home.com`
- **Exit:** 0
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-3: Switch to nonexistent account exits 2

- **Given:** Account store contains only `work@acme.com.credentials.json`. No `ghost@example.com.credentials.json` exists.
- **When:** `clp .account.switch name::ghost@example.com`
- **Then:** Error message on stderr containing "not found", exit 2.; stderr contains "not found"; no state mutation
- **Exit:** 2
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-4: Switch with non-email name exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`. `_active` is `work@acme.com`.
- **When:** `clp .account.switch name::notanemail`
- **Then:** Error message on stderr indicating the name must be a valid email address, exit 1.; no state mutation
- **Exit:** 1
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-5: Dry run prints action without modifying credentials

- **Given:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`. Record SHA-256 of `~/.claude/.credentials.json` and `_active` before command.
- **When:** `clp .account.switch name::personal@home.com dry::1`
- **Then:** `[dry-run] would switch to 'personal@home.com'` on stdout, exit 0.; stdout contains dry-run message; no files modified
- **Exit:** 0
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-6: Credential file content matches source account after switch

- **Given:** Account `personal@home.com` saved with known credential content containing specific `expiresAt`, `oauthAccessToken`, and `claudeAiSubscriptionType` values.
- **When:** `clp .account.switch name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; credentials file is byte-identical to source account file
- **Exit:** 0
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-7: Other accounts in store not modified by switch

- **Given:** Three accounts saved: `work@acme.com`, `personal@home.com`, `backup@archive.com`. Record SHA-256 of all three `.credentials.json` files in `~/.persistent/claude/credential/`.
- **When:** `clp .account.switch name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; all non-target account files unchanged; source account file unchanged
- **Exit:** 0
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-8: Switch to already-active account succeeds

- **Given:** Account `work@acme.com` saved and active. `_active` contains `work@acme.com`. `~/.claude/.credentials.json` matches `work@acme.com` credentials.
- **When:** `clp .account.switch name::work@acme.com`
- **Then:** `switched to 'work@acme.com'`, exit 0.; state unchanged; no errors
- **Exit:** 0
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-9: Atomic write produces no partial file on simulated crash

- **Given:** Account `personal@home.com` saved. Set up filesystem observation to detect temporary files. Optionally, use a signal or filesystem constraint to interrupt mid-write.
- **When:** `clp .account.switch name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; no `.json.tmp` residue; credentials file is always complete
- **Exit:** 0
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)

---

### IT-10: Missing `name::` parameter exits 1

- **Given:** Account store contains `work@acme.com` account. No special state needed.
- **When:** `clp .account.switch`
- **Then:** Error message on stderr indicating missing required parameter `name::`, exit 1.; no state mutation; error message references missing parameter
- **Exit:** 1
- **Source:** [commands.md — .account.switch](../../../../docs/cli/commands.md#command--5-accountswitch)
