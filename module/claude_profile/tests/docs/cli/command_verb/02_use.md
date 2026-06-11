# Test: verb::use

Behavioral contract tests for the `use` verb. Verifies idempotency, atomic state transition,
and pre-condition enforcement as defined in [docs/cli/command_verb/002_use.md](../../../../docs/cli/command_verb/002_use.md).

**Idempotency:** Conditional — re-activating the already-active account is a no-op.
**State Pattern:** Transitions state (`saved → active`; prior `active → saved`).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Re-activating the already-active account is a no-op | Idempotency |
| BV-2 | Use transitions a saved account to active and prior active to saved | State Transition |
| BV-3 | Use on a non-existent account exits 2 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Re-activating the already-active account is a no-op

- **Given:** `alice@acme.com` is the currently active account. `~/.claude/.credentials.json` contains alice's credentials. Active marker `_active_{hostname}_{user}` contains `alice@acme.com`.
- **When:** `clp .account.use name::alice@acme.com`
- **Then:** Exit 0. `~/.claude/.credentials.json` content unchanged. Active marker unchanged. No error output.
- **Exit:** 0
- **Source:** [002_use.md — Idempotency](../../../../docs/cli/command_verb/002_use.md#idempotency)

---

### BV-2: Use transitions a saved account to active and prior active to saved

- **Given:** `alice@acme.com` is saved (not active) in credential store. `bob@acme.com` is currently active (`~/.claude/.credentials.json` contains bob's credentials; marker = `bob@acme.com`).
- **When:** `clp .account.use name::alice@acme.com`
- **Then:** Exit 0. `~/.claude/.credentials.json` now contains alice's credentials (written atomically). Active marker now contains `alice@acme.com`. `bob@acme.com` profile remains in store (his `saved` state is preserved).
- **Exit:** 0
- **Source:** [002_use.md — State Transition Pattern](../../../../docs/cli/command_verb/002_use.md#state-transition-pattern)

---

### BV-3: Use on a non-existent account exits 2

- **Given:** `nonexistent@acme.com` does NOT exist in credential store. `alice@acme.com` is currently active.
- **When:** `clp .account.use name::nonexistent@acme.com`
- **Then:** Exit 2. `~/.claude/.credentials.json` unchanged (still alice's credentials). Active marker unchanged. Error message on stderr referencing account not found.
- **Exit:** 2
- **Source:** [002_use.md — Behavioral Contract](../../../../docs/cli/command_verb/002_use.md#behavioral-contract)
