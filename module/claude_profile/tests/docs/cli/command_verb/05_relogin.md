# Test: verb::relogin

Behavioral contract tests for the `relogin` verb. Verifies non-idempotency, in-place
credential refresh (lifecycle state preserved), and pre-condition enforcement as defined in
[docs/cli/command_verb/005_relogin.md](../../../../docs/cli/command_verb/005_relogin.md).

**Idempotency:** No — each invocation opens a new OAuth browser flow; tokens differ across calls.
**State Pattern:** Transitions credentials in-place; lifecycle state (`saved`/`active`) preserved.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Repeated relogin calls produce different tokens (non-idempotent) | Idempotency |
| BV-2 | Relogin updates credentials in-place; lifecycle state preserved | State Transition |
| BV-3 | Relogin on absent account profile exits 1 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Repeated relogin calls produce different tokens (non-idempotent)

- **Given:** `alice@acme.com` profile exists in credential store. TTY available. Two successive relogin invocations both complete successfully (user completes OAuth flow each time).
- **When:** `clp .account.relogin name::alice@acme.com` called (first invocation, OAuth completed), then `clp .account.relogin name::alice@acme.com` called again (second OAuth flow completed)
- **Then:** `alice@acme.com.credentials.json` after second call contains different `accessToken` than after first call. Each call produces a fresh token set — not idempotent across calls.
- **Exit:** 0
- **Source:** [005_relogin.md — Idempotency](../../../../docs/cli/command_verb/005_relogin.md#idempotency)

---

### BV-2: Relogin updates credentials in-place; lifecycle state preserved

- **Given:** `alice@acme.com` is in `[saved]` state (not active). `alice@acme.com.credentials.json` contains expired tokens. TTY available. OAuth flow completes successfully.
- **When:** `clp .account.relogin name::alice@acme.com`
- **Then:** Exit 0. `alice@acme.com.credentials.json` updated with new tokens. Account remains in `[saved]` state (NOT promoted to active). `~/.claude/.credentials.json` unmodified (prior active session unchanged).
- **Exit:** 0
- **Source:** [005_relogin.md — State Transition Pattern](../../../../docs/cli/command_verb/005_relogin.md#state-transition-pattern)

---

### BV-3: Relogin on absent account profile exits 1

- **Given:** `nobody@acme.com` does NOT exist in credential store.
- **When:** `clp .account.relogin name::nobody@acme.com`
- **Then:** Exit 1. No browser flow spawned. Error message on stderr referencing account not found. No files created.
- **Exit:** 1
- **Source:** [005_relogin.md — Behavioral Contract](../../../../docs/cli/command_verb/005_relogin.md#behavioral-contract)
