# Test: verb::delete

Behavioral contract tests for the `delete` verb. Verifies conditional non-idempotency,
file removal state transition, and pre-condition enforcement as defined in
[docs/cli/command_verb/003_delete.md](../../../../docs/cli/command_verb/003_delete.md).

**Idempotency:** Conditional — deleting an absent account exits 2 (not a silent no-op).
**State Pattern:** Removes state (`saved → absent`; `active → absent` with marker cleared).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Second delete of absent account exits 2 (non-idempotent) | Idempotency |
| BV-2 | Delete transitions account from saved to absent | State Transition |
| BV-3 | Delete on non-existent account exits 2 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Second delete of absent account exits 2 (non-idempotent)

- **Given:** `alice@acme.com` exists in credential store. First `clp .account.delete name::alice@acme.com` completes with exit 0 (account removed).
- **When:** `clp .account.delete name::alice@acme.com` (same command again; account now absent)
- **Then:** Exit 2. Error message on stderr indicating account not found. No unexpected file modifications.
- **Exit:** 2
- **Source:** [003_delete.md — Idempotency](../../../../docs/cli/command_verb/003_delete.md#idempotency)

---

### BV-2: Delete transitions account from saved to absent

- **Given:** `alice@acme.com` profile exists in credential store (`alice@acme.com.credentials.json` and `alice@acme.com.json` both present). Account is in `[saved]` state (not currently active).
- **When:** `clp .account.delete name::alice@acme.com`
- **Then:** Exit 0. `alice@acme.com.credentials.json` absent. `alice@acme.com.json` absent. No error output. `~/.claude/.credentials.json` unchanged (deleting a non-active account does not alter live session).
- **Exit:** 0
- **Source:** [003_delete.md — State Transition Pattern](../../../../docs/cli/command_verb/003_delete.md#state-transition-pattern)

---

### BV-3: Delete on non-existent account exits 2

- **Given:** `nobody@acme.com` does NOT exist in credential store. Credential store contains at least one other account.
- **When:** `clp .account.delete name::nobody@acme.com`
- **Then:** Exit 2. No files in credential store modified. Error message on stderr referencing account not found.
- **Exit:** 2
- **Source:** [003_delete.md — Behavioral Contract](../../../../docs/cli/command_verb/003_delete.md#behavioral-contract)
