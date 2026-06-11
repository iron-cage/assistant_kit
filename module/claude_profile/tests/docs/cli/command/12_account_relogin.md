# Test: `.account.relogin`

Integration test planning for the `.account.relogin` command. See [command/001_account.md](../../../../docs/cli/command/001_account.md#command--12-accountrelogin) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| IT-1 | Missing `name::` with active account uses active | Default Behavior |
| IT-2 | Missing `name::` with no active account exits 2 | Error Handling |
| IT-3 | Empty `name::` value exits 1 | Validation |
| IT-4 | Account not found exits 2 | Error Handling |
| IT-5 | `dry::1` with explicit name prints message without mutation | Dry Run |
| IT-6 | `dry::1` with no name uses active account | Dry Run |
| IT-7 | Positional bare arg resolves name | Positional Syntax |
| IT-8 | Prefix resolves to single account | Prefix Resolution |
| IT-9 | Invalid characters in name exits 1 | Validation |

### Test Coverage Summary

- Default Behavior: 1 test
- Validation: 2 tests
- Error Handling: 2 tests
- Dry Run: 2 tests
- Positional Syntax: 1 test
- Prefix Resolution: 1 test

**Total:** 9 integration tests

**Note:** Live credential-update tests (spawn `claude` → detect mtime change → `save`) require an
interactive TTY and live Anthropic credentials. Those are covered by manual testing only
(`tests/manual/`); automated tests cover only error/validation/dry-run paths.

---

### IT-1: Missing `name::` with active account uses active account

- **Given:** Account store contains `work@acme.com.credentials.json`. Per-machine active marker points to `work@acme.com`. `dry::1` used to avoid spawning claude.
- **When:** `clp .account.relogin dry::1` (no `name::`)
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0. No state mutation.
- **Exit:** 0
- **Source:** [feature/019_account_relogin.md AC-02](../../../../docs/feature/019_account_relogin.md), [invariant/006_param_defaults.md](../../../../docs/invariant/006_param_defaults.md)

---

### IT-2: Missing `name::` with no active account exits 2

- **Given:** Account store contains `work@acme.com.credentials.json`. No per-machine active marker present.
- **When:** `clp .account.relogin`
- **Then:** Error message on stderr containing "no active account" or equivalent, exit 2. No state mutation.
- **Exit:** 2
- **Source:** [feature/019_account_relogin.md AC-03](../../../../docs/feature/019_account_relogin.md)

---

### IT-3: Empty `name::` value exits 1

- **Given:** Account store contains `work@acme.com.credentials.json`.
- **When:** `clp .account.relogin name::`
- **Then:** Error message on stderr indicating empty name, exit 1. No state mutation.
- **Exit:** 1
- **Source:** [command/001_account.md — .account.relogin](../../../../docs/cli/command/001_account.md#command--12-accountrelogin)

---

### IT-4: Account not found exits 2

- **Given:** Account store contains only `work@acme.com.credentials.json`. No `ghost@example.com` saved.
- **When:** `clp .account.relogin name::ghost@example.com`
- **Then:** Error message on stderr containing "not found", exit 2. No state mutation.
- **Exit:** 2
- **Source:** [feature/019_account_relogin.md AC-04](../../../../docs/feature/019_account_relogin.md)

---

### IT-5: Dry run with explicit name prints action without mutation

- **Given:** Account `work@acme.com` saved. Record SHA-256 of `work@acme.com.credentials.json` and the per-machine active marker before command.
- **When:** `clp .account.relogin name::work@acme.com dry::1`
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0. No credential files modified; per-machine active marker unchanged.
- **Exit:** 0
- **Source:** [feature/019_account_relogin.md AC-01](../../../../docs/feature/019_account_relogin.md)

---

### IT-6: Dry run with no name uses active account

- **Given:** Account `work@acme.com` saved. Per-machine active marker points to `work@acme.com`. `dry::1` used.
- **When:** `clp .account.relogin dry::1`
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0. No credential files modified.
- **Exit:** 0
- **Source:** [feature/019_account_relogin.md AC-02](../../../../docs/feature/019_account_relogin.md)

---

### IT-7: Positional bare arg resolves name

- **Given:** Account `work@acme.com` saved. `dry::1` is used to avoid spawning claude.
- **When:** `clp .account.relogin work@acme.com dry::1` (no `name::` prefix)
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0.
- **Exit:** 0
- **Source:** [feature/019_account_relogin.md AC-05](../../../../docs/feature/019_account_relogin.md)

---

### IT-8: Prefix resolves to single account

- **Given:** Two accounts saved: `work@acme.com` and `personal@home.com`. `dry::1` used to avoid spawning claude.
- **When:** `clp .account.relogin work dry::1` (prefix form)
- **Then:** `[dry-run] would re-authenticate 'work@acme.com' via browser login` on stdout, exit 0.
- **Exit:** 0
- **Source:** [feature/019_account_relogin.md AC-06](../../../../docs/feature/019_account_relogin.md)

---

### IT-9: Invalid characters in name exits 1

- **Given:** Account store contains `work@acme.com`.
- **When:** `clp .account.relogin name::bad/name`
- **Then:** Error message on stderr indicating invalid characters, exit 1. No state mutation.
- **Exit:** 1
- **Source:** [command/001_account.md — .account.relogin](../../../../docs/cli/command/001_account.md#command--12-accountrelogin)
