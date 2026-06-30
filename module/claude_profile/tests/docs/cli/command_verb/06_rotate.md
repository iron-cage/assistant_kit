# Test: verb::rotate

> **DEPRECATED** — `rotate` verb removed. Behavioral contract superseded by `tests/docs/feature/038_usage_strategy_rotate.md`.

Behavioral contract tests for the `rotate` verb. Verifies non-idempotency, best-inactive
selection state transition, and pre-condition enforcement as defined in
[docs/cli/command_verb/006_rotate.md](../../../../docs/cli/command_verb/006_rotate.md).

**Idempotency:** No — selected account depends on current `expiresAt` values; repeated calls may select different accounts as tokens expire.
**State Pattern:** Transitions state — selects highest-expiry inactive account, activates it atomically.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Rotate selects best-expiry inactive account — outcome changes as tokens expire | Idempotency |
| BV-2 | Rotate activates highest-expiry inactive account and deactivates prior active | State Transition |
| BV-3 | Rotate with no inactive accounts exits 2 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Rotate selects best-expiry inactive account — outcome changes as tokens expire

- **Given:** Three saved accounts: `alice@acme.com` (expires +5h), `bob@acme.com` active, `carol@acme.com` (expires +3h). First `clp .account.rotate` selects alice. Now alice's token expires (simulate by setting `expiresAt` to past). Carol is now the best inactive account.
- **When:** `clp .account.rotate` (second call, after alice's token has expired)
- **Then:** Exit 0. Carol activated (not alice — her token expired). Confirms non-idempotency: outcome differs from first call based on current token state.
- **Exit:** 0
- **Source:** [006_rotate.md — Idempotency](../../../../docs/cli/command_verb/006_rotate.md#idempotency)

---

### BV-2: Rotate activates highest-expiry inactive account and deactivates prior active

- **Given:** `alice@acme.com` is inactive with `expiresAt` = T+7h. `carol@acme.com` is inactive with `expiresAt` = T+3h. `bob@acme.com` is currently active.
- **When:** `clp .account.rotate`
- **Then:** Exit 0. Alice activated (highest `expiresAt` among inactive accounts). `~/.claude/.credentials.json` atomically updated to alice's credentials. Active marker updated to `alice@acme.com`. Bob transitions to `[saved]`. Carol remains `[saved]`.
- **Exit:** 0
- **Source:** [006_rotate.md — State Transition Pattern](../../../../docs/cli/command_verb/006_rotate.md#state-transition-pattern)

---

### BV-3: Rotate with no inactive accounts exits 2

- **Given:** Only one account exists in credential store and it is currently active (no inactive saved accounts available).
- **When:** `clp .account.rotate`
- **Then:** Exit 2. `~/.claude/.credentials.json` unchanged. Error message on stderr referencing no eligible inactive accounts.
- **Exit:** 2
- **Source:** [006_rotate.md — Behavioral Contract](../../../../docs/cli/command_verb/006_rotate.md#behavioral-contract)
