# Test: verb::inspect

Behavioral contract tests for the `inspect` verb. Verifies full idempotency, read-only
state behavior across three API endpoints, and pre-condition enforcement as defined in
[docs/cli/command_verb/008_inspect.md](../../../../docs/cli/command_verb/008_inspect.md).

**Idempotency:** Yes — pure diagnostic read from three live API endpoints; no side effects accumulate.
**State Pattern:** Reads state — no local files written (unless `refresh::1` triggers token refresh).

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Repeated inspect calls produce no local side effects | Idempotency |
| BV-2 | Inspect is purely non-mutating — no credential store files modified | State Transition |
| BV-3 | Inspect without accessible account credentials exits 2 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Repeated inspect calls produce no local side effects

- **Given:** `alice@acme.com` credentials accessible. All three API endpoints reachable. Record mtime of all files in credential store.
- **When:** `clp .account.inspect name::alice@acme.com` called twice in sequence
- **Then:** Both calls exit 0. Identity and subscription data reported both times. No credential store files written or modified between calls (all mtimes unchanged).
- **Exit:** 0
- **Source:** [008_inspect.md — Idempotency](../../../../docs/cli/command_verb/008_inspect.md#idempotency)

---

### BV-2: Inspect is purely non-mutating — no credential store files modified

- **Given:** `alice@acme.com` exists. `alice@acme.com.json` has a known `_renewal_at` value. Record mtime of `alice@acme.com.json`.
- **When:** `clp .account.inspect name::alice@acme.com`
- **Then:** Exit 0. mtime of `alice@acme.com.json` unchanged. `alice@acme.com.credentials.json` mtime unchanged. `~/.claude/.credentials.json` mtime unchanged. No new files in credential store.
- **Exit:** 0
- **Source:** [008_inspect.md — State Transition Pattern](../../../../docs/cli/command_verb/008_inspect.md#state-transition-pattern)

---

### BV-3: Inspect without accessible account credentials exits 2

- **Given:** `~/.claude/.credentials.json` is absent (no active session). Credential store is empty (no named accounts either). `name::` not provided.
- **When:** `clp .account.inspect`
- **Then:** Exit 2. Error message on stderr referencing absent or unreadable credentials. No network requests attempted.
- **Exit:** 2
- **Source:** [008_inspect.md — Behavioral Contract](../../../../docs/cli/command_verb/008_inspect.md#behavioral-contract)
