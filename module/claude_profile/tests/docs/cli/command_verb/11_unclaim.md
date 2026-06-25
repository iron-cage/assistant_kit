# Test: verb::unclaim

Behavioral contract tests for the `unclaim` verb behavior. The `unclaim::1` param is REMOVED in
Feature 064 (exits 1 with migration message). The unclaim behavior is now delivered via
`owner::0 name::X`. Tests verify the current interface.
See [docs/cli/command_verb/011_unclaim.md](../../../../docs/cli/command_verb/011_unclaim.md).

**Idempotency:** Yes — clearing an already-unowned account produces identical stored state.
**State Pattern:** Metadata-only mutation — writes `owner: ""` to `{name}.json` only; `{name}.credentials.json` untouched; active marker untouched.
**Migration:** `unclaim::1 name::X` (Feature 037) → `owner::0 name::X` (Feature 064).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Re-clearing an already-unowned account is idempotent | Idempotency |
| BV-2 | `owner::0 name::X` writes `owner: ""` without touching credential files | State Transition |
| BV-3 | `owner::0 name::X` on account owned by different identity exits 1 (G8) | Pre-condition (G8) |
| BV-4 | `unclaim::1` REMOVED_TOGGLE exits 1 with migration message | Migration |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test
- Migration: 1 test

**Total:** 4 behavioral contract tests

---

### BV-1: Re-clearing an already-unowned account is idempotent (Feature 064)

- **Given:** `alice@acme.com` exists in credential store. `alice.json` already contains `"owner": ""` (unowned). Current identity = `testuser@testmachine`.
- **When:** `clp .accounts owner::0 name::alice@acme.com` (formerly `unclaim::1 name::alice@acme.com`)
- **Then:** Exit 0. `alice.json` still contains `"owner": ""` (unchanged). `alice.credentials.json` unmodified. No error output.
- **Exit:** 0
- **Source:** [011_unclaim.md — Idempotency](../../../../docs/cli/command_verb/011_unclaim.md#idempotency)

---

### BV-2: `owner::0 name::X` writes `owner: ""` without touching credential files (Feature 064)

- **Given:** `alice@acme.com` exists. `alice.json` has `"owner": "testuser@testmachine"`. `current_identity()` = `testuser@testmachine`. Record mtime of `alice.credentials.json` and `~/.claude/.credentials.json`.
- **When:** `clp .accounts owner::0 name::alice@acme.com` (formerly `unclaim::1 name::alice@acme.com`)
- **Then:** Exit 0. `alice.json` contains `"owner": ""`. mtime of `alice.credentials.json` unchanged. mtime of `~/.claude/.credentials.json` unchanged (no credential rotation). Active marker `_active_{hostname}_{user}` unchanged.
- **Exit:** 0
- **Source:** [011_unclaim.md — State Transition Pattern](../../../../docs/cli/command_verb/011_unclaim.md#state-transition-pattern)

---

### BV-3: `owner::0 name::X` on account owned by different identity exits 1 (G8) (Feature 064)

- **Given:** `alice@acme.com` exists. `alice.json` has `"owner": "other@remote"`. Current identity ≠ `other@remote`.
- **When:** `clp .accounts owner::0 name::alice@acme.com` (formerly `unclaim::1 name::alice@acme.com`)
- **Then:** Exit 1. Stderr contains `ownership violation: this account is owned by other@remote`. `alice.json` unchanged (`owner` remains `"other@remote"`). No file written.
- **Exit:** 1
- **Source:** [011_unclaim.md — Ownership Gate (G8)](../../../../docs/cli/command_verb/011_unclaim.md#ownership-gate-g8)

---

### BV-4: `unclaim::1` REMOVED_TOGGLE — exits 1 with migration message (Feature 064)

- **Given:** Any environment.
- **When:** `clp .accounts unclaim::1 name::alice@acme.com`
- **Then:** Exit 1. Migration message: "REMOVED — use `owner::0 name::X`". No file written.
- **Exit:** 1
- **Source:** [011_unclaim.md — Migration](../../../../docs/cli/command_verb/011_unclaim.md#migration-feature-037--feature-064)
