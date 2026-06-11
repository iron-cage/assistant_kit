# Test: verb::save

Behavioral contract tests for the `save` verb. Verifies idempotency, state creation, and
pre-condition enforcement as defined in [docs/cli/command_verb/001_save.md](../../../../docs/cli/command_verb/001_save.md).

**Idempotency:** Conditional — same credentials re-saved produces same state; changed credentials overwrite.
**State Pattern:** Creates state (`absent → saved`; `saved → saved` via read-merge).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Re-save with same credentials produces identical stored state | Idempotency |
| BV-2 | Save transitions account from absent to saved | State Transition |
| BV-3 | Save without readable credentials exits 2 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Re-save with same credentials produces identical stored state

- **Given:** `~/.claude/.credentials.json` exists with valid content V1. `{credential_store}/alice@acme.com.credentials.json` already exists with content V1 (first save already done). `{credential_store}/alice@acme.com.json` exists with supplementary metadata.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exit 0. `alice@acme.com.credentials.json` contains V1 (unchanged). `alice@acme.com.json` supplementary fields preserved. No error output.
- **Exit:** 0
- **Source:** [001_save.md — Idempotency](../../../../docs/cli/command_verb/001_save.md#idempotency)

---

### BV-2: Save transitions account from absent to saved

- **Given:** `~/.claude/.credentials.json` exists with valid credential content. No `alice@acme.com.*` files exist in credential store.
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** `{credential_store}/alice@acme.com.credentials.json` exists. `{credential_store}/alice@acme.com.json` exists. stdout: `saved current credentials as 'alice@acme.com'`. Exit 0.
- **Exit:** 0
- **Source:** [001_save.md — State Transition Pattern](../../../../docs/cli/command_verb/001_save.md#state-transition-pattern)

---

### BV-3: Save without readable credentials exits 2

- **Given:** `~/.claude/.credentials.json` does NOT exist. Credential store directory exists (empty).
- **When:** `clp .account.save name::alice@acme.com`
- **Then:** Exit 2. No files created in credential store. Error message on stderr referencing absent or unreadable credentials file.
- **Exit:** 2
- **Source:** [001_save.md — Behavioral Contract](../../../../docs/cli/command_verb/001_save.md#behavioral-contract)
