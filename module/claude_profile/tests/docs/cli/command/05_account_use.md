# Test: `.account.use`

Integration test planning for the `.account.use` command. See [commands.md](../../../../docs/cli/commands.md#command--5-accountuse) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Use overwrites `~/.claude/.credentials.json` with named account | Basic Invocation |
| IT-2 | Use updates `_active` marker to new name | Marker Update |
| IT-3 | Use with nonexistent account exits 2 with "not found" message | Error Handling |
| IT-4 | Use with non-email name exits 1 | Validation |
| IT-5 | `dry::1` prints action without modifying credentials | Dry Run |
| IT-6 | Credential file content matches source account after use | Data Integrity |
| IT-7 | Other accounts in store are not modified by use | Isolation |
| IT-8 | Use with already-active account succeeds (idempotent) | Idempotency |
| IT-9 | Atomic write: no partial file on simulated crash | Atomicity |
| IT-10 | Missing `name::` parameter exits 1 (required) | Required Param |
| IT-11 | `.credentials.status` shows new account email after use | Email Consistency |
| IT-12 | Use with path-unsafe chars in email local part exits 1 | Validation |
| IT-13 | Positional bare arg `alice@home.com` (no `name::`) switches account | Positional Syntax |
| IT-14 | Prefix `i3` resolves to `i3@wbox.pro` and switches account | Prefix Resolution |
| IT-15 | Ambiguous prefix matches two accounts → exit 1 | Prefix Resolution / Error |

### Test Coverage Summary

- Basic Invocation: 1 test
- Marker Update: 1 test
- Error Handling: 1 test
- Validation: 2 tests
- Dry Run: 1 test
- Data Integrity: 1 test
- Isolation: 1 test
- Idempotency: 1 test
- Atomicity: 1 test
- Required Param: 1 test
- Email Consistency: 1 test
- Positional Syntax: 1 test
- Prefix Resolution: 2 tests

**Total:** 15 integration tests

---

### IT-1: Use overwrites credentials with named account

- **Given:** Two accounts saved in `~/.persistent/claude/credential/`: `work@acme.com.credentials.json` and `personal@home.com.credentials.json`. `_active` marker set to `work`. `~/.claude/.credentials.json` contains `work` credentials.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; credentials file replaced with `personal` account content
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-2: Use updates `_active` marker to new name

- **Given:** Two accounts saved: `work@acme.com` and `personal@home.com`. `_active` contains `work@acme.com`.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'` on stdout, exit 0.; `_active` marker reads `personal@home.com`
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-3: Use with nonexistent account exits 2

- **Given:** Account store contains only `work@acme.com.credentials.json`. No `ghost@example.com.credentials.json` exists.
- **When:** `clp .account.use name::ghost@example.com`
- **Then:** Error message on stderr containing "not found", exit 2.; stderr contains "not found"; no state mutation
- **Exit:** 2
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-4: Use with non-email name exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`. `_active` is `work@acme.com`.
- **When:** `clp .account.use name::notanemail`
- **Then:** Error message on stderr indicating the name must be a valid email address, exit 1.; no state mutation
- **Exit:** 1
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-5: Dry run prints action without modifying credentials

- **Given:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`. Record SHA-256 of `~/.claude/.credentials.json` and `_active` before command.
- **When:** `clp .account.use name::personal@home.com dry::1`
- **Then:** `[dry-run] would switch to 'personal@home.com'` on stdout, exit 0.; stdout contains dry-run message; no files modified
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-6: Credential file content matches source account after use

- **Given:** Account `personal@home.com` saved with known credential content containing specific `expiresAt`, `oauthAccessToken`, and `claudeAiSubscriptionType` values.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; credentials file is byte-identical to source account file
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-7: Other accounts in store not modified by use

- **Given:** Three accounts saved: `work@acme.com`, `personal@home.com`, `backup@archive.com`. Record SHA-256 of all three `.credentials.json` files in `~/.persistent/claude/credential/`.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; all non-target account files unchanged; source account file unchanged
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-8: Use with already-active account succeeds

- **Given:** Account `work@acme.com` saved and active. `_active` contains `work@acme.com`. `~/.claude/.credentials.json` matches `work@acme.com` credentials.
- **When:** `clp .account.use name::work@acme.com`
- **Then:** `switched to 'work@acme.com'`, exit 0.; state unchanged; no errors
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-9: Atomic write produces no partial file on simulated crash

- **Given:** Account `personal@home.com` saved. Set up filesystem observation to detect temporary files. Optionally, use a signal or filesystem constraint to interrupt mid-write.
- **When:** `clp .account.use name::personal@home.com`
- **Then:** `switched to 'personal@home.com'`, exit 0.; no `.json.tmp` residue; credentials file is always complete
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-10: Missing `name::` parameter exits 1

- **Given:** Account store contains `work@acme.com` account. No special state needed.
- **When:** `clp .account.use`
- **Then:** Error message on stderr indicating missing required parameter `name::`, exit 1.; no state mutation; error message references missing parameter
- **Exit:** 1
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse)

---

### IT-11: `.credentials.status` shows new account email after use

- **Given:** Two accounts saved via `.account.save` in order: first `work@acme.com` (with `emailAddress: "work@acme.com"` in its `~/.claude.json` snapshot), then `personal@home.com` (with `emailAddress: "personal@home.com"` in its snapshot). After both saves, `personal@home.com` is the active account and `~/.claude.json` contains `"emailAddress": "personal@home.com"`.
- **When:** `clp .account.use name::work@acme.com` then `clp .credentials.status`
- **Then:** `.credentials.status` output contains `Email: work@acme.com` (not `personal@home.com`). Exit 0.; `~/.claude.json` restored from `work@acme.com`'s snapshot; `credentials.status Email:` field reflects the switched-to account
- **Exit:** 0
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse), [004_account_use.md AC-05](../../../../docs/feature/004_account_use.md)

---

### IT-12: Use with path-unsafe chars in email local part exits 1

- **Given:** Any account store state (the name is rejected before any store lookup).
- **When:** `clp .account.use name::a/b@c.com`
- **Then:** Error message on stderr indicating the name contains path-unsafe characters, exit 1. No filesystem modification.
- **Exit:** 1
- **Source:** [commands.md — .account.use](../../../../docs/cli/commands.md#command--5-accountuse), [004_account_use.md AC-06](../../../../docs/feature/004_account_use.md), [aw11 in account_mutations_test.rs]

---

### IT-13: Positional bare arg switches account

- **Given:** Two accounts saved: `work@acme.com` (active) and `personal@home.com`.
- **When:** `clp .account.use personal@home.com` (no `name::` prefix)
- **Then:** Exits 0; `switched to 'personal@home.com'` on stdout; outcome identical to `clp .account.use name::personal@home.com`.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-01](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-14: Prefix resolves to single account

- **Given:** Two accounts saved: `i3@wbox.pro` and `i5@wbox.pro`. `_active` = `i5@wbox.pro`.
- **When:** `clp .account.use i3` (prefix form, no `@`)
- **Then:** Exits 0; `switched to 'i3@wbox.pro'` on stdout; credentials overwritten with `i3@wbox.pro` content.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-05](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-15: Ambiguous prefix exits 1

- **Given:** Two accounts saved: `i3@wbox.pro` and `i5@wbox.pro`.
- **When:** `clp .account.use i` (prefix matches both accounts)
- **Then:** Exits 1; stderr contains "ambiguous" and lists both matching account names.
- **Exit:** 1
- **Source:** [015_name_shortcut_syntax.md AC-06](../../../../docs/feature/015_name_shortcut_syntax.md)
