# Test: verb::limits

Behavioral contract tests for the `limits` verb. Verifies full idempotency, read-only
state behavior, and pre-condition enforcement as defined in
[docs/cli/command_verb/004_limits.md](../../../../docs/cli/command_verb/004_limits.md).

**Idempotency:** Yes — pure read from live API headers; no side effects accumulate.
**State Pattern:** Reads state — no local files written.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Repeated limits calls produce no local side effects | Idempotency |
| BV-2 | Limits read is purely non-mutating — no files modified | State Transition |
| BV-3 | Limits without accessible account exits 2 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Repeated limits calls produce no local side effects

- **Given:** `alice@acme.com` credentials accessible in credential store. Claude API reachable. No local files have been modified by a prior limits call.
- **When:** `clp .account.limits name::alice@acme.com` called twice in sequence (without any other commands between)
- **Then:** Both calls exit 0. Rate-limit utilization output produced both times. No credential store files written or modified between calls. `alice@acme.com.credentials.json` mtime unchanged.
- **Exit:** 0
- **Source:** [004_limits.md — Idempotency](../../../../docs/cli/command_verb/004_limits.md#idempotency)

---

### BV-2: Limits read is purely non-mutating — no files modified

- **Given:** `alice@acme.com` exists in credential store. Record the mtime of `alice@acme.com.credentials.json` and `alice@acme.com.json`.
- **When:** `clp .account.limits name::alice@acme.com`
- **Then:** Exit 0. mtime of `alice@acme.com.credentials.json` unchanged. mtime of `alice@acme.com.json` unchanged. No new files created in credential store.
- **Exit:** 0
- **Source:** [004_limits.md — State Transition Pattern](../../../../docs/cli/command_verb/004_limits.md#state-transition-pattern)

---

### BV-3: Limits without accessible account exits 2

- **Given:** Credential store is empty (no saved accounts). No active account set.
- **When:** `clp .account.limits`
- **Then:** Exit 2. Error message on stderr referencing no active account or account not found.
- **Exit:** 2
- **Source:** [004_limits.md — Behavioral Contract](../../../../docs/cli/command_verb/004_limits.md#behavioral-contract)
