# Test: verb::assign

Behavioral contract tests for the `assign` verb behavior. The `assign::1` param is REMOVED in
Feature 064 (exits 1 with migration message). The assign behavior is now delivered via
`assignee::USER@MACHINE name::X`. Tests verify the current interface.
See [docs/cli/command_verb/009_assign.md](../../../../docs/cli/command_verb/009_assign.md).

**Idempotency:** Yes — writing the same active marker repeatedly produces identical stored state.
**State Pattern:** Accumulates state — writes `_active_{machine}_{user}` marker only; `~/.claude/.credentials.json` untouched.
**Migration:** `assign::1 name::X` (Feature 037) → `active::USER@MACHINE name::X` (Feature 064) → `assignee::USER@MACHINE name::X` (Feature 065).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Re-assigning the same account is idempotent | Idempotency |
| BV-2 | Assign writes active marker without touching credential files | State Transition |
| BV-3 | Assign on non-existent account exits 1 | Pre-condition |
| BV-4 | `assign::1` REMOVED_TOGGLE exits 1 with migration message | Migration |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test
- Migration: 1 test

**Total:** 4 behavioral contract tests

---

### BV-1: Re-assigning the same account is idempotent (Feature 064)

- **Given:** `alice@acme.com` exists in credential store. Active marker `_active_{hostname}_{user}` already contains `alice@acme.com`.
- **When:** `clp .accounts assignee::testuser@testmachine name::alice@acme.com` (formerly `assign::1 name::alice@acme.com`)
- **Then:** Exit 0. `_active_testmachine_testuser` still contains `alice@acme.com` (unchanged). `~/.claude/.credentials.json` unmodified. No error output.
- **Exit:** 0
- **Source:** [009_assign.md — Idempotency](../../../../docs/cli/command_verb/009_assign.md#idempotency)

---

### BV-2: Assign writes active marker without touching credential files (Feature 064)

- **Given:** `alice@acme.com` exists in credential store. No active marker exists. Record mtime of `alice@acme.com.credentials.json` and `~/.claude/.credentials.json`.
- **When:** `clp .accounts assignee::testuser@testmachine name::alice@acme.com` (formerly `assign::1 name::alice@acme.com`)
- **Then:** Exit 0. `_active_testmachine_testuser` created in credential store containing `alice@acme.com`. mtime of `alice@acme.com.credentials.json` unchanged. mtime of `~/.claude/.credentials.json` unchanged (no credential rotation performed).
- **Exit:** 0
- **Source:** [009_assign.md — State Transition Pattern](../../../../docs/cli/command_verb/009_assign.md#state-transition-pattern)

---

### BV-3: Assign on non-existent account exits 1 (Feature 064)

- **Given:** `nobody@acme.com` does NOT exist in credential store.
- **When:** `clp .accounts assignee::testuser@testmachine name::nobody@acme.com`
- **Then:** Exit 1 (or 2 — account-not-found). No marker file written. Error message on stderr referencing account not found.
- **Exit:** 1 or 2
- **Source:** [009_assign.md — Behavioral Contract](../../../../docs/cli/command_verb/009_assign.md#behavioral-contract)

---

### BV-4: `assign::1` REMOVED_TOGGLE — exits 1 with migration message (Feature 064)

- **Given:** Any environment.
- **When:** `clp .accounts assign::1 name::alice@acme.com`
- **Then:** Exit 1. Migration message: "REMOVED — use `assignee::USER@MACHINE name::X`". No marker file written.
- **Exit:** 1
- **Source:** [009_assign.md — Migration](../../../../docs/cli/command_verb/009_assign.md#migration-feature-037--feature-064)
