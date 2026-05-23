# Test: `.account.relogin`

Integration test planning for the `.account.relogin` command. See [command/account.md](../../../../docs/cli/command/account.md#command--12-accountrelogin) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Missing `name::` exits 1 | Validation |
| IT-2 | Empty `name::` exits 1 | Validation |
| IT-3 | Account not found exits 2 | Error Handling |
| IT-4 | `dry::1` prints message without mutation | Dry Run |
| IT-5 | Positional bare arg resolves name | Positional Syntax |
| IT-6 | Prefix resolves to single account | Prefix Resolution |
| IT-7 | Invalid characters in name exits 1 | Validation |

### Test Coverage Summary

- Validation: 3 tests
- Error Handling: 1 test
- Dry Run: 1 test
- Positional Syntax: 1 test
- Prefix Resolution: 1 test

**Total:** 7 integration tests

**Note:** Live credential-update tests (spawn `claude` → detect mtime change → `save`) require an
interactive TTY and live Anthropic credentials. Those are covered by manual testing only
(`tests/manual/`); automated tests cover only error/validation/dry-run paths.

---

### IT-1: Missing `name::` parameter exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`.
- **When:** `clp .account.relogin`
- **Then:** Error message on stderr referencing missing required parameter, exit 1. No state mutation.
- **Exit:** 1
- **Source:** [command/account.md — .account.relogin](../../../../docs/cli/command/account.md#command--12-accountrelogin)

---

### IT-2: Empty `name::` exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`.
- **When:** `clp .account.relogin name::`
- **Then:** Error message on stderr indicating empty name, exit 1. No state mutation.
- **Exit:** 1
- **Source:** [command/account.md — .account.relogin](../../../../docs/cli/command/account.md#command--12-accountrelogin)

---

### IT-3: Account not found exits 2

- **Given:** Account store contains only `work@acme.com.credentials.json`. No `ghost@example.com` saved.
- **When:** `clp .account.relogin name::ghost@example.com`
- **Then:** Error message on stderr containing "not found", exit 2. No state mutation.
- **Exit:** 2
- **Source:** [command/account.md — .account.relogin](../../../../docs/cli/command/account.md#command--12-accountrelogin)

---

### IT-4: Dry run prints action without mutation

- **Given:** Account `work@acme.com` saved. Record SHA-256 of `work@acme.com.credentials.json` and `_active` before command.
- **When:** `clp .account.relogin name::work@acme.com dry::1`
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0. No credential files modified; `_active` marker unchanged.
- **Exit:** 0
- **Source:** [command/account.md — .account.relogin](../../../../docs/cli/command/account.md#command--12-accountrelogin)

---

### IT-5: Positional bare arg resolves name

- **Given:** Account `work@acme.com` saved. `dry::1` is used to avoid spawning claude.
- **When:** `clp .account.relogin work@acme.com dry::1` (no `name::` prefix)
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-02](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-6: Prefix resolves to single account

- **Given:** Two accounts saved: `work@acme.com` and `personal@home.com`. `dry::1` used to avoid spawning claude.
- **When:** `clp .account.relogin work dry::1` (prefix form)
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0.
- **Exit:** 0
- **Source:** [015_name_shortcut_syntax.md AC-05](../../../../docs/feature/015_name_shortcut_syntax.md)

---

### IT-7: Invalid characters in prefix name exits 1

- **Given:** Account store contains `work@acme.com`.
- **When:** `clp .account.relogin name::bad/name`
- **Then:** Error message on stderr indicating invalid characters, exit 1. No state mutation.
- **Exit:** 1
- **Source:** [command/account.md — .account.relogin](../../../../docs/cli/command/account.md#command--12-accountrelogin)
