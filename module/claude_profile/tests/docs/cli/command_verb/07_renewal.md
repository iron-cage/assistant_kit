# Test: verb::renewal

Behavioral contract tests for the `renewal` verb. Verifies full idempotency, metadata
accumulation via read-merge, and pre-condition enforcement as defined in
[docs/cli/command_verb/007_renewal.md](../../../../docs/cli/command_verb/007_renewal.md).

**Idempotency:** Yes — same timestamp value produces identical stored state; repeated calls converge.
**State Pattern:** Accumulates state — updates `_renewal_at` in `{name}.json` via read-merge; lifecycle state unchanged.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| BV-1 | Re-setting the same renewal timestamp is idempotent | Idempotency |
| BV-2 | Renewal writes `_renewal_at` to `{name}.json` preserving all other fields | State Transition |
| BV-3 | Renewal without an operation parameter exits 1 | Pre-condition |

### Test Coverage Summary

- Idempotency: 1 test
- State Transition: 1 test
- Pre-condition: 1 test

**Total:** 3 behavioral contract tests

---

### BV-1: Re-setting the same renewal timestamp is idempotent

- **Given:** `alice@acme.com` exists. `alice@acme.com.json` has `_renewal_at = "2026-07-01T00:00:00Z"` and a custom field `role = "work"`.
- **When:** `clp .account.renewal name::alice@acme.com at::2026-07-01T00:00:00Z` (same timestamp as existing value)
- **Then:** Exit 0. `alice@acme.com.json` `_renewal_at` unchanged (`"2026-07-01T00:00:00Z"`). `role` field preserved. No error output.
- **Exit:** 0
- **Source:** [007_renewal.md — Idempotency](../../../../docs/cli/command_verb/007_renewal.md#idempotency)

---

### BV-2: Renewal writes `_renewal_at` to `{name}.json` preserving all other fields

- **Given:** `alice@acme.com` exists. `alice@acme.com.json` contains `{"role": "work", "host": "laptop"}` (no `_renewal_at` field).
- **When:** `clp .account.renewal name::alice@acme.com at::2026-07-01T00:00:00Z`
- **Then:** Exit 0. `alice@acme.com.json` now contains `_renewal_at = "2026-07-01T00:00:00Z"`. `role` and `host` fields preserved (read-merge semantics). Account lifecycle state unchanged.
- **Exit:** 0
- **Source:** [007_renewal.md — State Transition Pattern](../../../../docs/cli/command_verb/007_renewal.md#state-transition-pattern)

---

### BV-3: Renewal without an operation parameter exits 1

- **Given:** `alice@acme.com` exists in credential store.
- **When:** `clp .account.renewal name::alice@acme.com` (no `at::`, `from_now::`, or `clear::` provided)
- **Then:** Exit 1. `alice@acme.com.json` unmodified. Error message on stderr referencing missing required operation parameter.
- **Exit:** 1
- **Source:** [007_renewal.md — Behavioral Contract](../../../../docs/cli/command_verb/007_renewal.md#behavioral-contract)
