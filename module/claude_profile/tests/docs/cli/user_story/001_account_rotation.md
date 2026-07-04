# Test: user story 1 — Automatic Account Rotation

User acceptance tests for the "Automatic Account Rotation" story. Each UA-N case maps to one
Acceptance Criterion from [docs/cli/user_story/001_account_rotation.md](../../../../docs/cli/user_story/001_account_rotation.md).

**Persona:** SWE managing multiple Claude Max accounts across projects.

### Test Case Index

| ID | Test Name | Acceptance Criterion |
|----|-----------|---------------------|
| UA-1 | `.usage rotate::1` selects the best account using the active `next::` strategy | AC-1 |
| UA-2 | Rotation switch is atomic via write-then-rename | AC-2 |
| UA-3 | `dry::1` previews selected account without switching | AC-3 |
| UA-4 | `.account.use name::X` enables manual rotation to a known account | AC-4 |
| UA-5 | Exit 1 when no eligible accounts are available; quota table still rendered | AC-5 |

### Test Coverage Summary

- Strategy-based selection: 1 test
- Atomicity: 1 test
- Dry run: 1 test
- Manual fallback: 1 test
- Error handling: 1 test

**Total:** 5 user acceptance tests

---

### UA-1: `.usage rotate::1` selects the best account using the active `next::` strategy

- **Given:** Three saved accounts: `alice@acme.com` (inactive, 5h reset = T+2d), `carol@acme.com` (inactive, 5h reset = T+5d). `bob@acme.com` is currently active. Default strategy is `renew`.
- **When:** `clp .usage rotate::1`
- **Then:** Exit 0. `alice@acme.com` activated (she has the soonest 7d quota reset under `renew` strategy). Rotation confirmation on stdout identifying alice as the selected account. Quota table rendered before switch.
- **Exit:** 0
- **Source:** [001_account_rotation.md — AC-1](../../../../docs/cli/user_story/001_account_rotation.md)

---

### UA-2: Rotation switch is atomic via write-then-rename

- **Given:** `alice@acme.com` is the best inactive account. `bob@acme.com` is currently active.
- **When:** `clp .usage rotate::1`
- **Then:** Exit 0. `~/.claude/.credentials.json` is replaced atomically (write to temp file then rename — no partial credential state visible to concurrent readers). Alice is now active.
- **Exit:** 0
- **Source:** [001_account_rotation.md — AC-2](../../../../docs/cli/user_story/001_account_rotation.md)

---

### UA-3: `dry::1` previews selected account without switching

- **Given:** `alice@acme.com` is the best inactive account (highest `expiresAt`). `bob@acme.com` is active.
- **When:** `clp .usage rotate::1 dry::1`
- **Then:** Exit 0. stdout identifies `alice@acme.com` as the would-be selection. `~/.claude/.credentials.json` unchanged (still bob's credentials). Active marker unchanged. Quota table rendered.
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

### UA-5: Exit 1 when no eligible accounts are available

- **Given:** Only `alice@acme.com` exists in credential store and is currently active (no eligible accounts).
- **When:** `clp .usage rotate::1`
- **Then:** Exit 1. `~/.claude/.credentials.json` unchanged. Error message on stderr indicating no eligible accounts. Quota table still rendered before error.
- **Exit:** 1
- **Source:** [001_account_rotation.md — AC-5](../../../../docs/cli/user_story/001_account_rotation.md)
