# Test: verb::assign

Behavioral contract tests for the `assign` verb. Verifies full idempotency, marker-only
write behavior, and pre-condition enforcement as defined in
[docs/cli/command_verb/009_assign.md](../../../../docs/cli/command_verb/009_assign.md).

**Idempotency:** Yes — writing the same active marker repeatedly produces identical stored state.
**State Pattern:** Accumulates state — writes `_active_{machine}_{user}` marker only; `~/.claude/.credentials.json` untouched.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Re-assigning the same account is idempotent | Idempotency |
| BV-2 | Assign writes active marker without touching credential files | State Transition |
| BV-3 | Assign on non-existent account exits 2 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Re-assigning the same account is idempotent

- **Given:** `alice@acme.com` exists in credential store. Active marker `_active_{hostname}_{user}` already contains `alice@acme.com`.
- **When:** `clp .account.assign name::alice@acme.com`
- **Then:** Exit 0. `_active_{hostname}_{user}` still contains `alice@acme.com` (unchanged). `~/.claude/.credentials.json` unmodified. No error output.
- **Exit:** 0
- **Source:** [009_assign.md — Idempotency](../../../../docs/cli/command_verb/009_assign.md#idempotency)

---

### BV-2: Assign writes active marker without touching credential files

- **Given:** `alice@acme.com` exists in credential store. No active marker `_active_{hostname}_{user}` exists. Record mtime of `alice@acme.com.credentials.json` and `~/.claude/.credentials.json`.
- **When:** `clp .account.assign name::alice@acme.com`
- **Then:** Exit 0. `_active_{hostname}_{user}` created in credential store containing `alice@acme.com`. mtime of `alice@acme.com.credentials.json` unchanged. mtime of `~/.claude/.credentials.json` unchanged (no credential rotation performed).
- **Exit:** 0
- **Source:** [009_assign.md — State Transition Pattern](../../../../docs/cli/command_verb/009_assign.md#state-transition-pattern)

---

### BV-3: Assign on non-existent account exits 2

- **Given:** `nobody@acme.com` does NOT exist in credential store.
- **When:** `clp .account.assign name::nobody@acme.com`
- **Then:** Exit 2. No marker file written. Error message on stderr referencing account not found.
- **Exit:** 2
- **Source:** [009_assign.md — Behavioral Contract](../../../../docs/cli/command_verb/009_assign.md#behavioral-contract)
