# Test: user story 1 — Automatic Account Rotation

User acceptance tests for the "Automatic Account Rotation" story. Each UA-N case maps to one
Acceptance Criterion from [docs/cli/user_story/001_account_rotation.md](../../../../docs/cli/user_story/001_account_rotation.md).

**Persona:** SWE managing multiple Claude Max accounts across projects.

### Test Case Index

| ID | Test Name | Acceptance Criterion |
|----|-----------|---------------------|
| UA-1 | `.account.rotate` selects the inactive account with the highest remaining expiry | AC-1 |
| UA-2 | Rotation switch is atomic via write-then-rename | AC-2 |
| UA-3 | `dry::1` previews selected account without switching | AC-3 |
| UA-4 | `.account.use name::X` enables manual rotation to a known account | AC-4 |
| UA-5 | Exit 2 when no inactive accounts are available | AC-5 |

### Test Coverage Summary

- Automatic selection: 1 test
- Atomicity: 1 test
- Dry run: 1 test
- Manual fallback: 1 test
- Error handling: 1 test

**Total:** 5 user acceptance tests

---

### UA-1: `.account.rotate` selects the inactive account with the highest remaining expiry

- **Given:** Three saved accounts: `alice@acme.com` (inactive, `expiresAt` = T+7h), `carol@acme.com` (inactive, `expiresAt` = T+3h). `bob@acme.com` is currently active.
- **When:** `clp .account.rotate`
- **Then:** Exit 0. `alice@acme.com` activated (she has the higher remaining expiry among inactive accounts). Rotation confirmation on stdout identifying alice as the selected account.
- **Exit:** 0
- **Source:** [001_account_rotation.md — AC-1](../../../../docs/cli/user_story/001_account_rotation.md)

---

### UA-2: Rotation switch is atomic via write-then-rename

- **Given:** `alice@acme.com` is the best inactive account. `bob@acme.com` is currently active.
- **When:** `clp .account.rotate`
- **Then:** Exit 0. `~/.claude/.credentials.json` is replaced atomically (write to temp file then rename — no partial credential state visible to concurrent readers). Alice is now active.
- **Exit:** 0
- **Source:** [001_account_rotation.md — AC-2](../../../../docs/cli/user_story/001_account_rotation.md)

---

### UA-3: `dry::1` previews selected account without switching

- **Given:** `alice@acme.com` is the best inactive account (highest `expiresAt`). `bob@acme.com` is active.
- **When:** `clp .account.rotate dry::1`
- **Then:** Exit 0. stdout identifies `alice@acme.com` as the would-be selection. `~/.claude/.credentials.json` unchanged (still bob's credentials). Active marker unchanged.
- **Exit:** 0
- **Source:** [001_account_rotation.md — AC-3](../../../../docs/cli/user_story/001_account_rotation.md)

---

### UA-4: `.account.use name::X` enables manual rotation to a known account

- **Given:** `alice@acme.com` is saved (not active). `bob@acme.com` is active.
- **When:** `clp .account.use name::alice@acme.com`
- **Then:** Exit 0. Alice is now active. `~/.claude/.credentials.json` updated to alice's credentials.
- **Exit:** 0
- **Source:** [001_account_rotation.md — AC-4](../../../../docs/cli/user_story/001_account_rotation.md)

---

### UA-5: Exit 2 when no inactive accounts are available

- **Given:** Only `alice@acme.com` exists in credential store and is currently active (no inactive accounts).
- **When:** `clp .account.rotate`
- **Then:** Exit 2. `~/.claude/.credentials.json` unchanged. Error message on stderr indicating no eligible accounts.
- **Exit:** 2
- **Source:** [001_account_rotation.md — AC-5](../../../../docs/cli/user_story/001_account_rotation.md)
